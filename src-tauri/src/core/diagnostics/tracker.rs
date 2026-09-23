use crate::models::ResourceState;

#[derive(Debug, Clone, Copy)]
pub struct TrackerPolicy {
    pub high_activation_ms: u64,
    pub critical_activation_ms: u64,
    pub recovery_ms: u64,
    pub reopen_cooldown_ms: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct Observation {
    pub elevated: bool,
    pub high: bool,
    pub critical: bool,
    pub recovered: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActiveCondition {
    pub severity: ResourceState,
    pub started_at_epoch_ms: u64,
}

#[derive(Debug, Default)]
pub struct SustainedTracker {
    high_since: Option<u64>,
    critical_since: Option<u64>,
    recovery_since: Option<u64>,
    active: Option<ActiveCondition>,
    reopen_not_before: Option<u64>,
}

impl SustainedTracker {
    pub fn update(
        &mut self,
        now: u64,
        observation: Observation,
        policy: TrackerPolicy,
    ) -> Option<ActiveCondition> {
        update_since(&mut self.high_since, observation.high, now);
        update_since(&mut self.critical_since, observation.critical, now);

        let high_ready = elapsed_at_least(self.high_since, now, policy.high_activation_ms);
        let critical_ready =
            elapsed_at_least(self.critical_since, now, policy.critical_activation_ms);

        match self.active {
            None => {
                let cooldown_complete = self
                    .reopen_not_before
                    .map(|deadline| now >= deadline)
                    .unwrap_or(true);

                if critical_ready {
                    self.activate(ResourceState::Critical, now);
                } else if high_ready && cooldown_complete {
                    self.activate(ResourceState::High, now);
                }
            }
            Some(active) if active.severity == ResourceState::High => {
                if critical_ready {
                    self.active = Some(ActiveCondition {
                        severity: ResourceState::Critical,
                        started_at_epoch_ms: active.started_at_epoch_ms,
                    });
                    self.recovery_since = None;
                } else if observation.high {
                    self.recovery_since = None;
                } else if observation.recovered {
                    if recovery_elapsed(&mut self.recovery_since, now, policy.recovery_ms) {
                        self.close(now, policy.reopen_cooldown_ms);
                    }
                } else {
                    self.recovery_since = None;
                }
            }
            Some(active) if active.severity == ResourceState::Critical => {
                if observation.critical {
                    self.recovery_since = None;
                } else if observation.high {
                    if recovery_elapsed(&mut self.recovery_since, now, policy.recovery_ms) {
                        self.active = Some(ActiveCondition {
                            severity: ResourceState::High,
                            started_at_epoch_ms: active.started_at_epoch_ms,
                        });
                        self.recovery_since = None;
                    }
                } else if observation.recovered {
                    if recovery_elapsed(&mut self.recovery_since, now, policy.recovery_ms) {
                        self.close(now, policy.reopen_cooldown_ms);
                    }
                } else {
                    self.recovery_since = None;
                }
            }
            Some(_) => {
                self.active = None;
            }
        }

        self.active
    }

    pub fn active(&self) -> Option<ActiveCondition> {
        self.active
    }

    pub fn reset(&mut self) {
        self.high_since = None;
        self.critical_since = None;
        self.recovery_since = None;
        self.active = None;
        self.reopen_not_before = None;
    }

    fn activate(&mut self, severity: ResourceState, now: u64) {
        let started_at_epoch_ms = self.high_since.unwrap_or(now);
        self.active = Some(ActiveCondition {
            severity,
            started_at_epoch_ms,
        });
        self.recovery_since = None;
    }

    fn close(&mut self, now: u64, cooldown_ms: u64) {
        self.active = None;
        self.recovery_since = None;
        self.high_since = None;
        self.critical_since = None;
        self.reopen_not_before = Some(now.saturating_add(cooldown_ms));
    }
}

fn update_since(value: &mut Option<u64>, condition: bool, now: u64) {
    if condition {
        value.get_or_insert(now);
    } else {
        *value = None;
    }
}

fn elapsed_at_least(since: Option<u64>, now: u64, duration_ms: u64) -> bool {
    since
        .map(|started| now.saturating_sub(started) >= duration_ms)
        .unwrap_or(false)
}

fn recovery_elapsed(since: &mut Option<u64>, now: u64, duration_ms: u64) -> bool {
    let started = *since.get_or_insert(now);
    now.saturating_sub(started) >= duration_ms
}

#[cfg(test)]
mod tests {
    use super::*;

    const POLICY: TrackerPolicy = TrackerPolicy {
        high_activation_ms: 10_000,
        critical_activation_ms: 20_000,
        recovery_ms: 5_000,
        reopen_cooldown_ms: 10_000,
    };

    fn observed(high: bool, critical: bool, recovered: bool) -> Observation {
        Observation {
            elevated: high,
            high,
            critical,
            recovered,
        }
    }

    #[test]
    fn transient_high_condition_does_not_open_issue() {
        let mut tracker = SustainedTracker::default();
        assert!(tracker.update(0, observed(true, false, false), POLICY).is_none());
        assert!(tracker.update(5_000, observed(false, false, true), POLICY).is_none());
    }

    #[test]
    fn high_condition_requires_sustained_duration() {
        let mut tracker = SustainedTracker::default();
        tracker.update(0, observed(true, false, false), POLICY);
        let active = tracker
            .update(10_000, observed(true, false, false), POLICY)
            .expect("active");
        assert_eq!(active.severity, ResourceState::High);
    }

    #[test]
    fn recovery_uses_hysteresis_duration() {
        let mut tracker = SustainedTracker::default();
        tracker.update(0, observed(true, false, false), POLICY);
        tracker.update(10_000, observed(true, false, false), POLICY);

        assert!(tracker
            .update(11_000, observed(false, false, true), POLICY)
            .is_some());
        assert!(tracker
            .update(16_000, observed(false, false, true), POLICY)
            .is_none());
    }

    #[test]
    fn critical_escalation_keeps_original_start_time() {
        let mut tracker = SustainedTracker::default();
        tracker.update(0, observed(true, true, false), POLICY);
        tracker.update(10_000, observed(true, true, false), POLICY);
        let active = tracker
            .update(20_000, observed(true, true, false), POLICY)
            .expect("critical");

        assert_eq!(active.severity, ResourceState::Critical);
        assert_eq!(active.started_at_epoch_ms, 0);
    }
}
