use crate::{
    core::{error::ByteError, lifecycle::LifecycleState, state::AppState},
    models::{AppPreferences, VisibilitySuppressionReason},
    platform::windows::windowing,
};
use std::{
    ffi::OsString,
    mem::size_of,
    os::windows::ffi::OsStringExt,
    path::Path,
    ptr::{null, null_mut},
    sync::atomic::{AtomicU8, Ordering},
    thread,
    time::Duration,
};
use tauri::{AppHandle, Manager};
use windows_sys::Win32::{
    Foundation::{CloseHandle, HWND, LPARAM, LRESULT, RECT, WPARAM},
    Graphics::Gdi::{GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST},
    System::{
        LibraryLoader::GetModuleHandleW,
        Power::{
            RegisterPowerSettingNotification, UnregisterPowerSettingNotification, HPOWERNOTIFY,
            POWERBROADCAST_SETTING,
        },
        SystemServices::GUID_CONSOLE_DISPLAY_STATE,
        Threading::{
            GetCurrentProcessId, OpenProcess, QueryFullProcessImageNameW,
            PROCESS_QUERY_LIMITED_INFORMATION,
        },
    },
    UI::{
        Shell::SHQueryUserNotificationState,
        WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetForegroundWindow,
            GetWindowRect, GetWindowThreadProcessId, PeekMessageW, RegisterClassW,
            TranslateMessage, HWND_MESSAGE, MSG, PBT_POWERSETTINGCHANGE, PM_REMOVE,
            WM_POWERBROADCAST, WNDCLASSW,
        },
    },
};

const POLL_INTERVAL: Duration = Duration::from_millis(500);
const RECT_TOLERANCE_PX: i32 = 2;
const QUNS_NOT_PRESENT: i32 = 1;
const QUNS_BUSY: i32 = 2;
const QUNS_RUNNING_D3D_FULL_SCREEN: i32 = 3;
const QUNS_PRESENTATION_MODE: i32 = 4;
const DISPLAY_OFF: u8 = 0;
const DISPLAY_ON: u8 = 1;
const DISPLAY_DIMMED: u8 = 2;

static DISPLAY_STATE: AtomicU8 = AtomicU8::new(DISPLAY_ON);

#[derive(Debug, Clone, PartialEq, Eq)]
struct AwarenessObservation {
    display_off: bool,
    user_state: Option<i32>,
    fullscreen_geometry: bool,
    foreground_app: Option<String>,
    byte_owns_foreground: bool,
}

pub fn start(app: AppHandle) -> Result<(), ByteError> {
    let inspect_foreground_app = !app
        .state::<AppState>()
        .app_preferences()
        .hidden_foreground_apps
        .is_empty();
    apply_observation(&app, observe(inspect_foreground_app), true);

    let worker_app = app.clone();
    let worker = thread::Builder::new()
        .name("byte-desktop-awareness".into())
        .spawn(move || run(worker_app))
        .map_err(|error| ByteError::Io(error.to_string()))?;

    app.state::<AppState>().install_fullscreen_worker(worker);
    Ok(())
}

fn run(app: AppHandle) {
    let event_window = PowerEventWindow::new();

    loop {
        let state = app.state::<AppState>();
        if state.lifecycle.is_cancelled() {
            break;
        }

        if let Some(window) = event_window.as_ref() {
            window.pump_messages();
        }

        let inspect_foreground_app = !state.app_preferences().hidden_foreground_apps.is_empty();
        apply_observation(&app, observe(inspect_foreground_app), false);
        thread::sleep(POLL_INTERVAL);
    }
}

