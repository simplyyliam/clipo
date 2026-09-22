//! Application settings: the single source of configuration for every MVP
//! capability that is user configurable (shortcut, history limit, exclusions,
//! sensitive filtering, theme, startup).
//!
//! Settings are plain data. Applying them to the OS (shortcut registration,
//! autostart) happens in `crate::windowing` / `crate::commands` so this module
//! stays free of side effects.

use std::path::Path;

use serde::{Deserialize, Serialize};

/// Theme preference. The frontend resolves `System` against the OS theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    System,
    Light,
    Dark,
}

impl Default for Theme {
    fn default() -> Self {
        Theme::System
    }
}

pub const DEFAULT_SHORTCUT: &str = "Ctrl+Shift+V";
pub const DEFAULT_HISTORY_LIMIT: usize = 200;
pub const MIN_HISTORY_LIMIT: usize = 10;
pub const MAX_HISTORY_LIMIT: usize = 1000;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    /// Global shortcut that opens the clipboard popover, in Tauri accelerator
    /// syntax (e.g. `Ctrl+Shift+V`).
    pub shortcut: String,
    /// Maximum number of unpinned clips kept in history.
    pub history_limit: usize,
    /// Executable file names (lowercase, e.g. `keepass.exe`) whose clipboard
    /// content never enters history.
    pub excluded_apps: Vec<String>,
    /// When enabled, clips that look sensitive are not stored.
    pub block_sensitive: bool,
    pub theme: Theme,
    pub launch_at_startup: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            shortcut: DEFAULT_SHORTCUT.to_string(),
            history_limit: DEFAULT_HISTORY_LIMIT,
            excluded_apps: Vec::new(),
            block_sensitive: true,
            theme: Theme::default(),
            launch_at_startup: false,
        }
    }
}

impl Settings {
    /// Clamps values that would otherwise break invariants elsewhere
    /// (unbounded history, empty shortcut, duplicated exclusions).
    pub fn sanitized(mut self) -> Self {
        self.history_limit = self
            .history_limit
            .clamp(MIN_HISTORY_LIMIT, MAX_HISTORY_LIMIT);

        if self.shortcut.trim().is_empty() {
            self.shortcut = DEFAULT_SHORTCUT.to_string();
        } else {
            self.shortcut = self.shortcut.trim().to_string();
        }

        let mut apps: Vec<String> = self
            .excluded_apps
            .into_iter()
            .map(|app| app.trim().to_lowercase())
            .filter(|app| !app.is_empty())
            .collect();
        apps.sort();
        apps.dedup();
        self.excluded_apps = apps;

        self
    }

    /// True when clipboard content coming from `exe_name` must be ignored.
    pub fn is_app_excluded(&self, exe_name: &str) -> bool {
        let needle = exe_name.to_lowercase();
        self.excluded_apps.iter().any(|app| app == &needle)
    }

    pub fn load(path: &Path) -> Self {
        match std::fs::read(path) {
            Ok(bytes) => serde_json::from_slice::<Settings>(&bytes)
                .map(Settings::sanitized)
                .unwrap_or_else(|err| {
                    log::warn!("settings file unreadable, using defaults: {err}");
                    Settings::default()
                }),
            Err(_) => Settings::default(),
        }
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let bytes = serde_json::to_vec_pretty(self)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;
        crate::storage::write_atomic(path, &bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_clamps_and_normalizes() {
        let settings = Settings {
            history_limit: 99999,
            shortcut: "  ".into(),
            excluded_apps: vec!["KeePass.exe".into(), "keepass.exe".into(), " ".into()],
            ..Settings::default()
        }
        .sanitized();

        assert_eq!(settings.history_limit, MAX_HISTORY_LIMIT);
        assert_eq!(settings.shortcut, DEFAULT_SHORTCUT);
        assert_eq!(settings.excluded_apps, vec!["keepass.exe".to_string()]);
        assert!(settings.is_app_excluded("KEEPASS.EXE"));
    }
}
