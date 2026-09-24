use crate::{
    core::error::ByteError,
    models::{
        AppPreferences, Confidence, IssueCategory, NotificationCategory, ResourceState, SystemIssue,
    },
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use tempfile::NamedTempFile;

const STATE_SCHEMA_VERSION: u32 = 1;
const MINUTE_MS: u64 = 60_000;
const HOUR_MS: u64 = 60 * MINUTE_MS;
const RUNAWAY_MIN_MS: u64 = 10 * MINUTE_MS;

#[derive(Debug, Clone)]
pub struct SmartNotification {
    pub category: NotificationCategory,
    pub fingerprint: String,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedState {
    schema_version: u32,
    last_sent_epoch_ms: BTreeMap<NotificationCategory, u64>,
}

impl Default for PersistedState {
    fn default() -> Self {
        Self {
            schema_version: STATE_SCHEMA_VERSION,
            last_sent_epoch_ms: BTreeMap::new(),
        }
    }
}

pub struct SmartNotificationEngine {
    path: PathBuf,
    persisted: PersistedState,
    handled_active: BTreeSet<String>,
}

impl SmartNotificationEngine {
    pub fn load(path: PathBuf) -> Result<Self, ByteError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let (persisted, recovered) = match fs::read_to_string(&path) {
            Ok(raw) => match serde_json::from_str::<PersistedState>(&raw)
                .ok()
                .filter(|value| value.schema_version == STATE_SCHEMA_VERSION)
            {
                Some(value) => (value, false),
                None => (PersistedState::default(), true),
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                (PersistedState::default(), true)
            }
            Err(error) => return Err(error.into()),
        };

        let engine = Self {
            path,
            persisted,
            handled_active: BTreeSet::new(),
        };

        if recovered {
            engine.save()?;
        }

        Ok(engine)
    }

    pub fn evaluate(
        &mut self,
        now: u64,
        issues: &[SystemIssue],
        preferences: &AppPreferences,
    ) -> Option<SmartNotification> {
        let candidates = eligible_candidates(now, issues, preferences);
        let current = candidates
            .iter()
            .map(|candidate| candidate.fingerprint.clone())
            .collect::<BTreeSet<_>>();
        self.handled_active
            .retain(|fingerprint| current.contains(fingerprint));

        if !preferences.notifications_enabled
            || preferences.notification_quiet_mode
            || preferences
                .notification_snoozed_until_epoch_ms
                .map(|until| until > now)
                .unwrap_or(false)
        {
            return None;
        }

        let mut selected = None;

        for candidate in candidates {
            if self.handled_active.contains(&candidate.fingerprint) {
                continue;
            }

            let cooldown = cooldown_ms(candidate.category);
            let in_cooldown = self
                .persisted
                .last_sent_epoch_ms
                .get(&candidate.category)
                .map(|last| now.saturating_sub(*last) < cooldown)
                .unwrap_or(false);

            if in_cooldown {
                self.handled_active.insert(candidate.fingerprint);
                continue;
            }

            if selected.is_none() {
                selected = Some(candidate);
            } else {
                // One OS alert is enough for a simultaneous incident. Mark
                // lower-priority candidates as handled until they recover.
                self.handled_active.insert(candidate.fingerprint);
            }
        }

        selected
    }

    pub fn mark_sent(
        &mut self,
        notification: &SmartNotification,
        now: u64,
    ) -> Result<(), ByteError> {
        self.handled_active.insert(notification.fingerprint.clone());
        self.persisted
            .last_sent_epoch_ms
            .insert(notification.category, now);
        self.save()
    }

    fn save(&self) -> Result<(), ByteError> {
        let parent = self.path.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent)?;
        let payload = serde_json::to_vec_pretty(&self.persisted)?;
        let mut temp = NamedTempFile::new_in(parent)?;
        temp.write_all(&payload)?;
        temp.as_file_mut().sync_all()?;
        temp.persist(&self.path)
            .map_err(|error| ByteError::Io(error.error.to_string()))?;
        Ok(())
    }
}

fn eligible_candidates(
    now: u64,
    issues: &[SystemIssue],
    preferences: &AppPreferences,
) -> Vec<SmartNotification> {
    let mut candidates = issues
        .iter()
        .filter_map(|issue| candidate_for_issue(now, issue, preferences))
        .collect::<Vec<_>>();

    candidates.sort_by_key(|candidate| category_priority(candidate.category));
    candidates
}

