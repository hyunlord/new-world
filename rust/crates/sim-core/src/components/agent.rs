//! Canonical `Agent` component (V7 Phase 4-α → 5-α).
//!
//! Phase 4-α landed `Agent` as a zero-sized marker. Phase 5-α (P5α-1)
//! upgrades it to an identity-carrying struct: every agent now holds a
//! monotonically minted [`AgentId`]. Ids are issued by the engine via
//! `SimResources::issue_agent_id` (Phase 3-β `next_event_id` precedent)
//! and consumed by [`crate::components::agent::Agent::id`].
//!
//! `Default` is intentionally *not* derived. A zero-id default would
//! collide with the first id the engine mints, so callers must always
//! supply an id (or use `SimEngine::spawn_agent`, which mints internally).
//!
//! `Serialize + Deserialize` are retained so an agent's id and position
//! round-trip through the save format.

use serde::{Deserialize, Serialize};

/// Monotonically minted identity for an autonomous agent.
///
/// Allocated by `sim_engine::SimResources::issue_agent_id` (Relaxed
/// atomic `fetch_add`), mirroring the Phase 3-β `next_event_id` mint.
/// Stable across the entity's lifetime — even if the underlying hecs
/// [`Entity`](hecs::Entity) handle is recycled, the agent's id is not.
pub type AgentId = u64;

/// Identity component held by every autonomous agent.
///
/// Held alongside [`crate::components::Position`] on every spawned
/// agent. Future Phase 5 sub-stages extend the agent surface via
/// additional components rather than fields on `Agent` itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Agent {
    /// Monotonic identity minted at spawn time.
    pub id: AgentId,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_carries_public_id() {
        let a = Agent { id: 17 };
        assert_eq!(a.id, 17);
    }

    #[test]
    fn agent_is_copy_and_eq() {
        let a = Agent { id: 5 };
        let b = a;
        assert_eq!(a, b);
        assert_eq!(a.id, b.id);
    }

    /// Serde guard — round-trip via RON exercises both `Serialize` and
    /// `Deserialize`. Removal of either derive causes a compile failure,
    /// which is the desired build-time guard.
    #[test]
    fn serde_round_trip_preserves_id() {
        let original = Agent { id: 0xDEAD_BEEF };
        let encoded = ron::to_string(&original).expect("Agent must Serialize");
        let decoded: Agent =
            ron::from_str(&encoded).expect("Agent must Deserialize");
        assert_eq!(original, decoded);
        assert_eq!(decoded.id, 0xDEAD_BEEF);
    }

    /// Size guard — `Agent` is exactly the size of its id, no padding.
    /// Phase 4-α asserted `size_of::<Agent>() == 0` against the ZST form;
    /// Phase 5-α replaces that contract with size equality to AgentId.
    #[test]
    fn agent_size_equals_agent_id() {
        assert_eq!(std::mem::size_of::<Agent>(), std::mem::size_of::<AgentId>());
    }
}
