//! Phase 9-β Combat subsystem.
//!
//! Hosts [`CombatSystem`] (priority 137) — the runtime owner of
//! `Consuming{Agent(_)}` exit semantics for combat pairs tracked in
//! [`SimResources::combat_pairs`]. `AgentDecisionSystem` (priority 125)
//! is the sole emitter of `CombatStarted` and the sole inserter of
//! `combat_pairs` entries; this module's `CombatSystem` consumes the
//! pairs and emits `CombatCompleted`.
//!
//! [`SimResources::combat_pairs`]: sim_engine::SimResources::combat_pairs

pub mod combat_system;
pub use combat_system::CombatSystem;

/// Damage applied to defender per `CombatSystem` tick (P9Plan-5).
/// Single application resolves combat in one tick alongside
/// `REQUIRED_COMBAT_PROGRESS = 1`.
pub const DAMAGE_PER_COMBAT_TICK: f64 = 10.0;

/// Ticks of `Consuming{Agent}` required before `CombatCompleted` fires.
/// Locked at 1 — immediate resolution in the same tick as `CombatStarted`.
/// P9Plan-5.
pub const REQUIRED_COMBAT_PROGRESS: u32 = 1;
