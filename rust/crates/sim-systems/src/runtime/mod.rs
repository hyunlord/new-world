//! Runtime systems organization.
//!
//! Phase 2 (T7.6 land): 4 influence RuntimeSystems.
//! Phase 4-β land: `agent::AgentMovementSystem` (priority 120, every tick).
//! Phase 5-α land: `needs::HungerDecaySystem` (priority 130, every tick).
//! Phase 5-β land: `baseline::decision::AgentDecisionSystem` (priority 125, every tick) +
//! `needs::ThirstDecaySystem` (priority 131, every tick).
//!
//! Future phases:
//! - Phase 11 (Building Deep): `pub mod building;`
//! - Phase 17~20 (Wildlife/Disasters): 추가

pub mod agent;
pub mod agent_actions;
pub mod baseline;
pub(crate) mod chronicle_log;
pub mod combat;
pub mod construction;
pub mod food;
pub mod influence;
pub mod jobs;
pub mod knowledge;
pub mod memory;
pub mod mind;
pub mod mood;
pub mod nav;
pub mod needs;
pub mod neighbours;
pub mod resource_regen;
pub mod rooms;
pub mod settlement;
pub mod social;
pub mod steward;
pub mod story;
pub mod survival;
pub mod wildlife;

// Neutral public helper exports used by external runtime regression harnesses.
pub use agent_actions::memory_bias::BIAS_FLIP_THRESHOLD;
pub use agent_actions::needs::{
    minute_in_window, FAMILIARITY_BUMP, FATIGUE_CONSUME_AMOUNT, HUNGER_CONSUME_AMOUNT,
    HUNGER_THRESHOLD, REQUIRED_INTERACTION_PROGRESS, SOCIAL_CONSUME_AMOUNT, SOCIAL_THRESHOLD,
    THIRST_CONSUME_AMOUNT, THIRST_THRESHOLD,
};
pub use agent_actions::targets::{
    nearest_food_tile_in_radius, nearest_gather_tile, nearest_resource_tile, GATHER_SCAN_RADIUS,
};
