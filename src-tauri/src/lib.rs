//! Clipo - Windows Clipboard Manager Library

pub mod clipboard;
pub mod commands;
pub mod history;
pub mod intelligence;
pub mod platform;
pub mod settings;
pub mod state;
pub mod storage;
pub mod tray;
pub mod windowing;

use std::sync::Arc;
use tauri::Manager;
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use state::AppState;
use storage::Paths;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let handle = app.handle();
            let paths = Paths::resolve(&handle).expect("failed to resolve app paths");
            let state = Arc::new(AppState::new(paths));
            app.manage(Arc::clone(&state));

            // Start clipboard monitor background thread
            let _monitor = clipboard::monitor::start(handle.clone(), Arc::clone(&state));

            // Setup system tray
            let _ = tray::setup(&handle);

            // Register global shortcut
            let shortcut_str = state.settings().shortcut;
            if let Ok(shortcut) = shortcut_str.parse::<Shortcut>() {
                let app_handle = handle.clone();
                let state_clone = Arc::clone(&state);
                let _ = app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        windowing::toggle_popover(&app_handle, &state_clone);
                    }
                });
            }

            // Setup popover window events (hide on focus lost)
            if let Some(popover) = app.get_webview_window(windowing::POPOVER_LABEL) {
                windowing::setup_popover_events(&popover);
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_clips,
            commands::copy_clip,
            commands::paste_clip,
            commands::paste_plain,
            commands::toggle_pin,
            commands::delete_clip,
            commands::clear_history,
            commands::get_settings,
            commands::update_settings,
            commands::get_clip_thumbnail,
            commands::get_app_icon,
            commands::get_known_apps,
            commands::hide_window,
            commands::open_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
