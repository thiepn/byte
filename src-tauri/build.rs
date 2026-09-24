const COMMANDS: &[&str] = &[
    "get_snapshot",
    "get_activity_history",
    "inspect_apps",
    "get_collection",
    "record_collection_discovery",
    "get_preferences",
    "update_app_preferences",
    "get_desktop_awareness",
    "get_notification_permission",
    "request_notification_permission",
    "clear_activity_history",
    "open_release_page",
    "update_companion_preferences",
    "execute_recommended_action",
    "get_window_shell_state",
    "set_display_mode",
    "set_companion_size",
    "set_edge_anchor",
    "begin_move_mode",
    "drag_companion",
    "finish_move_mode",
    "set_companion_click_through",
    "show_main_window",
    "show_quick_panel",
    "hide_quick_panel",
    "show_companion",
    "hide_companion",
    "quit_byte",
];

fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(COMMANDS)),
    )
    .expect("failed to build Tauri command permissions");
}
