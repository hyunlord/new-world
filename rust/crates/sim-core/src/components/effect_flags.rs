use serde::{Deserialize, Serialize};

/// Lightweight per-entity effect flags toggled by queued effect primitives.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectFlags {
    /// Whether the entity is currently considered sheltered.
    pub sheltered: bool,
    /// Whether the entity is currently flagged as unsafe.
    pub is_unsafe: bool,
    /// Whether the entity is currently resting.
    pub resting: bool,
}
