use crate::core::error::ByteError;
use std::{
    fs::{File, OpenOptions},
    io,
    os::windows::fs::OpenOptionsExt,
    path::{Path, PathBuf},
};

const ERROR_SHARING_VIOLATION: i32 = 32;
const INSTANCE_LOCK_FILE: &str = "io.github.thiepn.byte.instance.lock";

pub struct SingleInstanceGuard {
    _file: File,
    path: PathBuf,
}

impl SingleInstanceGuard {
    pub fn acquire() -> Result<Option<Self>, ByteError> {
        Self::acquire_at(std::env::temp_dir().join(INSTANCE_LOCK_FILE))
    }

    fn acquire_at(path: PathBuf) -> Result<Option<Self>, ByteError> {
        match open_exclusive(&path) {
            Ok(file) => Ok(Some(Self { _file: file, path })),
            Err(error) if error.raw_os_error() == Some(ERROR_SHARING_VIOLATION) => Ok(None),
            Err(error) => Err(ByteError::Io(error.to_string())),
        }
    }

    pub fn lock_path(&self) -> &Path {
        &self.path
    }
}

fn open_exclusive(path: &Path) -> io::Result<File> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .share_mode(0)
        .open(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn second_open_is_rejected_until_first_guard_drops() {
        let temp = tempfile::tempdir().expect("temp");
        let path = temp.path().join("byte-instance.lock");

        let first = SingleInstanceGuard::acquire_at(path.clone())
            .expect("first acquire")
            .expect("first guard");
        assert_eq!(first.lock_path(), path.as_path());

        assert!(SingleInstanceGuard::acquire_at(path.clone())
            .expect("duplicate check")
            .is_none());

        drop(first);

        assert!(SingleInstanceGuard::acquire_at(path)
            .expect("reacquire")
            .is_some());
    }
}
