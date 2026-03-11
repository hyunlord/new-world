# WS-FEAT-004 Repo Scan

## 1. Where Belonging Need Is Stored
- Belonging is stored in [`NeedType::Belonging`](/Users/rexxa/github/new-world-wt/codex-ws-feat-004-social-gathering/rust/crates/sim-core/src/ids.rs) and read/written through [`Needs`](/Users/rexxa/github/new-world-wt/codex-ws-feat-004-social-gathering/rust/crates/sim-core/src/components/needs.rs).
- Spawn defaults are set in [`generate_needs()`](/Users/rexxa/github/new-world-wt/codex-ws-feat-004-social-gathering/rust/crates/sim-systems/src/entity_spawner.rs).
- Decay is applied in [`NeedsRuntimeSystem::run()`](/Users/rexxa/github/new-world-wt/codex-ws-feat-004-social-gathering/rust/crates/sim-systems/src/runtime/needs.rs).
- Existing direct belonging recovery already exists in:
  - [`BuildingEffectRuntimeSystem::run()`](/Users/rexxa/github/new-world-wt/codex-ws-feat-004-social-gathering/rust/crates/sim-systems/src/runtime/economy.rs) for campfire proximity
  - [`MovementRuntimeSystem::run()`](/Users/rexxa/github/new-world-wt/codex-ws-feat-004-social-gathering/rust/crates/sim-systems/src/runtime/world.rs) on `ActionType::Socialize` completion

## 2. Existing Campfire or Gathering Anchors
- The smallest existing social anchor is the completed `campfire` building in [`runtime/economy.rs`](/Users/rexxa/github/new-world-wt/codex-ws-feat-004-social-gathering/rust/crates/sim-systems/src/runtime/economy.rs).
- Campfires already function as:
  - warmth anchors through [`runtime/influence.rs`](/Users/rexxa/github/new-world-wt/codex-ws-feat-004-social-gathering/rust/crates/sim-systems/src/runtime/influence.rs)
  - belonging recovery anchors through [`BuildingEffectRuntimeSystem`](/Users/rexxa/github/new-world-wt/codex-ws-feat-004-social-gathering/rust/crates/sim-systems/src/runtime/economy.rs)
- Registry-backed campfire furniture is [`fire_pit`](/Users/rexxa/github/new-world-wt/codex-ws-feat-004-social-gathering/rust/crates/sim-data/data/furniture/basic.ron), which is already the correct place to declare additional influence emissions.

## 3. Possible Leader Entities
- There is a [`LeaderRuntimeSystem`](/Users/rexxa/github/new-world-wt/codex-ws-feat-004-social-gathering/rust/crates/sim-systems/src/runtime/social.rs), but no existing typed “leader influence emitter” component or bridge path.
- Using leaders as social anchors in this ticket would require broader authority/identity wiring than necessary.
- The clean minimum slice is therefore campfire-backed social gathering.

## 4. Current Steering Integration Points
- All current influence-driven movement bias is combined in [`influence_force_for_entity()`](/Users/rexxa/github/new-world-wt/codex-ws-feat-004-social-gathering/rust/crates/sim-systems/src/runtime/steering.rs).
- Existing terms are:
  - Food attraction
  - Warmth attraction
  - Room-aware shelter bias
  - Danger avoidance
- [`weighted_gradient()`](/Users/rexxa/github/new-world-wt/codex-ws-feat-004-social-gathering/rust/crates/sim-systems/src/runtime/steering.rs) is already the correct insertion point for a new `ChannelId::Social` term.

## 5. Recommended Insertion Point for Social Gathering
- Emitter source: add `ChannelId::Social` emission to registry-backed `fire_pit` furniture data, plus a campfire fallback in [`runtime/influence.rs`](/Users/rexxa/github/new-world-wt/codex-ws-feat-004-social-gathering/rust/crates/sim-systems/src/runtime/influence.rs).
- Sampling/movement bias: add a loneliness-weighted `ChannelId::Social` gradient term in [`runtime/steering.rs`](/Users/rexxa/github/new-world-wt/codex-ws-feat-004-social-gathering/rust/crates/sim-systems/src/runtime/steering.rs).
- Loneliness signal: derive directly from `1.0 - Needs::Belonging`, which is already authoritative and deterministic.
- This keeps the slice aligned with prior patterns:
  - emitters refreshed in `runtime/influence.rs`
  - agent response applied in `runtime/steering.rs`
  - no global nearest-agent lookup
