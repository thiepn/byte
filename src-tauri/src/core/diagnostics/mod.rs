mod processes;
mod tracker;

pub use processes::AppInspector;

use crate::models::{
    Confidence, IssueCategory, RecommendedAction, RecommendedActionKind, ResourceState,
    ResourceSummary, SystemIssue, SystemSnapshot, SystemStatus,
};
use processes::{Attribution, CulpritProvider, ProcessAttributor};
use tracker::{ActiveCondition, Observation, SustainedTracker, TrackerPolicy};

const CPU_POLICY: TrackerPolicy = TrackerPolicy {
    high_activation_ms: 60_000,
    critical_activation_ms: 300_000,
    recovery_ms: 15_000,
    reopen_cooldown_ms: 30_000,
};
const MEMORY_POLICY: TrackerPolicy = TrackerPolicy {
    high_activation_ms: 15_000,
    critical_activation_ms: 30_000,
    recovery_ms: 15_000,
    reopen_cooldown_ms: 30_000,
};
const THERMAL_POLICY: TrackerPolicy = TrackerPolicy {
    high_activation_ms: 60_000,
    critical_activation_ms: 30_000,
    recovery_ms: 30_000,
    reopen_cooldown_ms: 60_000,
};
const BATTERY_POLICY: TrackerPolicy = TrackerPolicy {
    high_activation_ms: 0,
    critical_activation_ms: 0,
    recovery_ms: 0,
    reopen_cooldown_ms: 15_000,
};
const STORAGE_POLICY: TrackerPolicy = TrackerPolicy {
    high_activation_ms: 0,
    critical_activation_ms: 0,
    recovery_ms: 0,
    reopen_cooldown_ms: 60_000,
};

pub struct DiagnosticEngine {
    cpu: SustainedTracker,
    memory: SustainedTracker,
    thermal: SustainedTracker,
    battery: SustainedTracker,
    storage: SustainedTracker,
    culprits: Box<dyn CulpritProvider>,
    last_issues: Vec<SystemIssue>,
}

impl DiagnosticEngine {
    pub fn new() -> Self {
        Self::with_culprit_provider(Box::new(ProcessAttributor::new()))
    }

    fn with_culprit_provider(culprits: Box<dyn CulpritProvider>) -> Self {
        Self {
            cpu: SustainedTracker::default(),
            memory: SustainedTracker::default(),
            thermal: SustainedTracker::default(),
            battery: SustainedTracker::default(),
            storage: SustainedTracker::default(),
            culprits,
            last_issues: Vec::new(),
        }
    }

