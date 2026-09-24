pub mod core;
pub mod ipc;
pub mod models;
pub mod platform;
pub mod telemetry;

use core::{
    activity::ActivityStore, collection::CollectionStore, config::ConfigStore,
    smart_notifications::SmartNotificationEngine, state::AppState,
};
use models::DisplayMode;
use platform::windows::{fullscreen, input::InputRuntime, startup, windowing};
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    let open = MenuItemBuilder::with_id("open", "Open Byte").build(app)?;
    let toggle = MenuItemBuilder::with_id("toggle", "Show / Hide Companion").build(app)?;
    let move_byte = MenuItemBuilder::with_id("move", "Move Byte").build(app)?;
    let click_through =
        MenuItemBuilder::with_id("click-through", "Toggle Click Through").build(app)?;

    let habitat = MenuItemBuilder::with_id("mode-habitat", "Habitat").build(app)?;
    let perch = MenuItemBuilder::with_id("mode-perch", "Perch").build(app)?;
    let mini = MenuItemBuilder::with_id("mode-mini", "Mini").build(app)?;
    let edge = MenuItemBuilder::with_id("mode-edge", "Edge").build(app)?;
    let tray_only = MenuItemBuilder::with_id("mode-tray", "Tray Only").build(app)?;
    let display_mode = SubmenuBuilder::new(app, "Display Mode")
        .items(&[&habitat, &perch, &mini, &edge, &tray_only])
        .build()?;

    let quit = MenuItemBuilder::with_id("quit", "Quit Byte").build(app)?;
    let menu = MenuBuilder::new(app)
        .items(&[
            &open,
            &toggle,
            &move_byte,
            &click_through,
            &display_mode,
            &quit,
        ])
        .build()?;

    let mut tray = TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("Byte");

    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }

    tray.on_menu_event(|app, event| match event.id().as_ref() {
        "open" => {
            let _ = windowing::show_main_window(app);
        }
        "toggle" => {
            let _ = windowing::toggle_companion(app);
        }
        "move" => {
            let _ = windowing::begin_move_mode(app);
        }
        "click-through" => {
            let _ = windowing::toggle_click_through(app);
        }
        "mode-habitat" => {
            let _ = windowing::set_display_mode(app, DisplayMode::Habitat);
        }
        "mode-perch" => {
            let _ = windowing::set_display_mode(app, DisplayMode::Perch);
        }
        "mode-mini" => {
            let _ = windowing::set_display_mode(app, DisplayMode::Mini);
        }
        "mode-edge" => {
            let _ = windowing::set_display_mode(app, DisplayMode::Edge);
        }
        "mode-tray" => {
            let _ = windowing::set_display_mode(app, DisplayMode::Tray);
        }
        "quit" => {
            app.state::<AppState>().stop_background_workers();
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
            let _ = windowing::show_quick_panel(tray.app_handle());
        }
    })
    .build(app)?;

    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let app_config_dir = app.path().app_config_dir()?;
            let config = ConfigStore::load(app_config_dir.join("config.json"))?;
            let activity = ActivityStore::load(app_config_dir.join("activity.json"))?;
            let collection = CollectionStore::load(app_config_dir.join("collection.json"))?;
            let smart_notifications =
                SmartNotificationEngine::load(app_config_dir.join("notifications.json"))?;
            let initial_app_preferences = config.snapshot().app;
            app.manage(AppState::new(
                config,
                activity,
                collection,
                smart_notifications,
            ));

            // Desktop awareness, capture exclusion, telemetry, and global input
            // are optional integrations. A Windows/API failure must not make
            // Byte itself fail to launch.
            let _ = fullscreen::start(app.handle().clone());
            let _ = windowing::apply_capture_affinity(
                app.handle(),
                initial_app_preferences.exclude_from_capture,
            );
            windowing::initialize(app.handle())?;
            let _ = startup::apply(initial_app_preferences.launch_at_startup);

            if telemetry::runtime::start(app.handle().clone()).is_err() {
                app.state::<AppState>().set_snapshot_unavailable();
            }

            if let Ok(input_runtime) = InputRuntime::start(app.handle().clone()) {
                app.state::<AppState>().install_input_runtime(input_runtime);
            }

            setup_tray(app)?;

            if !initial_app_preferences.onboarding_completed {
                let _ = windowing::show_main_window(app.handle());
            }

            if let Some(main_window) = app.get_webview_window("main") {
                let window_to_hide = main_window.clone();
                main_window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = window_to_hide.hide();
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::commands::get_snapshot,
            ipc::commands::get_activity_history,
            ipc::commands::inspect_apps,
            ipc::commands::get_collection,
            ipc::commands::record_collection_discovery,
            ipc::commands::get_preferences,
            ipc::commands::update_app_preferences,
            ipc::commands::get_desktop_awareness,
            ipc::commands::get_notification_permission,
            ipc::commands::request_notification_permission,
            ipc::commands::clear_activity_history,
            ipc::commands::open_release_page,
            ipc::commands::update_companion_preferences,
            ipc::commands::execute_recommended_action,
            ipc::commands::get_window_shell_state,
            ipc::commands::set_display_mode,
            ipc::commands::set_companion_size,
            ipc::commands::set_edge_anchor,
            ipc::commands::begin_move_mode,
            ipc::commands::drag_companion,
            ipc::commands::finish_move_mode,
            ipc::commands::set_companion_click_through,
            ipc::commands::show_main_window,
            ipc::commands::show_quick_panel,
            ipc::commands::hide_quick_panel,
            ipc::commands::show_companion,
            ipc::commands::hide_companion,
            ipc::commands::quit_byte
        ])
        .run(tauri::generate_context!())
        .expect("Byte failed to start");
}
