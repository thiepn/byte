use crate::{
    core::{error::ByteError, state::AppState},
    models::{ByteConfig, CompanionPreferences, SystemSnapshot},
};
use tauri::{AppHandle, Manager, State};

#[tauri::command]
pub fn get_snapshot(state: State<'_, AppState>) -> SystemSnapshot {
    state.snapshot.read().unwrap_or_else(|poisoned| poisoned.into_inner()).clone()
}

#[tauri::command]
pub fn get_preferences(state: State<'_, AppState>) -> ByteConfig {
    state.config.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).snapshot()
}

#[tauri::command]
pub fn update_companion_preferences(
    state: State<'_, AppState>,
    preferences: CompanionPreferences,
) -> Result<ByteConfig, ByteError> {
    state.config
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .update_companion(preferences)
}

fn show(app: &AppHandle, label: &str, focus: bool) -> Result<(), ByteError> {
    let window = app
        .get_webview_window(label)
        .ok_or_else(|| ByteError::Window(format!("window '{label}' is unavailable")))?;

    window.show().map_err(|error| ByteError::Window(error.to_string()))?;
    if focus {
        window.set_focus().map_err(|error| ByteError::Window(error.to_string()))?;
    }
    Ok(())
}

#[tauri::command]
pub fn show_main_window(app: AppHandle) -> Result<(), ByteError> { show(&app, "main", true) }

#[tauri::command]
pub fn show_quick_panel(app: AppHandle) -> Result<(), ByteError> { show(&app, "quick-panel", true) }

#[tauri::command]
pub fn hide_quick_panel(app: AppHandle) -> Result<(), ByteError> {
    let window = app.get_webview_window("quick-panel").ok_or_else(|| ByteError::Window("quick panel is unavailable".into()))?;
    window.hide().map_err(|error| ByteError::Window(error.to_string()))
}

#[tauri::command]
pub fn show_companion(app: AppHandle) -> Result<(), ByteError> { show(&app, "companion", false) }

#[tauri::command]
pub fn hide_companion(app: AppHandle) -> Result<(), ByteError> {
    let window = app.get_webview_window("companion").ok_or_else(|| ByteError::Window("companion window is unavailable".into()))?;
    window.hide().map_err(|error| ByteError::Window(error.to_string()))
}

#[tauri::command]
pub fn quit_byte(app: AppHandle, state: State<'_, AppState>) {
    state.lifecycle.cancel();
    app.exit(0);
}
