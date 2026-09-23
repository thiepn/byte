use crate::{core::{config::ConfigStore, lifecycle::LifecycleCoordinator}, models::SystemSnapshot};
use std::sync::{Mutex, RwLock};

pub struct AppState {
    pub snapshot: RwLock<SystemSnapshot>,
    pub config: Mutex<ConfigStore>,
    pub lifecycle: LifecycleCoordinator,
}

impl AppState {
    pub fn new(config: ConfigStore) -> Self {
        Self {
            snapshot: RwLock::new(SystemSnapshot::development_default()),
            config: Mutex::new(config),
            lifecycle: LifecycleCoordinator::default(),
        }
    }
}