fn apply_observation(app: &AppHandle, observation: AwarenessObservation, initial: bool) {
    let state = app.state::<AppState>();
    let config = state
        .config
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .snapshot();

    let previous_reason = state.desktop_awareness().reason;
    let preserve_external_suppression = observation.byte_owns_foreground
        && matches!(
            previous_reason,
            Some(
                VisibilitySuppressionReason::Fullscreen
                    | VisibilitySuppressionReason::Presentation
                    | VisibilitySuppressionReason::ExcludedApp
            )
        );
    let reason = if preserve_external_suppression {
        previous_reason
    } else {
        suppression_reason(&observation, &config.app)
    };
    let visible = windowing::is_companion_visible(app).unwrap_or(false);
    let visible_foreground_app = (reason == Some(VisibilitySuppressionReason::ExcludedApp))
        .then(|| observation.foreground_app.clone())
        .flatten();
    let transition = state.update_desktop_awareness(
        reason,
        visible_foreground_app,
        config.app.exclude_from_capture,
        visible,
    );

    if reason.is_some() {
        let _ = windowing::hide_quick_panel(app);
        if visible {
            let _ = windowing::hide_companion(app);
        }
    } else if transition.restore_on_exit && !initial {
        let _ = windowing::show_companion(app);
    }

    if transition.changed {
        let next_lifecycle = match reason {
            Some(VisibilitySuppressionReason::Locked) => LifecycleState::Locked,
            Some(
                VisibilitySuppressionReason::Fullscreen
                | VisibilitySuppressionReason::Presentation
                | VisibilitySuppressionReason::ExcludedApp,
            ) => LifecycleState::FullscreenReduced,
            // Phase 22 hides on display-off but deliberately leaves worker
            // suspension for Phase 23's coordinated power lifecycle.
            Some(VisibilitySuppressionReason::DisplaySleep) | None => LifecycleState::Active,
        };
        state.lifecycle.transition(next_lifecycle);
    }
}

fn observe(inspect_foreground_app: bool) -> AwarenessObservation {
    let display_off = DISPLAY_STATE.load(Ordering::Acquire) == DISPLAY_OFF;
    let user_state = user_notification_state();

    // SAFETY: window handles are used only for immediate Win32 queries, and
    // output buffers live for the duration of each call.
    unsafe {
        let window = GetForegroundWindow();
        if window.is_null() {
            return AwarenessObservation {
                display_off,
                user_state,
                fullscreen_geometry: false,
                foreground_app: None,
                byte_owns_foreground: false,
            };
        }

        let mut process_id = 0u32;
        GetWindowThreadProcessId(window, &mut process_id);
        if process_id == GetCurrentProcessId() {
            return AwarenessObservation {
                display_off,
                user_state,
                fullscreen_geometry: false,
                foreground_app: None,
                byte_owns_foreground: true,
            };
        }

        let foreground_app = inspect_foreground_app
            .then(|| process_name(process_id))
            .flatten();

        let mut window_rect: RECT = std::mem::zeroed();
        let fullscreen_geometry = if GetWindowRect(window, &mut window_rect) != 0 {
            let monitor = MonitorFromWindow(window, MONITOR_DEFAULTTONEAREST);
            if monitor.is_null() {
                false
            } else {
                let mut info: MONITORINFO = std::mem::zeroed();
                info.cbSize = size_of::<MONITORINFO>() as u32;
                GetMonitorInfoW(monitor, &mut info) != 0
                    && rect_matches(window_rect, info.rcMonitor)
            }
        } else {
            false
        };

        AwarenessObservation {
            display_off,
            user_state,
            fullscreen_geometry,
            foreground_app,
            byte_owns_foreground: false,
        }
    }
}

fn suppression_reason(
    observation: &AwarenessObservation,
    preferences: &AppPreferences,
) -> Option<VisibilitySuppressionReason> {
    if observation.display_off {
        return Some(VisibilitySuppressionReason::DisplaySleep);
    }

    if observation.user_state == Some(QUNS_NOT_PRESENT) {
        return Some(VisibilitySuppressionReason::Locked);
    }

    if observation
        .foreground_app
        .as_deref()
        .map(|name| {
            preferences
                .hidden_foreground_apps
                .iter()
                .any(|item| item == name)
        })
        .unwrap_or(false)
    {
        return Some(VisibilitySuppressionReason::ExcludedApp);
    }

    if preferences.hide_in_presentation && observation.user_state == Some(QUNS_PRESENTATION_MODE) {
        return Some(VisibilitySuppressionReason::Presentation);
    }

    if preferences.hide_in_fullscreen
        && (observation.fullscreen_geometry
            || observation.user_state == Some(QUNS_RUNNING_D3D_FULL_SCREEN))
    {
        return Some(VisibilitySuppressionReason::Fullscreen);
    }

    if observation.user_state == Some(QUNS_BUSY) {
        if preferences.hide_in_presentation && !observation.fullscreen_geometry {
            return Some(VisibilitySuppressionReason::Presentation);
        }
        if preferences.hide_in_fullscreen {
            return Some(VisibilitySuppressionReason::Fullscreen);
        }
    }

    None
}

