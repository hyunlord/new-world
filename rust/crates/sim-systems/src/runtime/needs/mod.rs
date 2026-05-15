//! Needs runtime systems (V7 Phase 5-α → 5-β).
//!
//! Phase 5-α landed [`HungerDecaySystem`] (priority 130). Phase 5-β
//! adds [`ThirstDecaySystem`] (priority 131) so both daily-routine
//! needs share the same decay slot, adjacent in the schedule, applied
//! AFTER `AgentDecisionSystem` (priority 125) reads pre-decay values
//! for the current tick.

pub mod hunger_decay;
pub mod thirst_decay;

pub use hunger_decay::HungerDecaySystem;
pub use thirst_decay::ThirstDecaySystem;
