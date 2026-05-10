// Note: #![forbid(unsafe_code)] removed — godot 0.5 requires
//   `unsafe impl ExtensionLibrary for SimBridgeExtension {}`
// The unsafe scope is restricted to the entry-point impl only.
#![deny(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]
//! sim-bridge — V7 reset FFI integration crate.
//!
//! T7.7.B land: WorldSimNode (`Node` subclass) exposing 3 FFI methods
//!   - `get_influence_overlay(channel: i32) -> PackedByteArray`
//!   - `get_tile_detail(x: i32, y: i32) -> Dictionary`
//!   - `on_building_placed(x: i32, y: i32, radius: i32) -> bool`
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

use godot::prelude::*;

/// FFI surface — `WorldSimNode` and the `enqueue_building_placed` delegate.
pub mod ffi;

/// GDExtension entry point — registered by Godot at load time.
struct SimBridgeExtension;

#[gdextension]
unsafe impl ExtensionLibrary for SimBridgeExtension {}
