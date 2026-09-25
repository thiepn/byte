use crate::core::error::ByteError;
use std::ptr::null;
use windows_sys::Win32::{
    Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE},
    System::Threading::CreateMutexW,
};

const INSTANCE_MUTEX_NAME: &str = r"Local\io.github.thiepn.byte";

pub struct SingleInstanceGuard {
    handle: HANDLE,
}

impl SingleInstanceGuard {
    pub fn acquire() -> Result<Option<Self>, ByteError> {
        let name = wide(INSTANCE_MUTEX_NAME);

        // SAFETY: the security attributes pointer is null, the initial-owner
        // flag is false, and name is a valid null-terminated UTF-16 string
        // for the duration of the call.
        let handle = unsafe { CreateMutexW(null(), 0, name.as_ptr()) };
        if handle.is_null() {
            return Err(ByteError::Io(std::io::Error::last_os_error().to_string()));
        }

        // SAFETY: GetLastError immediately follows CreateMutexW on this thread.
        let already_running = unsafe { GetLastError() } == ERROR_ALREADY_EXISTS;
        if already_running {
            // SAFETY: this process owns the returned mutex handle.
            unsafe {
                let _ = CloseHandle(handle);
            }
            return Ok(None);
        }

        Ok(Some(Self { handle }))
    }
}

impl Drop for SingleInstanceGuard {
    fn drop(&mut self) {
        // SAFETY: the handle was returned by CreateMutexW and remains owned by
        // this guard until drop.
        unsafe {
            let _ = CloseHandle(self.handle);
        }
    }
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instance_mutex_name_is_local_and_null_terminated() {
        assert!(INSTANCE_MUTEX_NAME.starts_with("Local\\"));
        let encoded = wide(INSTANCE_MUTEX_NAME);
        assert_eq!(encoded.last(), Some(&0));
    }
}
