use crate::{
    core::{config::ConfigStore, lifecycle::LifecycleCoordinator},
    models::SystemSnapshot,
};
use std::{
    sync::{Mutex, RwLock},
    thread::JoinHandle,
};

pub struct AppState {
    snapshot: RwLock<SystemSnapshot>,
    pub config: Mutex<ConfigStore>,
    pub lifecycle: LifecycleCoordinator,
    telemetry_worker: Mutex<Option<JoinHandle<()>>>,
}

impl AppState {
    pub fn new(config: ConfigStore) -> Self {
        Self {
            snapshot: RwLock::new(SystemSnapshot::unavailable()),
            config: Mutex::new(config),
            lifecycle: LifecycleCoordinator::default(),
            telemetry_worker: Mutex::new(None),
        }
    }

    pub fn snapshot(&self) -> SystemSnapshot {
        self.snapshot
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    pub fn replace_snapshot(&self, snapshot: SystemSnapshot) {
        *self
            .snapshot
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = snapshot;
    }

    pub fn install_telemetry_worker(&self, worker: JoinHandle<()>) {
        *self
            .telemetry_worker
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(worker);
    }

    pub fn stop_telemetry_worker(&self) {
        self.lifecycle.cancel();
        let worker = self
            .telemetry_worker
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take();

        if let Some(worker) = worker {
            let _ = worker.join();
        }
    }
}
