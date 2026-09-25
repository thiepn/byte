use crate::{
    core::{
        error::ByteError,
        persistence::{read_bounded_text, BoundedText},
        security::{
            normalize_and_validate_app_preferences, normalize_and_validate_config,
            validate_companion_preferences,
        },
    },
    models::{AppPreferences, ByteConfig, CompanionPreferences},
};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use tempfile::NamedTempFile;

pub const CURRENT_SCHEMA_VERSION: u32 = 9;
const MAX_CONFIG_FILE_BYTES: u64 = 256 * 1024;

pub struct ConfigStore {
    path: PathBuf,
    config: ByteConfig,
}

impl ConfigStore {
    pub fn load(path: PathBuf) -> Result<Self, ByteError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let config = match read_bounded_text(&path, MAX_CONFIG_FILE_BYTES)? {
            BoundedText::Present(raw) => match decode_and_migrate(&raw) {
                Ok(value) => value,
                Err(_) => {
                    quarantine_corrupt_config(&path);
                    ByteConfig::default()
                }
            },
            BoundedText::Invalid => {
                quarantine_corrupt_config(&path);
                ByteConfig::default()
            }
            BoundedText::Missing => ByteConfig::default(),
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
        validate_companion_preferences(&preferences)?;

        let mut next = self.config.clone();
        next.companion = preferences;
        self.save_config(&next)?;
        self.config = next;
        Ok(self.config.clone())
    }

    pub fn update_companion_with(
        &mut self,
        update: impl FnOnce(&mut CompanionPreferences),
    ) -> Result<ByteConfig, ByteError> {
        let mut next = self.config.clone();
        update(&mut next.companion);
        validate_companion_preferences(&next.companion)?;
        self.save_config(&next)?;
        self.config = next;
        Ok(self.config.clone())
    }

    pub fn update_app(&mut self, mut preferences: AppPreferences) -> Result<ByteConfig, ByteError> {
        normalize_and_validate_app_preferences(&mut preferences)?;

        let mut next = self.config.clone();
        next.app = preferences;
        self.save_config(&next)?;
        self.config = next;
        Ok(self.config.clone())
    }

    pub fn save(&self) -> Result<(), ByteError> {
        self.save_config(&self.config)
    }