    pub fn evaluate(&mut self, mut snapshot: SystemSnapshot) -> SystemSnapshot {
        let now = snapshot.timestamp_epoch_ms;

        let cpu_observation = observe_cpu(&snapshot.cpu);
        let memory_observation = observe_memory(&snapshot.memory);
        let storage_observation = observe_storage(&snapshot.storage);
        let battery_observation = snapshot.battery.as_ref().map(observe_battery);
        let thermal_observation = snapshot.thermal.as_ref().map(observe_thermal);

        let cpu_active = if snapshot.cpu.state == ResourceState::Unknown {
            self.cpu.reset();
            None
        } else {
            self.cpu.update(now, cpu_observation, CPU_POLICY)
        };
        let memory_active = if snapshot.memory.state == ResourceState::Unknown {
            self.memory.reset();
            None
        } else {
            self.memory.update(now, memory_observation, MEMORY_POLICY)
        };
        let storage_active = if snapshot.storage.state == ResourceState::Unknown {
            self.storage.reset();
            None
        } else {
            self.storage
                .update(now, storage_observation, STORAGE_POLICY)
        };
        let battery_active = match battery_observation {
            Some(observation) => self.battery.update(now, observation, BATTERY_POLICY),
            None => {
                self.battery.reset();
                None
            }
        };
        let thermal_active = match thermal_observation {
            Some(observation) => self.thermal.update(now, observation, THERMAL_POLICY),
            None => {
                self.thermal.reset();
                None
            }
        };

        let needs_process_attribution = cpu_active.is_some() || memory_active.is_some();

        if needs_process_attribution {
            self.culprits.refresh();
        }

        snapshot.cpu.state =
            visible_resource_state(snapshot.cpu.state, cpu_observation, cpu_active);
        snapshot.memory.state =
            visible_resource_state(snapshot.memory.state, memory_observation, memory_active);
        snapshot.storage.state =
            visible_resource_state(snapshot.storage.state, storage_observation, storage_active);

        if let (Some(battery), Some(observation)) = (snapshot.battery.as_mut(), battery_observation)
        {
            battery.state = visible_resource_state(battery.state, observation, battery_active);
        }

        if let (Some(thermal), Some(observation)) = (snapshot.thermal.as_mut(), thermal_observation)
        {
            thermal.state = visible_resource_state(thermal.state, observation, thermal_active);
        }

        let mut issues = Vec::with_capacity(5);

        if let Some(active) = cpu_active {
            issues.push(cpu_issue(
                active,
                self.culprits.attribution(IssueCategory::Cpu),
            ));
        }
        if let Some(active) = memory_active {
            issues.push(memory_issue(
                active,
                self.culprits.attribution(IssueCategory::Memory),
            ));
        }
        if let Some(active) = storage_active {
            issues.push(storage_issue(active, &snapshot));
        }
        if let Some(active) = battery_active {
            issues.push(battery_issue(active, &snapshot));
        }
        if let Some(active) = thermal_active {
            issues.push(thermal_issue(active, &snapshot));
        }

        issues.sort_by_key(|issue| std::cmp::Reverse(issue_priority(issue)));

        snapshot.overall_status = if issues
            .iter()
            .any(|issue| issue.severity == ResourceState::Critical)
        {
            SystemStatus::NeedsAttention
        } else if !issues.is_empty() {
            SystemStatus::Stressed
        } else if cpu_observation.elevated {
            SystemStatus::Busy
        } else {
            SystemStatus::Calm
        };

        self.last_issues = issues.clone();
        snapshot.secondary_issue_count = issues.len().saturating_sub(1).min(u8::MAX as usize) as u8;
        snapshot.primary_issue = issues.into_iter().next();
        snapshot
    }

    pub fn active_issues(&self) -> &[SystemIssue] {
        &self.last_issues
    }
}

impl Default for DiagnosticEngine {
    fn default() -> Self {
        Self::new()
    }
}

fn observe_cpu(resource: &ResourceSummary) -> Observation {
    if resource.state == ResourceState::Unknown {
        return unavailable_observation();
    }

    Observation {
        elevated: resource.value >= 70.0,
        high: resource.value >= 95.0,
        critical: resource.value >= 99.0,
        recovered: resource.value <= 90.0,
    }
}

fn observe_memory(resource: &ResourceSummary) -> Observation {
    if resource.state == ResourceState::Unknown {
        return unavailable_observation();
    }

    let available = resource.available;
    let elevated = resource.value >= 88.0
        || (resource.value >= 80.0 && available.map(|value| value <= 4.0).unwrap_or(false));
    let high = resource.value >= 96.0
        || (resource.value >= 90.0 && available.map(|value| value <= 2.0).unwrap_or(false));
    let critical = resource.value >= 99.0
        || (resource.value >= 96.0 && available.map(|value| value <= 1.0).unwrap_or(false));
    let recovered = resource.value <= 88.0 || available.map(|value| value >= 2.5).unwrap_or(false);

    Observation {
        elevated,
        high,
        critical,
        recovered,
    }
}

fn observe_storage(resource: &ResourceSummary) -> Observation {
    if resource.state == ResourceState::Unknown || resource.available.is_none() {
        return unavailable_observation();
    }

    let available = resource.available.unwrap_or(f32::INFINITY);
    Observation {
        elevated: available <= 30.0 || resource.value >= 90.0,
        high: available <= 15.0 || resource.value >= 95.0,
        critical: available <= 5.0 || resource.value >= 98.0,
        recovered: available >= 18.0 && resource.value <= 94.0,
    }
}

fn observe_battery(battery: &crate::models::BatterySummary) -> Observation {
    if battery.charging {
        return Observation {
            elevated: false,
            high: false,
            critical: false,
            recovered: true,
        };
    }

    Observation {
        elevated: battery.percent <= 20.0,
        high: battery.percent <= 10.0,
        critical: battery.percent <= 5.0,
        recovered: battery.percent >= 12.0,
    }
}

fn observe_thermal(resource: &ResourceSummary) -> Observation {
    if resource.state == ResourceState::Unknown {
        return unavailable_observation();
    }

    Observation {
        elevated: resource.value >= 90.0,
        high: resource.value >= 100.0,
        critical: resource.value >= 110.0,
        recovered: resource.value <= 95.0,
    }
}

