//! Putting a stored clip back onto the Windows clipboard.
//!
//! Quick copy, Quick paste and Paste as plain text all go through `write_clip`;
//! they differ only in what happens afterwards.

use crate::history::{Clip, ClipKind};
use crate::platform::clipboard_raw;
use crate::storage::Paths;

/// How text clips are placed on the clipboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextMode {
    /// Restore formatting when the clip carries rich text.
    Rich,
    /// Plain text only. The stored clip is never modified.
    Plain,
}

/// Writes a clip to the Windows clipboard. Returns false when the payload could
/// not be restored (missing image file, clipboard locked by another process).
pub fn write_clip(clip: &Clip, paths: &Paths, mode: TextMode) -> bool {
    match clip.kind {
        ClipKind::Files => {
            let existing: Vec<String> = clip
                .files
                .iter()
                .filter(|file| std::path::Path::new(&file.path).exists())
                .map(|file| file.path.clone())
                .collect();

            if existing.is_empty() {
                log::warn!("file clip {} no longer exists on disk", clip.id);
                return false;
            }

            clipboard_raw::write_file_drop(&existing)
        }
        ClipKind::Image => write_image(clip, paths),
        ClipKind::Text | ClipKind::Link => write_text(clip, mode),
    }
}

fn write_text(clip: &Clip, mode: TextMode) -> bool {
    let Some(text) = clip.text.as_ref() else {
        return false;
    };

    let Ok(mut clipboard) = arboard::Clipboard::new() else {
        return false;
    };

    let result = match (mode, clip.html.as_ref()) {
        (TextMode::Rich, Some(html)) => clipboard.set_html(html, Some(text)),
        _ => clipboard.set_text(text),
    };

    if let Err(err) = result {
        log::warn!("could not write text clip: {err}");
        return false;
    }

    true
}

fn write_image(clip: &Clip, paths: &Paths) -> bool {
    let path = paths.image(&clip.id);

    let decoded = match image::open(&path) {
        Ok(image) => image.to_rgba8(),
        Err(err) => {
            log::warn!("could not read image clip {}: {err}", clip.id);
            return false;
        }
    };

    let Ok(mut clipboard) = arboard::Clipboard::new() else {
        return false;
    };

    let (width, height) = (decoded.width() as usize, decoded.height() as usize);
    let data = arboard::ImageData {
        width,
        height,
        bytes: std::borrow::Cow::Owned(decoded.into_raw()),
    };

    if let Err(err) = clipboard.set_image(data) {
        log::warn!("could not write image clip: {err}");
        return false;
    }

    true
}