    fn save_config(&self, config: &ByteConfig) -> Result<(), ByteError> {
        let parent = self.path.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent)?;
        let payload = serde_json::to_vec_pretty(config)?;
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
        CURRENT_SCHEMA_VERSION => {}
        1..=5 => {
            config.schema_version = CURRENT_SCHEMA_VERSION;
            // Installations predating Phase 20 already passed through Byte
            // without onboarding. Do not force first-run setup on them.
            config.app.onboarding_completed = true;
        }
        6..=8 => {
            config.schema_version = CURRENT_SCHEMA_VERSION;
            // Phase 20+ already persisted onboarding state; preserve it.
            // Phase 28 adds a serde-defaulted Stable update channel.
        }
        other => {
            return Err(ByteError::Config(format!(
                "Unsupported configuration schema version: {other}"
            )));
        }
    }

    normalize_and_validate_config(&mut config)?;
    Ok(config)
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
    fn rejected_companion_mutation_does_not_poison_in_memory_state() {
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("config.json");
        let mut store = ConfigStore::load(path).expect("load");

        let result = store.update_companion_with(|preferences| {
            preferences.character = "../../invalid".into();
        });

        assert!(result.is_err());
        assert_eq!(store.snapshot().companion.character, "BYTE");
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
    fn v5_config_gains_phase_20_defaults_without_forcing_onboarding() {
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("config.json");
        let legacy = r#"{
          "schema_version": 5,
          "companion": {
            "character": "BYTE",
            "palette": "mint",
            "habitat": "DESK",
            "display_mode": "HABITAT",
            "size": "MEDIUM",
            "interaction_level": "NORMAL",
            "personality": "CURIOUS",
            "edge_anchor": "RIGHT",
            "placements": {
              "habitat": null,
              "perch": null,
              "mini": null,
              "edge": null
            },
            "customization": {
              "headwear": "none",
              "face_accessory": "none",
              "body_accessory": "none",
              "back_accessory": "none",
              "hand_prop": "none",
              "decorations": {
                "large_background": "none",
                "wall_or_sky": "none",
                "surface_left": "none",
                "surface_right": "none",
                "small_prop": "none",
                "ambient": "none"
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
        let app = migrated.snapshot().app;

        assert!(app.onboarding_completed);
        assert!(app.system_monitoring_enabled);
        assert!(app.notifications_enabled);
        assert_eq!(app.text_scale_percent, 100);
    }

    #[test]
    fn v6_config_gains_smart_notification_defaults() {
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("config.json");
        let config = ByteConfig {
            schema_version: 6,
            ..ByteConfig::default()
        };
        let mut value = serde_json::to_value(config).expect("serialize");
        let app = value
            .get_mut("app")
            .and_then(serde_json::Value::as_object_mut)
            .expect("app");
        app.remove("notification_memory_enabled");
        app.remove("notification_thermal_enabled");
        app.remove("notification_storage_enabled");
        app.remove("notification_battery_enabled");
        app.remove("notification_runaway_process_enabled");
        app.remove("notification_quiet_mode");
        app.remove("notification_snoozed_until_epoch_ms");
        fs::write(&path, serde_json::to_vec_pretty(&value).expect("json")).expect("write");

        let migrated = ConfigStore::load(path).expect("migrate");
        let app = migrated.snapshot().app;

        assert!(app.notification_memory_enabled);
        assert!(app.notification_thermal_enabled);
        assert!(app.notification_storage_enabled);
        assert!(app.notification_battery_enabled);
        assert!(app.notification_runaway_process_enabled);
        assert!(!app.notification_quiet_mode);
        assert!(app.notification_snoozed_until_epoch_ms.is_none());
    }

    #[test]
    fn v7_config_gains_phase_22_visibility_defaults() {
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("config.json");
        let config = ByteConfig {
            schema_version: 7,
            ..ByteConfig::default()
        };
        let mut value = serde_json::to_value(config).expect("serialize");
        let app = value
            .get_mut("app")
            .and_then(serde_json::Value::as_object_mut)
            .expect("app");
        app.remove("hide_in_presentation");
        app.remove("exclude_from_capture");
        app.remove("hidden_foreground_apps");
        fs::write(&path, serde_json::to_vec_pretty(&value).expect("json")).expect("write");

        let migrated = ConfigStore::load(path).expect("migrate");
        let app = migrated.snapshot().app;

        assert!(app.hide_in_presentation);
        assert!(app.exclude_from_capture);
        assert!(app.hidden_foreground_apps.is_empty());
    }

    #[test]
    fn v8_config_migrates_to_stable_release_channel() {
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("config.json");
        let mut legacy = serde_json::to_value(ByteConfig::default()).expect("serialize");
        legacy["schema_version"] = serde_json::Value::from(8);
        legacy["app"]
            .as_object_mut()
            .expect("app object")
            .remove("update_channel");
        fs::write(&path, serde_json::to_vec_pretty(&legacy).expect("json")).expect("write");

        let migrated = ConfigStore::load(path).expect("migrate");

        assert_eq!(migrated.snapshot().schema_version, CURRENT_SCHEMA_VERSION);
        assert_eq!(
            migrated.snapshot().app.update_channel,
            crate::models::ReleaseChannel::Stable
        );
    }

    #[test]
    fn semantically_invalid_config_is_quarantined_before_defaults_are_restored() {
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("config.json");
        let mut value = serde_json::to_value(ByteConfig::default()).expect("serialize");
        value["companion"]["character"] = serde_json::Value::String("../../escape".into());
        fs::write(&path, serde_json::to_vec_pretty(&value).expect("json")).expect("write");

        let store = ConfigStore::load(path.clone()).expect("recover");

        assert_eq!(store.snapshot().companion.character, "BYTE");
        assert!(path.with_extension("corrupt.json").exists());
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
