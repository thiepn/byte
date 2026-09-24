use crate::models::{
    now_epoch_ms, AppAttribution, AppDiagnosticsSnapshot, AppUsageSummary, Confidence,
    IssueCategory, ProcessSummary,
};
use std::{
    collections::HashMap,
    thread,
    time::{Duration, Instant},
};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};

const BYTES_PER_MIB: f32 = 1_048_576.0;
const PROCESS_REFRESH_INTERVAL: Duration = Duration::from_secs(3);
const APP_INSPECTION_COOLDOWN: Duration = Duration::from_millis(750);
const FIRST_CPU_SAMPLE_DELAY: Duration = Duration::from_millis(250);
const APP_RESULT_CAP: usize = 12;

#[derive(Debug, Clone)]
pub struct Attribution {
    pub culprit: ProcessSummary,
    pub confidence: Confidence,
}

pub trait CulpritProvider: Send {
    fn refresh(&mut self);
    fn attribution(&self, category: IssueCategory) -> Option<Attribution>;
}

#[derive(Debug, Default, Clone)]
struct Aggregate {
    name: String,
    pid: Option<u32>,
    process_count: u32,
    cpu_percent: f32,
    memory_mb: f32,
}

pub struct ProcessAttributor {
    system: System,
    cpu_count: f32,
    aggregates: Vec<Aggregate>,
    last_refresh: Option<Instant>,
}

impl ProcessAttributor {
    pub fn new() -> Self {
        Self {
            system: System::new(),
            cpu_count: logical_cpu_count(),
            aggregates: Vec::new(),
            last_refresh: None,
        }
    }

    fn rebuild_aggregates(&mut self) {
        self.aggregates = aggregate_processes(&self.system, self.cpu_count);
    }

    fn top_cpu(&self) -> Option<Attribution> {
        attribution_from(&self.aggregates, |item| item.cpu_percent, 15.0, 0.50, 0.25)
    }

    fn top_memory(&self) -> Option<Attribution> {
        attribution_from(&self.aggregates, |item| item.memory_mb, 256.0, 0.35, 0.18)
    }
}

impl Default for ProcessAttributor {
    fn default() -> Self {
        Self::new()
    }
}

impl CulpritProvider for ProcessAttributor {
    fn refresh(&mut self) {
        if self
            .last_refresh
            .map(|last| last.elapsed() < PROCESS_REFRESH_INTERVAL)
            .unwrap_or(false)
        {
            return;
        }

        refresh_processes(&mut self.system);
        self.rebuild_aggregates();
        self.last_refresh = Some(Instant::now());
    }

    fn attribution(&self, category: IssueCategory) -> Option<Attribution> {
        match category {
            IssueCategory::Cpu => self.top_cpu(),
            IssueCategory::Memory => self.top_memory(),
            _ => None,
        }
    }
}

pub struct AppInspector {
    system: System,
    cpu_count: f32,
    primed: bool,
    last_scan: Option<Instant>,
    cached: Option<AppDiagnosticsSnapshot>,
}

impl AppInspector {
    pub fn new() -> Self {
        Self {
            system: System::new(),
            cpu_count: logical_cpu_count(),
            primed: false,
            last_scan: None,
            cached: None,
        }
    }

    pub fn inspect(&mut self) -> AppDiagnosticsSnapshot {
        if let (Some(last_scan), Some(cached)) = (self.last_scan, self.cached.as_ref()) {
            if last_scan.elapsed() < APP_INSPECTION_COOLDOWN {
                return cached.clone();
            }
        }

        refresh_processes(&mut self.system);

        if !self.primed {
            thread::sleep(FIRST_CPU_SAMPLE_DELAY);
            refresh_processes(&mut self.system);
            self.primed = true;
        }

        let aggregates = aggregate_processes(&self.system, self.cpu_count);
        let snapshot = build_app_snapshot(&aggregates);
        self.last_scan = Some(Instant::now());
        self.cached = Some(snapshot.clone());
        snapshot
    }
}

impl Default for AppInspector {
    fn default() -> Self {
        Self::new()
    }
}

fn logical_cpu_count() -> f32 {
    std::thread::available_parallelism()
        .map(|value| value.get() as f32)
        .unwrap_or(1.0)
}

