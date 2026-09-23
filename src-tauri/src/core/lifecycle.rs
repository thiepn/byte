use serde::{Deserialize, Serialize};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Condvar, Mutex,
    },
    time::Duration,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LifecycleState {
    Active,
    FullscreenReduced,
    Locked,
    DisplaySleep,
    SystemSleep,
    ShuttingDown,
}

pub struct LifecycleCoordinator {
    state: Mutex<LifecycleState>,
    changed: Condvar,
    cancelled: AtomicBool,
}

impl Default for LifecycleCoordinator {
    fn default() -> Self {
        Self {
            state: Mutex::new(LifecycleState::Active),
            changed: Condvar::new(),
            cancelled: AtomicBool::new(false),
        }
    }
}

impl LifecycleCoordinator {
    pub fn current(&self) -> LifecycleState {
        *self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn transition(&self, next: LifecycleState) {
        *self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = next;
        self.changed.notify_all();
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
        *self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = LifecycleState::ShuttingDown;
        self.changed.notify_all();
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }

    pub fn wait_until_sampling_allowed(&self) -> bool {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        while is_sampling_suspended(*state) && !self.is_cancelled() {
            state = self
                .changed
                .wait(state)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }

        !self.is_cancelled()
    }

    pub fn wait_for_change_or_timeout(&self, timeout: Duration) -> bool {
        if self.is_cancelled() {
            return false;
        }

        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let _ = self
            .changed
            .wait_timeout(state, timeout)
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        !self.is_cancelled()
    }
}

fn is_sampling_suspended(state: LifecycleState) -> bool {
    matches!(
        state,
        LifecycleState::DisplaySleep | LifecycleState::SystemSleep | LifecycleState::ShuttingDown
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancellation_moves_lifecycle_to_shutdown() {
        let lifecycle = LifecycleCoordinator::default();
        lifecycle.cancel();

        assert!(lifecycle.is_cancelled());
        assert_eq!(lifecycle.current(), LifecycleState::ShuttingDown);
        assert!(!lifecycle.wait_until_sampling_allowed());
    }
}
