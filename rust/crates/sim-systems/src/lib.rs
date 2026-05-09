//! WorldSim runtime systems.
//!
//! Phase 2 (T7.6 land): 4 influence systems via [`register_phase2_systems`].
//!
//! - [`runtime::influence::BuildingStampSystem`]       priority 90,   every tick
//! - [`runtime::influence::InfluenceUpdateSystem`]     priority 100,  every tick
//! - [`runtime::influence::AgentInfluenceSampleSystem`] priority 110, every tick
//! - [`runtime::influence::InfluenceVisualizationSystem`] priority 1000, every 6 ticks
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