fn refresh_processes(system: &mut System) {
    system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_cpu().with_memory(),
    );
}

fn aggregate_processes(system: &System, cpu_count: f32) -> Vec<Aggregate> {
    let mut groups: HashMap<String, Aggregate> = HashMap::new();

    for (pid, process) in system.processes() {
        let display_name = friendly_process_name(&process.name().to_string_lossy());
        if display_name.is_empty() {
            continue;
        }

        let key = display_name.to_lowercase();
        let entry = groups.entry(key).or_insert_with(|| Aggregate {
            name: display_name,
            pid: Some(pid.as_u32()),
            ..Aggregate::default()
        });

        entry.process_count = entry.process_count.saturating_add(1);
        if entry.process_count > 1 {
            entry.pid = None;
        }
        entry.cpu_percent += (process.cpu_usage() / cpu_count).max(0.0);
        entry.memory_mb += process.memory() as f32 / BYTES_PER_MIB;
    }

    groups.into_values().collect()
}

fn build_app_snapshot(aggregates: &[Aggregate]) -> AppDiagnosticsSnapshot {
    let total_cpu = positive_total(aggregates, |item| item.cpu_percent);
    let total_memory = positive_total(aggregates, |item| item.memory_mb);

    let cpu_leader = app_attribution(
        aggregates,
        |item| item.cpu_percent,
        15.0,
        0.50,
        0.25,
        total_cpu,
    );
    let memory_leader = app_attribution(
        aggregates,
        |item| item.memory_mb,
        256.0,
        0.35,
        0.18,
        total_memory,
    );

    let mut apps = aggregates
        .iter()
        .filter(|item| item.cpu_percent >= 0.1 || item.memory_mb >= 1.0)
        .map(|item| {
            let cpu_share = share(item.cpu_percent, total_cpu);
            let memory_share = share(item.memory_mb, total_memory);
            AppUsageSummary {
                name: item.name.clone(),
                process_count: item.process_count,
                cpu_percent: item.cpu_percent,
                memory_mb: item.memory_mb,
                cpu_share,
                memory_share,
                cpu_confidence: confidence_for(
                    item.cpu_percent,
                    cpu_share,
                    15.0,
                    0.50,
                    0.25,
                ),
                memory_confidence: confidence_for(
                    item.memory_mb,
                    memory_share,
                    256.0,
                    0.35,
                    0.18,
                ),
            }
        })
        .collect::<Vec<_>>();

    apps.sort_by(|left, right| {
        relevance(right)
            .total_cmp(&relevance(left))
            .then_with(|| right.memory_mb.total_cmp(&left.memory_mb))
            .then_with(|| left.name.cmp(&right.name))
    });
    apps.truncate(APP_RESULT_CAP);

    AppDiagnosticsSnapshot {
        timestamp_epoch_ms: now_epoch_ms(),
        apps,
        cpu_leader,
        memory_leader,
    }
}

fn relevance(item: &AppUsageSummary) -> f32 {
    item.cpu_share.max(item.memory_share)
}

fn positive_total(aggregates: &[Aggregate], metric: impl Fn(&Aggregate) -> f32) -> f32 {
    aggregates
        .iter()
        .map(metric)
        .filter(|value| value.is_finite() && *value > 0.0)
        .sum::<f32>()
}

fn share(value: f32, total: f32) -> f32 {
    if total <= 0.0 || !value.is_finite() {
        0.0
    } else {
        (value / total).clamp(0.0, 1.0)
    }
}

fn confidence_for(
    value: f32,
    share: f32,
    minimum_value: f32,
    high_share: f32,
    medium_share: f32,
) -> Option<Confidence> {
    if value < minimum_value {
        return None;
    }

    if share >= high_share {
        Some(Confidence::High)
    } else if share >= medium_share {
        Some(Confidence::Medium)
    } else {
        None
    }
}

fn app_attribution(
    aggregates: &[Aggregate],
    metric: impl Fn(&Aggregate) -> f32 + Copy,
    minimum_value: f32,
    high_share: f32,
    medium_share: f32,
    total: f32,
) -> Option<AppAttribution> {
    let top = aggregates
        .iter()
        .filter(|item| metric(item).is_finite())
        .max_by(|left, right| metric(left).total_cmp(&metric(right)))?;

    let value = metric(top);
    let share = share(value, total);
    let confidence = confidence_for(value, share, minimum_value, high_share, medium_share)?;

    Some(AppAttribution {
        name: top.name.clone(),
        confidence,
        share,
        value,
    })
}

