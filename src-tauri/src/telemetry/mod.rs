pub mod engine;
pub mod raw;
pub mod runtime;
mod smoothing;

#[cfg(target_os = "windows")]
mod windows;

use crate::core::error::ByteError;
use raw::RawTelemetrySample;
use std::collections::VecDeque;

pub trait TelemetrySource: Send {
    fn sample(&mut self) -> Result<RawTelemetrySample, ByteError>;
}

pub struct MockTelemetrySource {
    samples: VecDeque<RawTelemetrySample>,
    fallback: RawTelemetrySample,
}

impl MockTelemetrySource {
    pub fn new(sample: RawTelemetrySample) -> Self {
        Self {
            samples: VecDeque::from([sample.clone()]),
            fallback: sample,
        }
    }

    pub fn healthy() -> Self {
        Self::new(RawTelemetrySample::development_default())
    }

    pub fn push(&mut self, sample: RawTelemetrySample) {
        self.fallback = sample.clone();
        self.samples.push_back(sample);
    }
}

impl TelemetrySource for MockTelemetrySource {
    fn sample(&mut self) -> Result<RawTelemetrySample, ByteError> {
        Ok(self
            .samples
            .pop_front()
            .unwrap_or_else(|| self.fallback.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_source_is_deterministic_after_queue_is_exhausted() {
        let mut source = MockTelemetrySource::healthy();
        let first = source.sample().expect("first sample");
        let second = source.sample().expect("second sample");
        assert_eq!(first.cpu_percent, second.cpu_percent);
    }
}
