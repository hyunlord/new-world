//! Phase 2 influence RuntimeSystems (T7.6 land).
//!
//! Four systems in priority order:
//! - 90  : [`BuildingStampSystem`] — building → influence stamp (no-op shell, Phase 11)
//! - 100 : [`InfluenceUpdateSystem`] — clear pending + swap double-buffer
//! - 110 : [`AgentInfluenceSampleSystem`] — read current buffer per agent
//! - 1000: [`InfluenceVisualizationSystem`] — debug digest every 6 ticks

pub mod agent_sample;
pub mod building_stamp;
pub mod update;
pub mod visualization;

pub use agent_sample::AgentInfluenceSampleSystem;
pub use building_stamp::BuildingStampSystem;
pub use update::InfluenceUpdateSystem;
pub use visualization::InfluenceVisualizationSystem;
