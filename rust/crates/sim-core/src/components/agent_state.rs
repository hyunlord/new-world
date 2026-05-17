//! `AgentState` finite-state machine (V7 Phase 5-β / P5β-2).
//!
//! Three explicit states drive the first agent-originated causal chain:
//!
//! ```text
//! Idle ──── threshold breach ───▶ Seeking { target } ──── tile reached ───▶ Consuming { target }
//!  ▲                                                                              │
//!  └────────────────────────── consume completed ────────────────────────────────┘
//! ```
//!
//! Phase 5-β scope was intentionally minimal: only `Food` and `Water`
//! targets existed. `Sleep` landed in γ alongside the day/night clock —
//! see `Sleep` component and `SleepDecaySystem`.
//! Mood/morale states are deferred to δ.
//!
//! Serde is enabled so save/load round-trip is preserved across the new
//! component surface — the FSM is part of agent identity, not transient.

use serde::{Deserialize, Serialize};

use crate::components::agent::AgentId;

/// Resource an agent is actively pursuing or consuming during the
/// [`AgentState::Seeking`] / [`AgentState::Consuming`] phases.
///
/// Phase 7-α scope: five variants. Adding a target requires updating
/// both the AgentDecisionSystem FSM and any UI that surfaces agent state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetKind {
    /// Solid food tile (decremented from `SimResources::food_tiles`).
    Food,
    /// Water tile (decremented from `SimResources::water_tiles`).
    Water,
    /// Sleep tile (decremented from `SimResources::sleep_tiles`).
    /// V7 Phase 5-γ / P5γ-5 — Path b symmetry (Plan §2.3 line 286-289).
    Sleep,
    /// Construction site tile — progress is advanced by
    /// `ConstructionSite::advance` once Phase 6-β lands the runtime
    /// system. V7 Phase 6-α / P6α-6 — Phase 5-γ Path (b) symmetry
    /// precedent (new TargetKind variant, NOT a new AgentState variant).
    ConstructionSite,
    /// V7 Phase 7-α / P7α-8 — Co-located partner agent for the
    /// Multi-agent Social System (Phase 5-γ Sleep / Phase 6-α
    /// ConstructionSite payload symmetry extended to AgentId). First
    /// payload-carrying `TargetKind` variant. The embedded `AgentId`
    /// identifies the partner; canonicalisation across a pair is
    /// handled by `RelationshipKey::new`, not by this variant. Runtime
    /// resolution lands in Phase 7-β; α arms in `AgentDecisionSystem`
    /// are intentionally inert.
    Agent(AgentId),
}

/// Per-agent FSM state. Lives alongside [`Hunger`]/[`Thirst`] and is
/// driven by `AgentDecisionSystem` (priority 125).
///
/// [`Hunger`]: crate::components::Hunger
/// [`Thirst`]: crate::components::Thirst
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentState {
    /// Default state — no active goal. Brownian motion proceeds normally.
    #[default]
    Idle,
    /// A threshold has been breached and the agent is pathing toward
    /// a matching resource tile. Brownian motion is suppressed
    /// (`AgentMovementSystem` skips entities in this state).
    Seeking {
        /// Resource the agent is en route to.
        target: TargetKind,
    },
    /// The agent has reached a matching resource tile and is consuming
    /// it. The consume effect (Hunger/Thirst decrement + tile counter
    /// decrement) is applied during this state.
    Consuming {
        /// Resource currently being consumed.
        target: TargetKind,
    },
}

impl AgentState {
    /// Convenience: `Some(target)` for the two active states, `None`
    /// for `Idle`.
    pub fn target(&self) -> Option<TargetKind> {
        match self {
            AgentState::Idle => None,
            AgentState::Seeking { target } | AgentState::Consuming { target } => Some(*target),
        }
    }

