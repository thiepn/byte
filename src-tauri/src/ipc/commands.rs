use crate::{
    core::{
        activity::ActivitySnapshot,
        error::ByteError,
        security::{
            normalize_and_validate_app_preferences, validate_companion_preferences,
            validate_snooze,
        },
        state::AppState,
    },
    models::{
        AppDiagnosticsSnapshot, AppPreferences, ByteConfig, CollectionDiscoveryKind,
        CollectionSnapshot, CompanionPreferences, CompanionSize, DesktopAwarenessSnapshot,
        DisplayMode, EdgeAnchor, NotificationPermissionState, RecommendedActionKind,
        SystemSnapshot, WindowShellState,
    },
    platform::windows::{actions, startup, windowing},
};
use tauri::{plugin::PermissionState, AppHandle, Emitter, State};
use tauri_plugin_notification::NotificationExt;

#[tauri::command]
pub fn get_snapshot(state: State<'_, AppState>) -> SystemSnapshot {
    state.snapshot()
}

#[tauri::command]
pub fn get_activity_history(state: State<'_, AppState>) -> ActivitySnapshot {
    state.activity_snapshot()
}

#[tauri::command]
pub fn inspect_apps(state: State<'_, AppState>) -> AppDiagnosticsSnapshot {
    state.inspect_apps()
}

#[tauri::command]
pub fn get_collection(state: State<'_, AppState>) -> Result<CollectionSnapshot, ByteError> {
    state.collection_snapshot()
}

#[tauri::command]
pub fn record_collection_discovery(
    app: AppHandle,
    state: State<'_, AppState>,
    discovery: CollectionDiscoveryKind,
) -> Result<CollectionSnapshot, ByteError> {
    let (snapshot, changed) = state.record_collection_discovery(discovery)?;
    if changed {
        // A rare idle discovery should be allowed to finish naturally. The
        // Studio refreshes immediately; companion celebrations are reserved
        // for non-discovery milestone unlocks emitted by input/telemetry.
        let _ = app.emit_to("main", "byte://collection-updated", snapshot.clone());
    }
    Ok(snapshot)
}

#[tauri::command]
pub fn get_preferences(state: State<'_, AppState>) -> ByteConfig {
    state
        .config
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .snapshot()
}

#[tauri::command]
pub fn update_app_preferences(
    app: AppHandle,
    state: State<'_, AppState>,
    mut preferences: AppPreferences,
) -> Result<ByteConfig, ByteError> {
    normalize_and_validate_app_preferences(&mut preferences)?;
    validate_snooze(
        preferences.notification_snoozed_until_epoch_ms,
        crate::models::now_epoch_ms(),
    )?;

    let previous = state.app_preferences();
    if previous.launch_at_startup != preferences.launch_at_startup {
        startup::apply(preferences.launch_at_startup)?;
    }

    let config = state
        .config
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .update_app(preferences.clone())?;

    if !preferences.system_monitoring_enabled {
        state.set_snapshot_unavailable();
    }

    let _ = windowing::apply_capture_affinity(&app, preferences.exclude_from_capture);
    state.lifecycle.wake_waiters();

    let _ = app.emit("byte://app-preferences-changed", preferences);
    Ok(config)
}

#[tauri::command]
pub fn get_desktop_awareness(state: State<'_, AppState>) -> DesktopAwarenessSnapshot {
    state.desktop_awareness()
}

#[tauri::command]
pub fn get_notification_permission(
    app: AppHandle,
) -> Result<NotificationPermissionState, ByteError> {
    Ok(map_notification_permission(
        app.notification()
            .permission_state()
            .map_err(|error| ByteError::Window(error.to_string()))?,
    ))
}

#[tauri::command]
pub fn request_notification_permission(
    app: AppHandle,
) -> Result<NotificationPermissionState, ByteError> {
    Ok(map_notification_permission(
        app.notification()
            .request_permission()
            .map_err(|error| ByteError::Window(error.to_string()))?,
    ))
}

