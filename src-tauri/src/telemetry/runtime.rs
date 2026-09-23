use super::engine::TelemetryEngine;
use crate::{
    core::{diagnostics::DiagnosticEngine, error::ByteError, state::AppState},
};
use std::{thread, time::Duration};
use tauri::{AppHandle, Manager};

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

    loop {
        let state = app.state::<AppState>();
        if !state.lifecycle.wait_until_sampling_allowed() {
            break;
        }

        if let Ok(snapshot) = telemetry.sample_snapshot() {
            state.replace_snapshot(diagnostics.evaluate(snapshot));
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