fn unavailable_observation() -> Observation {
    Observation {
        elevated: false,
        high: false,
        critical: false,
        recovered: false,
    }
}

fn visible_resource_state(
    original: ResourceState,
    observation: Observation,
    active: Option<ActiveCondition>,
) -> ResourceState {
    if original == ResourceState::Unknown {
        return ResourceState::Unknown;
    }

    if let Some(active) = active {
        return active.severity;
    }

    if observation.elevated || observation.high || observation.critical {
        ResourceState::Elevated
    } else {
        ResourceState::Normal
    }
}

fn cpu_issue(active: ActiveCondition, attribution: Option<Attribution>) -> SystemIssue {
    let (culprit, culprit_confidence, explanation) = attribution_parts(
        attribution,
        "is currently responsible for a large share of processor use.",
        "appears to be one of the main processor users right now.",
        "Processor usage has stayed extremely high for an extended period.",
    );

    SystemIssue {
        id: "cpu-pressure".into(),
        category: IssueCategory::Cpu,
        severity: active.severity,
        headline: if active.severity == ResourceState::Critical {
            "Processor usage has stayed extremely high".into()
        } else {
            "Your computer has been working very hard".into()
        },
        explanation,
        culprit,
        confidence: Confidence::High,
        culprit_confidence,
        recommended_action: Some(RecommendedAction {
            kind: RecommendedActionKind::OpenTaskManager,
            label: "Open Task Manager".into(),
        }),
        started_at_epoch_ms: active.started_at_epoch_ms,
    }
}

fn memory_issue(active: ActiveCondition, attribution: Option<Attribution>) -> SystemIssue {
    let (culprit, culprit_confidence, explanation) = attribution_parts(
        attribution,
        "is currently one of the largest memory users.",
        "appears to be using a significant share of memory.",
        "Available memory has stayed unusually low.",
    );

    SystemIssue {
        id: "memory-pressure".into(),
        category: IssueCategory::Memory,
        severity: active.severity,
        headline: if active.severity == ResourceState::Critical {
            "Memory is critically low".into()
        } else {
            "Memory is getting tight".into()
        },
        explanation,
        culprit,
        confidence: Confidence::High,
        culprit_confidence,
        recommended_action: Some(RecommendedAction {
            kind: RecommendedActionKind::OpenTaskManager,
            label: "Open Task Manager".into(),
        }),
        started_at_epoch_ms: active.started_at_epoch_ms,
    }
}

fn storage_issue(active: ActiveCondition, snapshot: &SystemSnapshot) -> SystemIssue {
    let available = snapshot.storage.available.unwrap_or(0.0);
    SystemIssue {
        id: "storage-pressure".into(),
        category: IssueCategory::Storage,
        severity: active.severity,
        headline: if active.severity == ResourceState::Critical {
            "Storage is almost full".into()
        } else {
            "Storage is getting low".into()
        },
        explanation: format!(
            "About {} remains available on the monitored drive.",
            format_gb(available)
        ),
        culprit: None,
        confidence: Confidence::High,
        culprit_confidence: None,
        recommended_action: Some(RecommendedAction {
            kind: RecommendedActionKind::OpenStorageSettings,
            label: "Open Storage Settings".into(),
        }),
        started_at_epoch_ms: active.started_at_epoch_ms,
    }
}

fn battery_issue(active: ActiveCondition, snapshot: &SystemSnapshot) -> SystemIssue {
    let percent = snapshot
        .battery
        .as_ref()
        .map(|battery| battery.percent)
        .unwrap_or(0.0);

    SystemIssue {
        id: "battery-low".into(),
        category: IssueCategory::Battery,
        severity: active.severity,
        headline: if active.severity == ResourceState::Critical {
            "Your battery is almost empty".into()
        } else {
            "Your battery is getting low".into()
        },
        explanation: format!("Battery level is about {}%.", percent.round()),
        culprit: None,
        confidence: Confidence::High,
        culprit_confidence: None,
        recommended_action: Some(RecommendedAction {
            kind: RecommendedActionKind::OpenBatterySettings,
            label: "Open Battery Settings".into(),
        }),
        started_at_epoch_ms: active.started_at_epoch_ms,
    }
}

