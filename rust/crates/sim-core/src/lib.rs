// sim-core: Data structures only (no gameplay logic)
// All components, world data, config, and IDs live here.

pub mod calendar;
pub mod causal_log;
pub mod children_index;
pub mod config;
pub mod effect;
pub mod enums;
pub mod ids;
pub mod influence_channel;
pub mod influence_grid;
pub mod item;
pub mod item_store;
pub mod room;
pub mod temperament;
pub mod tile_grid;
pub mod wall_mask;

pub mod components;
pub mod scales;
pub mod world;

pub mod building;
pub mod band;
pub mod territory_grid;
pub mod settlement;

// Re-export commonly used types for convenience
pub use band::{Band, BandStore};
pub use building::{Building, FurniturePlan, WallPlan};
pub use calendar::GameCalendar;
pub use causal_log::{CausalEvent, CausalLog, CauseRef};
pub use children_index::ChildrenIndex;
pub use components::{EffectFlags, Inventory};
pub use config::ConfigSummary;
pub use config::SimConfig;
pub use effect::{
    EffectEntry, EffectFlag, EffectPrimitive, EffectQueue, EffectSource, EffectStat,
    InfluenceEmitter, InfluenceReceiver, ScheduledEffect,
};
pub use enums::*;
pub use ids::{BandId, BuildingId, EntityId, SettlementId, SkillId, TechId, TraitId};
pub use influence_channel::{ChannelClampPolicy, ChannelId, ChannelMeta};
pub use influence_grid::{EmitterRecord, FalloffType, InfluenceGrid};
pub use item::{EquipSlot, ItemDerivedStats, ItemId, ItemInstance, ItemOwner};
pub use item_store::ItemStore;
pub use room::{assign_room_ids, detect_rooms, Room, RoomId, RoomRole};
pub use settlement::{Settlement, STONE_AGE_TECH_IDS};
pub use temperament::{
    Temperament, TemperamentAxes, TemperamentBiasRow, TemperamentPrsWeightRow, TemperamentRuleSet,
    TemperamentShiftRuleView,
};
pub use tile_grid::{StructuralTile, TileGrid};
pub use wall_mask::WallBlockingMask;
pub use world::{Tile, WorldMap};
