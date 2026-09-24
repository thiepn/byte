use crate::{
    core::error::ByteError,
    models::{AppPreferences, ByteConfig, CompanionPreferences, SavedPlacement},
};
use std::collections::BTreeSet;

const MAX_EXCLUDED_APPS: usize = 32;
const MAX_EXCLUDED_APP_LEN: usize = 96;
const MAX_MONITOR_NAME_LEN: usize = 256;

const CHARACTERS: &[&str] = &["BYTE", "MOCHI", "PIP", "KIWI"];
const HABITATS: &[&str] = &["MEADOW", "DESK", "BEDROOM", "SPACE", "AQUARIUM", "ROOFTOP"];

const BYTE_PALETTES: &[&str] = &[
    "default", "mint", "peach", "lavender", "cream", "charcoal", "rose", "retro", "aurora",
];
const MOCHI_PALETTES: &[&str] = &[
    "default",
    "ginger",
    "tuxedo",
    "gray",
    "lavender",
    "strawberry",
    "cocoa",
    "snow",
    "aurora",
];
const PIP_PALETTES: &[&str] = &[
    "default", "mint", "peach", "grape", "lemon", "rose", "aqua", "midnight", "aurora",
];
const KIWI_PALETTES: &[&str] = &[
    "default", "lime", "autumn", "bluebird", "lavender", "peach", "snow", "midnight", "aurora",
];

const HEADWEAR: &[&str] = &[
    "none",
    "beanie",
    "crown",
    "sprout",
    "night_cap",
    "headphones",
];
const FACE_ACCESSORY: &[&str] = &["none", "round_glasses", "star_glasses"];
const BODY_ACCESSORY: &[&str] = &["none", "scarf", "bow_tie"];
const BACK_ACCESSORY: &[&str] = &["none", "backpack", "wings"];
const HAND_PROP: &[&str] = &["none", "mug", "book", "star_wand"];

const LARGE_BACKGROUND: &[&str] = &[
    "none",
    "pennant_banner",
    "memory_frame",
    "constellation_frame",
];
const WALL_OR_SKY: &[&str] = &["none", "string_lights", "signal_kite", "paper_cloud"];
const SURFACE_LEFT: &[&str] = &["none", "potted_plant", "book_stack"];
const SURFACE_RIGHT: &[&str] = &["none", "table_lamp", "tiny_radio"];
const SMALL_PROP: &[&str] = &["none", "tiny_mug", "charging_orb", "little_crystal"];
const AMBIENT: &[&str] = &["none", "star_mobile", "sparkle_chimes"];

pub fn normalize_and_validate_config(config: &mut ByteConfig) -> Result<(), ByteError> {
    normalize_and_validate_app_preferences(&mut config.app)?;
    validate_companion_preferences(&config.companion)
}

pub fn normalize_and_validate_app_preferences(
    preferences: &mut AppPreferences,
) -> Result<(), ByteError> {
    if !matches!(preferences.text_scale_percent, 100 | 110 | 125) {
        return Err(ByteError::Config(
            "Text scale must be 100, 110, or 125 percent".into(),
        ));
    }

    if preferences.hidden_foreground_apps.len() > MAX_EXCLUDED_APPS {
        return Err(ByteError::Config(
            "At most 32 foreground app exclusions are supported".into(),
        ));
    }

    let mut normalized = Vec::new();
    let mut seen = BTreeSet::new();

    for raw in &preferences.hidden_foreground_apps {
        if raw.len() > MAX_EXCLUDED_APP_LEN
            || raw
                .chars()
                .any(|character| character.is_control() || matches!(character, '\\' | '/' | ':'))
        {
            return Err(ByteError::Config(
                "Excluded app names must be executable names, not paths".into(),
            ));
        }

        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }

        let lowercase = trimmed.to_lowercase();
        let stem = lowercase.strip_suffix(".exe").unwrap_or(&lowercase).trim();
        if stem.is_empty() {
            continue;
        }

        if seen.insert(stem.to_string()) {
            normalized.push(stem.to_string());
        }
    }

    preferences.hidden_foreground_apps = normalized;
    Ok(())
}

pub fn validate_snooze(until: Option<u64>, now: u64) -> Result<(), ByteError> {
    if let Some(until) = until {
        let maximum = now.saturating_add(7 * 24 * 60 * 60 * 1_000);
        if until > maximum {
            return Err(ByteError::Config(
                "Notification snooze cannot exceed seven days".into(),
            ));
        }
    }
    Ok(())
}

