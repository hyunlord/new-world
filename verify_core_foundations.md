# WS-REF-003 Verification

## Commands

```bash
cd /Users/rexxa/github/new-world-wt/codex-refactor-core-foundations/rust
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

## Required Checks

### Systems compile
- `sim-core` exports:
  - `InfluenceGrid`
  - `EffectPrimitive`
  - `CausalLog`
  - `TileGrid`
  - `Room`, `RoomId`, `detect_rooms`
  - `Temperament`
- `sim-engine::SimResources` owns:
  - `influence_grid`
  - `tile_grid`
  - `rooms`
  - `causal_log`

### Systems registered / integrated
- spawned agents receive:
  - `InfluenceReceiver`
  - `Temperament`
- engine resources initialize tile/room/causal scaffolds
- `sim-data::DataRegistry` exposes typed hooks for:
  - materials
  - furniture/object influence
  - structures
  - action effects
  - world rules
  - temperament rules

### Basic simulation runs
- workspace tests pass
- engine tests confirm `SimResources` initializes the new foundations
- room detection test passes
- temperament derivation test passes
- influence emitter/receiver scaffold tests pass

## Concrete Test Evidence

### Influence propagation scaffold
- `sim-core::effect::tests::influence_emitter_converts_into_grid_record`
- `sim-core::effect::tests::influence_receiver_default_listens_to_all_channels`
- existing influence-grid suite remains green

### Room detection
- `sim-core::room::tests::room_detection_finds_enclosed_floor_region`
- `sim-core::room::tests::room_assignment_writes_room_ids_back_to_grid`

### Temperament assignment
- `sim-core::temperament::tests::temperament_from_personality_maps_hexaco_axes`
- `sim-systems::entity_spawner::tests::traits_and_faith_components_present`

### Engine integration
- `sim-engine::engine::tests::sim_resources_initialize_influence_grid_from_map_dimensions`

## Acceptance
- if all commands above pass and the listed tests remain green, `core_foundations_ready = true`
