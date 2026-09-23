use super::{raw::RawTelemetrySample, smoothing::Ewma, TelemetrySource};
use crate::{
    core::error::ByteError,
    models::{
        BatterySummary, NetworkSummary, ResourceState, ResourceSummary, SystemSnapshot,
        SystemStatus,
    },
};

const BYTES_PER_GIB: f32 = 1_073_741_824.0;
const BYTES_PER_MEGABIT: f32 = 125_000.0;
const DEFAULT_EWMA_ALPHA: f32 = 0.35;

pub struct TelemetryEngine<S: TelemetrySource> {
    source: S,
    cpu: Ewma,
    memory: Ewma,
    network_download: Ewma,
    network_upload: Ewma,
    thermal: Ewma,
}

impl<S: TelemetrySource> TelemetryEngine<S> {
    pub fn new(source: S) -> Self {
        Self {
            source,
            cpu: Ewma::new(DEFAULT_EWMA_ALPHA),
            memory: Ewma::new(DEFAULT_EWMA_ALPHA),
            network_download: Ewma::new(DEFAULT_EWMA_ALPHA),
            network_upload: Ewma::new(DEFAULT_EWMA_ALPHA),
            thermal: Ewma::new(DEFAULT_EWMA_ALPHA),
        }
    }

    pub fn sample_snapshot(&mut self) -> Result<SystemSnapshot, ByteError> {
        let raw = self.source.sample()?;
        Ok(self.snapshot_from_raw(raw))
    }

    fn snapshot_from_raw(&mut self, raw: RawTelemetrySample) -> SystemSnapshot {
        let raw_memory_percent = percent(raw.memory_used_bytes, raw.memory_total_bytes);
        let memory_percent = self.memory.update(raw_memory_percent);

        let storage = match raw.storage {
            Some(storage) if storage.total_bytes > 0 => ResourceSummary {
                value: percent(
                    storage.total_bytes.saturating_sub(storage.available_bytes),
                    storage.total_bytes,
                ),
                unit: "%".into(),
                state: ResourceState::Normal,
                available: Some(storage.available_bytes as f32 / BYTES_PER_GIB),
                available_unit: Some("GB".into()),
            },
            _ => unavailable_resource("%"),
        };

        let battery = raw.battery.map(|battery| BatterySummary {
            percent: battery.percent.clamp(0.0, 100.0),
            charging: battery.charging,
            state: ResourceState::Normal,
        });

        let thermal = raw.thermal_celsius.and_then(|temperature| {
            if temperature.is_finite() {
                Some(ResourceSummary {
                    value: self.thermal.update(temperature),
                    unit: "°C".into(),
                    state: ResourceState::Normal,
                    available: None,
                    available_unit: None,
                })
            } else {
                None
            }
        });

        SystemSnapshot {
            timestamp_epoch_ms: raw.timestamp_epoch_ms,
            overall_status: SystemStatus::Calm,
            cpu: ResourceSummary {
                value: self.cpu.update(raw.cpu_percent.clamp(0.0, 100.0)),
                unit: "%".into(),
                state: ResourceState::Normal,
                available: None,
                available_unit: None,
            },
            memory: ResourceSummary {
                value: memory_percent,
                unit: "%".into(),
                state: if raw.memory_total_bytes == 0 {
                    ResourceState::Unknown
                } else {
                    ResourceState::Normal
                },
                available: if raw.memory_total_bytes == 0 {
                    None
                } else {
                    Some(raw.memory_available_bytes as f32 / BYTES_PER_GIB)
                },
                available_unit: if raw.memory_total_bytes == 0 {
                    None
                } else {
                    Some("GB".into())
                },
            },
            storage,
            battery,
            network: NetworkSummary {
                download_mbps: self
                    .network_download
                    .update(raw.network.download_bytes_per_second.max(0.0))
                    / BYTES_PER_MEGABIT,
                upload_mbps: self
                    .network_upload
                    .update(raw.network.upload_bytes_per_second.max(0.0))
                    / BYTES_PER_MEGABIT,
            },
            thermal,
            primary_issue: None,
            secondary_issue_count: 0,
        }
    }
}

fn percent(used: u64, total: u64) -> f32 {
    if total == 0 {
        return 0.0;
    }
    (used as f64 * 100.0 / total as f64).clamp(0.0, 100.0) as f32
}

fn unavailable_resource(unit: &str) -> ResourceSummary {
    ResourceSummary {
        value: 0.0,
        unit: unit.into(),
        state: ResourceState::Unknown,
        available: None,
        available_unit: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::MockTelemetrySource;

    fn fixture(name: &str) -> RawTelemetrySample {
        let raw = match name {
            "healthy" => include_str!("../../tests/fixtures/healthy.json"),
            "busy" => include_str!("../../tests/fixtures/busy.json"),
            "optional_missing" => include_str!("../../tests/fixtures/optional_missing.json"),
            _ => panic!("unknown fixture"),
        };
        serde_json::from_str(raw).expect("valid telemetry fixture")
    }

    #[test]
    fn fixture_maps_to_user_snapshot_without_diagnostics() {
        let mut engine = TelemetryEngine::new(MockTelemetrySource::new(fixture("healthy")));
        let snapshot = engine.sample_snapshot().expect("snapshot");

        assert_eq!(snapshot.overall_status, SystemStatus::Calm);
        assert!((snapshot.cpu.value - 18.0).abs() < 0.001);
        assert!((snapshot.memory.value - 50.0).abs() < 0.001);
        assert!(snapshot.storage.available.is_some());
        assert!(snapshot.battery.is_some());
        assert!(snapshot.thermal.is_some());
        assert!(snapshot.primary_issue.is_none());
    }

    #[test]
    fn optional_signals_can_be_absent_without_fake_values() {
        let mut engine =
            TelemetryEngine::new(MockTelemetrySource::new(fixture("optional_missing")));
        let snapshot = engine.sample_snapshot().expect("snapshot");

        assert!(snapshot.battery.is_none());
        assert!(snapshot.thermal.is_none());
        assert_eq!(snapshot.storage.state, ResourceState::Unknown);
        assert!(snapshot.storage.available.is_none());
    }

    #[test]
    fn cpu_smoothing_starts_from_first_real_sample() {
        let mut source = MockTelemetrySource::new(fixture("healthy"));
        source.push(fixture("busy"));
        let mut engine = TelemetryEngine::new(source);

        let first = engine.sample_snapshot().expect("first");
        let second = engine.sample_snapshot().expect("second");

        assert!((first.cpu.value - 18.0).abs() < 0.001);
        assert!(second.cpu.value > first.cpu.value);
        assert!(second.cpu.value < 92.0);
    }
}
