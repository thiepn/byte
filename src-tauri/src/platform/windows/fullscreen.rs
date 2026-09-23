#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FullscreenObservation { pub active: bool }

impl FullscreenObservation {
    pub const fn inactive() -> Self { Self { active: false } }
}
