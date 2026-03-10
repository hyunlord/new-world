//! sim-systems: Simulation systems and hot-path algorithms.
//!
//! This crate will progressively absorb performance-critical logic from GDScript.
//! The first migrated hot path is A* pathfinding used by movement.

// Scientific simulation functions naturally have many parameters (entity state vectors).
// Numerical array operations (Cholesky decomposition, culture vectors) use index-based loops.
#![allow(clippy::too_many_arguments)]
#![allow(clippy::needless_range_loop)]

pub mod body;
pub mod entity_spawner;
pub mod math_utils;
pub mod name_generator;
pub mod pathfinding;
pub mod runtime;
pub mod stat_curve;
pub mod values_init;

pub use runtime::drain_and_apply_llm_responses;
