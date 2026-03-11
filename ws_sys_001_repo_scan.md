## Movement Decision Flow

1. `BehaviorRuntimeSystem` in [rust/crates/sim-systems/src/runtime/cognition.rs](/Users/rexxa/github/new-world-wt/codex-ws-sys-001-steering-layer/rust/crates/sim-systems/src/runtime/cognition.rs) chooses `Behavior.current_action` and writes `action_target_x/y`.
2. `SteeringRuntimeSystem` in [rust/crates/sim-systems/src/runtime/steering.rs](/Users/rexxa/github/new-world-wt/codex-ws-sys-001-steering-layer/rust/crates/sim-systems/src/runtime/steering.rs) derives per-agent velocity from:
   - desired action force
   - influence-driven force
   - neighbor separation / cohesion
   - direct target blend when a behavior target exists
3. `MovementRuntimeSystem` in [rust/crates/sim-systems/src/runtime/world.rs](/Users/rexxa/github/new-world-wt/codex-ws-sys-001-steering-layer/rust/crates/sim-systems/src/runtime/world.rs) consumes `Position.vel_x/vel_y` and advances entity positions.

## Systems Producing Movement Bias

- Food attraction: [rust/crates/sim-systems/src/runtime/steering.rs](/Users/rexxa/github/new-world-wt/codex-ws-sys-001-steering-layer/rust/crates/sim-systems/src/runtime/steering.rs)
- Danger avoidance: [rust/crates/sim-systems/src/runtime/steering.rs](/Users/rexxa/github/new-world-wt/codex-ws-sys-001-steering-layer/rust/crates/sim-systems/src/runtime/steering.rs)
- Shelter warmth bias: [rust/crates/sim-systems/src/runtime/steering.rs](/Users/rexxa/github/new-world-wt/codex-ws-sys-001-steering-layer/rust/crates/sim-systems/src/runtime/steering.rs)
- Social gathering: [rust/crates/sim-systems/src/runtime/steering.rs](/Users/rexxa/github/new-world-wt/codex-ws-sys-001-steering-layer/rust/crates/sim-systems/src/runtime/steering.rs)

The existing slices already share one file, but they were still computed as ad-hoc force terms instead of an explicit steering context and arbitration layer.

## Influence Sampling Usage

- Warmth diagnostics / recovery sampling: [rust/crates/sim-systems/src/runtime/needs.rs](/Users/rexxa/github/new-world-wt/codex-ws-sys-001-steering-layer/rust/crates/sim-systems/src/runtime/needs.rs)
- Food-driven forage target choice: [rust/crates/sim-systems/src/runtime/cognition.rs](/Users/rexxa/github/new-world-wt/codex-ws-sys-001-steering-layer/rust/crates/sim-systems/src/runtime/cognition.rs)
- Runtime movement bias: [rust/crates/sim-systems/src/runtime/steering.rs](/Users/rexxa/github/new-world-wt/codex-ws-sys-001-steering-layer/rust/crates/sim-systems/src/runtime/steering.rs)

## Current Movement Ownership Map

- Action selection owner: `BehaviorRuntimeSystem`
- Influence sampling owner: steering runtime
- Final velocity write owner: steering runtime
- Position integration owner: `MovementRuntimeSystem`
- Influence field ownership: `InfluenceRuntimeSystem`

## Recommended Integration Point

The correct insertion point for WS-SYS-001 is [rust/crates/sim-systems/src/runtime/steering.rs](/Users/rexxa/github/new-world-wt/codex-ws-sys-001-steering-layer/rust/crates/sim-systems/src/runtime/steering.rs), because that is already the sole runtime layer that:

- samples influence gradients
- turns them into velocity bias
- writes causal steering log entries
- runs immediately before `MovementRuntimeSystem`

This keeps cognition responsible for action choice, while the steering layer becomes the single owner of influence-driven movement composition and priority arbitration.
