#![forbid(unsafe_code)]
//! sim-bridge — V7 reset FFI integration crate.
//!
//! T7.7.A: Empty scaffold (Cargo.toml + workspace registration + this lib.rs).
//! T7.7.B will introduce:
//!   - godot dependency (workspace.dependencies)
//!   - WorldSimNode (Godot Node binding)
//!   - 3 FFI methods: get_influence_overlay / get_tile_detail / on_building_placed
//!
//! Cold-tier admission: V7 Hook Governance v3.3.8 §1 (Signal A whitelist).
//! Behavior gate (Signal D `impl RuntimeSystem for X`) remains the
//! authoritative trigger for hot-tier classification once T7.7.B ships
//! actual FFI shims that touch InfluenceGrid buffers and dispatch
//! BuildingPlacedEvent into SimResources.building_event_queue.
