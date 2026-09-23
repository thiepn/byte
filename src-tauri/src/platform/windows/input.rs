use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InputActivityKind { KeyboardActivity, MouseLeft, MouseRight, Scroll }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputActivityEvent {
    pub kind: InputActivityKind,
    pub timestamp_epoch_ms: u64,
}

/// Privacy boundary: there is deliberately no field for key identity or typed content.
pub fn anonymous_keyboard_activity(timestamp_epoch_ms: u64) -> InputActivityEvent {
    InputActivityEvent { kind: InputActivityKind::KeyboardActivity, timestamp_epoch_ms }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyboard_event_contains_activity_only() {
        let event = anonymous_keyboard_activity(42);
        assert_eq!(event.kind, InputActivityKind::KeyboardActivity);
        assert_eq!(event.timestamp_epoch_ms, 42);
    }
}
