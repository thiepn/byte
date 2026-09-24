use crate::{
    core::{error::ByteError, lifecycle::LifecycleState, state::AppState},
    models::DisplayMode,
    platform::windows::windowing,
};
use std::{mem::size_of, thread, time::Duration};
use tauri::{AppHandle, Manager};
use windows_sys::Win32::{
    Foundation::RECT,
    Graphics::Gdi::{
        GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    },
    System::Threading::GetCurrentProcessId,
    UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowRect, GetWindowThreadProcessId,
    },
};

const POLL_INTERVAL: Duration = Duration::from_millis(750);
const RECT_TOLERANCE_PX: i32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FullscreenObservation {
    pub active: bool,
}

impl FullscreenObservation {
    pub const fn inactive() -> Self {
        Self { active: false }
    }
}

pub fn start(app: AppHandle) -> Result<(), ByteError> {
    let worker_app = app.clone();
    let worker = thread::Builder::new()
        .name("byte-fullscreen-watch".into())
        .spawn(move || run(worker_app))
        .map_err(|error| ByteError::Io(error.to_string()))?;

    app.state::<AppState>().install_fullscreen_worker(worker);
    Ok(())
}

fn run(app: AppHandle) {
    let mut last = FullscreenObservation::inactive();
    let mut hidden_by_fullscreen = false;

    loop {
        let state = app.state::<AppState>();
        if state.lifecycle.is_cancelled() {
            break;
        }

        let current = observe();
        let config = state
            .config
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .snapshot();

        if current.active != last.active {
            state.lifecycle.transition(if current.active {
                LifecycleState::FullscreenReduced
            } else {
                LifecycleState::Active
            });
            last = current;
        }

        let should_hide = current.active
            && config.app.hide_in_fullscreen
            && config.companion.display_mode != DisplayMode::Tray;

        if should_hide && !hidden_by_fullscreen {
            if windowing::is_companion_visible(&app).unwrap_or(false) {
                let _ = windowing::hide_quick_panel(&app);
                if windowing::hide_companion(&app).is_ok() {
                    hidden_by_fullscreen = true;
                }
            }
        } else if !should_hide && hidden_by_fullscreen {
            let _ = windowing::show_companion(&app);
            hidden_by_fullscreen = false;
        }

        thread::sleep(POLL_INTERVAL);
    }
}

fn observe() -> FullscreenObservation {
    // SAFETY: all handles and output buffers are checked before use, and no
    // pointer from the Win32 calls is retained beyond this function.
    unsafe {
        let window = GetForegroundWindow();
        if window.is_null() {
            return FullscreenObservation::inactive();
        }

        let mut process_id = 0u32;
        GetWindowThreadProcessId(window, &mut process_id);
        if process_id == GetCurrentProcessId() {
            return FullscreenObservation::inactive();
        }

        let mut window_rect: RECT = std::mem::zeroed();
        if GetWindowRect(window, &mut window_rect) == 0 {
            return FullscreenObservation::inactive();
        }

        let monitor = MonitorFromWindow(window, MONITOR_DEFAULTTONEAREST);
        if monitor.is_null() {
            return FullscreenObservation::inactive();
        }

        let mut info: MONITORINFO = std::mem::zeroed();
        info.cbSize = size_of::<MONITORINFO>() as u32;
        if GetMonitorInfoW(monitor, &mut info) == 0 {
            return FullscreenObservation::inactive();
        }

        FullscreenObservation {
            active: rect_matches(window_rect, info.rcMonitor),
        }
    }
}

fn rect_matches(window: RECT, monitor: RECT) -> bool {
    close(window.left, monitor.left)
        && close(window.top, monitor.top)
        && close(window.right, monitor.right)
        && close(window.bottom, monitor.bottom)
}

fn close(left: i32, right: i32) -> bool {
    (left - right).abs() <= RECT_TOLERANCE_PX
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_monitor_rect_is_fullscreen() {
        let monitor = RECT {
            left: 0,
            top: 0,
            right: 1920,
            bottom: 1080,
        };
        assert!(rect_matches(monitor, monitor));
    }

    #[test]
    fn ordinary_maximized_window_is_not_fullscreen() {
        let window = RECT {
            left: 0,
            top: 0,
            right: 1920,
            bottom: 1040,
        };
        let monitor = RECT {
            left: 0,
            top: 0,
            right: 1920,
            bottom: 1080,
        };
        assert!(!rect_matches(window, monitor));
    }
}
