//! Needs runtime systems (V7 Phase 5-α → 5-β → 5-γ).
//!
//! Phase 5-α landed [`HungerDecaySystem`] (priority 130). Phase 5-β
//! added [`ThirstDecaySystem`] (priority 131). Phase 5-γ adds
//! [`SleepDecaySystem`] (priority 132) so all three daily-routine
//! needs share adjacent decay slots, applied AFTER
//! `AgentDecisionSystem` (priority 125) reads pre-decay values for
//! the current tick.

pub mod hunger_decay;
pub mod sleep_decay;
pub mod social_decay;
pub mod thirst_decay;

pub use hunger_decay::HungerDecaySystem;
pub use sleep_decay::SleepDecaySystem;
pub use social_decay::SocialDecaySystem;
pub use thirst_decay::ThirstDecaySystem;