pub fn validate_companion_preferences(preferences: &CompanionPreferences) -> Result<(), ByteError> {
    require_member("character", &preferences.character, CHARACTERS)?;
    require_member("habitat", &preferences.habitat, HABITATS)?;
    require_member(
        "palette",
        &preferences.palette,
        palettes_for_character(&preferences.character)?,
    )?;

    let customization = &preferences.customization;
    require_member("headwear", &customization.headwear, HEADWEAR)?;
    require_member(
        "face accessory",
        &customization.face_accessory,
        FACE_ACCESSORY,
    )?;
    require_member(
        "body accessory",
        &customization.body_accessory,
        BODY_ACCESSORY,
    )?;
    require_member(
        "back accessory",
        &customization.back_accessory,
        BACK_ACCESSORY,
    )?;
    require_member("hand prop", &customization.hand_prop, HAND_PROP)?;

    let decorations = &customization.decorations;
    require_member(
        "large background decoration",
        &decorations.large_background,
        LARGE_BACKGROUND,
    )?;
    require_member(
        "wall or sky decoration",
        &decorations.wall_or_sky,
        WALL_OR_SKY,
    )?;
    require_member(
        "left surface decoration",
        &decorations.surface_left,
        SURFACE_LEFT,
    )?;
    require_member(
        "right surface decoration",
        &decorations.surface_right,
        SURFACE_RIGHT,
    )?;
    require_member("small prop decoration", &decorations.small_prop, SMALL_PROP)?;
    require_member("ambient decoration", &decorations.ambient, AMBIENT)?;

    for (name, placement) in [
        ("habitat", preferences.placements.habitat.as_ref()),
        ("perch", preferences.placements.perch.as_ref()),
        ("mini", preferences.placements.mini.as_ref()),
        ("edge", preferences.placements.edge.as_ref()),
    ] {
        if let Some(placement) = placement {
            validate_placement(name, placement)?;
        }
    }

    Ok(())
}

fn palettes_for_character(character: &str) -> Result<&'static [&'static str], ByteError> {
    match character {
        "BYTE" => Ok(BYTE_PALETTES),
        "MOCHI" => Ok(MOCHI_PALETTES),
        "PIP" => Ok(PIP_PALETTES),
        "KIWI" => Ok(KIWI_PALETTES),
        _ => Err(ByteError::Config("Unknown character".into())),
    }
}

fn require_member(label: &str, value: &str, allowed: &[&str]) -> Result<(), ByteError> {
    if value.len() > 64 || !allowed.contains(&value) {
        return Err(ByteError::Config(format!("Unsupported {label}")));
    }
    Ok(())
}

fn validate_placement(label: &str, placement: &SavedPlacement) -> Result<(), ByteError> {
    if !placement.x.is_finite()
        || !placement.y.is_finite()
        || !(0.0..=1.0).contains(&placement.x)
        || !(0.0..=1.0).contains(&placement.y)
    {
        return Err(ByteError::Config(format!(
            "Invalid normalized {label} placement"
        )));
    }

    if let Some(name) = placement.monitor_name.as_deref() {
        if name.len() > MAX_MONITOR_NAME_LEN || name.chars().any(char::is_control) {
            return Err(ByteError::Config("Invalid monitor name".into()));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::CompanionPreferences;

    #[test]
    fn valid_defaults_pass() {
        let mut config = ByteConfig::default();
        assert!(normalize_and_validate_config(&mut config).is_ok());
    }

    #[test]
    fn excluded_app_names_are_canonicalized_without_paths() {
        let mut app = AppPreferences {
            hidden_foreground_apps: vec![
                " OBS64.exe ".into(),
                "obs64".into(),
                "POWERPNT.EXE".into(),
            ],
            ..AppPreferences::default()
        };
        normalize_and_validate_app_preferences(&mut app).expect("normalize");
        assert_eq!(app.hidden_foreground_apps, vec!["obs64", "powerpnt"]);

        app.hidden_foreground_apps = vec!["C:\\Apps\\game.exe".into()];
        assert!(normalize_and_validate_app_preferences(&mut app).is_err());
    }

    #[test]
    fn unknown_asset_ids_and_wrong_slots_are_rejected() {
        let preferences = CompanionPreferences {
            character: "../../evil".into(),
            ..CompanionPreferences::default()
        };
        assert!(validate_companion_preferences(&preferences).is_err());

        let mut preferences = CompanionPreferences::default();
        preferences.customization.face_accessory = "night_cap".into();
        assert!(validate_companion_preferences(&preferences).is_err());

        let mut preferences = CompanionPreferences::default();
        preferences.customization.decorations.ambient = "charging_orb".into();
        assert!(validate_companion_preferences(&preferences).is_err());
    }

    #[test]
    fn palette_must_belong_to_selected_character() {
        let mut preferences = CompanionPreferences {
            character: "MOCHI".into(),
            palette: "mint".into(),
            ..CompanionPreferences::default()
        };
        assert!(validate_companion_preferences(&preferences).is_err());

        preferences.palette = "ginger".into();
        assert!(validate_companion_preferences(&preferences).is_ok());
    }

    #[test]
    fn untrusted_placements_are_bounded() {
        let mut preferences = CompanionPreferences::default();
        preferences.placements.mini = Some(SavedPlacement {
            monitor_name: Some("display".into()),
            x: 1.2,
            y: 0.5,
        });
        assert!(validate_companion_preferences(&preferences).is_err());

        preferences.placements.mini = Some(SavedPlacement {
            monitor_name: Some("display".into()),
            x: 0.2,
            y: 0.5,
        });
        assert!(validate_companion_preferences(&preferences).is_ok());
    }

    #[test]
    fn notification_snooze_is_bounded() {
        let day = 24 * 60 * 60 * 1_000;
        assert!(validate_snooze(Some(7 * day), 0).is_ok());
        assert!(validate_snooze(Some(7 * day + 1), 0).is_err());
    }
}
