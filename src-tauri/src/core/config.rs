use crate::{
    core::error::ByteError,
    models::{ByteConfig, CompanionPreferences},
};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use tempfile::NamedTempFile;

pub const CURRENT_SCHEMA_VERSION: u32 = 1;

pub struct ConfigStore {
    path: PathBuf,
    config: ByteConfig,
}

impl ConfigStore {
    pub fn load(path: PathBuf) -> Result<Self, ByteError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let config = match fs::read_to_string(&path) {
            Ok(raw) => match serde_json::from_str::<ByteConfig>(&raw) {
                Ok(value) if value.schema_version == CURRENT_SCHEMA_VERSION => value,
                Ok(_) | Err(_) => {
                    quarantine_corrupt_config(&path);
                    ByteConfig::default()
                }
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => ByteConfig::default(),
            Err(error) => return Err(error.into()),
        };

        let store = Self { path, config };
        store.save()?;
        Ok(store)
    }

    pub fn snapshot(&self) -> ByteConfig {
        self.config.clone()
    }

    pub fn update_companion(
        &mut self,
        preferences: CompanionPreferences,
    ) -> Result<ByteConfig, ByteError> {
        self.config.companion = preferences;
        self.save()?;
        Ok(self.config.clone())
    }

    pub fn save(&self) -> Result<(), ByteError> {
        let parent = self.path.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent)?;
        let payload = serde_json::to_vec_pretty(&self.config)?;
        let mut temp = NamedTempFile::new_in(parent)?;
        temp.write_all(&payload)?;
        temp.as_file_mut().sync_all()?;
        temp.persist(&self.path)
            .map_err(|error| ByteError::Io(error.error.to_string()))?;
        Ok(())
    }
}

fn quarantine_corrupt_config(path: &Path) {
    if !path.exists() {
        return;
    }
    let quarantine = path.with_extension("corrupt.json");
    let _ = fs::remove_file(&quarantine);
    let _ = fs::rename(path, quarantine);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::DisplayMode;

    #[test]
    fn defaults_are_persisted_and_reloaded() {
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("config.json");
        let mut store = ConfigStore::load(path.clone()).expect("load");
        let mut preferences = store.snapshot().companion;
        preferences.display_mode = DisplayMode::Mini;
        store.update_companion(preferences).expect("persist");
        let reloaded = ConfigStore::load(path).expect("reload");
        assert_eq!(
            reloaded.snapshot().companion.display_mode,
            DisplayMode::Mini
        );
    }

    #[test]
    fn corrupt_config_recovers_to_defaults() {
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("config.json");
        fs::write(&path, "{ definitely not json").expect("fixture");
        let store = ConfigStore::load(path).expect("recover");
        assert_eq!(store.snapshot().schema_version, CURRENT_SCHEMA_VERSION);
    }
}
