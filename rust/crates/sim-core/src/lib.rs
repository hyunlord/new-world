// sim-core: Data structures only (no gameplay logic)
// All components, world data, config, and IDs live here.

pub mod calendar;
pub mod config;
pub mod causal_log;
pub mod effect;
pub mod enums;
pub mod ids;
pub mod influence_channel;
pub mod influence_grid;
pub mod room;
pub mod temperament;
pub mod tile_grid;
pub mod wall_mask;

pub mod components;
pub mod scales;
pub mod world;

pub mod building;
pub mod settlement;

// Re-export commonly used types for convenience
pub use building::Building;
pub use calendar::GameCalendar;
pub use causal_log::{CauseRef, CausalEvent, CausalLog};
pub use config::ConfigSummary;
pub use config::SimConfig;
pub use effect::{EffectFlag, EffectPrimitive, EffectStat, InfluenceEmitter, InfluenceReceiver};
pub use enums::*;
pub use ids::{BuildingId, EntityId, SettlementId, SkillId, TechId, TraitId};
pub use influence_channel::{ChannelClampPolicy, ChannelId, ChannelMeta};
pub use influence_grid::{EmitterRecord, FalloffType, InfluenceGrid};
pub use room::{assign_room_ids, detect_rooms, Room, RoomId, RoomRole};
pub use settlement::Settlement;
pub use temperament::{Temperament, TemperamentAxes};
pub use tile_grid::{StructuralTile, TileGrid};
pub use wall_mask::WallBlockingMask;
pub use world::{Tile, WorldMap};
