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

pub mod world_node;

pub use world_node::enqueue_building_placed;
pub use world_node::{
    collect_event_chain, collect_tile_causal_history, tile_idx_from_coords,
    try_collect_event_chain, try_collect_tile_causal_history, CausalEventView,
};
pub use world_node::{agent_rows_split, collect_agent_snapshot, AgentSnapshotRow};
pub use world_node::{collect_relationship_snapshot, RelationshipSnapshotRow};
pub use world_node::WorldSimNode;
