# Core Systems Validation

## InfluenceGrid

- Present: yes
- Path: `rust/crates/sim-core/src/influence_grid.rs`
- Runtime resource: yes, via `SimResources.influence_grid`

## Effect primitives

- Present: yes
- Path: `rust/crates/sim-core/src/effect.rs`
- Components exposed:
  - `InfluenceEmitter`
  - `InfluenceReceiver`

## CausalLog

- Present: yes
- Path: `rust/crates/sim-core/src/causal_log.rs`
- Runtime resource: yes, via `SimResources.causal_log`

## TileGrid

- Present: yes
- Path: `rust/crates/sim-core/src/tile_grid.rs`
- Runtime resource: yes, via `SimResources.tile_grid`

## Room system

- Present: scaffold/basic detection
- Paths:
  - `rust/crates/sim-core/src/room.rs`
  - `RoomId` exposed in components
- Runtime integration:
  - room data structures exist
  - full gameplay integration remains limited

## Temperament component

- Present: yes
- Path: `rust/crates/sim-core/src/temperament.rs`
- Spawn integration: yes, via `sim-systems/src/entity_spawner.rs`

## Validation verdict

- required foundations exist and compile
- they are partially integrated into ECS/resources/spawn paths
- they are not all yet fully exploited by gameplay systems