fn attribution_from(
    aggregates: &[Aggregate],
    metric: impl Fn(&Aggregate) -> f32,
    minimum_value: f32,
    high_share: f32,
    medium_share: f32,
) -> Option<Attribution> {
    let total = positive_total(aggregates, &metric);

    let top = aggregates
        .iter()
        .filter(|item| metric(item).is_finite())
        .max_by(|left, right| metric(left).total_cmp(&metric(right)))?;

    let top_value = metric(top);
    let share = share(top_value, total);
    let confidence =
        confidence_for(top_value, share, minimum_value, high_share, medium_share)?;

    Some(Attribution {
        culprit: ProcessSummary {
            name: top.name.clone(),
            pid: top.pid,
            cpu_percent: Some(top.cpu_percent),
            memory_mb: Some(top.memory_mb),
        },
        confidence,
    })
}

fn friendly_process_name(raw: &str) -> String {
    let trimmed = raw.trim();
    let without_exe = trimmed
        .strip_suffix(".exe")
        .or_else(|| trimmed.strip_suffix(".EXE"))
        .unwrap_or(trimmed);
    without_exe.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn aggregate(name: &str, cpu: f32, memory: f32) -> Aggregate {
        Aggregate {
            name: name.into(),
            pid: Some(1),
            process_count: 1,
            cpu_percent: cpu,
            memory_mb: memory,
        }
    }

    #[test]
    fn strips_windows_executable_suffix() {
        assert_eq!(friendly_process_name("chrome.exe"), "chrome");
        assert_eq!(friendly_process_name("APP.EXE"), "APP");
    }

    #[test]
    fn refuses_to_blame_a_process_without_a_meaningful_share() {
        let values = vec![
            aggregate("A", 12.0, 100.0),
            aggregate("B", 11.0, 100.0),
            aggregate("C", 10.0, 100.0),
        ];

        assert!(attribution_from(&values, |item| item.cpu_percent, 15.0, 0.5, 0.25).is_none());
    }

    #[test]
    fn returns_high_confidence_for_dominant_process() {
        let values = vec![
            aggregate("Chrome", 60.0, 2000.0),
            aggregate("Other", 20.0, 500.0),
        ];

        let result =
            attribution_from(&values, |item| item.cpu_percent, 15.0, 0.5, 0.25).expect("culprit");
        assert_eq!(result.culprit.name, "Chrome");
        assert_eq!(result.confidence, Confidence::High);
    }

    #[test]
    fn app_snapshot_keeps_context_rows_without_accusing_every_app() {
        let values = vec![
            aggregate("Browser", 45.0, 1800.0),
            aggregate("Editor", 20.0, 1200.0),
            aggregate("Music", 2.0, 500.0),
        ];

        let result = build_app_snapshot(&values);
        assert_eq!(result.apps.len(), 3);
        assert_eq!(result.apps[0].name, "Browser");
        assert_eq!(result.cpu_leader.as_ref().map(|item| item.name.as_str()), Some("Browser"));
        assert!(result.apps[2].cpu_confidence.is_none());
    }

    #[test]
    fn app_snapshot_can_identify_different_cpu_and_memory_leaders() {
        let values = vec![
            aggregate("Compiler", 70.0, 400.0),
            aggregate("Browser", 10.0, 4000.0),
            aggregate("Other", 5.0, 500.0),
        ];

        let result = build_app_snapshot(&values);
        assert_eq!(result.cpu_leader.expect("cpu").name, "Compiler");
        assert_eq!(result.memory_leader.expect("memory").name, "Browser");
    }

    #[test]
    fn app_snapshot_is_capped() {
        let values = (0..30)
            .map(|index| aggregate(&format!("App{index}"), index as f32 + 1.0, 100.0 + index as f32))
            .collect::<Vec<_>>();

        assert_eq!(build_app_snapshot(&values).apps.len(), APP_RESULT_CAP);
    }
}
