//! Application-wide state.
//!
//! Owns the in-memory history, settings, resolved storage paths, icon cache,
//! and the foreground window recorded when the popover opens.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicIsize, Ordering};

use parking_lot::Mutex;
use tauri::{AppHandle, Emitter};

use crate::history::History;
use crate::platform::app::RawWindow;
use crate::platform::icon;
use crate::settings::Settings;
use crate::storage::Paths;

pub struct AppState {
    pub paths: Paths,
    pub settings: Mutex<Settings>,
    pub history: Mutex<History>,
    /// Last known foreground window before Clipo's popover took focus.
    foreground_window: AtomicIsize,
    /// Cached base64-encoded PNG icons per executable name.
    app_icons: Mutex<HashMap<String, Option<String>>>,
}

impl AppState {
    pub fn new(paths: Paths) -> Self {
        let settings = Settings::load(&paths.settings);
        let history = History::load(&paths);

        Self {
            paths,
            settings: Mutex::new(settings),
            history: Mutex::new(history),
            foreground_window: AtomicIsize::new(0),
            app_icons: Mutex::new(HashMap::new()),
        }
    }

    /// Snapshot of current settings.
    pub fn settings(&self) -> Settings {
        self.settings.lock().clone()
    }

    /// Records the window that had focus before Clipo was activated.
    pub fn record_foreground_window(&self, window: Option<RawWindow>) {
        let raw = window.unwrap_or(0);
        self.foreground_window.store(raw, Ordering::Relaxed);
    }

    /// Retrieves the recorded foreground window for capture source attribution.
    pub fn foreground_window_for_capture(&self) -> Option<RawWindow> {
        let raw = self.foreground_window.load(Ordering::Relaxed);
        (raw != 0).then_some(raw)
    }

    /// Takes the recorded foreground window for focus restoration (clearing it).
    pub fn take_foreground_window(&self) -> Option<RawWindow> {
        let raw = self.foreground_window.swap(0, Ordering::Relaxed);
        (raw != 0).then_some(raw)
    }

    /// Emits a Tauri event to all windows notifying that clips changed.
    pub fn notify_clips_changed(&self, app: &AppHandle) {
        let _ = app.emit("clips-changed", ());
    }

    /// Retrieves or extracts the base64-encoded PNG icon for an executable.
    pub fn app_icon(&self, exe_path: Option<&str>, exe_name: &str) -> Option<String> {
        let mut cache = self.app_icons.lock();

        if let Some(cached) = cache.get(exe_name) {
            return cached.clone();
        }

        let icon_b64 = exe_path
            .map(PathBuf::from)
            .and_then(|path| icon::executable_icon_png(&path))
            .map(|bytes| {
                use base64::Engine;
                base64::engine::general_purpose::STANDARD.encode(bytes)
            });

        cache.insert(exe_name.to_string(), icon_b64.clone());
        icon_b64
    }
}
