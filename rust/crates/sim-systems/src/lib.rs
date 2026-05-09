//! WorldSim runtime systems.
//!
//! V7 reset 후 sim-systems crate (T7.5.5.B). Phase 2 4 RuntimeSystems
//! (T7.6에서 land 의무)의 home:
//! - InfluenceUpdateSystem (priority 100, Hot/Warm/Cold dispatch)
//! - BuildingStampSystem (priority 90, event-driven)
//! - AgentInfluenceSampleSystem (priority 110, agent ECS query)
//! - InfluenceVisualizationSystem (priority 1000, every 6 ticks)
//!
//! Phase 0 v0.1.3 patch Section 4.3 base.
//!
//! Dependencies:
//! - [`sim_core`] — ECS components, materials, influence, tile data
//! - [`sim_engine`] — `RuntimeSystem` trait + `SimResources` host

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod runtime;