    /// True when the agent should NOT take its Brownian step this tick.
    ///
    /// Truth table (per V7 Phase 7-γ plan §γ A15):
    ///   - `Idle`                                       → false
    ///   - `Seeking { _ }`                              → true  (all variants)
    ///   - `Consuming { target: TargetKind::Agent(_) }` → true  (P7-γ A15)
    ///   - `Consuming { target: Food|Water|Sleep|ConstructionSite }` → false
    ///
    /// The `Consuming{Agent(_)}` case suppresses Brownian motion at the API
    /// level because the social interaction loop requires both participants
    /// to remain on the shared tile for the entire `REQUIRED_INTERACTION_
    /// PROGRESS` window. For all other `Consuming { _ }` variants, the
    /// movement system applies an additional internal freeze (`AgentMovement
    /// System::tick` has its own `Consuming { .. }` short-circuit), so the
    /// observable behaviour is the same — but only the `Agent(_)` variant
    /// surfaces it via the public `suppresses_movement()` predicate.
    pub fn suppresses_movement(&self) -> bool {
        matches!(self, AgentState::Seeking { .. })
            || matches!(self, AgentState::Consuming { target: TargetKind::Agent(_) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_idle() {
        assert_eq!(AgentState::default(), AgentState::Idle);
    }

    #[test]
    fn target_helper_returns_inner_kind() {
        assert_eq!(AgentState::Idle.target(), None);
        assert_eq!(
            AgentState::Seeking { target: TargetKind::Food }.target(),
            Some(TargetKind::Food)
        );
        assert_eq!(
            AgentState::Consuming { target: TargetKind::Water }.target(),
            Some(TargetKind::Water)
        );
    }

    #[test]
    fn suppresses_movement_truth_table() {
        // Idle never suppresses.
        assert!(!AgentState::Idle.suppresses_movement());
        // Seeking always suppresses (any target).
        assert!(AgentState::Seeking { target: TargetKind::Food }.suppresses_movement());
        assert!(AgentState::Seeking { target: TargetKind::Water }.suppresses_movement());
        assert!(AgentState::Seeking { target: TargetKind::Sleep }.suppresses_movement());
        assert!(AgentState::Seeking { target: TargetKind::ConstructionSite }
            .suppresses_movement());
        assert!(AgentState::Seeking { target: TargetKind::Agent(7) }.suppresses_movement());
        // Consuming{Agent(_)} suppresses (P7-γ A15) — agents in a social
        // interaction must stay on the shared tile through the full
        // REQUIRED_INTERACTION_PROGRESS window.
        assert!(AgentState::Consuming { target: TargetKind::Agent(7) }.suppresses_movement());
        // Consuming{Food|Water|Sleep|ConstructionSite} does NOT surface
        // suppression at the API level (the movement system applies its
        // own Consuming-tick freeze for these variants).
        assert!(!AgentState::Consuming { target: TargetKind::Food }.suppresses_movement());
        assert!(!AgentState::Consuming { target: TargetKind::Water }.suppresses_movement());
        assert!(!AgentState::Consuming { target: TargetKind::Sleep }.suppresses_movement());
        assert!(!AgentState::Consuming { target: TargetKind::ConstructionSite }
            .suppresses_movement());
    }

    #[test]
    fn serde_round_trip_each_variant() {
        let cases = [
            AgentState::Idle,
            AgentState::Seeking { target: TargetKind::Food },
            AgentState::Seeking { target: TargetKind::Water },
            AgentState::Seeking { target: TargetKind::Sleep },
            AgentState::Seeking { target: TargetKind::ConstructionSite },
            AgentState::Seeking { target: TargetKind::Agent(7) },
            AgentState::Consuming { target: TargetKind::Food },
            AgentState::Consuming { target: TargetKind::Water },
            AgentState::Consuming { target: TargetKind::Sleep },
            AgentState::Consuming { target: TargetKind::ConstructionSite },
            AgentState::Consuming { target: TargetKind::Agent(7) },
        ];
        for state in cases {
            let encoded = ron::to_string(&state).unwrap();
            let decoded: AgentState = ron::from_str(&encoded).unwrap();
            assert_eq!(state, decoded);
        }
    }

    #[test]
    fn target_kind_serde_round_trip() {
        for kind in [
            TargetKind::Food,
            TargetKind::Water,
            TargetKind::Sleep,
            TargetKind::ConstructionSite,
            TargetKind::Agent(7),
        ] {
            let encoded = ron::to_string(&kind).unwrap();
            let decoded: TargetKind = ron::from_str(&encoded).unwrap();
            assert_eq!(kind, decoded);
        }
    }
}
