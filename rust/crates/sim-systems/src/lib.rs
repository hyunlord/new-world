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
//! - [`runtime::needs::SleepDecaySystem`]               priority 132, every tick (Phase 5-γ)
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

/// Register the Phase 5-α/β/γ needs stack on `engine`.
///
/// Registers (in priority order after sorting):
/// - 130 : [`runtime::needs::HungerDecaySystem`]
/// - 131 : [`runtime::needs::ThirstDecaySystem`] (Phase 5-β)
/// - 132 : [`runtime::needs::SleepDecaySystem`] (Phase 5-γ)
/// - 135 : [`runtime::needs::SocialDecaySystem`] (Phase 7-β re-plan)
///
/// All four run after `AgentDecisionSystem` (priority 125) so the
/// decision system reads pre-decay need values, and before
/// `InfluenceVisualizationSystem` (1000) so the visualisation observes
/// the post-decay values. `SocialDecaySystem` sits at 135 (after
/// `SocialInteractionSystem` at 134) so the social handshake/completion
/// fires on pre-decay `loneliness`, and the loneliness advance applies
/// afterward — mirroring the Hunger/Thirst/Sleep pre-decision pattern.
pub fn register_needs_systems(engine: &mut SimEngine) {
    engine.register_system(Box::new(
        runtime::needs::HungerDecaySystem::new(),
    ));
    engine.register_system(Box::new(
        runtime::needs::ThirstDecaySystem::new(),
    ));
    engine.register_system(Box::new(
        runtime::needs::SleepDecaySystem::new(),
    ));
    engine.register_system(Box::new(
        runtime::needs::SocialDecaySystem::new(),
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

/// Register the Phase 6-β construction stack on `engine`.
///
/// Registers (in priority order after sorting):
/// - 133 : [`runtime::construction::ConstructionSystem`]
///
/// Slots strictly after `SleepDecaySystem` (priority 132) and strictly
/// before `InfluenceVisualizationSystem` (priority 1000). Owns agent
/// `Consuming { ConstructionSite }` exit semantics + completion-edge
/// CausalEvent emission. See `runtime::construction::ConstructionSystem`
/// for full responsibilities.
pub fn register_construction_systems(engine: &mut SimEngine) {
    engine.register_system(Box::new(
        runtime::construction::ConstructionSystem::new(),
    ));
}

/// Register the Phase 7-β social interaction stack on `engine`.
///
/// Registers (in priority order after sorting):
/// - 134 : [`runtime::social::SocialInteractionSystem`]
///
/// Slots strictly after `ConstructionSystem` (priority 133) and strictly
/// before `InfluenceVisualizationSystem` (priority 1000). Owns agent
/// `Consuming { Agent(_) }` exit semantics + completion-edge
/// `SocialInteractionCompleted` emission. `AgentDecisionSystem`
/// (priority 125) owns the Idle→Seeking and Seeking→Consuming
/// transitions plus `SocialInteractionStarted` emission.
pub fn register_social_systems(engine: &mut SimEngine) {
    engine.register_system(Box::new(
        runtime::social::SocialInteractionSystem::new(),
    ));
}

/// Register the Phase 8-β memory stack on `engine`.
///
/// Registers (in priority order after sorting):
/// - 136 : [`runtime::memory::MemorySystem`]
///
/// Slots strictly after `SocialDecaySystem` (priority 135) so the encoding
/// pass observes every same-tick actor event emitted by upstream
/// production systems (Hunger/Thirst/Sleep/Construction/Social) and the
/// `AgentDecisionSystem`. The decay pass applies uniformly to all `Memory`
/// components, no eviction at the per-tick boundary.
pub fn register_memory_systems(engine: &mut SimEngine) {
    engine.register_system(Box::new(runtime::memory::MemorySystem::new()));
}

/// V7 Phase 7-β / P7β-15 — canonical production system registration.
///
/// Single source of truth for "what systems run in a production engine":
/// callable from both `sim-bridge::ffi::world_node::WorldSimNode::init`
/// (the live Godot path) and harness tests that need a production-
/// equivalent system list without a Godot runtime.
///
/// Registration order (post-sort by priority):
/// - 90   BuildingStampSystem
/// - 100  InfluenceUpdateSystem
/// - 110  AgentInfluenceSampleSystem
/// - 120  AgentMovementSystem
/// - 125  AgentDecisionSystem
/// - 130  HungerDecaySystem
/// - 131  ThirstDecaySystem
/// - 132  SleepDecaySystem
/// - 133  ConstructionSystem
/// - 134  SocialInteractionSystem
/// - 135  SocialDecaySystem (Phase 7-β re-plan)
/// - 1000 InfluenceVisualizationSystem
///
/// Harness A1b inspects this registry to verify
/// `SocialInteractionSystem` appears exactly once.
pub fn register_default_runtime_systems(engine: &mut SimEngine) {
    register_phase2_systems(engine);
    register_agent_systems(engine);
    register_decision_systems(engine);
    register_needs_systems(engine);
    register_construction_systems(engine);
    register_social_systems(engine);
    register_memory_systems(engine);
}
