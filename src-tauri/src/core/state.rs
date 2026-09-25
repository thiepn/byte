use crate::{
    core::{
        activity::{ActivitySnapshot, ActivityStore},
        collection::CollectionStore,
        config::ConfigStore,
        diagnostics::AppInspector,
        lifecycle::LifecycleCoordinator,
        smart_notifications::{SmartNotification, SmartNotificationEngine},
    },
    models::{
        AppDiagnosticsSnapshot, AppPreferences, CollectionDiscoveryKind, CollectionSnapshot,
        CompanionPreferences, DesktopAwarenessSnapshot, SystemIssue, SystemSnapshot,
        VisibilitySuppressionReason, WindowShellState,
    },
};
use std::{
    sync::{Mutex, RwLock},
    thread::JoinHandle,
};

#[cfg(target_os = "windows")]
use crate::platform::windows::input::InputRuntime;

#[derive(Debug, Default)]
struct DesktopVisibilityState {
    snapshot: DesktopAwarenessSnapshot,
    restore_companion: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DesktopVisibilityTransition {
    pub entered: bool,
    pub restore_on_exit: bool,
    pub changed: bool,
}

pub struct AppState {
    snapshot: RwLock<SystemSnapshot>,
    pub config: Mutex<ConfigStore>,
    pub lifecycle: LifecycleCoordinator,
    pub window_shell: Mutex<WindowShellState>,
    desktop_visibility: Mutex<DesktopVisibilityState>,
    activity: Mutex<ActivityStore>,
    collection: Mutex<CollectionStore>,
    app_inspector: Mutex<AppInspector>,
    smart_notifications: Mutex<SmartNotificationEngine>,
    telemetry_worker: Mutex<Option<JoinHandle<()>>>,
    #[cfg(target_os = "windows")]
    fullscreen_worker: Mutex<Option<JoinHandle<()>>>,
    #[cfg(target_os = "windows")]
    input_runtime: Mutex<Option<InputRuntime>>,
}

impl AppState {
    pub fn new(
        config: ConfigStore,
        activity: ActivityStore,
        collection: CollectionStore,
        smart_notifications: SmartNotificationEngine,
    ) -> Self {
        Self {
            snapshot: RwLock::new(SystemSnapshot::unavailable()),
            config: Mutex::new(config),
            lifecycle: LifecycleCoordinator::default(),
            window_shell: Mutex::new(WindowShellState::default()),
            desktop_visibility: Mutex::new(DesktopVisibilityState::default()),
            activity: Mutex::new(activity),
            collection: Mutex::new(collection),
            app_inspector: Mutex::new(AppInspector::new()),
            smart_notifications: Mutex::new(smart_notifications),
            telemetry_worker: Mutex::new(None),
            #[cfg(target_os = "windows")]
            fullscreen_worker: Mutex::new(None),
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

    pub fn clear_activity(&self) -> Result<ActivitySnapshot, crate::core::error::ByteError> {
        self.activity
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear()
    }

    pub fn reset_activity_observation_baseline(&self) {
        self.activity
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .reset_observation_baseline();
    }

    pub fn app_preferences(&self) -> AppPreferences {
        self.config
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .snapshot()
            .app
    }

    pub fn set_snapshot_unavailable(&self) {
        *self
            .snapshot
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = SystemSnapshot::unavailable();
    }

    pub fn desktop_awareness(&self) -> DesktopAwarenessSnapshot {
        self.desktop_visibility
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .snapshot
            .clone()
    }

    pub fn is_visibility_suppressed(&self) -> bool {
        self.desktop_visibility
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .snapshot
            .suppressed
    }

    pub fn update_desktop_awareness(
        &self,
        reason: Option<VisibilitySuppressionReason>,
        foreground_app: Option<String>,
        capture_exclusion_enabled: bool,
        companion_visible: bool,
    ) -> DesktopVisibilityTransition {
        let mut state = self
            .desktop_visibility
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        let was_suppressed = state.snapshot.suppressed;
        let suppressed = reason.is_some();
        let entered = suppressed && !was_suppressed;
        let exited = !suppressed && was_suppressed;

        if entered {
            state.restore_companion = companion_visible;
        }

        let next = DesktopAwarenessSnapshot {
            suppressed,
            reason,
            foreground_app,
            capture_exclusion_enabled,
        };
        let changed = state.snapshot != next;
        state.snapshot = next;

        let restore_on_exit = exited && state.restore_companion;
        if exited {
            state.restore_companion = false;
        }

        DesktopVisibilityTransition {
            entered,
            restore_on_exit,
            changed,
        }
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

    pub fn next_smart_notification(
        &self,
        now: u64,
        issues: &[SystemIssue],
        preferences: &AppPreferences,
    ) -> Option<SmartNotification> {
        self.smart_notifications
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .evaluate(now, issues, preferences)
    }

    pub fn mark_smart_notification_sent(
        &self,
        notification: &SmartNotification,
        now: u64,
    ) -> Result<(), crate::core::error::ByteError> {
        self.smart_notifications
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .mark_sent(notification, now)
    }

    pub fn inspect_apps(&self) -> AppDiagnosticsSnapshot {
        self.app_inspector
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .inspect()
    }

    pub fn telemetry_worker_running(&self) -> bool {
        self.telemetry_worker
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
            .map(|worker| !worker.is_finished())
            .unwrap_or(false)
    }

    pub fn install_telemetry_worker(&self, worker: JoinHandle<()>) {
        *self
            .telemetry_worker
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(worker);
    }

    #[cfg(target_os = "windows")]
    pub fn fullscreen_worker_running(&self) -> bool {
        self.fullscreen_worker
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
            .map(|worker| !worker.is_finished())
            .unwrap_or(false)
    }

    #[cfg(target_os = "windows")]
    pub fn input_runtime_installed(&self) -> bool {
        self.input_runtime
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .is_some()
    }

    #[cfg(target_os = "windows")]
    pub fn install_fullscreen_worker(&self, worker: JoinHandle<()>) {
        *self
            .fullscreen_worker
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
    pub fn notify_input_lifecycle_changed(&self) {
        if let Some(runtime) = self
            .input_runtime
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
        {
            runtime.notify_lifecycle_changed();
        }
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

        self.lifecycle.cancel();
        self.stop_telemetry_worker();

        #[cfg(target_os = "windows")]
        {
            let worker = self
                .fullscreen_worker
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .take();
            if let Some(worker) = worker {
                let _ = worker.join();
            }
        }

        let _ = self
            .collection
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .flush();
    }
}
