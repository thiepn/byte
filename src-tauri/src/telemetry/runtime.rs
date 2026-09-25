use super::engine::TelemetryEngine;
use crate::{
    core::{
        diagnostics::DiagnosticEngine,
        error::ByteError,
        lifecycle::{background_work_suspended, LifecycleState},
        state::AppState,
    },
    models::SystemStatus,
};
use std::{
    thread,
    time::{Duration, Instant},
};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

#[cfg(target_os = "windows")]
use super::windows::WindowsTelemetrySource;

const ALERT_SAMPLE_INTERVAL: Duration = Duration::from_millis(1_500);
const BUSY_SAMPLE_INTERVAL: Duration = Duration::from_millis(2_500);
const CALM_SAMPLE_INTERVAL: Duration = Duration::from_secs(5);
const FULLSCREEN_SAMPLE_INTERVAL: Duration = Duration::from_secs(8);
const MONITORING_DISABLED_INTERVAL: Duration = Duration::from_secs(30);
const SUSPEND_GAP_RESET_INTERVAL: Duration = Duration::from_secs(12);
const SAMPLE_FAILURE_UNAVAILABLE_THRESHOLD: u8 = 2;

pub fn start(app: AppHandle) -> Result<(), ByteError> {
    let worker_app = app.clone();
    let handle = thread::Builder::new()
        .name("byte-telemetry".into())
        .spawn(move || run_worker(worker_app))
        .map_err(|error| ByteError::Io(error.to_string()))?;

    app.state::<AppState>().install_telemetry_worker(handle);
    Ok(())
}

fn scheduling_gap_requires_reset(elapsed: Duration) -> bool {
    elapsed >= SUSPEND_GAP_RESET_INTERVAL
}

fn sampling_interval(
    lifecycle: LifecycleState,
    status: Option<SystemStatus>,
    monitoring_enabled: bool,
) -> Duration {
    if !monitoring_enabled {
        return MONITORING_DISABLED_INTERVAL;
    }

    if lifecycle == LifecycleState::FullscreenReduced {
        return FULLSCREEN_SAMPLE_INTERVAL;
    }

    match status {
        Some(SystemStatus::NeedsAttention | SystemStatus::Stressed) => ALERT_SAMPLE_INTERVAL,
        Some(SystemStatus::Busy) => BUSY_SAMPLE_INTERVAL,
        Some(SystemStatus::Calm) | None => CALM_SAMPLE_INTERVAL,
    }
}

