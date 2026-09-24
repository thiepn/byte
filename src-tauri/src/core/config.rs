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

pub const CURRENT_SCHEMA_VERSION: u32 = 5;

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
            Ok(raw) => match decode_and_migrate(&raw) {
                Ok(value) => value,
                Err(_) => {
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

    pub fn update_companion_with(
        &mut self,
        update: impl FnOnce(&mut CompanionPreferences),
    ) -> Result<ByteConfig, ByteError> {
        update(&mut self.config.companion);
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

fn decode_and_migrate(raw: &str) -> Result<ByteConfig, ByteError> {
    let mut config = serde_json::from_str::<ByteConfig>(raw)?;

    match config.schema_version {
        CURRENT_SCHEMA_VERSION => Ok(config),
        1..=4 => {
            config.schema_version = CURRENT_SCHEMA_VERSION;
            Ok(config)
        }
        other => Err(ByteError::Config(format!(
            "Unsupported configuration schema version: {other}"
        ))),
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
        preferences.customization.headwear = "beanie".into();
        store.update_companion(preferences).expect("persist");
        let reloaded = ConfigStore::load(path).expect("reload");
        assert_eq!(
            reloaded.snapshot().companion.display_mode,
            DisplayMode::Mini
        );
        assert_eq!(
            reloaded.snapshot().companion.customization.headwear,
            "beanie"
        );
    }

    #[test]
    fn v1_config_migrates_without_losing_companion_preferences() {
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("config.json");
        let legacy = r#"{
          "schema_version": 1,
          "companion": {
            "character": "BYTE",
            "habitat": "MEADOW",
            "display_mode": "MINI",
            "size": "MEDIUM",
            "interaction_level": "NORMAL"
          },
          "app": {
            "hide_in_fullscreen": true,
            "sound_enabled": false,
            "launch_at_startup": false,
            "activity_history_enabled": true
          }
        }"#;
        fs::write(&path, legacy).expect("write");

        let migrated = ConfigStore::load(path).expect("migrate");

        assert_eq!(migrated.snapshot().schema_version, CURRENT_SCHEMA_VERSION);
        assert_eq!(
            migrated.snapshot().companion.display_mode,
            DisplayMode::Mini
        );
        assert_eq!(
            migrated.snapshot().companion.edge_anchor,
            crate::models::EdgeAnchor::Right
        );
        assert!(migrated.snapshot().companion.placements.mini.is_none());
        assert_eq!(migrated.snapshot().companion.palette, "default");
        assert_eq!(
            migrated.snapshot().companion.personality,
            crate::models::Personality::Curious
        );
        assert_eq!(migrated.snapshot().companion.customization.headwear, "none");
    }

    #[test]
    fn v2_config_adds_default_palette_and_customization() {
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("config.json");
        let legacy = r#"{
          "schema_version": 2,
          "companion": {
            "character": "MOCHI",
            "habitat": "MEADOW",
            "display_mode": "HABITAT",
            "size": "MEDIUM",
            "interaction_level": "NORMAL",
            "edge_anchor": "RIGHT",
            "placements": {
              "habitat": null,
              "perch": null,
              "mini": null,
              "edge": null
            }
          },
          "app": {
            "hide_in_fullscreen": true,
            "sound_enabled": false,
            "launch_at_startup": false,
            "activity_history_enabled": true
          }
        }"#;
        fs::write(&path, legacy).expect("write");

        let migrated = ConfigStore::load(path).expect("migrate");

        assert_eq!(migrated.snapshot().schema_version, CURRENT_SCHEMA_VERSION);
        assert_eq!(migrated.snapshot().companion.character, "MOCHI");
        assert_eq!(migrated.snapshot().companion.palette, "default");
        assert_eq!(
            migrated
                .snapshot()
                .companion
                .customization
                .decorations
                .ambient,
            "none"
        );
    }

    #[test]
    fn v3_config_gains_phase_13_customization_defaults() {
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("config.json");
        let legacy = r#"{
          "schema_version": 3,
          "companion": {
            "character": "KIWI",
            "palette": "autumn",
            "habitat": "ROOFTOP",
            "display_mode": "PERCH",
            "size": "LARGE",
            "interaction_level": "PLAYFUL",
            "edge_anchor": "LEFT",
            "placements": {
              "habitat": null,
              "perch": null,
              "mini": null,
              "edge": null
            }
          },
          "app": {
            "hide_in_fullscreen": true,
            "sound_enabled": false,
            "launch_at_startup": false,
            "activity_history_enabled": true
          }
        }"#;
        fs::write(&path, legacy).expect("write");

        let migrated = ConfigStore::load(path).expect("migrate");
        let companion = migrated.snapshot().companion;

        assert_eq!(companion.character, "KIWI");
        assert_eq!(companion.palette, "autumn");
        assert_eq!(companion.habitat, "ROOFTOP");
        assert_eq!(companion.customization.back_accessory, "none");
        assert_eq!(companion.customization.decorations.surface_left, "none");
    }

    #[test]
    fn v4_config_gains_default_personality_without_losing_customization() {
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("config.json");
        let legacy = r#"{
          "schema_version": 4,
          "companion": {
            "character": "BYTE",
            "palette": "mint",
            "habitat": "DESK",
            "display_mode": "HABITAT",
            "size": "MEDIUM",
            "interaction_level": "NORMAL",
            "edge_anchor": "RIGHT",
            "placements": {
              "habitat": null,
              "perch": null,
              "mini": null,
              "edge": null
            },
            "customization": {
              "headwear": "beanie",
              "face_accessory": "none",
              "body_accessory": "scarf",
              "back_accessory": "none",
              "hand_prop": "mug",
              "decorations": {
                "large_background": "pennant_banner",
                "wall_or_sky": "none",
                "surface_left": "potted_plant",
                "surface_right": "none",
                "small_prop": "none",
                "ambient": "star_mobile"
              }
            }
          },
          "app": {
            "hide_in_fullscreen": true,
            "sound_enabled": false,
            "launch_at_startup": false,
            "activity_history_enabled": true
          }
        }"#;
        fs::write(&path, legacy).expect("write");

        let migrated = ConfigStore::load(path).expect("migrate");
        let companion = migrated.snapshot().companion;

        assert_eq!(companion.personality, crate::models::Personality::Curious);
        assert_eq!(companion.customization.headwear, "beanie");
        assert_eq!(
            companion.customization.decorations.large_background,
            "pennant_banner"
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
