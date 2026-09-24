use crate::core::error::ByteError;
use std::env;
use winreg::{enums::HKEY_CURRENT_USER, RegKey};

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const VALUE_NAME: &str = "Byte";

pub fn apply(enabled: bool) -> Result<(), ByteError> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu.create_subkey(RUN_KEY)?;

    if enabled {
        let executable = env::current_exe()?;
        let command = format!("\"{}\"", executable.display());
        key.set_value(VALUE_NAME, &command)?;
    } else {
        match key.delete_value(VALUE_NAME) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_location_is_current_user_run_key() {
        assert_eq!(RUN_KEY, r"Software\Microsoft\Windows\CurrentVersion\Run");
        assert_eq!(VALUE_NAME, "Byte");
    }
}