fn candidate_for_issue(
    now: u64,
    issue: &SystemIssue,
    preferences: &AppPreferences,
) -> Option<SmartNotification> {
    let category = match issue.category {
        IssueCategory::Memory
            if preferences.notification_memory_enabled
                && issue.severity == ResourceState::Critical =>
        {
            NotificationCategory::Memory
        }
        IssueCategory::Thermal
            if preferences.notification_thermal_enabled
                && matches!(
                    issue.severity,
                    ResourceState::High | ResourceState::Critical
                ) =>
        {
            NotificationCategory::Thermal
        }
        IssueCategory::Storage
            if preferences.notification_storage_enabled
                && issue.severity == ResourceState::Critical =>
        {
            NotificationCategory::Storage
        }
        IssueCategory::Battery
            if preferences.notification_battery_enabled
                && issue.severity == ResourceState::Critical =>
        {
            NotificationCategory::Battery
        }
        IssueCategory::Cpu
            if preferences.notification_runaway_process_enabled
                && issue.severity == ResourceState::Critical
                && now.saturating_sub(issue.started_at_epoch_ms) >= RUNAWAY_MIN_MS
                && issue.culprit.is_some()
                && matches!(
                    issue.culprit_confidence,
                    Some(Confidence::Medium | Confidence::High)
                ) =>
        {
            NotificationCategory::RunawayProcess
        }
        _ => return None,
    };

    Some(build_notification(category, issue))
}

fn build_notification(category: NotificationCategory, issue: &SystemIssue) -> SmartNotification {
    let fingerprint = format!("{:?}:{}:{}", category, issue.id, issue.started_at_epoch_ms);

    let title = match category {
        NotificationCategory::Memory => "Byte: memory is critically low".into(),
        NotificationCategory::Thermal => "Byte: your computer is running very hot".into(),
        NotificationCategory::Storage => "Byte: storage is almost full".into(),
        NotificationCategory::Battery => "Byte: battery is almost empty".into(),
        NotificationCategory::RunawayProcess => issue
            .culprit
            .as_ref()
            .map(|culprit| format!("Byte: {} has kept the CPU busy", culprit.name))
            .unwrap_or_else(|| "Byte: processor pressure has persisted".into()),
    };

    let next_step = issue
        .recommended_action
        .as_ref()
        .map(|action| format!(" Next: {}.", action.label))
        .unwrap_or_else(|| " Open Byte for details.".into());

    let body = match category {
        NotificationCategory::RunawayProcess => {
            let culprit = issue
                .culprit
                .as_ref()
                .map(|value| value.name.as_str())
                .unwrap_or("An app");
            format!(
                "{culprit} has remained a major processor user for about 10 minutes.{next_step}"
            )
        }
        _ => format!("{}{}", issue.explanation, next_step),
    };

    SmartNotification {
        category,
        fingerprint,
        title,
        body,
    }
}

fn category_priority(category: NotificationCategory) -> u8 {
    match category {
        NotificationCategory::Thermal => 0,
        NotificationCategory::Memory => 1,
        NotificationCategory::Battery => 2,
        NotificationCategory::Storage => 3,
        NotificationCategory::RunawayProcess => 4,
    }
}

