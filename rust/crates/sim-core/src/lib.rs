// sim-core: Data structures only (no gameplay logic)
// All components, world data, config, and IDs live here.

pub mod ids;
pub mod enums;
pub mod config;
pub mod calendar;

pub mod components;
pub mod world;

pub mod settlement;
pub mod building;

// Re-export commonly used types for convenience
pub use ids::{EntityId, SettlementId, BuildingId, TraitId, SkillId, TechId};
pub use enums::*;
pub use config::ConfigSummary;
pub use calendar::GameCalendar;
pub use settlement::Settlement;
pub use building::Building;
pub use world::{WorldMap, Tile};
