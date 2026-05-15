//! Runtime systems organization.
//!
//! Phase 2 (T7.6 land): 4 influence RuntimeSystems.
//! Phase 4-β land: `agent::AgentMovementSystem` (priority 120, every tick).
//! Phase 5-α land: `needs::HungerDecaySystem` (priority 130, every tick).
//! Phase 5-β land: `decision::AgentDecisionSystem` (priority 125, every tick) +
//! `needs::ThirstDecaySystem` (priority 131, every tick).
//!
//! Future phases:
//! - Phase 11 (Building Deep): `pub mod building;`
//! - Phase 17~20 (Wildlife/Disasters): 추가

pub mod agent;
pub mod decision;
pub mod influence;
pub mod needs;
