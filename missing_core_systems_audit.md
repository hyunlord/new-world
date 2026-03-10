# WS-REF-003 Missing Core Systems Audit

## Scope
- Repository: `/Users/rexxa/github/new-world-wt/codex-refactor-core-foundations`
- Focus crates:
  - `rust/crates/sim-core`
  - `rust/crates/sim-engine`
  - `rust/crates/sim-systems`
  - `rust/crates/sim-data`

## Required Systems vs Repo Reality

| System | Required by ticket | Repo state before this ticket | Evidence |
|---|---|---|---|
| Influence Grid | Yes | Already existed from prior A-2 work | `rust/crates/sim-core/src/influence_grid.rs` |
| Effect primitives | Yes | Missing | no `rust/crates/sim-core/src/effect.rs` before ticket |
| CausalLog | Yes | Missing in `sim-core`; only `sim-engine/src/explain_log.rs` existed as a different diagnostic ring buffer | `rust/crates/sim-engine/src/explain_log.rs` |
| TileGrid | Yes | Missing in `sim-core` | no `rust/crates/sim-core/src/tile_grid.rs` before ticket |
| Room system | Yes | Missing in `sim-core`; shelter blocking logic was embedded in runtime economy code | `rust/crates/sim-systems/src/runtime/economy.rs` |
| Temperament component/model | Yes | Missing in `sim-core`; only declarative rules in `sim-data` and legacy personality component existed | `rust/crates/sim-data/src/defs/temperament.rs`, `rust/crates/sim-core/src/components/personality.rs` |

## ECS Foundation Gaps Identified

### Missing shared ECS components
- `InfluenceEmitter`
- `InfluenceReceiver`
- `Temperament`
- `RoomId`

### Missing shared resources
- structural tile grid resource
- detected room cache
- causal event log

### Missing typed data hooks
- wall-blocking hint from material data
- furniture/structure influence emission access
- action effect access
- authoritative world-rules access
- authoritative temperament-rules access
- legacy trait lookup bridge

## Ticket Resolution Direction

### Reuse
- Keep the existing `InfluenceGrid`, `ChannelId`, and `WallBlockingMask` implementation.

### Add
- shared simulation scaffold modules in `sim-core`
- ECS components and resource integration
- typed registry accessors in `sim-data`
- minimal spawn/resource integration in `sim-systems` and `sim-engine`

### Do not add yet
- full runtime logic for effects
- full room semantics
- full causal event production
- full temperament-driven behavior systems

## Exit Criteria for This Ticket
- all six foundation areas exist in code
- new modules compile
- ECS can carry the new scaffold components
- engine resources include tile/room/causal state
- data hooks expose foundation inputs
- basic tests prove influence, room, and temperament scaffolds are live