fn map_notification_permission(state: PermissionState) -> NotificationPermissionState {
    match state {
        PermissionState::Granted => NotificationPermissionState::Granted,
        PermissionState::Denied => NotificationPermissionState::Denied,
        PermissionState::Prompt | PermissionState::PromptWithRationale => {
            NotificationPermissionState::Prompt
        }
    }
}

#[tauri::command]
pub fn clear_activity_history(state: State<'_, AppState>) -> Result<ActivitySnapshot, ByteError> {
    state.clear_activity()
}

#[tauri::command]
pub fn open_release_page() -> Result<(), ByteError> {
    actions::open_release_page()
}

#[tauri::command]
pub fn update_companion_preferences(
    app: AppHandle,
    state: State<'_, AppState>,
    preferences: CompanionPreferences,
) -> Result<ByteConfig, ByteError> {
    validate_companion_preferences(&preferences)?;
    state.validate_collection_preferences(&preferences)?;

    let config = state
        .config
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .update_companion(preferences)?;

    windowing::apply_companion_layout(&app, &config.companion)?;
    let _ = app.emit_to(
        "companion",
        "byte://companion-preferences-changed",
        config.companion.clone(),
    );
    Ok(config)
}

#[tauri::command]
pub fn execute_recommended_action(
    app: AppHandle,
    action: RecommendedActionKind,
) -> Result<(), ByteError> {
    actions::execute(&app, action)?;
    let _ = windowing::hide_quick_panel(&app);
    Ok(())
}

#[tauri::command]
pub fn get_window_shell_state(app: AppHandle) -> WindowShellState {
    windowing::shell_state(&app)
}

#[tauri::command]
pub fn set_display_mode(app: AppHandle, mode: DisplayMode) -> Result<ByteConfig, ByteError> {
    windowing::set_display_mode(&app, mode)
}

#[tauri::command]
pub fn set_companion_size(app: AppHandle, size: CompanionSize) -> Result<ByteConfig, ByteError> {
    windowing::set_companion_size(&app, size)
}

#[tauri::command]
pub fn set_edge_anchor(app: AppHandle, anchor: EdgeAnchor) -> Result<ByteConfig, ByteError> {
    windowing::set_edge_anchor(&app, anchor)
}

#[tauri::command]
pub fn begin_move_mode(app: AppHandle) -> Result<WindowShellState, ByteError> {
    windowing::begin_move_mode(&app)
}

#[tauri::command]
pub fn drag_companion(app: AppHandle) -> Result<WindowShellState, ByteError> {
    windowing::drag_companion(&app)
}

#[tauri::command]
pub fn finish_move_mode(app: AppHandle) -> Result<WindowShellState, ByteError> {
    windowing::finish_move_mode(&app)
}

#[tauri::command]
pub fn set_companion_click_through(
    app: AppHandle,
    enabled: bool,
) -> Result<WindowShellState, ByteError> {
    windowing::set_click_through(&app, enabled)
}

#[tauri::command]
pub fn show_main_window(app: AppHandle) -> Result<(), ByteError> {
    windowing::show_main_window(&app)
}

#[tauri::command]
pub fn show_quick_panel(app: AppHandle) -> Result<(), ByteError> {
    windowing::show_quick_panel(&app)
}

#[tauri::command]
pub fn hide_quick_panel(app: AppHandle) -> Result<(), ByteError> {
    windowing::hide_quick_panel(&app)
}

#[tauri::command]
pub fn show_companion(app: AppHandle) -> Result<(), ByteError> {
    windowing::show_companion(&app)
}

#[tauri::command]
pub fn hide_companion(app: AppHandle) -> Result<(), ByteError> {
    windowing::hide_companion(&app)
}

#[tauri::command]
pub fn quit_byte(app: AppHandle, state: State<'_, AppState>) {
    state.stop_background_workers();
    app.exit(0);
}
