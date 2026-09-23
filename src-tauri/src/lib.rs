pub mod core;
pub mod ipc;
pub mod models;
pub mod platform;
pub mod telemetry;

use core::{config::ConfigStore, state::AppState};
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    Manager,
};

fn show_window(app: &tauri::AppHandle, label: &str, focus: bool) {
    if let Some(window) = app.get_webview_window(label) {
        let _ = window.show();
        if focus {
            let _ = window.set_focus();
        }
    }
}

fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    let open = MenuItemBuilder::with_id("open", "Open Byte").build(app)?;
    let toggle = MenuItemBuilder::with_id("toggle", "Show Companion").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit Byte").build(app)?;
    let menu = MenuBuilder::new(app)
        .items(&[&open, &toggle, &quit])
        .build()?;

    let mut tray = TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("Byte");

    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }

    tray.on_menu_event(|app, event| match event.id().as_ref() {
        "open" => show_window(app, "main", true),
        "toggle" => show_window(app, "companion", false),
        "quit" => {
            app.state::<AppState>().lifecycle.cancel();
            app.exit(0);
        },
        _ => {}
    })
    .build(app)?;

    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let config_path = app.path().app_config_dir()?.join("config.json");
            let config = ConfigStore::load(config_path)?;
            app.manage(AppState::new(config));

            setup_tray(app)?;

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
            ipc::commands::get_preferences,
            ipc::commands::update_companion_preferences,
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
