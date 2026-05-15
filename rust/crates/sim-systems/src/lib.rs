//! WorldSim runtime systems.
//!
//! Phase 2 (T7.6 land): 4 influence systems via [`register_phase2_systems`].
//!
//! - [`runtime::influence::BuildingStampSystem`]       priority 90,   every tick
//! - [`runtime::influence::InfluenceUpdateSystem`]     priority 100,  every tick
//! - [`runtime::influence::AgentInfluenceSampleSystem`] priority 110, every tick
//! - [`runtime::influence::InfluenceVisualizationSystem`] priority 1000, every 6 ticks
//!
//! Phase 4-β land: 1 agent system via [`register_agent_systems`].
//!
//! - [`runtime::agent::AgentMovementSystem`]            priority 120, every tick
//!
//! Phase 5-α land: 1 needs system via [`register_needs_systems`].
//!
//! - [`runtime::needs::HungerDecaySystem`]              priority 130, every tick
//! - [`runtime::needs::ThirstDecaySystem`]              priority 131, every tick (Phase 5-β)
//!
//! Phase 5-β land: 1 decision system via [`register_decision_systems`].
//!
//! - [`runtime::decision::AgentDecisionSystem`]         priority 125, every tick
//!
//! Phase 0 v0.1.3 patch Section 4.3 base.
//!
//! Dependencies:
//! - [`sim_core`] — ECS components, materials, influence, tile data
//! - [`sim_engine`] — `RuntimeSystem` trait + `SimResources` host

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use sim_engine::SimEngine;

pub mod runtime;

/// Register the Phase 2 influence stack on `engine` in priority order.
///
/// Registers (in priority order after sorting):
/// - 90  : [`runtime::influence::BuildingStampSystem`]
/// - 100 : [`runtime::influence::InfluenceUpdateSystem`]
/// - 110 : [`runtime::influence::AgentInfluenceSampleSystem`]
/// - 1000: [`runtime::influence::InfluenceVisualizationSystem`] (every 6 ticks)
pub fn register_phase2_systems(engine: &mut SimEngine) {
    engine.register_system(Box::new(
        runtime::influence::BuildingStampSystem::new(),
    ));
    engine.register_system(Box::new(
        runtime::influence::InfluenceUpdateSystem::new(),
    ));
    engine.register_system(Box::new(
        runtime::influence::AgentInfluenceSampleSystem::new(),
    ));
    engine.register_system(Box::new(
        runtime::influence::InfluenceVisualizationSystem::new(),
    ));
}

/// Register the Phase 4-β agent stack on `engine`.
///
/// Registers (in priority order after sorting):
/// - 120 : [`runtime::agent::AgentMovementSystem`]
///
/// Runs after `AgentInfluenceSampleSystem` (priority 110) so movement
/// operates on the post-sample tile.
pub fn register_agent_systems(engine: &mut SimEngine) {
    engine.register_system(Box::new(
        runtime::agent::AgentMovementSystem::new(),
    ));
}

/// Register the Phase 5-α/β needs stack on `engine`.
///
/// Registers (in priority order after sorting):
/// - 130 : [`runtime::needs::HungerDecaySystem`]
/// - 131 : [`runtime::needs::ThirstDecaySystem`] (Phase 5-β)
///
/// Both run after `AgentDecisionSystem` (priority 125) so the decision
/// system reads pre-decay need values, and before
/// `InfluenceVisualizationSystem` (1000) so the visualisation observes
/// the post-decay values.
pub fn register_needs_systems(engine: &mut SimEngine) {
    engine.register_system(Box::new(
        runtime::needs::HungerDecaySystem::new(),
    ));
    engine.register_system(Box::new(
        runtime::needs::ThirstDecaySystem::new(),
    ));
}

/// Register the Phase 5-β decision stack on `engine`.
///
/// Registers (in priority order after sorting):
/// - 125 : [`runtime::decision::AgentDecisionSystem`]
///
/// Slots between `AgentMovementSystem` (priority 120) and
/// `HungerDecaySystem` (priority 130) so it observes the post-move
/// position and the pre-decay need values for the current tick.
pub fn register_decision_systems(engine: &mut SimEngine) {
    engine.register_system(Box::new(
        runtime::decision::AgentDecisionSystem::new(),
    ));
}
