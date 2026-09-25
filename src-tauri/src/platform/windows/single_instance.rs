use crate::{core::error::ByteError, platform::windows::windowing};
use std::{
    io,
    ptr::null,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
};
use tauri::AppHandle;
use windows_sys::Win32::{
    Foundation::{
        CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT,
    },
    System::Threading::{CreateEventW, SetEvent, WaitForSingleObject},
};

const ACTIVATION_EVENT_NAME: &str = "Local\\io.github.thiepn.byte.activate";
const ACTIVATION_WAIT_MS: u32 = 500;

pub struct SingleInstanceGuard {
    activation_event: usize,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl SingleInstanceGuard {
    pub fn acquire() -> Result<Option<Self>, ByteError> {
        Self::acquire_named(ACTIVATION_EVENT_NAME)
    }

    fn acquire_named(name: &str) -> Result<Option<Self>, ByteError> {
        let (activation_event, last_error) =
            create_activation_event(name).map_err(|error| ByteError::Io(error.to_string()))?;

        if last_error == ERROR_ALREADY_EXISTS {
            // A running Byte owns this session-scoped event. Signal it before
            // exiting so the existing process can reveal/focus its main window.
            unsafe {
                let _ = SetEvent(activation_event as HANDLE);
                let _ = CloseHandle(activation_event as HANDLE);
            }
            return Ok(None);
        }

        Ok(Some(Self {
            activation_event,
            stop: Arc::new(AtomicBool::new(false)),
            worker: None,
        }))
    }

    pub fn start_activation_listener(&mut self, app: AppHandle) -> Result<(), ByteError> {
        if self.worker.is_some() {
            return Ok(());
        }

        let activation_event = self.activation_event;
        let stop = Arc::clone(&self.stop);
        let worker = thread::Builder::new()
            .name("byte-single-instance-activation".into())
            .spawn(move || activation_loop(activation_event, stop, app))
            .map_err(|error| ByteError::Io(error.to_string()))?;

        self.worker = Some(worker);
        Ok(())
    }
}

impl Drop for SingleInstanceGuard {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);

        // Wake a listener immediately instead of waiting for its bounded poll.
        unsafe {
            let _ = SetEvent(self.activation_event as HANDLE);
        }

        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }

        unsafe {
            let _ = CloseHandle(self.activation_event as HANDLE);
        }
    }
}

fn create_activation_event(name: &str) -> io::Result<(usize, u32)> {
    let wide_name = wide(name);

    // SAFETY: Byte passes a valid, null-terminated name, requests an auto-reset
    // event, and owns the returned handle until it is explicitly closed.
    let event = unsafe { CreateEventW(null(), 0, 0, wide_name.as_ptr()) };
    // CreateEventW documents ERROR_ALREADY_EXISTS through GetLastError even
    // when it succeeds, so capture it immediately before any other OS call.
    let last_error = unsafe { GetLastError() };

    if event.is_null() {
        return Err(io::Error::from_raw_os_error(last_error as i32));
    }

    Ok((event as usize, last_error))
}

fn activation_loop(activation_event: usize, stop: Arc<AtomicBool>, app: AppHandle) {
    loop {
        let result = unsafe { WaitForSingleObject(activation_event as HANDLE, ACTIVATION_WAIT_MS) };

        if stop.load(Ordering::Acquire) {
            break;
        }

        match result {
            WAIT_OBJECT_0 => {
                let _ = windowing::show_main_window(&app);
            }
            WAIT_TIMEOUT => {}
            _ => break,
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
    fn duplicate_acquire_signals_the_existing_instance_and_releases_cleanly() {
        let name = format!("Local\\io.github.thiepn.byte.test.{}", std::process::id());

        let first = SingleInstanceGuard::acquire_named(&name)
            .expect("first acquire")
            .expect("first guard");

        assert!(SingleInstanceGuard::acquire_named(&name)
            .expect("duplicate acquire")
            .is_none());

        assert_eq!(
            unsafe { WaitForSingleObject(first.activation_event as HANDLE, 0) },
            WAIT_OBJECT_0
        );

        drop(first);

        assert!(SingleInstanceGuard::acquire_named(&name)
            .expect("reacquire")
            .is_some());
    }
}
