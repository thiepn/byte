use crate::models::now_epoch_ms;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawStorageSample {
    pub total_bytes: u64,
    pub available_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawBatterySample {
    pub percent: f32,
    pub charging: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawNetworkSample {
    pub download_bytes_per_second: f32,
    pub upload_bytes_per_second: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawTelemetrySample {
    pub timestamp_epoch_ms: u64,
    pub cpu_percent: f32,
    pub memory_used_bytes: u64,
    pub memory_total_bytes: u64,
    pub memory_available_bytes: u64,
    pub storage: Option<RawStorageSample>,
    pub battery: Option<RawBatterySample>,
    pub network: RawNetworkSample,
    pub thermal_celsius: Option<f32>,
}

impl RawTelemetrySample {
    pub fn development_default() -> Self {
        const GIB: u64 = 1_073_741_824;

        Self {
            timestamp_epoch_ms: now_epoch_ms(),
            cpu_percent: 18.0,
            memory_used_bytes: 8 * GIB,
            memory_total_bytes: 16 * GIB,
            memory_available_bytes: 8 * GIB,
            storage: Some(RawStorageSample {
                total_bytes: 512 * GIB,
                available_bytes: 287 * GIB,
            }),
            battery: Some(RawBatterySample {
                percent: 82.0,
                charging: true,
            }),
            network: RawNetworkSample {
                download_bytes_per_second: 50_000.0,
                upload_bytes_per_second: 12_500.0,
            },
            thermal_celsius: None,
        }
    }
}
