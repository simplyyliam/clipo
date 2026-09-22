//! System tray icon and menu setup.

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

use crate::state::AppState;
use crate::windowing;

pub fn setup(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let show_item = MenuItem::with_id(app, "show_clipboard", "Open Clipboard", true, None::<&str>)?;
    let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let clear_item = MenuItem::with_id(app, "clear_history", "Clear History", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit Clipo", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[&show_item, &settings_item, &clear_item, &quit_item],
    )?;

    let mut builder = TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("Clipo - Clipboard Manager");

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    builder
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show_clipboard" => {
                if let Some(state) = app.try_state::<std::sync::Arc<AppState>>() {
                    windowing::show_popover(app, &state);
                }
            }
            "settings" => {
                windowing::show_settings(app);
            }
            "clear_history" => {
                if let Some(state) = app.try_state::<std::sync::Arc<AppState>>() {
                    let evicted = {
                        let mut history = state.history.lock();
                        history.clear()
                    };
                    for id in evicted {
                        state.paths.remove_clip_assets(&id);
                    }
                    if let Ok(paths) = crate::storage::Paths::resolve(app) {
                        let history = state.history.lock();
                        let _ = history.save(&paths);
                    }
                    state.notify_clips_changed(app);
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(state) = app.try_state::<std::sync::Arc<AppState>>() {
                    windowing::toggle_popover(app, &state);
                }
            }
        })
        .build(app)?;

    Ok(())
}
