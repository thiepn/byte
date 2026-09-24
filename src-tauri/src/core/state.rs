use crate::{
    core::{
        activity::{ActivitySnapshot, ActivityStore},
        config::ConfigStore,
        diagnostics::AppInspector,
        lifecycle::LifecycleCoordinator,
    },
    models::{AppDiagnosticsSnapshot, SystemSnapshot, WindowShellState},
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
    app_inspector: Mutex<AppInspector>,
    telemetry_worker: Mutex<Option<JoinHandle<()>>>,
    #[cfg(target_os = "windows")]
    input_runtime: Mutex<Option<InputRuntime>>,
}

impl AppState {
    pub fn new(config: ConfigStore, activity: ActivityStore) -> Self {
        Self {
            snapshot: RwLock::new(SystemSnapshot::unavailable()),
            config: Mutex::new(config),
            lifecycle: LifecycleCoordinator::default(),
            window_shell: Mutex::new(WindowShellState::default()),
            activity: Mutex::new(activity),
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
    }
}