fn thermal_issue(active: ActiveCondition, snapshot: &SystemSnapshot) -> SystemIssue {
    let temperature = snapshot
        .thermal
        .as_ref()
        .map(|thermal| thermal.value)
        .unwrap_or(0.0);

    SystemIssue {
        id: "thermal-pressure".into(),
        category: IssueCategory::Thermal,
        severity: active.severity,
        headline: if active.severity == ResourceState::Critical {
            "Your computer is running very hot".into()
        } else {
            "Your computer is running hotter than usual".into()
        },
        explanation: format!(
            "Byte has detected sustained high temperature readings around {:.0}°C.",
            temperature
        ),
        culprit: None,
        confidence: Confidence::Medium,
        culprit_confidence: None,
        recommended_action: Some(RecommendedAction {
            kind: RecommendedActionKind::ViewDetails,
            label: "View details".into(),
        }),
        started_at_epoch_ms: active.started_at_epoch_ms,
    }
}

fn attribution_parts(
    attribution: Option<Attribution>,
    high_confidence_suffix: &str,
    medium_confidence_suffix: &str,
    fallback: &str,
) -> (
    Option<crate::models::ProcessSummary>,
    Option<Confidence>,
    String,
) {
    match attribution {
        Some(attribution) => {
            let suffix = if attribution.confidence == Confidence::High {
                high_confidence_suffix
            } else {
                medium_confidence_suffix
            };
            let explanation = format!("{} {}", attribution.culprit.name, suffix);
            (
                Some(attribution.culprit),
                Some(attribution.confidence),
                explanation,
            )
        }
        None => (None, None, fallback.into()),
    }
}

fn issue_priority(issue: &SystemIssue) -> (u8, u8) {
    let severity = match issue.severity {
        ResourceState::Critical => 3,
        ResourceState::High => 2,
        ResourceState::Elevated => 1,
        _ => 0,
    };

    let category = match issue.category {
        IssueCategory::Thermal => 5,
        IssueCategory::Memory => 4,
        IssueCategory::Battery => 4,
        IssueCategory::Storage => 3,
        IssueCategory::Cpu => 2,
    };

    (severity, category)
}

