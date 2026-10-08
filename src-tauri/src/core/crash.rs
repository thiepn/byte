//! Small local panic diagnostic, deliberately without panic messages, paths or
//! other potentially sensitive user data. This is not an analytics service.

use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    panic,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

const MAX_DIAGNOSTIC_BYTES: u64 = 64 * 1024;
const LOG_FILENAME: &str = "runtime-diagnostics.log";

pub fn install(directory: &Path) {
    let log = directory.join(LOG_FILENAME);
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|value| value.as_secs())
            .unwrap_or(0);
        let location = info.location().map(|location| (location.file(), location.line()));
        let record = diagnostic_record(timestamp, location);
        let _ = append_record(&log, &record);
        default_hook(info);
    }));
}

// Panic payloads and absolute paths may contain private information. Only
// store a timestamp, the basename of the code file and its line number.
fn diagnostic_record(timestamp: u64, location: Option<(&str, u32)>) -> String {
    let origin = location
        .and_then(|(path, line)| {
            Path::new(path)
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| format!("{name}:{line}"))
        })
        .unwrap_or_else(|| "unknown".into());
    format!("panic epoch_seconds={timestamp} source={origin}\n")
}

fn append_record(path: &Path, record: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    if fs::metadata(path)
        .map(|metadata| metadata.len().saturating_add(record.len() as u64) > MAX_DIAGNOSTIC_BYTES)
        .unwrap_or(false)
    {
        let previous = path.with_extension("previous.log");
        match fs::remove_file(&previous) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
        fs::rename(path, previous)?;
    }

    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?
        .write_all(record.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panic_record_omits_sensitive_paths_and_payloads() {
        let record = diagnostic_record(1_234, Some(("C:\\private\\person\\src\\runtime.rs", 42)));
        assert_eq!(record, "panic epoch_seconds=1234 source=runtime.rs:42\n");
        assert_eq!(diagnostic_record(0, None), "panic epoch_seconds=0 source=unknown\n");
        assert!(!record.contains("person"));
    }

    #[test]
    fn diagnostics_rotate_in_bounded_local_files() {
        let dir = tempfile::tempdir().expect("temp");
        let path = dir.path().join(LOG_FILENAME);
        for _ in 0..600 {
            append_record(&path, &format!("{}\n", "x".repeat(127))).expect("append");
        }
        let previous = path.with_extension("previous.log");
        assert!(previous.exists());
        assert!(fs::metadata(path).expect("current").len() <= MAX_DIAGNOSTIC_BYTES);
        assert!(fs::metadata(previous).expect("previous").len() <= MAX_DIAGNOSTIC_BYTES);
    }
}