fn cooldown_ms(category: NotificationCategory) -> u64 {
    match category {
        NotificationCategory::Thermal => 30 * MINUTE_MS,
        NotificationCategory::Battery => HOUR_MS,
        NotificationCategory::Memory => 4 * HOUR_MS,
        NotificationCategory::RunawayProcess => 4 * HOUR_MS,
        NotificationCategory::Storage => 24 * HOUR_MS,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ProcessSummary, RecommendedAction, RecommendedActionKind};

    fn issue(category: IssueCategory, severity: ResourceState, started: u64) -> SystemIssue {
        SystemIssue {
            id: format!("{category:?}-issue"),
            category,
            severity,
            headline: "Headline".into(),
            explanation: "Plain-language explanation.".into(),
            culprit: None,
            confidence: Confidence::High,
            culprit_confidence: None,
            recommended_action: Some(RecommendedAction {
                kind: RecommendedActionKind::ViewDetails,
                label: "View details".into(),
            }),
            started_at_epoch_ms: started,
        }
    }

    fn engine() -> (tempfile::TempDir, SmartNotificationEngine) {
        let temp = tempfile::tempdir().expect("temp");
        let engine =
            SmartNotificationEngine::load(temp.path().join("notifications.json")).expect("load");
        (temp, engine)
    }

    #[test]
    fn only_critical_memory_is_eligible() {
        let (_temp, mut engine) = engine();
        let prefs = AppPreferences::default();
        assert!(engine
            .evaluate(
                1_000,
                &[issue(IssueCategory::Memory, ResourceState::High, 0)],
                &prefs
            )
            .is_none());
        assert_eq!(
            engine
                .evaluate(
                    2_000,
                    &[issue(IssueCategory::Memory, ResourceState::Critical, 0)],
                    &prefs
                )
                .expect("critical")
                .category,
            NotificationCategory::Memory
        );
    }

    #[test]
    fn serious_thermal_pressure_can_alert_at_high() {
        let (_temp, mut engine) = engine();
        let result = engine
            .evaluate(
                1_000,
                &[issue(IssueCategory::Thermal, ResourceState::High, 0)],
                &AppPreferences::default(),
            )
            .expect("thermal");
        assert_eq!(result.category, NotificationCategory::Thermal);
    }

    #[test]
    fn runaway_process_requires_time_and_confident_culprit() {
        let (_temp, mut engine) = engine();
        let mut cpu = issue(IssueCategory::Cpu, ResourceState::Critical, 0);
        cpu.culprit = Some(ProcessSummary {
            name: "Compiler".into(),
            pid: Some(1),
            cpu_percent: Some(80.0),
            memory_mb: Some(500.0),
        });
        cpu.culprit_confidence = Some(Confidence::High);

        assert!(engine
            .evaluate(5 * MINUTE_MS, &[cpu.clone()], &AppPreferences::default())
            .is_none());
        assert_eq!(
            engine
                .evaluate(10 * MINUTE_MS, &[cpu], &AppPreferences::default())
                .expect("runaway")
                .category,
            NotificationCategory::RunawayProcess
        );
    }

    #[test]
    fn quiet_mode_and_snooze_suppress_without_consuming_issue() {
        let (_temp, mut engine) = engine();
        let memory = issue(IssueCategory::Memory, ResourceState::Critical, 0);
        let mut prefs = AppPreferences::default();
        prefs.notification_quiet_mode = true;
        assert!(engine.evaluate(1_000, &[memory.clone()], &prefs).is_none());

        prefs.notification_quiet_mode = false;
        prefs.notification_snoozed_until_epoch_ms = Some(10_000);
        assert!(engine.evaluate(2_000, &[memory.clone()], &prefs).is_none());

        prefs.notification_snoozed_until_epoch_ms = None;
        assert!(engine.evaluate(11_000, &[memory], &prefs).is_some());
    }

    #[test]
    fn per_category_switch_disables_only_that_category() {
        let (_temp, mut engine) = engine();
        let mut prefs = AppPreferences::default();
        prefs.notification_battery_enabled = false;

        assert!(engine
            .evaluate(
                1_000,
                &[issue(IssueCategory::Battery, ResourceState::Critical, 0)],
                &prefs
            )
            .is_none());
        assert!(engine
            .evaluate(
                1_000,
                &[issue(IssueCategory::Storage, ResourceState::Critical, 0)],
                &prefs
            )
            .is_some());
    }

    #[test]
    fn duplicate_active_issue_is_suppressed_and_recovery_clears_fingerprint() {
        let (_temp, mut engine) = engine();
        let prefs = AppPreferences::default();
        let memory = issue(IssueCategory::Memory, ResourceState::Critical, 0);
        let first = engine
            .evaluate(1_000, &[memory.clone()], &prefs)
            .expect("first");
        engine.mark_sent(&first, 1_000).expect("record");

        assert!(engine.evaluate(2_000, &[memory], &prefs).is_none());
        assert!(engine.evaluate(3_000, &[], &prefs).is_none());
    }

    #[test]
    fn category_cooldown_survives_engine_reload() {
        let temp = tempfile::tempdir().expect("temp");
        let path = temp.path().join("notifications.json");
        let prefs = AppPreferences::default();

        {
            let mut engine = SmartNotificationEngine::load(path.clone()).expect("load");
            let first = engine
                .evaluate(
                    1_000,
                    &[issue(IssueCategory::Storage, ResourceState::Critical, 0)],
                    &prefs,
                )
                .expect("first");
            engine.mark_sent(&first, 1_000).expect("record");
        }

        let mut reloaded = SmartNotificationEngine::load(path).expect("reload");
        let reopened = issue(IssueCategory::Storage, ResourceState::Critical, 2_000);
        assert!(reloaded.evaluate(3_000, &[reopened], &prefs).is_none());
    }

    #[test]
    fn simultaneous_conditions_produce_only_highest_priority_alert() {
        let (_temp, mut engine) = engine();
        let prefs = AppPreferences::default();
        let result = engine
            .evaluate(
                1_000,
                &[
                    issue(IssueCategory::Storage, ResourceState::Critical, 0),
                    issue(IssueCategory::Thermal, ResourceState::High, 0),
                    issue(IssueCategory::Memory, ResourceState::Critical, 0),
                ],
                &prefs,
            )
            .expect("alert");
        assert_eq!(result.category, NotificationCategory::Thermal);
    }
}
