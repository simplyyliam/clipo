//! Local-only storage. Everything Clipo persists lives under the OS app-data
//! directory; nothing leaves the machine.
//!
//! Layout:
//! ```text
//! %APPDATA%/com.clipo.app/
//!   settings.json      user configuration
//!   history.json       clip metadata (text, urls, file paths, pin state)
//!   images/<id>.png    full image payload of image clips
//!   thumbs/<id>.png    downscaled preview used by the UI
//! ```

use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager};

#[derive(Debug, Clone)]
pub struct Paths {
    pub root: PathBuf,
    pub settings: PathBuf,
    pub history: PathBuf,
    pub images: PathBuf,
    pub thumbs: PathBuf,
}

impl Paths {
    pub fn resolve(app: &AppHandle) -> std::io::Result<Self> {
        let root = app
            .path()
            .app_data_dir()
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::NotFound, err.to_string()))?;

        let paths = Self {
            settings: root.join("settings.json"),
            history: root.join("history.json"),
            images: root.join("images"),
            thumbs: root.join("thumbs"),
            root,
        };

        std::fs::create_dir_all(&paths.root)?;
        std::fs::create_dir_all(&paths.images)?;
        std::fs::create_dir_all(&paths.thumbs)?;

        Ok(paths)
    }

    pub fn image(&self, id: &str) -> PathBuf {
        self.images.join(format!("{id}.png"))
    }

    pub fn thumb(&self, id: &str) -> PathBuf {
        self.thumbs.join(format!("{id}.png"))
    }

    /// Removes the binary payload owned by a clip. Best effort: a missing file
    /// is not an error, the clip is going away either way.
    pub fn remove_clip_assets(&self, id: &str) {
        let _ = std::fs::remove_file(self.image(id));
        let _ = std::fs::remove_file(self.thumb(id));
    }
}

/// Writes through a temporary file so a crash mid-write cannot corrupt the
/// existing data.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, path)
}
