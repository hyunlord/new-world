//! `SeekTarget` goal-coordinate component (V7 Section 16-α).
//!
//! α0 (`53075aff`) laid the resource substrate (`food_tiles` /
//! `water_tiles` / `sleep_tiles`, 12 non-depleting `u8::MAX` sources) and
//! confirmed agents transition `Idle → Seeking{Food/Water/Sleep}` on a
//! need-threshold breach. But `Seeking` carries no goal coordinate — the
//! agent doesn't know *which* tile to head for.
//!
//! α = goal recognition only. When an agent is seeking a resource, the
//! `AgentDecisionSystem` post-decision pass records the **nearest matching
//! resource tile** in this component. β consumes `SeekTarget` for
//! directional movement; α itself produces **no movement and no visible
//! change** — purely internal state that β builds on.
//!
//! `Eq` is valid (the field is `(u32, u32)`), so exact equality and a
//! serde RON round-trip are both well-defined. Mirrors the `Position` /
//! `Hunger` component conventions (derives + doc + `#[cfg(test)]`).

use serde::{Deserialize, Serialize};

/// Goal resource-tile coordinate for an agent in
/// `AgentState::Seeking{Food/Water/Sleep}`.
///
/// `tile` is the nearest matching resource tile at the moment the agent
/// entered `Seeking` (chosen by Manhattan distance with a deterministic
/// `(x, y)` tie-break — see `sim_systems` `nearest_resource_tile`).
/// Consumed by β movement. Attached/cleared by the `AgentDecisionSystem`
/// post-decision pass; never attached to `Idle`, `Consuming`, or
/// `Seeking{ConstructionSite|Agent}` agents (those are co-located, not
/// resource tiles).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeekTarget {
    /// Goal resource-tile coordinate (the nearest matching tile at the
    /// moment `Seeking{Food/Water/Sleep}` was entered). Consumed by β.
    pub tile: (u32, u32),
}

impl SeekTarget {
    /// Construct a `SeekTarget` pointing at `tile`.
    pub const fn new(tile: (u32, u32)) -> Self {
        Self { tile }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_tile() {
        let st = SeekTarget::new((7, 42));
        assert_eq!(st.tile, (7, 42));
    }

    #[test]
    fn equality_holds_for_same_tile() {
        assert_eq!(SeekTarget::new((3, 4)), SeekTarget::new((3, 4)));
        assert_ne!(SeekTarget::new((3, 4)), SeekTarget::new((4, 3)));
    }

    /// Serde guard — round-trip via RON exercises both `Serialize` and
    /// `Deserialize`. If serde derives are removed this fails to compile
    /// before it runs, which is the desired build-time guard. Mirrors the
    /// `hunger.rs` / `position.rs` round-trip pattern.
    #[test]
    fn serde_round_trip() {
        let original = SeekTarget::new((7, 42));
        let encoded = ron::to_string(&original).expect("SeekTarget must Serialize");
        let decoded: SeekTarget =
            ron::from_str(&encoded).expect("SeekTarget must Deserialize");
        assert_eq!(original, decoded);
    }
}
