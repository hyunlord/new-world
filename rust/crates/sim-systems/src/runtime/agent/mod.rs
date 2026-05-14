//! V7 Phase 4-β — Agent runtime systems.
//!
//! Owns the per-tick motion system for canonical
//! [`sim_core::components::Agent`] entities. Phase 4-α landed the canonical
//! components and `SimEngine::spawn_agent`; this module adds the priority-120
//! [`AgentMovementSystem`] so agents actually move on the tile grid.
//!
//! Future Phase 4 sub-stages (γ sprite, δ BodyHealth) extend this module
//! with rendering hooks and a health subsystem respectively.

pub mod movement;

pub use movement::{AgentMovementSystem, MovementRng};
