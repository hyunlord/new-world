# WS-FEAT-001 Report

## Implementation Intent

Food Attraction is the correct next vertical slice after Warmth because it is the first survival loop where agents must not just react to ambient conditions, but actively move toward a resource source. That makes it the clearest test of whether the Influence Grid can drive real survival behavior rather than only passive status effects.

This ticket validates Influence Grid as a survival-behavior system because the full path is now:

- food-bearing tile
- Food emitter refresh
- Food influence propagation
- hunger-sensitive sampling
- movement bias / forage target choice
- causal log entry

Direct omniscient nearest-resource attraction was intentionally avoided because it would hide whether the spatial causality layer is actually working. Hunger-sensitive response matters because survival behavior should scale with actual need pressure: hungry agents should react more strongly than sated agents under the same local signal.

## How It Was Implemented

Food emitters are sourced from map tiles that already contain positive `ResourceType::Food` deposits. That source model already existed in `InfluenceRuntimeSystem`, so this ticket kept the smallest correct representation instead of inventing a new content layer.

The runtime refresh path remains:

- `/Users/rexxa/github/new-world-wt/codex-ws-feat-001-food-attraction/rust/crates/sim-systems/src/runtime/influence.rs`
- `InfluenceRuntimeSystem`
- `collect_map_emitters(...)`
- `tile_food_intensity(...)`

This ticket strengthened that path with focused tests proving that:

- food emitters are registered
- emitter count does not leak across refreshes
- near tiles receive stronger Food signal than far tiles

Food influence is sampled in two places:

- `/Users/rexxa/github/new-world-wt/codex-ws-feat-001-food-attraction/rust/crates/sim-systems/src/runtime/steering.rs`
  - `influence_force_for_entity(...)`
  - hunger pressure scales the Food gradient force
- `/Users/rexxa/github/new-world-wt/codex-ws-feat-001-food-attraction/rust/crates/sim-systems/src/runtime/cognition.rs`
  - `behavior_assign_action(...)`
  - forage target selection now uses bounded local Food influence rather than direct nearest-food lookup

The key vertical-slice change is in `BehaviorRuntimeSystem`. The old direct resource search path was replaced with `find_best_influence_tile(...)`, which:

- scans only a bounded local radius
- samples `ChannelId::Food`
- ignores passability-invalid tiles
- requires a minimum Food signal
- prefers stronger signal, then shorter distance, then deterministic coordinate order

Movement integration stays deliberately minimal and compatible with the current hybrid architecture:

- behavior still decides whether to forage
- steering still produces hunger-weighted Food force
- movement still advances the agent through the existing runtime path

Causal logging already existed in the steering path and remains the meaningful place to explain food-driven movement. When a Food gradient actually contributes force, the system writes a causal entry tied to the agent and the applied influence cause.

Changed files:

- `/Users/rexxa/github/new-world-wt/codex-ws-feat-001-food-attraction/rust/crates/sim-core/src/config.rs`
  - added bounded constants for influence-driven forage targeting
- `/Users/rexxa/github/new-world-wt/codex-ws-feat-001-food-attraction/rust/crates/sim-systems/src/runtime/cognition.rs`
  - replaced direct nearest-food lookup with bounded Food influence target selection
  - added behavior tests
- `/Users/rexxa/github/new-world-wt/codex-ws-feat-001-food-attraction/rust/crates/sim-systems/src/runtime/influence.rs`
  - added food emitter refresh / attenuation tests
- `/Users/rexxa/github/new-world-wt/codex-ws-feat-001-food-attraction/rust/crates/sim-systems/src/runtime/steering.rs`
  - added hunger-sensitive force and movement outcome tests

## What Feature It Adds

Hungry agents now begin moving toward food-rich areas through the Influence Grid instead of only through omniscient direct resource search. Food sources become local attractors, and agents under stronger hunger pressure respond more strongly than agents who are relatively sated.

This makes several future behaviors structurally possible without changing the architectural pattern:

- clustering around edible resource regions
- competition around shared food gradients
- migration toward richer areas
- later conflict between Food, Danger, Warmth, and Social signals

## Verification After Implementation

Commands run:

- `cd rust && cargo test -p sim-systems -- --nocapture`
- `cd rust && cargo check --workspace`
- `cd rust && cargo test --workspace`
- `cd rust && cargo clippy --workspace -- -D warnings`

Results:

- `cargo test -p sim-systems` PASS
- `cargo check --workspace` PASS
- `cargo test --workspace` PASS
- `cargo clippy --workspace -- -D warnings` PASS

Key behavioral evidence:

- food emitter registration test proves food-bearing tiles create emitters and refresh without stale leak
- food signal attenuation test proves nearby Food signal is stronger than distant Food signal
- hunger-sensitive attraction test proves high-hunger force magnitude is stronger than low-hunger force magnitude
- movement outcome test proves a hungry agent ends closer to food than a sated agent under otherwise equivalent conditions
- no-signal safety test proves no false Food attraction is generated when no emitter exists

Current limitations still remaining are listed below rather than hidden.

## In-Game Checks (한국어)

반드시 아래 내용을 실제 게임에서 확인해야 한다.

- 배고픈 에이전트가 음식이 있는 방향으로 이동하는지
- 배고픔이 낮은 에이전트는 같은 음식에 덜 민감하게 반응하는지
- 가까운 음식이 먼 음식보다 더 강하게 끌어당기는지
- 음식이 없는 곳으로는 이상한 쏠림이 생기지 않는지
- 여러 에이전트가 같은 음식원 주변으로 자연스럽게 모이는지
- 이동이 순간이동처럼 부자연스럽지 않은지
- 기존 Warmth 관련 동작이 회귀하지 않았는지
- 시뮬레이션이 정상적으로 계속 진행되는지

## Remaining Risks

- 현재 이 슬라이스에서 Food emitter로 연결된 것은 맵 타일의 `ResourceType::Food` 뿐이다.
- 이동은 여전히 기존 hybrid 행동/이동 구조 위에 얹힌 heuristic bias이며, 전용 steering-only navigation 체계는 아니다.
- `MovementRuntimeSystem`에는 아직 `Forage` 완료 시 직접적인 허기 회복 경로가 남아 있다. 즉, 채집 성공과 소비 성공의 완전한 분리는 아직 안 됐다.
- wood / stone 같은 다른 자원 대상 선택에는 여전히 직접 자원 조회 경로가 남아 있으며, 이 티켓 범위 밖이다.
- danger / food / warmth 간의 본격적인 다중 채널 갈등 해소는 아직 구현하지 않았다.
