use crate::{
    core::error::ByteError,
    models::{
        now_epoch_ms, CollectionDiscoveryKind, CollectionItemKind, CollectionItemProgress,
        CollectionSnapshot, CompanionPreferences, SystemSnapshot,
    },
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use tempfile::NamedTempFile;

const COLLECTION_SCHEMA_VERSION: u32 = 1;
const DAY_MS: u64 = 86_400_000;
const WEEK_DAYS: u64 = 7;
const MONTH_DAYS: u64 = 30;
const TYPING_FIRST: u64 = 2_500;
const TYPING_SECOND: u64 = 10_000;
const CHARGING_TARGET: u64 = 5;
const NETWORK_TARGET: u64 = 5;
const NETWORK_MIN_MBPS: f32 = 0.5;
const NETWORK_MOMENT_INTERVAL_MS: u64 = 60_000;
const TYPING_FLUSH_INTERVAL: u64 = 250;

const NIGHT_CAP: &str = "cosmetic:night_cap";
const MEMORY_FRAME: &str = "decoration:memory_frame";
const HEADPHONES: &str = "cosmetic:headphones";
const AURORA: &str = "palette:aurora";
const CHARGING_ORB: &str = "decoration:charging_orb";
const SIGNAL_KITE: &str = "decoration:signal_kite";
const RARE_A: &str = "idle:rare_a";
const RARE_B: &str = "idle:rare_b";

#[derive(Debug, Serialize, Deserialize)]
struct CollectionData {
    schema_version: u32,
    first_seen_epoch_ms: u64,
    typing_events: u64,
    charging_sessions: u64,
    network_moments: u64,
    last_network_moment_epoch_ms: Option<u64>,
    discoveries: BTreeSet<String>,
    unlocked_ids: BTreeSet<String>,
}

impl CollectionData {
    fn new(now: u64) -> Self {
        Self {
            schema_version: COLLECTION_SCHEMA_VERSION,
            first_seen_epoch_ms: now,
            typing_events: 0,
            charging_sessions: 0,
            network_moments: 0,
            last_network_moment_epoch_ms: None,
            discoveries: BTreeSet::new(),
            unlocked_ids: BTreeSet::new(),
        }
    }
}

pub struct CollectionStore {
    path: PathBuf,
    data: CollectionData,
    previous_charging: Option<bool>,
    dirty: bool,
}

impl CollectionStore {
    pub fn load(path: PathBuf) -> Result<Self, ByteError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let now = now_epoch_ms();
        let data = match fs::read_to_string(&path) {
            Ok(raw) => serde_json::from_str::<CollectionData>(&raw)
                .ok()
                .filter(|value| value.schema_version == COLLECTION_SCHEMA_VERSION)
                .unwrap_or_else(|| CollectionData::new(now)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => CollectionData::new(now),
            Err(error) => return Err(error.into()),
        };

        let mut store = Self {
            path,
            data,
            previous_charging: None,
            dirty: false,
        };
        let changed = store.evaluate_unlocks(now);
        if changed || !store.path.exists() {
            store.save()?;
        }
        Ok(store)
    }

    pub fn snapshot(&mut self) -> Result<CollectionSnapshot, ByteError> {
        let now = now_epoch_ms();
        if self.evaluate_unlocks(now) {
            self.save()?;
        }
        Ok(self.build_snapshot(now))
    }

    pub fn record_typing(
        &mut self,
        timestamp_epoch_ms: u64,
    ) -> Result<Option<CollectionSnapshot>, ByteError> {
        self.data.typing_events = self.data.typing_events.saturating_add(1);
        self.dirty = true;
        let unlocked = self.evaluate_unlocks(timestamp_epoch_ms);

        if unlocked || self.data.typing_events % TYPING_FLUSH_INTERVAL == 0 {
            self.save()?;
        }

        Ok(unlocked.then(|| self.build_snapshot(timestamp_epoch_ms)))
    }

    pub fn observe_system(
        &mut self,
        snapshot: &SystemSnapshot,
    ) -> Result<Option<CollectionSnapshot>, ByteError> {
        let mut progress_changed = false;
        let charging = snapshot.battery.as_ref().map(|battery| battery.charging);

        if matches!(
            (self.previous_charging, charging),
            (Some(false), Some(true))
        ) {
            self.data.charging_sessions = self.data.charging_sessions.saturating_add(1);
            self.dirty = true;
            progress_changed = true;
        }
        self.previous_charging = charging;

        let throughput = snapshot.network.download_mbps + snapshot.network.upload_mbps;
        let network_due = self
            .data
            .last_network_moment_epoch_ms
            .map(|last| {
                snapshot.timestamp_epoch_ms.saturating_sub(last) >= NETWORK_MOMENT_INTERVAL_MS
            })
            .unwrap_or(true);

        if self.data.network_moments < NETWORK_TARGET
            && throughput >= NETWORK_MIN_MBPS
            && network_due
        {
            self.data.network_moments = self.data.network_moments.saturating_add(1);
            self.data.last_network_moment_epoch_ms = Some(snapshot.timestamp_epoch_ms);
            self.dirty = true;
            progress_changed = true;
        }

        let unlocked = self.evaluate_unlocks(snapshot.timestamp_epoch_ms);
        if progress_changed || unlocked {
            self.save()?;
        }

        Ok(unlocked.then(|| self.build_snapshot(snapshot.timestamp_epoch_ms)))
    }

    pub fn record_discovery(
        &mut self,
        discovery: CollectionDiscoveryKind,
    ) -> Result<(CollectionSnapshot, bool), ByteError> {
        let id = match discovery {
            CollectionDiscoveryKind::RareA => "rare_a",
            CollectionDiscoveryKind::RareB => "rare_b",
        };
        let inserted = self.data.discoveries.insert(id.into());
        if inserted {
            self.dirty = true;
        }

        let now = now_epoch_ms();
        let unlocked = self.evaluate_unlocks(now);
        if inserted || unlocked {
            self.save()?;
        }

        Ok((self.build_snapshot(now), unlocked))
    }

    pub fn validate_preferences(
        &self,
        preferences: &CompanionPreferences,
    ) -> Result<(), ByteError> {
        let customization = &preferences.customization;

        let gated = [
            (customization.headwear.as_str() == "night_cap", NIGHT_CAP),
            (customization.headwear.as_str() == "headphones", HEADPHONES),
            (
                customization.decorations.large_background.as_str() == "memory_frame",
                MEMORY_FRAME,
            ),
            (
                customization.decorations.small_prop.as_str() == "charging_orb",
                CHARGING_ORB,
            ),
            (
                customization.decorations.wall_or_sky.as_str() == "signal_kite",
                SIGNAL_KITE,
            ),
            (preferences.palette.as_str() == "aurora", AURORA),
        ];

        if let Some((_, unlock_id)) = gated
            .into_iter()
            .find(|(selected, unlock_id)| *selected && !self.data.unlocked_ids.contains(*unlock_id))
        {
            return Err(ByteError::Config(format!(
                "Customization item is not unlocked: {unlock_id}"
            )));
        }

        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), ByteError> {
        if self.dirty {
            self.save()?;
        }
        Ok(())
    }

    fn evaluate_unlocks(&mut self, now: u64) -> bool {
        let elapsed_days = now.saturating_sub(self.data.first_seen_epoch_ms) / DAY_MS;
        let candidates = [
            (elapsed_days >= WEEK_DAYS, NIGHT_CAP),
            (elapsed_days >= MONTH_DAYS, MEMORY_FRAME),
            (self.data.typing_events >= TYPING_FIRST, HEADPHONES),
            (self.data.typing_events >= TYPING_SECOND, AURORA),
            (self.data.charging_sessions >= CHARGING_TARGET, CHARGING_ORB),
            (self.data.network_moments >= NETWORK_TARGET, SIGNAL_KITE),
            (self.data.discoveries.contains("rare_a"), RARE_A),
            (self.data.discoveries.contains("rare_b"), RARE_B),
        ];

        let mut changed = false;
        for (condition, unlock_id) in candidates {
            if condition && self.data.unlocked_ids.insert(unlock_id.into()) {
                changed = true;
                self.dirty = true;
            }
        }
        changed
    }

    fn build_snapshot(&self, now: u64) -> CollectionSnapshot {
        let elapsed_days = now.saturating_sub(self.data.first_seen_epoch_ms) / DAY_MS;

        CollectionSnapshot {
            first_seen_epoch_ms: self.data.first_seen_epoch_ms,
            typing_events: self.data.typing_events,
            charging_sessions: self.data.charging_sessions,
            network_moments: self.data.network_moments,
            unlocked_ids: self.data.unlocked_ids.iter().cloned().collect(),
            items: vec![
                item(
                    NIGHT_CAP,
                    "night_cap",
                    CollectionItemKind::Cosmetic,
                    "Night Cap",
                    "A sleepy headwear extra for spending time with Byte.",
                    "Keep Byte around for one week.",
                    self.is_unlocked(NIGHT_CAP),
                    Some(elapsed_days.min(WEEK_DAYS)),
                    Some(WEEK_DAYS),
                ),
                item(
                    MEMORY_FRAME,
                    "memory_frame",
                    CollectionItemKind::Decoration,
                    "Memory Frame",
                    "A small anniversary keepsake for habitat backgrounds.",
                    "One month since Byte first joined you.",
                    self.is_unlocked(MEMORY_FRAME),
                    Some(elapsed_days.min(MONTH_DAYS)),
                    Some(MONTH_DAYS),
                ),
                item(
                    HEADPHONES,
                    "headphones",
                    CollectionItemKind::Cosmetic,
                    "Pixel Headphones",
                    "A typing-inspired headwear extra earned through normal computer use.",
                    "Type naturally while Byte is running.",
                    self.is_unlocked(HEADPHONES),
                    Some(self.data.typing_events.min(TYPING_FIRST)),
                    Some(TYPING_FIRST),
                ),
                item(
                    AURORA,
                    "aurora",
                    CollectionItemKind::Palette,
                    "Aurora Palette",
                    "An extra colorway available across all four companions.",
                    "A longer natural typing milestone.",
                    self.is_unlocked(AURORA),
                    Some(self.data.typing_events.min(TYPING_SECOND)),
                    Some(TYPING_SECOND),
                ),
                item(
                    CHARGING_ORB,
                    "charging_orb",
                    CollectionItemKind::Decoration,
                    "Charging Orb",
                    "A tiny glowing prop unlocked by ordinary charging sessions.",
                    "Let Byte notice a few normal charging sessions.",
                    self.is_unlocked(CHARGING_ORB),
                    Some(self.data.charging_sessions.min(CHARGING_TARGET)),
                    Some(CHARGING_TARGET),
                ),
                item(
                    SIGNAL_KITE,
                    "signal_kite",
                    CollectionItemKind::Decoration,
                    "Signal Kite",
                    "A connected-world decoration discovered through everyday network activity.",
                    "Use the computer normally while connected.",
                    self.is_unlocked(SIGNAL_KITE),
                    None,
                    None,
                ),
                item(
                    RARE_A,
                    "rare_a",
                    CollectionItemKind::Idle,
                    "Rare Idle I",
                    "Replay a rare animation after Byte performs it naturally.",
                    "Notice this rare idle in the desktop companion.",
                    self.is_unlocked(RARE_A),
                    None,
                    None,
                ),
                item(
                    RARE_B,
                    "rare_b",
                    CollectionItemKind::Idle,
                    "Rare Idle II",
                    "Replay a second rare animation after Byte performs it naturally.",
                    "Notice this rare idle in the desktop companion.",
                    self.is_unlocked(RARE_B),
                    None,
                    None,
                ),
            ],
        }
    }

    fn is_unlocked(&self, unlock_id: &str) -> bool {
        self.data.unlocked_ids.contains(unlock_id)
    }

    fn save(&mut self) -> Result<(), ByteError> {
        let parent = self.path.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent)?;
        let payload = serde_json::to_vec_pretty(&self.data)?;
        let mut temp = NamedTempFile::new_in(parent)?;
        temp.write_all(&payload)?;
        temp.as_file_mut().sync_all()?;
        temp.persist(&self.path)
            .map_err(|error| ByteError::Io(error.error.to_string()))?;
        self.dirty = false;
        Ok(())
    }
}

