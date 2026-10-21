//! V7 Phase 4-β — Agent runtime systems.
//!
//! Owns the per-tick motion system for canonical
//! [`sim_core::components::Agent`] entities. Phase 4-α landed the canonical
//! components and `SimEngine::spawn_agent`; this module adds the priority-120
//! [`AgentMovementSystem`] so agents actually move on the tile grid.
//!
//! Future Phase 4 sub-stages (γ sprite, δ BodyHealth) extend this module
//! with rendering hooks and a health subsystem respectively.

pub mod aging;
pub mod beds;
/// M1.5 per-woman fertility (`BirthRule::Fertility`).
pub mod fertility;
/// Shared household partition and family-aware departure selection.
pub mod households;
pub mod maturation;
pub mod movement;
/// Daily partner matching and family event observation.
pub mod pairing;

pub use movement::{haul_due, hauls_home, AgentMovementSystem, MovementRng};

pub use aging::AgingSystem;
pub use beds::BedAssignmentSystem;
pub use fertility::FertilitySystem;
pub use maturation::MaturationSystem;
pub use pairing::PairingSystem;
