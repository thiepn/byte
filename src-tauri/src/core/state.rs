use crate::{
    core::{
        activity::{ActivitySnapshot, ActivityStore},
        collection::CollectionStore,
        config::ConfigStore,
        diagnostics::AppInspector,
        lifecycle::LifecycleCoordinator,
    },
    models::{
        AppDiagnosticsSnapshot, CollectionDiscoveryKind, CollectionSnapshot, CompanionPreferences,
        SystemSnapshot, WindowShellState,
    },
};
use std::{
    sync::{Mutex, RwLock},
    thread::JoinHandle,
};

#[cfg(target_os = "windows")]
use crate::platform::windows::input::InputRuntime;

pub struct AppState {
    snapshot: RwLock<SystemSnapshot>,
    pub config: Mutex<ConfigStore>,
    pub lifecycle: LifecycleCoordinator,
    pub window_shell: Mutex<WindowShellState>,
    activity: Mutex<ActivityStore>,
    collection: Mutex<CollectionStore>,
    app_inspector: Mutex<AppInspector>,
    telemetry_worker: Mutex<Option<JoinHandle<()>>>,
    #[cfg(target_os = "windows")]
    input_runtime: Mutex<Option<InputRuntime>>,
}

impl AppState {
    pub fn new(config: ConfigStore, activity: ActivityStore, collection: CollectionStore) -> Self {
        Self {
            snapshot: RwLock::new(SystemSnapshot::unavailable()),
            config: Mutex::new(config),
            lifecycle: LifecycleCoordinator::default(),
            window_shell: Mutex::new(WindowShellState::default()),
            activity: Mutex::new(activity),
            collection: Mutex::new(collection),
            app_inspector: Mutex::new(AppInspector::new()),
            telemetry_worker: Mutex::new(None),
            #[cfg(target_os = "windows")]
            input_runtime: Mutex::new(None),
        }
    }

    pub fn snapshot(&self) -> SystemSnapshot {
        self.snapshot
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    pub fn replace_snapshot(&self, snapshot: SystemSnapshot) {
        let history_enabled = self
            .config
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .snapshot()
            .app
            .activity_history_enabled;

        let _ = self
            .activity
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .record(&snapshot, history_enabled);

        *self
            .snapshot
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = snapshot;
    }

    pub fn activity_snapshot(&self) -> ActivitySnapshot {
        self.activity
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .snapshot()
    }

    pub fn collection_snapshot(&self) -> Result<CollectionSnapshot, crate::core::error::ByteError> {
        self.collection
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .snapshot()
    }

    pub fn record_collection_typing(
        &self,
        timestamp_epoch_ms: u64,
    ) -> Result<Option<CollectionSnapshot>, crate::core::error::ByteError> {
        self.collection
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .record_typing(timestamp_epoch_ms)
    }

    pub fn observe_collection_system(
        &self,
        snapshot: &SystemSnapshot,
    ) -> Result<Option<CollectionSnapshot>, crate::core::error::ByteError> {
        self.collection
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .observe_system(snapshot)
    }

    pub fn record_collection_discovery(
        &self,
        discovery: CollectionDiscoveryKind,
    ) -> Result<(CollectionSnapshot, bool), crate::core::error::ByteError> {
        self.collection
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .record_discovery(discovery)
    }

    pub fn validate_collection_preferences(
        &self,
        preferences: &CompanionPreferences,
    ) -> Result<(), crate::core::error::ByteError> {
        self.collection
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .validate_preferences(preferences)
    }

    pub fn inspect_apps(&self) -> AppDiagnosticsSnapshot {
        self.app_inspector
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .inspect()
    }

    pub fn install_telemetry_worker(&self, worker: JoinHandle<()>) {
        *self
            .telemetry_worker
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(worker);
    }

    #[cfg(target_os = "windows")]
    pub fn install_input_runtime(&self, runtime: InputRuntime) {
        *self
            .input_runtime
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(runtime);
    }

    #[cfg(target_os = "windows")]
    pub fn stop_input_runtime(&self) {
        let runtime = self
            .input_runtime
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take();

        if let Some(runtime) = runtime {
            runtime.stop();
        }
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

    pub fn stop_background_workers(&self) {
        #[cfg(target_os = "windows")]
        self.stop_input_runtime();
        self.stop_telemetry_worker();
        let _ = self
            .collection
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .flush();
    }
}
