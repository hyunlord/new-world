//! Needs runtime systems (V7 Phase 5-α / P5α-4).
//!
//! First need-decay system: [`HungerDecaySystem`] (priority 130, every
//! tick). Slots between `AgentMovementSystem` (priority 120) and
//! `InfluenceVisualizationSystem` (priority 1000) so movement still
//! reads pre-need state but viz still observes the post-need value.

pub mod hunger_decay;

pub use hunger_decay::HungerDecaySystem;
