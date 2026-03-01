//! sim-systems: Simulation systems and hot-path algorithms.
//!
//! This crate will progressively absorb performance-critical logic from GDScript.
//! The first migrated hot path is A* pathfinding used by movement.

pub mod pathfinding;
pub mod stat_curve;
pub mod body;
