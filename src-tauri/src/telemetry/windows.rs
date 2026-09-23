use super::{
    raw::{RawBatterySample, RawNetworkSample, RawStorageSample, RawTelemetrySample},
    TelemetrySource,
};
use crate::{core::error::ByteError, models::now_epoch_ms};
use std::{
    path::Path,
    thread,
    time::{Duration, Instant},
};
use sysinfo::{
    Components, CpuRefreshKind, DiskRefreshKind, Disks, MemoryRefreshKind, Networks, RefreshKind,
    System, MINIMUM_CPU_UPDATE_INTERVAL,
};
use windows_sys::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};

const STORAGE_REFRESH_INTERVAL: Duration = Duration::from_secs(15);
const BATTERY_REFRESH_INTERVAL: Duration = Duration::from_secs(5);
const THERMAL_REFRESH_INTERVAL: Duration = Duration::from_secs(10);

pub struct WindowsTelemetrySource {
    system: System,
    disks: Disks,
    networks: Networks,
    components: Components,
    last_network_refresh: Instant,
    last_storage_refresh: Instant,
    last_battery_refresh: Instant,
    last_thermal_refresh: Instant,
    cached_storage: Option<RawStorageSample>,
    cached_battery: Option<RawBatterySample>,
    cached_thermal_celsius: Option<f32>,
}

impl WindowsTelemetrySource {
    pub fn new() -> Self {
        let refreshes = RefreshKind::nothing()
            .with_cpu(CpuRefreshKind::nothing().with_cpu_usage())
            .with_memory(MemoryRefreshKind::nothing().with_ram());

        let system = System::new_with_specifics(refreshes);
        let disks = Disks::new_with_refreshed_list_specifics(
            DiskRefreshKind::nothing().with_storage(),
        );
        let networks = Networks::new_with_refreshed_list();
        let components = Components::new_with_refreshed_list();

        let cached_storage = read_system_storage(&disks);
        let cached_battery = read_battery();
        let cached_thermal_celsius = read_hottest_temperature(&components);

        thread::sleep(MINIMUM_CPU_UPDATE_INTERVAL);

        let now = Instant::now();
        Self {
            system,
            disks,
            networks,
            components,
            last_network_refresh: now,
            last_storage_refresh: now,
            last_battery_refresh: now,
            last_thermal_refresh: now,
            cached_storage,
            cached_battery,
            cached_thermal_celsius,
        }
    }

    fn refresh_slow_signals(&mut self) {
        if self.last_storage_refresh.elapsed() >= STORAGE_REFRESH_INTERVAL {
            self.disks.refresh_specifics(
                true,
                DiskRefreshKind::nothing().with_storage(),
            );
            self.cached_storage = read_system_storage(&self.disks);
            self.last_storage_refresh = Instant::now();
        }

        if self.last_battery_refresh.elapsed() >= BATTERY_REFRESH_INTERVAL {
            self.cached_battery = read_battery();
            self.last_battery_refresh = Instant::now();
        }

        if self.last_thermal_refresh.elapsed() >= THERMAL_REFRESH_INTERVAL {
            self.components.refresh(true);
            self.cached_thermal_celsius = read_hottest_temperature(&self.components);
            self.last_thermal_refresh = Instant::now();
        }
    }
}

impl TelemetrySource for WindowsTelemetrySource {
    fn sample(&mut self) -> Result<RawTelemetrySample, ByteError> {
        self.system.refresh_cpu_usage();
        self.system
            .refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());

        let elapsed = self.last_network_refresh.elapsed().as_secs_f32().max(0.001);
        self.networks.refresh(true);
        self.last_network_refresh = Instant::now();

        let received_bytes = self
            .networks
            .list()
            .values()
            .fold(0_u64, |total, data| total.saturating_add(data.received()));
        let transmitted_bytes = self
            .networks
            .list()
            .values()
            .fold(0_u64, |total, data| total.saturating_add(data.transmitted()));

        self.refresh_slow_signals();

        Ok(RawTelemetrySample {
            timestamp_epoch_ms: now_epoch_ms(),
            cpu_percent: self.system.global_cpu_usage(),
            memory_used_bytes: self.system.used_memory(),
            memory_total_bytes: self.system.total_memory(),
            memory_available_bytes: self.system.available_memory(),
            storage: self.cached_storage.clone(),
            battery: self.cached_battery.clone(),
            network: RawNetworkSample {
                download_bytes_per_second: received_bytes as f32 / elapsed,
                upload_bytes_per_second: transmitted_bytes as f32 / elapsed,
            },
            thermal_celsius: self.cached_thermal_celsius,
        })
    }
}

fn read_system_storage(disks: &Disks) -> Option<RawStorageSample> {
    let executable = std::env::current_exe().ok();

    let preferred = executable
        .as_deref()
        .and_then(|path| find_disk_for_path(disks, path))
        .or_else(|| disks.list().iter().max_by_key(|disk| disk.total_space()));

    preferred.map(|disk| RawStorageSample {
        total_bytes: disk.total_space(),
        available_bytes: disk.available_space(),
    })
}

fn find_disk_for_path<'a>(disks: &'a Disks, path: &Path) -> Option<&'a sysinfo::Disk> {
    disks
        .list()
        .iter()
        .filter(|disk| path.starts_with(disk.mount_point()))
        .max_by_key(|disk| disk.mount_point().as_os_str().len())
}

fn read_hottest_temperature(components: &Components) -> Option<f32> {
    components
        .list()
        .iter()
        .filter_map(|component| component.temperature())
        .filter(|temperature| temperature.is_finite() && (0.0..=150.0).contains(temperature))
        .max_by(|left, right| left.total_cmp(right))
}

fn read_battery() -> Option<RawBatterySample> {
    let mut status = SYSTEM_POWER_STATUS {
        ACLineStatus: 0,
        BatteryFlag: 0,
        BatteryLifePercent: 0,
        SystemStatusFlag: 0,
        BatteryLifeTime: 0,
        BatteryFullLifeTime: 0,
    };

    // SAFETY: status is a valid writable SYSTEM_POWER_STATUS for the duration
    // of the Win32 call, and the API does not retain the pointer.
    let ok = unsafe { GetSystemPowerStatus(&mut status) };
    if ok == 0 {
        return None;
    }

    const NO_SYSTEM_BATTERY: u8 = 128;
    const BATTERY_CHARGING: u8 = 8;
    const UNKNOWN_PERCENT: u8 = 255;

    if status.BatteryFlag & NO_SYSTEM_BATTERY != 0
        || status.BatteryLifePercent == UNKNOWN_PERCENT
        || status.BatteryLifePercent > 100
    {
        return None;
    }

    Some(RawBatterySample {
        percent: status.BatteryLifePercent as f32,
        charging: status.BatteryFlag & BATTERY_CHARGING != 0,
    })
}
