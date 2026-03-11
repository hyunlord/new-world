# WS-FEAT-001 Repository Scan

## 1. Food Source Ownership Map

- Authoritative world food sources live in Rust world tiles, not Godot.
- Runtime representation:
  - `sim-core::world::Tile.resources`
  - each entry is a `TileResource`
  - food-bearing deposits are identified by `ResourceType::Food`
- Existing runtime ownership:
  - `/Users/rexxa/github/new-world-wt/codex-ws-feat-001-food-attraction/rust/crates/sim-systems/src/runtime/influence.rs`
  - `InfluenceRuntimeSystem::collect_map_emitters()`
  - `tile_food_intensity(...)`
- Current first-slice emitter model:
  - a map tile with positive `ResourceType::Food` amount becomes a `ChannelId::Food` emitter
  - intensity scales from deposit amount through `tile_food_intensity(...)`

## 2. Current Movement Ownership Map

- Action choice ownership:
  - `/Users/rexxa/github/new-world-wt/codex-ws-feat-001-food-attraction/rust/crates/sim-systems/src/runtime/cognition.rs`
  - `BehaviorRuntimeSystem`
  - decides whether the agent is currently foraging
- Movement bias ownership:
  - `/Users/rexxa/github/new-world-wt/codex-ws-feat-001-food-attraction/rust/crates/sim-systems/src/runtime/steering.rs`
  - `SteeringRuntimeSystem`
  - samples influence and produces steering force / causal log
- Position update ownership:
  - `/Users/rexxa/github/new-world-wt/codex-ws-feat-001-food-attraction/rust/crates/sim-systems/src/runtime/world.rs`
  - `MovementRuntimeSystem`
  - consumes current action plus movement state and advances the agent
- Movement style today:
  - hybrid
  - action/target based behavior selection
  - influence-driven steering force layered into movement
  - tile-step/path heuristic movement rather than pure continuous steering

## 3. Current Hunger Data Ownership Map

- Shared hunger state:
  - `sim-core::components::Needs`
  - `NeedType::Hunger`
- Hunger decay / diagnostics ownership:
  - `/Users/rexxa/github/new-world-wt/codex-ws-feat-001-food-attraction/rust/crates/sim-systems/src/runtime/needs.rs`
  - `NeedsRuntimeSystem`
- Hunger action selection ownership:
  - `/Users/rexxa/github/new-world-wt/codex-ws-feat-001-food-attraction/rust/crates/sim-systems/src/runtime/cognition.rs`
  - hunger deficit contributes to `ActionType::Forage`
- Existing hunger restore path:
  - `/Users/rexxa/github/new-world-wt/codex-ws-feat-001-food-attraction/rust/crates/sim-systems/src/runtime/world.rs`
  - `MovementRuntimeSystem`
  - explicit `Forage` completion still restores hunger directly after arrival/completion

## 4. Current Influence Sampling Locations

- Food / danger / warmth steering samples:
  - `/Users/rexxa/github/new-world-wt/codex-ws-feat-001-food-attraction/rust/crates/sim-systems/src/runtime/steering.rs`
  - `influence_force_for_entity(...)`
  - uses `InfluenceGrid::sample_gradient(...)`
- Warmth need sampling precedent:
  - `/Users/rexxa/github/new-world-wt/codex-ws-feat-001-food-attraction/rust/crates/sim-systems/src/runtime/needs.rs`
  - uses `ChannelId::Warmth`
- Runtime emitter refresh precedent:
  - `/Users/rexxa/github/new-world-wt/codex-ws-feat-001-food-attraction/rust/crates/sim-systems/src/runtime/influence.rs`
  - `InfluenceRuntimeSystem`

## 5. Recommended Insertion Point For Food Attraction

- Best insertion point for this ticket:
  - `/Users/rexxa/github/new-world-wt/codex-ws-feat-001-food-attraction/rust/crates/sim-systems/src/runtime/cognition.rs`
  - `behavior_assign_action(...)`
- Reason:
  - this is where forage targets were previously assigned through direct nearest-food lookup
  - replacing only the target acquisition step avoids a framework rewrite
  - behavior still decides to forage from hunger pressure
  - movement still uses the current runtime path
  - target selection can now be influence-driven instead of omniscient resource search

## 6. Architectural Risk Discovered During Scan

- The largest architecture violation for this ticket was not missing Food influence propagation.
- It was the direct nearest-food lookup in `BehaviorRuntimeSystem`, which bypassed the grid:
  - old path used direct world search for `ResourceType::Food`
  - this made forage targeting omniscient even though Food influence already existed
- This ticket's recommended correction is therefore:
  - keep map-tile food emitters in `InfluenceRuntimeSystem`
  - keep hunger-weighted steering in `SteeringRuntimeSystem`
  - replace direct forage target lookup with bounded local Food influence lookup in `BehaviorRuntimeSystem`
