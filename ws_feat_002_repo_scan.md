# WS-FEAT-002 Repository Scan

## 1. Current Danger Representations

- Typed danger channel already exists:
  - `/Users/rexxa/github/new-world-wt/codex-ws-feat-002-danger-avoidance/rust/crates/sim-core/src/influence_channel.rs`
  - `ChannelId::Danger`
- Current map/runtime danger representations already present before this ticket:
  - hazardous terrain in `/Users/rexxa/github/new-world-wt/codex-ws-feat-002-danger-avoidance/rust/crates/sim-systems/src/runtime/influence.rs`
  - `tile_danger_intensity(...)`
  - sources include `DeepWater`, `ShallowWater`, `Mountain`, and some non-passable tiles
- Current agent-side danger pressure signals:
  - `NeedType::Safety`
  - `Stress.level`
  - `EmotionType::Fear` exists in runtime psychology but was not yet wired into influence avoidance before this ticket

## 2. Possible Danger Emitter Sources

- Existing and smallest correct source already supported by runtime:
  - hazardous terrain tiles via `tile_danger_intensity(...)`
- Existing fire-adjacent source available in content/runtime:
  - completed `campfire` buildings
  - registry-backed furniture id: `fire_pit`
  - file: `/Users/rexxa/github/new-world-wt/codex-ws-feat-002-danger-avoidance/rust/crates/sim-data/data/furniture/basic.ron`
- Not chosen for this ticket:
  - predator entities
  - combat zones
  - special hostile groups
  - these are either absent or would expand scope beyond the smallest vertical slice

## 3. Agent Movement Hook Location

- Action selection ownership:
  - `/Users/rexxa/github/new-world-wt/codex-ws-feat-002-danger-avoidance/rust/crates/sim-systems/src/runtime/cognition.rs`
  - `BehaviorRuntimeSystem`
- Influence sampling and force composition:
  - `/Users/rexxa/github/new-world-wt/codex-ws-feat-002-danger-avoidance/rust/crates/sim-systems/src/runtime/steering.rs`
  - `SteeringRuntimeSystem`
  - `influence_force_for_entity(...)`
- Position update ownership:
  - `/Users/rexxa/github/new-world-wt/codex-ws-feat-002-danger-avoidance/rust/crates/sim-systems/src/runtime/world.rs`
  - `MovementRuntimeSystem`

Recommended insertion point for this ticket:

- keep emitter refresh inside `InfluenceRuntimeSystem`
- keep avoidance weighting in `SteeringRuntimeSystem`
- do not rewrite `MovementRuntimeSystem`

This is the narrowest place to add danger avoidance without introducing a generic steering rewrite.

## 4. Interaction With Food Attraction Logic

- Food attraction already exists in:
  - `/Users/rexxa/github/new-world-wt/codex-ws-feat-002-danger-avoidance/rust/crates/sim-systems/src/runtime/steering.rs`
  - `ChannelId::Food`
  - hunger-weighted positive gradient
- Existing steering composition already had the correct narrow shape for this ticket:
  - `food + warmth - danger`
- The gap found during scan was not missing danger subtraction itself.
- The important missing piece for this ticket was:
  - making danger response explicitly fear-driven
  - proving that real danger sources refresh correctly
  - proving that danger can beat food when fear is high

## 5. Recommended Ticket Scope

The smallest correct danger-avoidance slice is:

- keep typed `ChannelId::Danger`
- keep `InfluenceRuntimeSystem` as the refresh owner
- add a real fire-adjacent danger source through `campfire` / `fire_pit`
- feed `EmotionType::Fear` into danger weighting
- add focused tests for:
  - emitter registration
  - distance attenuation
  - no-signal safety
  - wall blocking
  - food-vs-danger conflict

This preserves the existing architecture and avoids a framework rewrite.

Known limitation kept in scope:

- the current runtime still reaches registry content through the existing `campfire -> fire_pit` and `shelter -> lean_to_structure` bridge helpers in `runtime/influence.rs`
- ticket work should preserve that bridge instead of trying to redesign the full building content identity model
