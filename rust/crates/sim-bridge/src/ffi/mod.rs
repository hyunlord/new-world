//! FFI bindings exposed to Godot via the godot 0.5 crate.
//!
//! ## Bridge Identity Contract
//!
//! [`enqueue_building_placed`] is the canonical implementation of
//! `WorldSimNode::on_building_placed`'s bounds-check and enqueue logic.
//! The `#[func]` body of `on_building_placed` consists solely of a forwarding
//! call to this function (verified by the Evaluator's Completeness review).
//!
//! V7 Phase 3-γ (γ-1) extends the contract with two read-only causal
//! getters. [`collect_tile_causal_history`] and [`collect_event_chain`]
//! are the canonical pure-Rust collectors backing
//! `WorldSimNode::get_tile_causal_history` and `WorldSimNode::get_event_chain`;
//! the `#[func]` bodies marshal the returned [`CausalEventView`] slices into
//! Godot dictionaries. Sim-test exercises the collectors directly.

pub mod activity;
pub mod causal;
pub mod chronicle;
pub mod colony;
pub mod economy;
pub mod engine_build;
pub mod family;
pub mod knowledge;
pub mod node;
pub mod reads_agent;
pub mod reads_settlement;
pub mod relations;
pub mod save;
pub mod snapshot;
pub mod space;

pub use activity::{collect_activity_snapshot, ActivitySnapshotRow};
pub use colony::{
    can_place, clip_rect, collect_agent_jobs, collect_colony_overlay, collect_colony_versions,
    collect_trees, submit_blueprint, submit_rect, submit_tiles, ColonyOverlay, ColonyVersions,
    OverlaySite, RectCommand,
};
pub use economy::{
    collect_agent_economy, collect_recipes, collect_work_priorities, collect_workshops,
    levels_in_work_order, parse_economy, submit_economy, AgentEconomyRow, BillRow, EconomyCommand,
    PriorityRow, WorkshopRow,
};
pub use family::{collect_agent_family, AgentFamilyRow};
pub use knowledge::{
    collect_agent_knowledge, collect_knowledge_waits, collect_teach_orders, collect_tech,
    collect_tech_node, AgentKnowledgeView, KnowledgeRow, TeachRow, TechNodeView, TechRow, TechView,
    WaitRow,
};
pub use space::{
    collect_place_preview, collect_room_detail, collect_rooms, collect_space, submit_space,
    PlacePreview, RoomDetail, RoomRow, RoomsView, SpaceCommand, SpaceView,
};

pub use causal::{
    collect_event_chain, collect_tile_causal_history, event_view_to_owned_dict,
    tile_idx_from_coords, try_collect_event_chain, try_collect_tile_causal_history,
    CausalEventView, FfiFieldValue,
};
pub use engine_build::{
    bootstrap_spawn_agents, init_production_engine, seed_finite_resource_scarcity,
};
pub use engine_build::{build_game_engine, build_stress_engine};
pub use engine_build::{
    INITIAL_FOOD, INITIAL_SLEEP, INITIAL_WATER, SOURCE_FOOD, SOURCE_SLEEP, SOURCE_WATER,
};
pub use node::enqueue_building_placed;
pub use node::WorldSimNode;
pub use node::{clamp_sim_speed, seconds_per_day, MAX_TICKS_PER_FRAME, TICKS_PER_SECOND_AT_1X};
pub use reads_agent::{collect_agent_detail, AgentDetailRow, AGENT_DETAIL_DICT_KEYS};
pub use reads_agent::{collect_relationship_snapshot, RelationshipSnapshotRow};
pub use reads_settlement::{
    collect_settlement_snapshot, collect_settlement_wood_targets, SettlementSnapshotRow,
};
pub use save::{load_engine_from_file, save_engine_to_file};
pub use snapshot::{agent_rows_split, collect_agent_snapshot, AgentSnapshotRow};
pub use snapshot::{
    collect_agent_ages_days, collect_agent_ages_years, collect_agent_colds,
    collect_agent_infancy_and_pregnancy,
};
pub use snapshot::{collect_building_snapshot, collect_terrain_snapshot, collect_world_info};
pub use snapshot::{collect_construction_snapshot, ConstructionSnapshotRow};
pub use snapshot::{collect_recent_deaths, recent_death_rows_split, RecentDeathRow};
pub use snapshot::{
    collect_resource_snapshot, resource_rows_amounts, resource_rows_split, ResourceSnapshotRow,
};
pub use snapshot::{collect_tallies, collect_time_snapshot};