#[cfg(target_os = "windows")]
fn run_worker(app: AppHandle) {
    let mut telemetry = TelemetryEngine::new(WindowsTelemetrySource::new());
    let mut diagnostics = DiagnosticEngine::new();
    let mut last_sample_at: Option<Instant> = None;
    let mut consecutive_sample_failures = 0_u8;

    loop {
        let state = app.state::<AppState>();
        let lifecycle_before_wait = state.lifecycle.current();
        if !state.lifecycle.wait_until_sampling_allowed() {
            break;
        }
        let lifecycle_after_wait = state.lifecycle.current();

        let resumed_from_suspension = background_work_suspended(lifecycle_before_wait)
            && !background_work_suspended(lifecycle_after_wait);
        let resumed_from_long_gap = last_sample_at
            .map(|last| scheduling_gap_requires_reset(last.elapsed()))
            .unwrap_or(false);

        if resumed_from_suspension || resumed_from_long_gap {
            telemetry = TelemetryEngine::new(WindowsTelemetrySource::new());
            diagnostics = DiagnosticEngine::new();
            state.set_snapshot_unavailable();
            last_sample_at = None;
        }

        let app_preferences = state.app_preferences();
        let interval = if !app_preferences.system_monitoring_enabled {
            state.set_snapshot_unavailable();
            sampling_interval(lifecycle_after_wait, None, false)
        } else if let Ok(snapshot) = telemetry.sample_snapshot() {
            consecutive_sample_failures = 0;
            last_sample_at = Some(Instant::now());
            let evaluated = diagnostics.evaluate(snapshot);

            if let Ok(Some(collection)) = state.observe_collection_system(&evaluated) {
                let _ = app.emit_to("main", "byte://collection-updated", collection.clone());
                let _ = app.emit_to("companion", "byte://collection-updated", collection);
            }

            let issues = diagnostics.active_issues();
            if !state.is_visibility_suppressed() {
                if let Some(notification) = state.next_smart_notification(
                    evaluated.timestamp_epoch_ms,
                    issues,
                    &app_preferences,
                ) {
                    let permission_granted = app
                        .notification()
                        .permission_state()
                        .map(|state| state == tauri::plugin::PermissionState::Granted)
                        .unwrap_or(false);

                    if permission_granted {
                        let builder = app
                            .notification()
                            .builder()
                            .title(&notification.title)
                            .body(&notification.body);
                        let builder = if app_preferences.sound_enabled {
                            builder
                        } else {
                            builder.silent()
                        };

                        if builder.show().is_ok() {
                            let _ = state.mark_smart_notification_sent(
                                &notification,
                                evaluated.timestamp_epoch_ms,
                            );
                        }
                    }
                }
            }

            let status = evaluated.overall_status;
            let event_snapshot = evaluated.clone();
            state.replace_snapshot(evaluated);
            let _ = app.emit_to("companion", "byte://snapshot-updated", event_snapshot);
            sampling_interval(lifecycle_after_wait, Some(status), true)
        } else {
            consecutive_sample_failures = consecutive_sample_failures.saturating_add(1);

            if consecutive_sample_failures == SAMPLE_FAILURE_UNAVAILABLE_THRESHOLD {
                diagnostics = DiagnosticEngine::new();
                telemetry = TelemetryEngine::new(WindowsTelemetrySource::new());
                last_sample_at = None;

                let unavailable = crate::models::SystemSnapshot::unavailable();
                state.replace_snapshot(unavailable.clone());
                let _ = app.emit_to("companion", "byte://snapshot-updated", unavailable);
            }

            CALM_SAMPLE_INTERVAL
        };

        if !state.lifecycle.wait_for_change_or_timeout(interval) {
            break;
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn run_worker(app: AppHandle) {
    app.state::<AppState>().lifecycle.cancel();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn long_scheduler_gaps_are_treated_as_resume_boundaries() {
        assert!(!scheduling_gap_requires_reset(Duration::from_secs(8)));
        assert!(scheduling_gap_requires_reset(Duration::from_secs(12)));
        assert!(scheduling_gap_requires_reset(Duration::from_secs(60)));
    }

    #[test]
    fn repeated_sampling_failures_cross_unavailable_threshold() {
        assert!(1 < SAMPLE_FAILURE_UNAVAILABLE_THRESHOLD);
        assert_eq!(SAMPLE_FAILURE_UNAVAILABLE_THRESHOLD, 2);
    }

    #[test]
    fn calm_sampling_is_slow_and_pressure_sampling_is_fast() {
        assert_eq!(
            sampling_interval(LifecycleState::Active, Some(SystemStatus::Calm), true),
            CALM_SAMPLE_INTERVAL
        );
        assert_eq!(
            sampling_interval(LifecycleState::Active, Some(SystemStatus::Busy), true),
            BUSY_SAMPLE_INTERVAL
        );
        assert_eq!(
            sampling_interval(
                LifecycleState::Active,
                Some(SystemStatus::NeedsAttention),
                true
            ),
            ALERT_SAMPLE_INTERVAL
        );
    }

    #[test]
    fn fullscreen_and_disabled_monitoring_use_low_power_cadence() {
        assert_eq!(
            sampling_interval(
                LifecycleState::FullscreenReduced,
                Some(SystemStatus::NeedsAttention),
                true
            ),
            FULLSCREEN_SAMPLE_INTERVAL
        );
        assert_eq!(
            sampling_interval(LifecycleState::Active, Some(SystemStatus::Calm), false),
            MONITORING_DISABLED_INTERVAL
        );
    }
}
