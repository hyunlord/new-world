//! Shared agent mechanics and legacy-compatible selection helpers.
//!
//! Prioritized need selection, memory cascade labels and routine lottery remain
//! legacy-compatible APIs; they are not the data-driven mind evaluator.

pub(crate) mod consume;
pub(crate) mod forage;
pub(crate) mod memory_bias;
pub(crate) mod needs;
pub(crate) mod routine;
pub(crate) mod targets;
pub(crate) mod wood;

pub use needs::{
    actionable_need_with_fatigue_threshold, clock_in_window, fatigue_threshold_at, NeedAvailability,
};
