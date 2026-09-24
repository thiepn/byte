use crate::models::{Confidence, IssueCategory, ProcessSummary};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};

const BYTES_PER_MIB: f32 = 1_048_576.0;
const PROCESS_REFRESH_INTERVAL: Duration = Duration::from_secs(3);

#[derive(Debug, Clone)]
pub struct Attribution {
    pub culprit: ProcessSummary,
    pub confidence: Confidence,
}

pub trait CulpritProvider: Send {
    fn refresh(&mut self);
    fn attribution(&self, category: IssueCategory) -> Option<Attribution>;
}

#[derive(Debug, Default)]
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
            cpu_count: std::thread::available_parallelism()
                .map(|value| value.get() as f32)
                .unwrap_or(1.0),
            aggregates: Vec::new(),
            last_refresh: None,
        }
    }

    fn rebuild_aggregates(&mut self) {
        let mut groups: HashMap<String, Aggregate> = HashMap::new();

        for (pid, process) in self.system.processes() {
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
            entry.cpu_percent += (process.cpu_usage() / self.cpu_count).max(0.0);
            entry.memory_mb += process.memory() as f32 / BYTES_PER_MIB;
        }

        self.aggregates = groups.into_values().collect();
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

        self.system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing().with_cpu().with_memory(),
        );
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

fn attribution_from(
    aggregates: &[Aggregate],
    metric: impl Fn(&Aggregate) -> f32,
    minimum_value: f32,
    high_share: f32,
    medium_share: f32,
) -> Option<Attribution> {
    let total = aggregates
        .iter()
        .map(&metric)
        .filter(|value| value.is_finite() && *value > 0.0)
        .sum::<f32>();

    let top = aggregates
        .iter()
        .filter(|item| metric(item).is_finite())
        .max_by(|left, right| metric(left).total_cmp(&metric(right)))?;

    let top_value = metric(top);
    if top_value < minimum_value || total <= 0.0 {
        return None;
    }

    let share = (top_value / total).clamp(0.0, 1.0);
    let confidence = if share >= high_share {
        Confidence::High
    } else if share >= medium_share {
        Confidence::Medium
    } else {
        Confidence::Low
    };

    if confidence == Confidence::Low {
        return None;
    }

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
}
