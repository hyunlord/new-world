//! sim-systems: Simulation systems and hot-path algorithms.
//!
//! This crate will progressively absorb performance-critical logic from GDScript.
//! The first migrated hot path is A* pathfinding used by movement.

// Scientific simulation functions naturally have many parameters (entity state vectors).
// Numerical array operations (Cholesky decomposition, culture vectors) use index-based loops.
#![allow(clippy::too_many_arguments)]
#![allow(clippy::needless_range_loop)]

pub mod math_utils;
pub mod pathfinding;
pub mod stat_curve;
pub mod body;
pub mod runtime;
pub mod entity_spawner;
