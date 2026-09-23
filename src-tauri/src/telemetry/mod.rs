use crate::{core::error::ByteError, models::SystemSnapshot};

pub trait TelemetrySource: Send {
    fn sample(&mut self) -> Result<SystemSnapshot, ByteError>;
}

pub struct MockTelemetrySource { snapshot: SystemSnapshot }

impl MockTelemetrySource {
    pub fn new(snapshot: SystemSnapshot) -> Self { Self { snapshot } }
    pub fn healthy() -> Self { Self::new(SystemSnapshot::development_default()) }
}

impl TelemetrySource for MockTelemetrySource {
    fn sample(&mut self) -> Result<SystemSnapshot, ByteError> { Ok(self.snapshot.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SystemStatus;

    #[test]
    fn mock_source_is_deterministic() {
        let mut source = MockTelemetrySource::healthy();
        let first = source.sample().expect("first");
        let second = source.sample().expect("second");
        assert_eq!(first.overall_status, SystemStatus::Calm);
        assert_eq!(first.cpu.value, second.cpu.value);
    }
}
