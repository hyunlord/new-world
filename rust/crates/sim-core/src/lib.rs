// sim-core: Data structures only (no gameplay logic)
// All components, world data, config, and IDs live here.

pub mod calendar;
pub mod config;
pub mod enums;
pub mod ids;
pub mod influence_channel;
pub mod influence_grid;
pub mod wall_mask;

pub mod components;
pub mod scales;
pub mod world;

pub mod building;
pub mod settlement;

// Re-export commonly used types for convenience
pub use building::Building;
pub use calendar::GameCalendar;
pub use config::ConfigSummary;
pub use config::SimConfig;
pub use enums::*;
pub use ids::{BuildingId, EntityId, SettlementId, SkillId, TechId, TraitId};
pub use influence_channel::{ChannelId, ChannelMeta};
pub use influence_grid::{EmitterRecord, FalloffType, InfluenceGrid};
pub use settlement::Settlement;
pub use wall_mask::WallBlockingMask;
pub use world::{Tile, WorldMap};
