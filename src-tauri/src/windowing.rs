//! Window management: popover window (transient clipboard history) and settings window.

use std::sync::Arc;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

use crate::platform::app as platform_app;
use crate::state::AppState;

pub const POPOVER_LABEL: &str = "main";
pub const SETTINGS_LABEL: &str = "settings";

/// Toggles the clipboard popover window.
pub fn toggle_popover(app: &AppHandle, state: &Arc<AppState>) {
    if let Some(window) = app.get_webview_window(POPOVER_LABEL) {
        if window.is_visible().unwrap_or(false) {
            hide_popover(app);
        } else {
            show_popover(app, state);
        }
    }
}

/// Shows the popover window near the current cursor / active context and captures the foreground window.
pub fn show_popover(app: &AppHandle, state: &Arc<AppState>) {
    // Record foreground window before showing popover so we can identify source app or paste back.
    let fg = platform_app::foreground_window();
    state.record_foreground_window(fg);

    if let Some(window) = app.get_webview_window(POPOVER_LABEL) {
        // Position window near cursor if desired or center
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Hides the popover window.
pub fn hide_popover(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(POPOVER_LABEL) {
        let _ = window.hide();
    }
}

/// Opens or focuses the dedicated Settings window.
pub fn show_settings(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(SETTINGS_LABEL) {
        let _ = window.show();
        let _ = window.set_focus();
    } else {
        let builder = WebviewWindowBuilder::new(
            app,
            SETTINGS_LABEL,
            WebviewUrl::App("settings".into()),
        )
        .title("Clipo Settings")
        .inner_size(520.0, 580.0)
        .resizable(false)
        .maximizable(false)
        .minimizable(true);

        if let Ok(window) = builder.build() {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

/// Sets up focus-lost handler on popover to auto-hide when clicking outside.
pub fn setup_popover_events(window: &WebviewWindow) {
    let window_clone = window.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::Focused(focused) = event {
            if !focused {
                let _ = window_clone.hide();
            }
        }
    });
}
