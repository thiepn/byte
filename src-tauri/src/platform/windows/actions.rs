use crate::{
    core::error::ByteError,
    models::{RecommendedActionKind, ReleaseChannel},
    platform::windows::windowing,
};
use std::{ffi::OsStr, iter, os::windows::ffi::OsStrExt, ptr::null};
use tauri::AppHandle;
use windows_sys::Win32::{
    Foundation::HWND,
    UI::{Shell::ShellExecuteW, WindowsAndMessaging::SW_SHOWNORMAL},
};

const STABLE_RELEASES_URL: &str = "https://github.com/thiepn/byte/releases/latest";
const BETA_RELEASES_URL: &str = "https://github.com/thiepn/byte/releases";

pub fn execute(app: &AppHandle, action: RecommendedActionKind) -> Result<(), ByteError> {
    match action {
        RecommendedActionKind::ViewDetails => windowing::show_main_window(app),
        RecommendedActionKind::OpenTaskManager => launch("taskmgr.exe"),
        RecommendedActionKind::OpenStorageSettings => launch("ms-settings:storagesense"),
        RecommendedActionKind::OpenBatterySettings => launch("ms-settings:batterysaver-settings"),
    }
}

pub fn open_release_page(channel: ReleaseChannel) -> Result<(), ByteError> {
    match channel {
        ReleaseChannel::Stable => launch(STABLE_RELEASES_URL),
        ReleaseChannel::Beta => launch(BETA_RELEASES_URL),
    }
}

fn launch(target: &str) -> Result<(), ByteError> {
    let operation = wide("open");
    let target = wide(target);

    // SAFETY: ShellExecuteW only receives null-terminated local strings selected
    // by the enum above. There is no user-provided executable, URI, or argument.
    let result = unsafe {
        ShellExecuteW(
            0 as HWND,
            operation.as_ptr(),
            target.as_ptr(),
            null(),
            null(),
            SW_SHOWNORMAL,
        )
    };

    if result as isize <= 32 {
        return Err(ByteError::Window(
            "Windows could not open the requested system destination".into(),
        ));
    }

    Ok(())
}

fn wide(value: &str) -> Vec<u16> {
    OsStr::new(value)
        .encode_wide()
        .chain(iter::once(0))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_channels_are_fixed_allowlisted_destinations() {
        assert_eq!(
            STABLE_RELEASES_URL,
            "https://github.com/thiepn/byte/releases/latest"
        );
        assert_eq!(BETA_RELEASES_URL, "https://github.com/thiepn/byte/releases");
    }

    #[test]
    fn wide_strings_are_null_terminated() {
        let value = wide("ms-settings:storagesense");
        assert_eq!(value.last(), Some(&0));
        assert_eq!(
            String::from_utf16_lossy(&value[..value.len() - 1]),
            "ms-settings:storagesense"
        );
    }
}
