// Note: #![forbid(unsafe_code)] removed — godot 0.5 requires
//   `unsafe impl ExtensionLibrary for SimBridgeExtension {}`
// The unsafe scope is restricted to the entry-point impl only.
#![deny(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]
//! sim-bridge — V7 reset FFI integration crate.
//!
//! T7.7.B land + V7 Phase 3-γ (γ-1): WorldSimNode (`Node` subclass) exposing
//! 5 FFI methods.
//!
//! T7.7.B (3 methods):
//!   - `get_influence_overlay(channel: i32) -> PackedByteArray`
//!   - `get_tile_detail(x: i32, y: i32) -> Dictionary`
//!   - `on_building_placed(x: i32, y: i32, radius: i32) -> bool`
//!
//! γ-1 (2 methods — read-only causal log surface for the upcoming "왜?" UI):
//!   - `get_tile_causal_history(x: i32, y: i32) -> Array<Dictionary>`
//!   - `get_event_chain(x: i32, y: i32, event_id: i64) -> Array<Dictionary>`
//!
//! Cold-tier admission: governance v3.3.8 §1 (Signal A whitelist).
//! Behavior gate: Signal D unchanged — sim-bridge contains no
//! `impl RuntimeSystem for X`; FFI methods exit through `#[func]`.
//!
//! ## Bridge Identity Contract
//!
//! `on_building_placed`'s complete bounds-check and enqueue logic lives in
//! [`ffi::enqueue_building_placed`], a `pub fn` with a Rust-only signature.
//! The `on_building_placed` `#[func]` body consists solely of a forwarding
//! call to that function. Sim-test imports `enqueue_building_placed` directly
//! to exercise Assertions 5 and 6 without requiring Godot runtime.
//!
//! γ-1 extends this contract: [`ffi::collect_tile_causal_history`] and
//! [`ffi::collect_event_chain`] are the canonical pure-Rust collectors
//! backing the two new `#[func]` methods. The `#[func]` bodies marshal the
//! returned `Vec<CausalEventView>` into a Godot `Array<Dictionary>`. Sim-test
//! exercises the collectors directly (no Godot runtime).

use godot::prelude::*;

/// FFI surface — `WorldSimNode` and the `enqueue_building_placed` delegate.
pub mod ffi;

/// GDExtension entry point — registered by Godot at load time.
struct SimBridgeExtension;

#[gdextension]
unsafe impl ExtensionLibrary for SimBridgeExtension {}
