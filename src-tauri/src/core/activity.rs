use crate::{
    core::{
        error::ByteError,
        persistence::{read_bounded_text, BoundedText},
    },
    models::{IssueCategory, ResourceState, SystemIssue, SystemSnapshot},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use tempfile::NamedTempFile;

const EVENT_CAP: usize = 200;
const TREND_CAP: usize = 240;
const TREND_INTERVAL_MS: u64 = 15_000;
const ACTIVITY_SCHEMA_VERSION: u32 = 1;
const MAX_ACTIVITY_FILE_BYTES: u64 = 512 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ActivityEventKind {
    IssueOpened,
    IssueResolved,
    Power,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ActivityTone {
    Normal,
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityEvent {
    pub id: u64,
    pub timestamp_epoch_ms: u64,
    pub kind: ActivityEventKind,
    pub tone: ActivityTone,
    pub title: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendPoint {
    pub timestamp_epoch_ms: u64,
    pub cpu_percent: f32,
    pub memory_percent: f32,
    pub storage_percent: f32,
    pub battery_percent: Option<f32>,
    pub network_mbps: f32,
    pub thermal_c: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivitySnapshot {
    pub events: Vec<ActivityEvent>,
    pub trends: Vec<TrendPoint>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedActivity {
    schema_version: u32,
    events: Vec<ActivityEvent>,
}

pub struct ActivityStore {
    path: PathBuf,
    events: VecDeque<ActivityEvent>,
    trends: VecDeque<TrendPoint>,
    previous: Option<SystemSnapshot>,
    last_trend_at: Option<u64>,
    next_id: u64,
}

impl ActivityStore {
    pub fn load(path: PathBuf) -> Result<Self, ByteError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let events = match read_bounded_text(&path, MAX_ACTIVITY_FILE_BYTES)? {
            BoundedText::Present(raw) => serde_json::from_str::<PersistedActivity>(&raw)
                .ok()
                .filter(|value| value.schema_version == ACTIVITY_SCHEMA_VERSION)
                .map(|value| {
                    value
                        .events
                        .into_iter()
                        .rev()
                        .take(EVENT_CAP)
                        .collect::<Vec<_>>()
                        .into_iter()
                        .rev()
                        .collect::<VecDeque<_>>()
                })
                .unwrap_or_default(),
            BoundedText::Invalid | BoundedText::Missing => VecDeque::new(),
        };

        let next_id = events
            .back()
            .map(|event| event.id.saturating_add(1))
            .unwrap_or(1);

        Ok(Self {
            path,
            events,
            trends: VecDeque::new(),
            previous: None,
            last_trend_at: None,
            next_id,
        })
    }

    pub fn snapshot(&self) -> ActivitySnapshot {
        ActivitySnapshot {
            events: self.events.iter().rev().cloned().collect(),
            trends: self.trends.iter().cloned().collect(),
        }
    }

    pub fn clear(&mut self) -> Result<ActivitySnapshot, ByteError> {
        self.events.clear();
        self.trends.clear();
        self.previous = None;
        self.last_trend_at = None;
        self.next_id = 1;
        self.save()?;
        Ok(self.snapshot())
    }

    pub fn record(
        &mut self,
        snapshot: &SystemSnapshot,
        history_enabled: bool,
    ) -> Result<(), ByteError> {
        let ready = snapshot.cpu.state != ResourceState::Unknown
            || snapshot.memory.state != ResourceState::Unknown;

        if ready && history_enabled {
            self.record_trend(snapshot);
        }

        let mut changed = false;
        if ready && history_enabled {
            if let Some(previous) = self.previous.clone().filter(snapshot_ready) {
                changed |= self.record_issue_changes(&previous, snapshot);
                changed |= self.record_power_change(&previous, snapshot);
            }
        }

        self.previous = Some(snapshot.clone());

        if changed {
            self.save()?;
        }

        Ok(())
    }

    fn record_trend(&mut self, snapshot: &SystemSnapshot) {
        let should_record = self
            .last_trend_at
            .map(|last| snapshot.timestamp_epoch_ms.saturating_sub(last) >= TREND_INTERVAL_MS)
            .unwrap_or(true);

        if !should_record {
            return;
        }

        self.trends.push_back(TrendPoint {
            timestamp_epoch_ms: snapshot.timestamp_epoch_ms,
            cpu_percent: snapshot.cpu.value,
            memory_percent: snapshot.memory.value,
            storage_percent: snapshot.storage.value,
            battery_percent: snapshot.battery.as_ref().map(|battery| battery.percent),
            network_mbps: snapshot.network.download_mbps + snapshot.network.upload_mbps,
            thermal_c: snapshot.thermal.as_ref().and_then(|thermal| {
                (thermal.state != ResourceState::Unknown).then_some(thermal.value)
            }),
        });
        self.last_trend_at = Some(snapshot.timestamp_epoch_ms);

        while self.trends.len() > TREND_CAP {
            self.trends.pop_front();
        }
    }

    fn record_issue_changes(
        &mut self,
        previous: &SystemSnapshot,
        current: &SystemSnapshot,
    ) -> bool {
        let previous_id = previous
            .primary_issue
            .as_ref()
            .map(|issue| issue.id.as_str());
        let current_id = current
            .primary_issue
            .as_ref()
            .map(|issue| issue.id.as_str());

        if previous_id == current_id {
            return false;
        }

        let mut changed = false;

        if let Some(issue) = previous.primary_issue.as_ref() {
            self.push_event(
                current.timestamp_epoch_ms,
                ActivityEventKind::IssueResolved,
                ActivityTone::Normal,
                format!("Resolved: {}", issue.headline),
                "The condition is no longer active.".into(),
            );
            changed = true;
        }

        if let Some(issue) = current.primary_issue.as_ref() {
            self.push_event(
                current.timestamp_epoch_ms,
                ActivityEventKind::IssueOpened,
                issue_tone(issue),
                issue.headline.clone(),
                persisted_issue_detail(issue),
            );
            changed = true;
        }

        changed
    }

    fn record_power_change(&mut self, previous: &SystemSnapshot, current: &SystemSnapshot) -> bool {
        let previous_charging = previous.battery.as_ref().map(|battery| battery.charging);
        let current_charging = current.battery.as_ref().map(|battery| battery.charging);

        if previous_charging == current_charging {
            return false;
        }

        match current_charging {
            Some(true) => self.push_event(
                current.timestamp_epoch_ms,
                ActivityEventKind::Power,
                ActivityTone::Normal,
                "Charging started".into(),
                current
                    .battery
                    .as_ref()
                    .map(|battery| format!("Battery is at about {:.0}%.", battery.percent))
                    .unwrap_or_else(|| "Byte detected external power.".into()),
            ),
            Some(false) => self.push_event(
                current.timestamp_epoch_ms,
                ActivityEventKind::Power,
                ActivityTone::Info,
                "Running on battery".into(),
                current
                    .battery
                    .as_ref()
                    .map(|battery| format!("Battery is at about {:.0}%.", battery.percent))
                    .unwrap_or_else(|| "External power was disconnected.".into()),
            ),
            None => return false,
        }

        true
    }

    fn push_event(
        &mut self,
        timestamp_epoch_ms: u64,
        kind: ActivityEventKind,
        tone: ActivityTone,
        title: String,
        detail: String,
    ) {
        self.events.push_back(ActivityEvent {
            id: self.next_id,
            timestamp_epoch_ms,
            kind,
            tone,
            title,
            detail,
        });
        self.next_id = self.next_id.saturating_add(1);

        while self.events.len() > EVENT_CAP {
            self.events.pop_front();
        }
    }

    fn save(&self) -> Result<(), ByteError> {
        let parent = self.path.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent)?;

        let payload = PersistedActivity {
            schema_version: ACTIVITY_SCHEMA_VERSION,
            events: self.events.iter().cloned().collect(),
        };
        let bytes = serde_json::to_vec_pretty(&payload)?;
        let mut temp = NamedTempFile::new_in(parent)?;
        temp.write_all(&bytes)?;
        temp.as_file_mut().sync_all()?;
        temp.persist(&self.path)
            .map_err(|error| ByteError::Io(error.error.to_string()))?;
        Ok(())
    }
}

fn persisted_issue_detail(issue: &SystemIssue) -> String {
    match issue.category {
        IssueCategory::Cpu => {
            "Byte detected sustained processor pressure. Current app attribution is not stored in Activity history.".into()
        }
        IssueCategory::Memory => {
            "Byte detected sustained memory pressure. Current app attribution is not stored in Activity history.".into()
        }
        _ => issue.explanation.clone(),
    }
}

fn snapshot_ready(snapshot: &SystemSnapshot) -> bool {
    snapshot.cpu.state != ResourceState::Unknown || snapshot.memory.state != ResourceState::Unknown
}

fn issue_tone(issue: &SystemIssue) -> ActivityTone {
    match issue.severity {
        ResourceState::Critical => ActivityTone::Critical,
        ResourceState::High => ActivityTone::Warning,
        ResourceState::Elevated => ActivityTone::Info,
        ResourceState::Normal | ResourceState::Unknown => ActivityTone::Info,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        BatterySummary, Confidence, IssueCategory, NetworkSummary, ProcessSummary,
        RecommendedAction, RecommendedActionKind, ResourceSummary, SystemStatus,
    };

    fn resource(value: f32, state: ResourceState) -> ResourceSummary {
        ResourceSummary {
            value,
            unit: "%".into(),
            state,
            available: None,
            available_unit: None,
        }
    }

    fn snapshot(time: u64) -> SystemSnapshot {
        SystemSnapshot {
            timestamp_epoch_ms: time,
            overall_status: SystemStatus::Calm,
            cpu: resource(20.0, ResourceState::Normal),
            memory: resource(45.0, ResourceState::Normal),
            storage: resource(50.0, ResourceState::Normal),
            battery: Some(BatterySummary {
                percent: 80.0,
                charging: false,
                state: ResourceState::Normal,
            }),
            network: NetworkSummary {
                download_mbps: 1.0,
                upload_mbps: 0.5,
            },
            thermal: None,
            primary_issue: None,
            secondary_issue_count: 0,
        }
    }

    fn issue() -> SystemIssue {
        SystemIssue {
            id: "memory-pressure".into(),
            category: IssueCategory::Memory,
            severity: ResourceState::High,
            headline: "Memory is getting tight".into(),
            explanation: "Available memory has stayed unusually low.".into(),
            culprit: None,
            confidence: Confidence::High,
            culprit_confidence: None,
            recommended_action: Some(RecommendedAction {
                kind: RecommendedActionKind::OpenTaskManager,
                label: "Open Task Manager".into(),
            }),
            started_at_epoch_ms: 20_000,
        }
    }

    #[test]
    fn persisted_cpu_and_memory_events_omit_process_names() {
        let mut current = issue();
        current.category = IssueCategory::Cpu;
        current.culprit = Some(ProcessSummary {
            name: "SensitiveApp".into(),
            pid: Some(7),
            cpu_percent: Some(80.0),
            memory_mb: Some(500.0),
        });
        current.explanation = "SensitiveApp is currently responsible for CPU use.".into();

        let detail = persisted_issue_detail(&current);
        assert!(!detail.contains("SensitiveApp"));
        assert!(detail.contains("not stored"));
    }

    #[test]
    fn records_meaningful_events_but_not_every_snapshot() {
        let temp = tempfile::tempdir().expect("temp dir");
        let mut store = ActivityStore::load(temp.path().join("activity.json")).expect("load");

        store.record(&snapshot(1_000), true).expect("first");
        store.record(&snapshot(2_000), true).expect("second");
        assert!(store.snapshot().events.is_empty());

        let mut pressured = snapshot(20_000);
        pressured.primary_issue = Some(issue());
        store.record(&pressured, true).expect("issue");

        assert_eq!(store.snapshot().events.len(), 1);
        assert_eq!(
            store.snapshot().events[0].kind,
            ActivityEventKind::IssueOpened
        );
    }

    #[test]
    fn records_issue_resolution_and_power_changes() {
        let temp = tempfile::tempdir().expect("temp dir");
        let mut store = ActivityStore::load(temp.path().join("activity.json")).expect("load");

        let mut pressured = snapshot(10_000);
        pressured.primary_issue = Some(issue());
        store.record(&pressured, true).expect("start");

        let mut resolved = snapshot(30_000);
        resolved.battery.as_mut().expect("battery").charging = true;
        store.record(&resolved, true).expect("resolve");

        let events = store.snapshot().events;
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].kind, ActivityEventKind::Power);
        assert_eq!(events[1].kind, ActivityEventKind::IssueResolved);
    }

    #[test]
    fn trend_sampling_is_bounded_and_interval_limited() {
        let temp = tempfile::tempdir().expect("temp dir");
        let mut store = ActivityStore::load(temp.path().join("activity.json")).expect("load");

        store.record(&snapshot(0), true).expect("first");
        store.record(&snapshot(5_000), true).expect("too soon");
        store.record(&snapshot(15_000), true).expect("interval");

        assert_eq!(store.snapshot().trends.len(), 2);
    }

    #[test]
    fn disabling_history_suppresses_events_and_trends() {
        let temp = tempfile::tempdir().expect("temp dir");
        let mut store = ActivityStore::load(temp.path().join("activity.json")).expect("load");

        store.record(&snapshot(1_000), false).expect("first");
        let mut next = snapshot(20_000);
        next.primary_issue = Some(issue());
        store.record(&next, false).expect("disabled");

        let result = store.snapshot();
        assert!(result.events.is_empty());
        assert!(result.trends.is_empty());
    }

    #[test]
    fn persisted_events_reload_but_session_trends_do_not() {
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("activity.json");

        {
            let mut store = ActivityStore::load(path.clone()).expect("load");
            store.record(&snapshot(1_000), true).expect("first");
            let mut next = snapshot(20_000);
            next.battery.as_mut().expect("battery").charging = true;
            store.record(&next, true).expect("power");
            assert!(!store.snapshot().trends.is_empty());
        }

        let reloaded = ActivityStore::load(path).expect("reload");
        assert_eq!(reloaded.snapshot().events.len(), 1);
        assert!(reloaded.snapshot().trends.is_empty());
    }
}
