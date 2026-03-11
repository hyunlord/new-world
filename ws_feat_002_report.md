# WS-FEAT-002 Report

## Implementation Intent

Danger avoidance is the correct second survival slice because Food attraction alone only proves positive attraction. Survival behavior also needs the opposite case: agents must move away from harmful areas when threat pressure is high. That makes danger avoidance the first real negative steering slice on top of the Influence Grid.

Influence-based avoidance is preferred over direct targeting because it preserves the same spatial-causality architecture established by Food and Warmth:

- danger source
- Danger emitter
- local propagation
- agent sampling
- fear-weighted avoidance
- causal movement

This keeps agents local and non-omniscient. They react to the shape of nearby threat in space, not to a perfect global lookup of all hazards.

## How It Was Implemented

Danger sources for this ticket are deliberately small and concrete:

- hazardous terrain tiles already emitted `ChannelId::Danger`
- completed campfires now also emit danger
- registry-backed `fire_pit` data in `/Users/rexxa/github/new-world-wt/codex-ws-feat-002-danger-avoidance/rust/crates/sim-data/data/furniture/basic.ron` now includes a declarative Danger emission
- the runtime still reaches that content through the existing `campfire -> fire_pit` bridge helper in `/Users/rexxa/github/new-world-wt/codex-ws-feat-002-danger-avoidance/rust/crates/sim-systems/src/runtime/influence.rs`
- fallback no-registry campfire logic in the same file also stamps Danger for compatibility when registry content is unavailable

Danger propagation ownership remains inside `InfluenceRuntimeSystem`. No new parallel authority path was introduced. The same runtime already rebuilding Food and Warmth emitters now refreshes danger emitters as part of the deterministic grid rebuild.

Agent sampling remains in `/Users/rexxa/github/new-world-wt/codex-ws-feat-002-danger-avoidance/rust/crates/sim-systems/src/runtime/steering.rs`. The key runtime change is that danger avoidance is now explicitly fear-driven:

- `NeedType::Safety` deficit still matters
- `Stress.level` still matters
- `EmotionType::Fear` is now also sampled directly
- the avoidance drive uses the maximum of safety deficit, stress, and fear

That drive is then multiplied by the existing danger influence weight and temperament fear scaling before the negative gradient is applied.

Movement integration remains narrow:

- `SteeringRuntimeSystem` computes `food + warmth - danger`
- `MovementRuntimeSystem` still consumes resulting velocity
- no generic steering rewrite was introduced

Causal logging remains in the steering layer. When danger dominates a meaningful movement force, the dominant cause is logged as `danger_gradient`.

Changed files:

- `/Users/rexxa/github/new-world-wt/codex-ws-feat-002-danger-avoidance/rust/crates/sim-core/src/config.rs`
  - added fallback campfire danger intensity constant
- `/Users/rexxa/github/new-world-wt/codex-ws-feat-002-danger-avoidance/rust/crates/sim-data/data/furniture/basic.ron`
  - added registry-backed Danger emission for `fire_pit`
- `/Users/rexxa/github/new-world-wt/codex-ws-feat-002-danger-avoidance/rust/crates/sim-data/src/registry.rs`
  - strengthened registry helper test so `fire_pit` must expose danger emission
- `/Users/rexxa/github/new-world-wt/codex-ws-feat-002-danger-avoidance/rust/crates/sim-systems/src/runtime/influence.rs`
  - added fallback campfire danger emitter
  - added focused danger emitter / wall blocking tests
- `/Users/rexxa/github/new-world-wt/codex-ws-feat-002-danger-avoidance/rust/crates/sim-systems/src/runtime/steering.rs`
  - wired `EmotionType::Fear` into avoidance weighting
  - added focused danger/fear/conflict tests

## What Feature It Adds

Agents can now treat danger as a true negative spatial influence instead of only a passive stat. This enables visible behaviors such as:

- agents moving away from harmful fire-adjacent areas when fear is high
- agents preferring safer local routes instead of blindly following food attraction
- resource zones becoming contested when they overlap with danger
- multiple agents reacting to the same hazard field at once

The important result is that the Influence Grid now drives both attraction and avoidance behavior through the same spatial medium.

## Verification After Implementation

Commands run:

- `cd rust && cargo check --workspace`
- `cd rust && cargo test -p sim-systems -- --nocapture`
- `cd rust && cargo test -p sim-data`
- `cd rust && cargo test --workspace`
- `cd rust && cargo clippy --workspace -- -D warnings`

Key behavioral evidence:

- danger emitter registration is proven for both terrain and campfire sources
- registry-backed campfire danger now has a distinct radius test, so registry and fallback paths are no longer conflated
- danger signal attenuates with distance
- no false avoidance appears without danger signal
- wall blocking reduces campfire danger signal
- high fear produces stronger avoidance than low fear
- in food-vs-danger conflict, high fear can override food attraction while low fear still moves toward food
- focused tests covering those cases now include:
  - `influence_runtime_system_registry_campfire_uses_registry_only_light_emission`
  - `influence_runtime_system_campfire_danger_attenuates_with_distance`
  - `influence_force_scales_with_fear_pressure`
  - `influence_force_has_no_false_danger_avoidance_without_signal`
  - `danger_overrides_food_when_fear_is_high`

Runtime properties confirmed:

- deterministic typed channel usage
- no direct global nearest-danger lookup added
- no new Godot-side simulation ownership
- avoidance still flows through the Influence Grid path

## In-Game Checks (한국어)

- 불이나 포식자 근처에서 에이전트가 자동으로 멀어지는지 확인한다.
- 음식이 가까워도 위험이 더 크면 피하는지 확인한다.
- 벽 뒤의 위험 신호가 약해지는지 확인한다.
- 위험이 없는 곳에서는 이상하게 도망치지 않는지 확인한다.
- 여러 에이전트가 동일한 위험원에서 동시에 도망치는지 확인한다.
- food attraction 동작이 깨지지 않았는지 확인한다.
- 시뮬레이션이 정상적으로 계속 진행되는지 확인한다.

## Remaining Risks

- this slice wires only terrain hazards and campfire/fire-pit danger, not predators or combat zones
- runtime building content still relies on the pre-existing `campfire -> fire_pit` and `shelter -> lean_to_structure` bridge helpers instead of a fully typed building-definition identity layer
- avoidance is still part of the current hybrid movement heuristic, not a dedicated steering-only navigation stack
- `MovementRuntimeSystem` still applies direct completion rewards for some actions, so the broader survival loop is not yet fully effect-driven
- danger response currently uses fear/stress/safety, but not a richer explicit threat-perception component
- multi-channel arbitration is still heuristic; this ticket only proves the danger-vs-food case