fn item(
    unlock_id: &str,
    item_id: &str,
    kind: CollectionItemKind,
    title: &str,
    description: &str,
    condition: &str,
    unlocked: bool,
    progress_current: Option<u64>,
    progress_target: Option<u64>,
) -> CollectionItemProgress {
    CollectionItemProgress {
        unlock_id: unlock_id.into(),
        item_id: item_id.into(),
        kind,
        title: title.into(),
        description: description.into(),
        condition: condition.into(),
        unlocked,
        progress_current,
        progress_target,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        BatterySummary, NetworkSummary, ResourceState, ResourceSummary, SystemStatus,
    };

    fn snapshot(time: u64, charging: bool, network: f32) -> SystemSnapshot {
        let normal = || ResourceSummary {
            value: 20.0,
            unit: "%".into(),
            state: ResourceState::Normal,
            available: None,
            available_unit: None,
        };
        SystemSnapshot {
            timestamp_epoch_ms: time,
            overall_status: SystemStatus::Calm,
            cpu: normal(),
            memory: normal(),
            storage: normal(),
            battery: Some(BatterySummary {
                percent: 80.0,
                charging,
                state: ResourceState::Normal,
            }),
            network: NetworkSummary {
                download_mbps: network,
                upload_mbps: 0.0,
            },
            thermal: None,
            primary_issue: None,
            secondary_issue_count: 0,
        }
    }

    #[test]
    fn core_customization_is_not_gated() {
        let temp = tempfile::tempdir().expect("temp dir");
        let store = CollectionStore::load(temp.path().join("collection.json")).expect("load");
        let mut preferences = CompanionPreferences::default();
        preferences.customization.headwear = "beanie".into();
        preferences.customization.decorations.ambient = "star_mobile".into();

        assert!(store.validate_preferences(&preferences).is_ok());
    }

    #[test]
    fn locked_phase_19_items_are_rejected() {
        let temp = tempfile::tempdir().expect("temp dir");
        let store = CollectionStore::load(temp.path().join("collection.json")).expect("load");
        let mut preferences = CompanionPreferences::default();
        preferences.customization.headwear = "night_cap".into();

        assert!(store.validate_preferences(&preferences).is_err());
    }

    #[test]
    fn charging_counts_only_false_to_true_transitions() {
        let temp = tempfile::tempdir().expect("temp dir");
        let mut store = CollectionStore::load(temp.path().join("collection.json")).expect("load");

        store
            .observe_system(&snapshot(1_000, true, 0.0))
            .expect("baseline");
        store
            .observe_system(&snapshot(2_000, true, 0.0))
            .expect("steady");
        assert_eq!(store.snapshot().expect("snapshot").charging_sessions, 0);

        store
            .observe_system(&snapshot(3_000, false, 0.0))
            .expect("battery");
        store
            .observe_system(&snapshot(4_000, true, 0.0))
            .expect("charge");
        assert_eq!(store.snapshot().expect("snapshot").charging_sessions, 1);
    }

    #[test]
    fn network_moments_are_time_separated_and_bounded() {
        let temp = tempfile::tempdir().expect("temp dir");
        let mut store = CollectionStore::load(temp.path().join("collection.json")).expect("load");

        store
            .observe_system(&snapshot(1_000, false, 2.0))
            .expect("first");
        store
            .observe_system(&snapshot(2_000, false, 2.0))
            .expect("too soon");
        store
            .observe_system(&snapshot(61_000, false, 2.0))
            .expect("second");

        assert_eq!(store.snapshot().expect("snapshot").network_moments, 2);
    }

    #[test]
    fn rare_discovery_unlocks_replay_without_gating_natural_idle() {
        let temp = tempfile::tempdir().expect("temp dir");
        let mut store = CollectionStore::load(temp.path().join("collection.json")).expect("load");

        let (snapshot, changed) = store
            .record_discovery(CollectionDiscoveryKind::RareA)
            .expect("discover");
        assert!(changed);
        assert!(snapshot.unlocked_ids.iter().any(|id| id == RARE_A));
    }
}
