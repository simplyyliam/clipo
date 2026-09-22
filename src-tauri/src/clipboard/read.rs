//! Reading the Windows clipboard into Clipo's single clip representation.

use std::path::PathBuf;

use crate::platform::clipboard_raw::{self, ClipboardGuard};

/// Raw clipboard payload, before intelligence rules are applied.
pub enum Capture {
    Files(Vec<PathBuf>),
    Image {
        rgba: Vec<u8>,
        width: u32,
        height: u32,
    },
    Text {
        text: String,
        /// Present when the source application also offered rich text.
        html: Option<String>,
    },
}

pub struct Reading {
    pub capture: Option<Capture>,
    /// The source application asked clipboard managers to ignore this content
    /// (password managers and browsers in private mode do this). Always honored,
    /// independently of the sensitive-content setting.
    pub opted_out: bool,
}

/// Reads the current clipboard once. Unsupported or empty clipboards yield
/// `capture: None` rather than an error: this runs on every clipboard change and
/// must never be noisy.
pub fn read() -> Reading {
    let mut opted_out = false;
    let mut files = Vec::new();
    let mut html = None;

    // Phase 1: formats that need direct Win32 access.
    if let Some(_guard) = ClipboardGuard::open() {
        opted_out = has_opt_out_format();

        if !opted_out {
            if clipboard_raw::has_format(clipboard_raw::CF_HDROP) {
                files = clipboard_raw::read_file_drop();
            }

            let html_format = clipboard_raw::register_format("HTML Format");
            if clipboard_raw::has_format(html_format) {
                html = clipboard_raw::format_bytes(html_format).and_then(parse_cf_html);
            }
        }
    }

    if opted_out {
        return Reading {
            capture: None,
            opted_out: true,
        };
    }

    if !files.is_empty() {
        return Reading {
            capture: Some(Capture::Files(files)),
            opted_out: false,
        };
    }

    // Phase 2: text and images, where arboard already does the format handling.
    let capture = read_with_arboard(html);

    Reading {
        capture,
        opted_out: false,
    }
}

fn read_with_arboard(html: Option<String>) -> Option<Capture> {
    let mut clipboard = match arboard::Clipboard::new() {
        Ok(clipboard) => clipboard,
        Err(err) => {
            log::warn!("clipboard unavailable: {err}");
            return None;
        }
    };

    if let Ok(text) = clipboard.get_text() {
        if !text.trim().is_empty() {
            return Some(Capture::Text { text, html });
        }
    }

    match clipboard.get_image() {
        Ok(image) => {
            let width = image.width as u32;
            let height = image.height as u32;

            if width == 0 || height == 0 {
                return None;
            }

            Some(Capture::Image {
                rgba: image.bytes.into_owned(),
                width,
                height,
            })
        }
        Err(_) => None,
    }
}

/// Formats applications register to stay out of clipboard history.
fn has_opt_out_format() -> bool {
    const OPT_OUT_FORMATS: [&str; 2] = [
        "ExcludeClipboardContentFromMonitorProcessing",
        "CanIncludeInClipboardHistory",
    ];

    for name in OPT_OUT_FORMATS {
        let format = clipboard_raw::register_format(name);
        if format == 0 || !clipboard_raw::has_format(format) {
            continue;
        }

        match name {
            // Presence alone means "do not process".
            "ExcludeClipboardContentFromMonitorProcessing" => return true,
            // A DWORD: 0 means "do not keep".
            _ => {
                let allowed = clipboard_raw::format_bytes(format)
                    .and_then(|bytes| bytes.get(0..4).map(|head| head != [0, 0, 0, 0]))
                    .unwrap_or(true);

                if !allowed {
                    return true;
                }
            }
        }
    }

    false
}

/// `CF_HTML` is a UTF-8 payload with an offset header. Only the fragment is the
/// actual copied markup.
fn parse_cf_html(bytes: Vec<u8>) -> Option<String> {
    let raw = String::from_utf8_lossy(&bytes);

    let offset = |key: &str| -> Option<usize> {
        let start = raw.find(key)? + key.len();
        let rest = &raw[start..];
        let end = rest.find(['\r', '\n'])?;
        rest[..end].trim().parse::<usize>().ok()
    };

    let fragment = match (offset("StartFragment:"), offset("EndFragment:")) {
        (Some(start), Some(end)) if end > start && end <= bytes.len() => {
            String::from_utf8_lossy(&bytes[start..end]).to_string()
        }
        _ => {
            let start = raw.find("<html").or_else(|| raw.find("<HTML"))?;
            raw[start..].to_string()
        }
    };

    let trimmed = fragment.trim().to_string();
    (!trimmed.is_empty()).then_some(trimmed)
}
