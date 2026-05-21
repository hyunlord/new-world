//! V7 Phase 10-β — Settlement runtime stack.
//!
//! Hosts [`SettlementSystem`] (priority 138). Sole owner of:
//!   - Auto-formation scan (P10Plan-2-a): cluster agents + buildings within
//!     `SETTLEMENT_PROXIMITY_RADIUS` and instantiate a new
//!     [`Settlement`](sim_core::components::Settlement) when both formation
//!     thresholds are met.
//!   - Membership sync: add/remove member agents based on proximity each
//!     tick; keep `population_stats.current` consistent with `member_agents`.
//!   - Community history ingestion: append `BuildingPlaced` /
//!     `CombatCompleted` / `AgentBorn` event ids to each settlement's
//!     bounded community history.
//!   - Birth trigger (P10Plan-5): spawn a fresh agent (with full
//!     need/state components) near a settlement member when the cooldown
//!     has elapsed and the settlement is below `SETTLEMENT_MAX_POP`.
//!   - Dissolution (P10Plan-3): remove settlements whose population fell
//!     to zero AND whose buildings are all gone; emit
//!     `CausalEvent::SettlementDissolved`.

pub mod settlement_system;
pub use settlement_system::{SettlementSystem, BIRTH_COOLDOWN_TICKS};
