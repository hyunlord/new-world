# WS-FEAT-004 Report

## Implementation Intent

Social gathering is the next vertical slice because the simulation already proves three kinds of spatial behavior: attraction to food, avoidance of danger, and cold-driven shelter bias. The next missing proof is that the same Influence Grid can also create **positive group formation** without explicit membership logic.

This slice uses spatial social influence instead of direct nearest-agent queries so clustering remains local, gradual, and explainable. Lonely agents should drift toward social anchors because nearby tiles feel more socially attractive, not because the runtime assigns them to a group.

## How It Was Implemented

- Added tightly scoped social steering constants in [/Users/rexxa/github/new-world-wt/codex-ws-feat-004-social-gathering/rust/crates/sim-core/src/config.rs](/Users/rexxa/github/new-world-wt/codex-ws-feat-004-social-gathering/rust/crates/sim-core/src/config.rs):
  - `STEERING_SOCIAL_INFLUENCE_WEIGHT`
  - `STEERING_SOCIAL_MIN_LONELINESS`
  - `INFLUENCE_CAMPFIRE_SOCIAL_INTENSITY`
- Extended the registry-backed campfire furniture definition in [/Users/rexxa/github/new-world-wt/codex-ws-feat-004-social-gathering/rust/crates/sim-data/data/furniture/basic.ron](/Users/rexxa/github/new-world-wt/codex-ws-feat-004-social-gathering/rust/crates/sim-data/data/furniture/basic.ron) so `fire_pit` now emits `ChannelId::Social` in addition to warmth/danger/light.
- Updated [/Users/rexxa/github/new-world-wt/codex-ws-feat-004-social-gathering/rust/crates/sim-data/src/registry.rs](/Users/rexxa/github/new-world-wt/codex-ws-feat-004-social-gathering/rust/crates/sim-data/src/registry.rs) test coverage so loaded `fire_pit` emissions must include `social`.
- Extended [/Users/rexxa/github/new-world-wt/codex-ws-feat-004-social-gathering/rust/crates/sim-systems/src/runtime/influence.rs](/Users/rexxa/github/new-world-wt/codex-ws-feat-004-social-gathering/rust/crates/sim-systems/src/runtime/influence.rs):
  - registry-backed campfires now propagate Social automatically
  - fallback campfires emit Social as well when registry data is absent
  - focused tests now verify Social emitter registration and distance attenuation
- Extended [/Users/rexxa/github/new-world-wt/codex-ws-feat-004-social-gathering/rust/crates/sim-systems/src/runtime/steering.rs](/Users/rexxa/github/new-world-wt/codex-ws-feat-004-social-gathering/rust/crates/sim-systems/src/runtime/steering.rs):
  - computes `loneliness_drive = 1.0 - NeedType::Belonging`
  - samples `ChannelId::Social` through the existing constant-time `weighted_gradient(...)`
  - adds `social_vector` to the current movement combination
  - only activates when loneliness exceeds `STEERING_SOCIAL_MIN_LONELINESS`
  - adds causal classification `CAUSE_INFLUENCE_SOCIAL_GRADIENT`
- Added focused runtime tests proving:
  - lonely agents move toward social anchors
  - socially satisfied agents ignore the same signal
  - multiple lonely agents cluster toward a shared social anchor
  - danger still wins when fear is high

## What Feature It Adds

This slice adds the first real **group-forming** behavior driven by Influence Grid.

- campfires become local social attractors
- lonely agents begin drifting toward small gatherings
- socially satisfied agents stop overreacting to the same anchor
- clustering can now emerge without explicit group membership logic
- future leader/festival/ritual gathering can build on the same `ChannelId::Social` path

## Verification After Implementation

Commands run:

- `cd rust && cargo test -p sim-systems steering -- --nocapture`
- `cd rust && cargo test -p sim-data foundation_helpers_expose_loaded_schema_hooks -- --nocapture`
- `cd rust && cargo check --workspace`
- `cd rust && cargo test --workspace`
- `cd rust && cargo clippy --workspace -- -D warnings`

Results:

- `cargo test -p sim-systems steering -- --nocapture`: PASS
- `cargo test -p sim-data foundation_helpers_expose_loaded_schema_hooks -- --nocapture`: PASS
- `cargo check --workspace`: PASS
- `cargo test --workspace`: PASS
- `cargo clippy --workspace -- -D warnings`: PASS

Key behavioral evidence:

- campfire-backed social emitters register and attenuate with distance
- high-loneliness agents receive a positive social gradient and move toward it
- high-belonging agents ignore the same social anchor
- two lonely agents move toward the same campfire-centered social field
- danger remains dominant over social gathering when fear is high

## Remaining Risks

- this slice only wires one social anchor type: campfire / `fire_pit`
- group formation is still heuristic movement bias, not explicit social planning
- direct belonging restoration at campfires and on `ActionType::Socialize` completion still exists beside the new spatial bias
- there is still no explicit food-vs-social preference arbitration beyond the current weighted sum

## In-Game Checks (한국어)

- 캠프파이어 주변에 에이전트들이 자연스럽게 모이는지 확인한다.
- 외로운 에이전트가 그룹 쪽으로 이동하는지 확인한다.
- 사회 욕구가 낮은 에이전트는 같은 군집 신호에 덜 반응하는지 확인한다.
- food attraction과 social gathering이 동시에 있을 때 이동이 크게 깨지지 않는지 확인한다.
- danger avoidance가 social attraction보다 우선되는 상황이 보이는지 확인한다.
- 군집이 한 타일에 폭발적으로 뭉치지 않고 완만하게 형성되는지 확인한다.
- 시뮬레이션이 정상적으로 계속 진행되는지 확인한다.
