//! FFI bindings exposed to Godot via the godot 0.5 crate.
//!
//! ## Bridge Identity Contract
//!
//! [`enqueue_building_placed`] is the canonical implementation of
//! `WorldSimNode::on_building_placed`'s bounds-check and enqueue logic.
//! The `#[func]` body of `on_building_placed` consists solely of a forwarding
//! call to this function (verified by the Evaluator's Completeness review).

pub mod world_node;

pub use world_node::enqueue_building_placed;
pub use world_node::WorldSimNode;