fn format_gb(value: f32) -> String {
    if value < 10.0 {
        format!("{value:.1} GB")
    } else {
        format!("{value:.0} GB")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{BatterySummary, NetworkSummary};
    use processes::Attribution;

    struct MockCulprits {
        cpu: Option<Attribution>,
        memory: Option<Attribution>,
    }

    impl CulpritProvider for MockCulprits {
        fn refresh(&mut self) {}

        fn attribution(&self, category: IssueCategory) -> Option<Attribution> {
            match category {
                IssueCategory::Cpu => self.cpu.clone(),
                IssueCategory::Memory => self.memory.clone(),
                _ => None,
            }
        }
    }

    fn snapshot_at(time: u64) -> SystemSnapshot {
        SystemSnapshot {
            timestamp_epoch_ms: time,
            overall_status: SystemStatus::Calm,
            cpu: resource(10.0, None),
            memory: resource(50.0, Some(8.0)),
            storage: resource(50.0, Some(250.0)),
            battery: Some(BatterySummary {
                percent: 80.0,
                charging: false,
                state: ResourceState::Normal,
            }),
            network: NetworkSummary {
                download_mbps: 0.0,
                upload_mbps: 0.0,
            },
            thermal: Some(ResourceSummary {
                value: 50.0,
                unit: "°C".into(),
                state: ResourceState::Normal,
                available: None,
                available_unit: None,
            }),
            primary_issue: None,
            secondary_issue_count: 0,
        }
    }

    fn resource(value: f32, available: Option<f32>) -> ResourceSummary {
        ResourceSummary {
            value,
            unit: "%".into(),
            state: ResourceState::Normal,
            available,
            available_unit: available.map(|_| "GB".into()),
        }
    }

    fn engine() -> DiagnosticEngine {
        DiagnosticEngine::with_culprit_provider(Box::new(MockCulprits {
            cpu: None,
            memory: None,
        }))
    }

    #[test]
    fn high_cpu_is_busy_before_it_becomes_an_issue() {
        let mut engine = engine();
        let mut snapshot = snapshot_at(0);
        snapshot.cpu.value = 97.0;

        let first = engine.evaluate(snapshot.clone());
        assert_eq!(first.overall_status, SystemStatus::Busy);
        assert!(first.primary_issue.is_none());

        snapshot.timestamp_epoch_ms = 60_000;
        let sustained = engine.evaluate(snapshot);
        assert_eq!(sustained.overall_status, SystemStatus::Stressed);
        assert_eq!(
            sustained.primary_issue.expect("cpu issue").category,
            IssueCategory::Cpu
        );
    }

    #[test]
    fn transient_cpu_spike_does_not_open_issue() {
        let mut engine = engine();
        let mut spike = snapshot_at(0);
        spike.cpu.value = 99.0;
        assert_eq!(engine.evaluate(spike).overall_status, SystemStatus::Busy);

        let normal = snapshot_at(10_000);
        let result = engine.evaluate(normal);
        assert_eq!(result.overall_status, SystemStatus::Calm);
        assert!(result.primary_issue.is_none());
    }

    #[test]
    fn critical_memory_pressure_escalates_after_sustained_interval() {
        let mut engine = engine();
        let mut snapshot = snapshot_at(0);
        snapshot.memory.value = 99.2;
        snapshot.memory.available = Some(0.5);

        engine.evaluate(snapshot.clone());

        snapshot.timestamp_epoch_ms = 15_000;
        let high = engine.evaluate(snapshot.clone());
        assert_eq!(
            high.primary_issue.expect("high memory").severity,
            ResourceState::High
        );

        snapshot.timestamp_epoch_ms = 30_000;
        let critical = engine.evaluate(snapshot);
        assert_eq!(critical.overall_status, SystemStatus::NeedsAttention);
        assert_eq!(
            critical.primary_issue.expect("critical memory").severity,
            ResourceState::Critical
        );
    }

    #[test]
    fn recovery_requires_hysteresis_time() {
        let mut engine = engine();
        let mut pressure = snapshot_at(0);
        pressure.memory.value = 96.5;
        pressure.memory.available = Some(0.8);
        engine.evaluate(pressure.clone());

        pressure.timestamp_epoch_ms = 15_000;
        assert!(engine.evaluate(pressure).primary_issue.is_some());

        let mut recovered = snapshot_at(16_000);
        recovered.memory.value = 60.0;
        recovered.memory.available = Some(6.0);
        assert!(engine.evaluate(recovered.clone()).primary_issue.is_some());

        recovered.timestamp_epoch_ms = 31_000;
        assert!(engine.evaluate(recovered).primary_issue.is_none());
    }

    #[test]
    fn low_battery_is_immediate_and_actionable() {
        let mut engine = engine();
        let mut snapshot = snapshot_at(0);
        snapshot.battery = Some(BatterySummary {
            percent: 4.0,
            charging: false,
            state: ResourceState::Normal,
        });

        let result = engine.evaluate(snapshot);
        let issue = result.primary_issue.expect("battery issue");
        assert_eq!(result.overall_status, SystemStatus::NeedsAttention);
        assert_eq!(issue.category, IssueCategory::Battery);
        assert_eq!(
            issue.recommended_action.expect("action").kind,
            RecommendedActionKind::OpenBatterySettings
        );
    }

    #[test]
    fn unavailable_storage_never_becomes_an_issue() {
        let mut engine = engine();
        let mut snapshot = snapshot_at(0);
        snapshot.storage = ResourceSummary {
            value: 0.0,
            unit: "%".into(),
            state: ResourceState::Unknown,
            available: None,
            available_unit: None,
        };

        let result = engine.evaluate(snapshot);
        assert!(result.primary_issue.is_none());
        assert_eq!(result.storage.state, ResourceState::Unknown);
    }

    #[test]
    fn higher_severity_issue_wins_primary_priority() {
        let mut engine = engine();

        let mut cpu = snapshot_at(0);
        cpu.cpu.value = 97.0;
        engine.evaluate(cpu.clone());
        cpu.timestamp_epoch_ms = 60_000;
        engine.evaluate(cpu);

        let mut combined = snapshot_at(61_000);
        combined.cpu.value = 97.0;
        combined.battery = Some(BatterySummary {
            percent: 4.0,
            charging: false,
            state: ResourceState::Normal,
        });

        let result = engine.evaluate(combined);
        assert_eq!(
            result.primary_issue.expect("primary").category,
            IssueCategory::Battery
        );
        assert_eq!(result.secondary_issue_count, 1);
    }
}
