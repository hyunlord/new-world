//! Canonical `Agent` marker component (V7 Phase 4-α first deliverable).
//!
//! Zero-sized marker (P4α-1-b — minimal scope). Identifies an ECS entity
//! as an autonomous agent. Personality / Body / Needs / BodyHealth are
//! NOT part of α and are deferred to later Phase 4 sub-stages (β/γ/δ).
//!
//! `Serialize + Deserialize` are derived so the marker can participate
//! in save-load round-trip alongside `Position`.

use serde::{Deserialize, Serialize};

/// Marker component identifying an entity as an autonomous agent.
///
/// Held alongside [`crate::components::Position`] on every spawned
/// agent. Future Phase 4 sub-stages extend the agent surface via
/// additional components rather than fields on `Agent` itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Agent;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marker_instantiates() {
        let _ = Agent;
    }

    #[test]
    fn marker_is_zero_sized() {
        assert_eq!(std::mem::size_of::<Agent>(), 0);
    }

    #[test]
    #[allow(clippy::default_constructed_unit_structs)]
    fn marker_default_equals_unit() {
        assert_eq!(Agent, Agent::default());
    }

    /// Serde guard — round-trip via RON exercises both `Serialize` and
    /// `Deserialize`. Removal of either derive causes a compile failure,
    /// which is the desired build-time guard.
    #[test]
    fn serde_round_trip_preserves_marker() {
        let original = Agent;
        let encoded = ron::to_string(&original).expect("Agent must Serialize");
        let decoded: Agent =
            ron::from_str(&encoded).expect("Agent must Deserialize");
        assert_eq!(original, decoded);
    }
}