fn user_notification_state() -> Option<i32> {
    let mut state = 0i32;
    // SAFETY: SHQueryUserNotificationState writes one enum-sized integer to
    // the provided valid stack pointer.
    let result = unsafe { SHQueryUserNotificationState(&mut state) };
    (result >= 0).then_some(state)
}

fn process_name(process_id: u32) -> Option<String> {
    // SAFETY: the process handle is closed before return; the output buffer is
    // local and sized according to the API contract.
    unsafe {
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, process_id);
        if process.is_null() {
            return None;
        }

        let mut buffer = [0u16; 1024];
        let mut size = buffer.len() as u32;
        let ok = QueryFullProcessImageNameW(process, 0, buffer.as_mut_ptr(), &mut size);
        let _ = CloseHandle(process);
        if ok == 0 || size == 0 {
            return None;
        }

        let path = OsString::from_wide(&buffer[..size as usize]);
        let stem = Path::new(&path).file_stem()?.to_string_lossy();
        normalize_app_name(&stem)
    }
}

pub fn normalize_excluded_apps(values: &[String]) -> Result<Vec<String>, ByteError> {
    if values.len() > 32 {
        return Err(ByteError::Config(
            "At most 32 foreground app exclusions are supported".into(),
        ));
    }

    let mut normalized = Vec::new();
    for value in values {
        if value.len() > 96 || value.contains(['\\', '/', ':']) {
            return Err(ByteError::Config(
                "Excluded app names must be executable names, not paths".into(),
            ));
        }
        let Some(value) = normalize_app_name(value) else {
            continue;
        };
        if !normalized.contains(&value) {
            normalized.push(value);
        }
    }

    Ok(normalized)
}

