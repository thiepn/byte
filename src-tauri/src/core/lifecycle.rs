use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    RwLock,
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
    state: RwLock<LifecycleState>,
    cancelled: AtomicBool,
}

impl Default for LifecycleCoordinator {
    fn default() -> Self {
        Self {
            state: RwLock::new(LifecycleState::Active),
            cancelled: AtomicBool::new(false),
        }
    }
}

impl LifecycleCoordinator {
    pub fn current(&self) -> LifecycleState {
        *self
            .state
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn transition(&self, next: LifecycleState) {
        *self
            .state
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = next;
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
        self.transition(LifecycleState::ShuttingDown);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
}
