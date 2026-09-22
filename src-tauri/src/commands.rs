//! Tauri IPC Commands.
//!
//! Provides frontend access to clips, actions (copy, paste, pin, delete, clear),
//! settings, thumbnails, and app icons.

use std::sync::Arc;

use tauri::{AppHandle, State};

use tauri_plugin_autostart::ManagerExt as AutostartManagerExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use crate::clipboard::write::{self, TextMode};
use crate::history::{ClipView, SourceApp};
use crate::platform::app as platform_app;
use crate::platform::input;
use crate::settings::Settings;
use crate::state::AppState;
use crate::windowing;

#[tauri::command]
pub fn get_clips(state: State<'_, Arc<AppState>>) -> Vec<ClipView> {
    let history = state.history.lock();
    history.views()
}

#[tauri::command]
pub fn copy_clip(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<bool, String> {
    let clip = {
        let history = state.history.lock();
        history.get(&id).cloned()
    };

    let Some(clip) = clip else {
        return Err("Clip not found".into());
    };

    let success = write::write_clip(&clip, &state.paths, TextMode::Rich);
    if success {
        windowing::hide_popover(&app);
    }
    Ok(success)
}

#[tauri::command]
pub fn paste_clip(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<bool, String> {
    let clip = {
        let history = state.history.lock();
        history.get(&id).cloned()
    };

    let Some(clip) = clip else {
        return Err("Clip not found".into());
    };

    let success = write::write_clip(&clip, &state.paths, TextMode::Rich);
    if !success {
        return Ok(false);
    }

    // Hide popover first so target window can receive focus
    windowing::hide_popover(&app);

    // Restore focus to original target window and simulate Ctrl+V
    let target = state.take_foreground_window();
    std::thread::spawn(move || {
        if let Some(hwnd) = target {
            platform_app::focus_window(hwnd);
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
        input::send_paste();
    });

    Ok(true)
}

#[tauri::command]
pub fn paste_plain(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<bool, String> {
    let clip = {
        let history = state.history.lock();
        history.get(&id).cloned()
    };

    let Some(clip) = clip else {
        return Err("Clip not found".into());
    };

    let success = write::write_clip(&clip, &state.paths, TextMode::Plain);
    if !success {
        return Ok(false);
    }

    windowing::hide_popover(&app);

    let target = state.take_foreground_window();
    std::thread::spawn(move || {
        if let Some(hwnd) = target {
            platform_app::focus_window(hwnd);
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
        input::send_paste();
    });

    Ok(true)
}

#[tauri::command]
pub fn toggle_pin(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<Option<bool>, String> {
    let pinned = {
        let mut history = state.history.lock();
        let p = history.toggle_pin(&id);
        let _ = history.save(&state.paths);
        p
    };

    state.notify_clips_changed(&app);
    Ok(pinned)
}

#[tauri::command]
pub fn delete_clip(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<bool, String> {
    let removed = {
        let mut history = state.history.lock();
        let r = history.remove(&id);
        let _ = history.save(&state.paths);
        r
    };

    if let Some(removed_id) = removed {
        state.paths.remove_clip_assets(&removed_id);
        state.notify_clips_changed(&app);
        Ok(true)
    } else {
        Ok(false)
    }
}

#[tauri::command]
pub fn clear_history(app: AppHandle, state: State<'_, Arc<AppState>>) -> Result<bool, String> {
    let evicted = {
        let mut history = state.history.lock();
        let ev = history.clear();
        let _ = history.save(&state.paths);
        ev
    };

    for id in evicted {
        state.paths.remove_clip_assets(&id);
    }

    state.notify_clips_changed(&app);
    Ok(true)
}

#[tauri::command]
pub fn get_settings(state: State<'_, Arc<AppState>>) -> Settings {
    state.settings()
}

#[tauri::command]
pub fn update_settings(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    settings: Settings,
) -> Result<Settings, String> {
    let sanitized = settings.sanitized();
    let prev_shortcut = state.settings().shortcut;
    {
        let mut current = state.settings.lock();
        *current = sanitized.clone();
        let _ = current.save(&state.paths.settings);
    }

    // Update global shortcut if changed
    if prev_shortcut != sanitized.shortcut {
        if let Ok(old_sc) = prev_shortcut.parse::<Shortcut>() {
            let _ = app.global_shortcut().unregister(old_sc);
        }
        if let Ok(new_sc) = sanitized.shortcut.parse::<Shortcut>() {
            let app_handle = app.clone();
            let state_clone = Arc::clone(&state);
            let _ = app.global_shortcut().on_shortcut(new_sc, move |_app, _shortcut, event| {
                if event.state() == ShortcutState::Pressed {
                    windowing::toggle_popover(&app_handle, &state_clone);
                }
            });
        }
    }

    // Update Windows autostart if configured
    let autostart_manager = app.autolaunch();
    if sanitized.launch_at_startup {
        let _ = autostart_manager.enable();
    } else {
        let _ = autostart_manager.disable();
    }

    // Apply limit to history if reduced
    let evicted = {
        let mut history = state.history.lock();
        let ev = history.apply_limit(sanitized.history_limit);
        let _ = history.save(&state.paths);
        ev
    };

    for id in evicted {
        state.paths.remove_clip_assets(&id);
    }

    Ok(sanitized)
}

#[tauri::command]
pub fn get_clip_thumbnail(state: State<'_, Arc<AppState>>, id: String) -> Option<String> {
    let path = state.paths.thumb(&id);
    let bytes = std::fs::read(path).ok()?;
    use base64::Engine;
    Some(base64::engine::general_purpose::STANDARD.encode(bytes))
}

#[tauri::command]
pub fn get_app_icon(
    state: State<'_, Arc<AppState>>,
    exe_name: String,
    exe_path: Option<String>,
) -> Option<String> {
    state.app_icon(exe_path.as_deref(), &exe_name)
}

#[tauri::command]
pub fn get_known_apps(state: State<'_, Arc<AppState>>) -> Vec<SourceApp> {
    let history = state.history.lock();
    history.known_apps()
}

#[tauri::command]
pub fn hide_window(app: AppHandle) {
    windowing::hide_popover(&app);
}

#[tauri::command]
pub fn open_settings(app: AppHandle) {
    windowing::show_settings(&app);
}