fn normalize_app_name(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let without_exe = trimmed
        .strip_suffix(".exe")
        .or_else(|| trimmed.strip_suffix(".EXE"))
        .unwrap_or(trimmed);
    let normalized = without_exe.trim().to_lowercase();
    (!normalized.is_empty()).then_some(normalized)
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

struct PowerEventWindow {
    hwnd: HWND,
    power_notification: HPOWERNOTIFY,
}

impl PowerEventWindow {
    fn new() -> Option<Self> {
        // SAFETY: this creates a message-only window owned by this process and
        // registers it for one documented power-setting notification.
        unsafe {
            let instance = GetModuleHandleW(null());
            if instance.is_null() {
                return None;
            }

            let class_name = wide("ByteDesktopAwarenessWindow");
            let mut class: WNDCLASSW = std::mem::zeroed();
            class.lpfnWndProc = Some(power_window_proc);
            class.hInstance = instance;
            class.lpszClassName = class_name.as_ptr();
            let _ = RegisterClassW(&class);

            let hwnd = CreateWindowExW(
                0,
                class_name.as_ptr(),
                class_name.as_ptr(),
                0,
                0,
                0,
                0,
                0,
                HWND_MESSAGE,
                null_mut(),
                instance,
                null(),
            );
            if hwnd.is_null() {
                return None;
            }

            let power_notification =
                RegisterPowerSettingNotification(hwnd as _, &GUID_CONSOLE_DISPLAY_STATE, 0);
            if power_notification.is_null() {
                let _ = DestroyWindow(hwnd);
                return None;
            }

            Some(Self {
                hwnd,
                power_notification,
            })
        }
    }

    fn pump_messages(&self) {
        // SAFETY: MSG is initialized before dispatch and messages are removed
        // only for this process-owned message window.
        unsafe {
            let mut message: MSG = std::mem::zeroed();
            while PeekMessageW(&mut message, self.hwnd, 0, 0, PM_REMOVE) != 0 {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
    }
}

impl Drop for PowerEventWindow {
    fn drop(&mut self) {
        // SAFETY: both handles were created/registered by PowerEventWindow.
        unsafe {
            let _ = UnregisterPowerSettingNotification(self.power_notification);
            let _ = DestroyWindow(self.hwnd);
        }
    }
}

unsafe extern "system" fn power_window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == WM_POWERBROADCAST && wparam as u32 == PBT_POWERSETTINGCHANGE && lparam != 0 {
        // SAFETY: Windows documents lParam for PBT_POWERSETTINGCHANGE as a
        // valid POWERBROADCAST_SETTING pointer for the duration of the call.
        let setting = unsafe { &*(lparam as *const POWERBROADCAST_SETTING) };
        if setting.PowerSetting == GUID_CONSOLE_DISPLAY_STATE && setting.DataLength >= 4 {
            let data = setting.Data.as_ptr() as *const u32;
            let value = unsafe { std::ptr::read_unaligned(data) };
            if value <= DISPLAY_DIMMED as u32 {
                DISPLAY_STATE.store(value as u8, Ordering::Release);
            }
        }
        return 1;
    }

    unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn preferences() -> AppPreferences {
        AppPreferences::default()
    }

    fn observation() -> AwarenessObservation {
        AwarenessObservation {
            display_off: false,
            user_state: None,
            fullscreen_geometry: false,
            foreground_app: Some("browser".into()),
            byte_owns_foreground: false,
        }
    }

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

    #[test]
    fn display_off_and_lock_have_priority() {
        let mut current = observation();
        current.display_off = true;
        assert_eq!(
            suppression_reason(&current, &preferences()),
            Some(VisibilitySuppressionReason::DisplaySleep)
        );

        current.display_off = false;
        current.user_state = Some(QUNS_NOT_PRESENT);
        assert_eq!(
            suppression_reason(&current, &preferences()),
            Some(VisibilitySuppressionReason::Locked)
        );
    }

    #[test]
    fn fullscreen_and_presentation_follow_preferences() {
        let mut current = observation();
        current.fullscreen_geometry = true;
        assert_eq!(
            suppression_reason(&current, &preferences()),
            Some(VisibilitySuppressionReason::Fullscreen)
        );

        current.fullscreen_geometry = false;
        current.user_state = Some(QUNS_PRESENTATION_MODE);
        assert_eq!(
            suppression_reason(&current, &preferences()),
            Some(VisibilitySuppressionReason::Presentation)
        );
    }

    #[test]
    fn only_external_foreground_reasons_need_byte_foreground_preservation() {
        for reason in [
            VisibilitySuppressionReason::Fullscreen,
            VisibilitySuppressionReason::Presentation,
            VisibilitySuppressionReason::ExcludedApp,
        ] {
            assert!(matches!(
                Some(reason),
                Some(
                    VisibilitySuppressionReason::Fullscreen
                        | VisibilitySuppressionReason::Presentation
                        | VisibilitySuppressionReason::ExcludedApp
                )
            ));
        }

        assert!(!matches!(
            Some(VisibilitySuppressionReason::Locked),
            Some(
                VisibilitySuppressionReason::Fullscreen
                    | VisibilitySuppressionReason::Presentation
                    | VisibilitySuppressionReason::ExcludedApp
            )
        ));
        assert!(!matches!(
            Some(VisibilitySuppressionReason::DisplaySleep),
            Some(
                VisibilitySuppressionReason::Fullscreen
                    | VisibilitySuppressionReason::Presentation
                    | VisibilitySuppressionReason::ExcludedApp
            )
        ));
    }

    #[test]
    fn excluded_foreground_app_hides_even_when_windowed() {
        let mut prefs = preferences();
        prefs.hidden_foreground_apps = vec!["obs64".into()];
        let mut current = observation();
        current.foreground_app = Some("obs64".into());

        assert_eq!(
            suppression_reason(&current, &prefs),
            Some(VisibilitySuppressionReason::ExcludedApp)
        );
    }

    #[test]
    fn excluded_app_names_are_normalized_and_deduplicated() {
        let result =
            normalize_excluded_apps(&[" OBS64.exe ".into(), "obs64".into(), "POWERPNT.EXE".into()])
                .expect("normalize");

        assert_eq!(result, vec!["obs64", "powerpnt"]);
    }

    #[test]
    fn excluded_app_paths_are_rejected() {
        assert!(normalize_excluded_apps(&["C:\\Apps\\game.exe".into()]).is_err());
    }
}
