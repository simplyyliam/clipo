//! Clipboard Core.
//!
//! One pipeline turns a Windows clipboard change into a stored clip:
//!
//! ```text
//! clipboard change -> read -> identify content -> identify source app
//!                  -> exclusions -> sensitive rules -> history -> persistence
//! ```
//!
//! Nothing else in the application is allowed to add clips.

pub mod monitor;
pub mod read;
pub mod write;

use std::path::Path;

use tauri::AppHandle;

use crate::history::{Clip, ClipKind, FileRef, ImageMeta, SourceApp};
use crate::intelligence;
use crate::platform::app as platform_app;
use crate::state::AppState;

use read::Capture;

/// Longest edge of the stored preview image. Full resolution stays on disk for
/// copy/paste; the UI only ever loads the thumbnail.
const THUMB_MAX_EDGE: u32 = 640;

/// Result of one ingestion attempt, useful for logs and for tests.
#[derive(Debug, PartialEq, Eq)]
pub enum Ingested {
    Stored,
    Empty,
    OptedOut,
    ExcludedApp,
    Sensitive,
}

/// Reads the clipboard and, if the content is allowed, adds it to history.
pub fn ingest(app: &AppHandle, state: &AppState) -> Ingested {
    let reading = read::read();

    if reading.opted_out {
        return Ingested::OptedOut;
    }

    let Some(capture) = reading.capture else {
        return Ingested::Empty;
    };

    // App-awareness: the foreground window at capture time is the source app.
    let source = current_source_app(state);
    let settings = state.settings();

    if let Some(source) = &source {
        if settings.is_app_excluded(&source.exe_name) {
            return Ingested::ExcludedApp;
        }
    }

    if settings.block_sensitive {
        if let Capture::Text { text, .. } = &capture {
            if intelligence::looks_sensitive(text) {
                return Ingested::Sensitive;
            }
        }
    }

    let id = uuid::Uuid::new_v4().to_string();

    let clip = match capture {
        Capture::Text { text, html } => {
            let url = intelligence::detect_url(&text);
            let kind = if intelligence::is_link(&text) {
                ClipKind::Link
            } else {
                ClipKind::Text
            };

            Clip {
                id,
                kind,
                created_at: crate::history::now_ms(),
                pinned: false,
                fingerprint: fingerprint("text", text.trim().as_bytes()),
                text: Some(text),
                html,
                url,
                image: None,
                files: Vec::new(),
                source,
            }
        }
        Capture::Files(paths) => {
            let files: Vec<FileRef> = paths
                .iter()
                .map(|path| FileRef {
                    name: path
                        .file_name()
                        .map(|name| name.to_string_lossy().to_string())
                        .unwrap_or_else(|| path.to_string_lossy().to_string()),
                    path: path.to_string_lossy().to_string(),
                })
                .collect();

            if files.is_empty() {
                return Ingested::Empty;
            }

            let key = files
                .iter()
                .map(|file| file.path.as_str())
                .collect::<Vec<_>>()
                .join("|");

            Clip {
                id,
                kind: ClipKind::Files,
                created_at: crate::history::now_ms(),
                pinned: false,
                fingerprint: fingerprint("files", key.as_bytes()),
                text: None,
                html: None,
                url: None,
                image: None,
                files,
                source,
            }
        }
        Capture::Image {
            rgba,
            width,
            height,
        } => {
            let fingerprint = fingerprint("image", &rgba);

            let Some(meta) = store_image(state, &id, &rgba, width, height) else {
                return Ingested::Empty;
            };

            Clip {
                id,
                kind: ClipKind::Image,
                created_at: crate::history::now_ms(),
                pinned: false,
                fingerprint,
                text: None,
                html: None,
                url: None,
                image: Some(meta),
                files: Vec::new(),
                source,
            }
        }
    };

    let discarded = {
        let mut history = state.history.lock();
        history.insert(clip, &settings)
    };

    for id in discarded {
        state.paths.remove_clip_assets(&id);
    }

    state.notify_clips_changed(app);
    Ingested::Stored
}

fn current_source_app(state: &AppState) -> Option<SourceApp> {
    // While the popover is open the foreground window is Clipo itself, so the
    // window recorded when the popover opened is the meaningful source.
    let window = state
        .foreground_window_for_capture()
        .or_else(platform_app::foreground_window)?;

    let executable = platform_app::window_executable(window)?;
    let exe_name = platform_app::executable_name(&executable);

    if exe_name.is_empty() {
        return None;
    }

    Some(SourceApp {
        name: platform_app::display_name(&executable),
        exe_name,
        exe_path: Some(executable.to_string_lossy().to_string()),
    })
}

/// Writes the full image and its thumbnail next to each other. Returns `None`
/// when the payload is malformed, which is treated as "nothing was copied".
fn store_image(
    state: &AppState,
    id: &str,
    rgba: &[u8],
    width: u32,
    height: u32,
) -> Option<ImageMeta> {
    let buffer = image::RgbaImage::from_raw(width, height, rgba.to_vec())?;

    if let Err(err) = save_png(&buffer, &state.paths.image(id)) {
        log::warn!("could not store image clip: {err}");
        return None;
    }

    let thumbnail = thumbnail(&buffer);
    if let Err(err) = save_png(&thumbnail, &state.paths.thumb(id)) {
        log::warn!("could not store image thumbnail: {err}");
    }

    let byte_len = std::fs::metadata(state.paths.image(id))
        .map(|meta| meta.len())
        .unwrap_or(0);

    Some(ImageMeta {
        width,
        height,
        byte_len,
    })
}

fn thumbnail(source: &image::RgbaImage) -> image::RgbaImage {
    let longest = source.width().max(source.height());

    if longest <= THUMB_MAX_EDGE {
        return source.clone();
    }

    let scale = THUMB_MAX_EDGE as f32 / longest as f32;
    let width = ((source.width() as f32 * scale).round() as u32).max(1);
    let height = ((source.height() as f32 * scale).round() as u32).max(1);

    image::imageops::resize(source, width, height, image::imageops::FilterType::Triangle)
}

fn save_png(image: &image::RgbaImage, path: &Path) -> std::io::Result<()> {
    let mut bytes = Vec::new();
    image
        .write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Png,
        )
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;

    crate::storage::write_atomic(path, &bytes)
}

/// Stable content hash (FNV-1a). Stable across restarts, so re-copying content
/// collapses onto the existing clip instead of duplicating it.
pub fn fingerprint(kind: &str, bytes: &[u8]) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;

    for byte in kind.as_bytes().iter().chain(bytes) {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }

    format!("{kind}:{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_is_stable_and_kind_scoped() {
        assert_eq!(fingerprint("text", b"hello"), fingerprint("text", b"hello"));
        assert_ne!(fingerprint("text", b"hello"), fingerprint("files", b"hello"));
    }
}
