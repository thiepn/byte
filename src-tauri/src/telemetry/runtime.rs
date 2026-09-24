use super::engine::TelemetryEngine;
use crate::core::{diagnostics::DiagnosticEngine, error::ByteError, state::AppState};
use std::{thread, time::Duration};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

#[cfg(target_os = "windows")]
use super::windows::WindowsTelemetrySource;

pub const SAMPLE_INTERVAL: Duration = Duration::from_millis(1_500);

pub fn start(app: AppHandle) -> Result<(), ByteError> {
    let worker_app = app.clone();
    let handle = thread::Builder::new()
        .name("byte-telemetry".into())
        .spawn(move || run_worker(worker_app))
        .map_err(|error| ByteError::Io(error.to_string()))?;

    app.state::<AppState>().install_telemetry_worker(handle);
    Ok(())
}

#[cfg(target_os = "windows")]
fn run_worker(app: AppHandle) {
    let mut telemetry = TelemetryEngine::new(WindowsTelemetrySource::new());
    let mut diagnostics = DiagnosticEngine::new();
    let mut last_notified_issue: Option<String> = None;

    loop {
        let state = app.state::<AppState>();
        if !state.lifecycle.wait_until_sampling_allowed() {
            break;
        }

        let app_preferences = state.app_preferences();
        if !app_preferences.system_monitoring_enabled {
            state.set_snapshot_unavailable();
        } else if let Ok(snapshot) = telemetry.sample_snapshot() {
            let evaluated = diagnostics.evaluate(snapshot);

            if let Ok(Some(collection)) = state.observe_collection_system(&evaluated) {
                let _ = app.emit_to("main", "byte://collection-updated", collection.clone());
                let _ = app.emit_to("companion", "byte://collection-updated", collection);
            }

            if evaluated.overall_status == crate::models::SystemStatus::NeedsAttention {
                if let Some(issue) = evaluated.primary_issue.as_ref() {
                    if app_preferences.notifications_enabled
                        && last_notified_issue.as_deref() != Some(issue.id.as_str())
                    {
                        let _ = app
                            .notification()
                            .builder()
                            .title("Byte needs attention")
                            .body(&issue.headline)
                            .show();
                    }
                    last_notified_issue = Some(issue.id.clone());
                }
            } else {
                last_notified_issue = None;
            }

            state.replace_snapshot(evaluated);
        }

        if !state.lifecycle.wait_for_change_or_timeout(SAMPLE_INTERVAL) {
            break;
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn run_worker(app: AppHandle) {
    app.state::<AppState>().lifecycle.cancel();
}
