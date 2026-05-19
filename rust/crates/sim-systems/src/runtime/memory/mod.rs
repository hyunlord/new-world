//! V7 Phase 8-β — Memory System module root.
//!
//! Hosts [`MemorySystem`] (priority 136, the only system in this module).
//! See `.harness/plans/phase8.md §3` Phase 8-β block for the full
//! per-event encoding table and cascade-bias semantics.

pub mod memory_system;

pub use memory_system::{MemorySystem, DECAY_RATE, MAX_RECENCY_TICKS, REINFORCEMENT_BOOST};
