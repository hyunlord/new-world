# WS-FEAT-003 Report

## Implementation Intent

Shelter-seeking is the next survival slice after Food attraction and Danger avoidance because it is the first slice that requires structure, room detection, and need pressure to work together. Food and Danger already prove that agents can move toward or away from spatial gradients. This ticket proves that the same Influence Grid can also encode an indoor survival advantage.

Indoor spaces matter because WorldSim is trying to model settlement behavior, not just open-field steering. If rooms retain or amplify warmth better than outdoor tiles, then agents can begin to treat shelters as spatially meaningful survival infrastructure instead of just static buildings.

## How It Was Implemented

- Added tightly scoped shelter-bias constants in [/Users/rexxa/github/new-world-wt/codex-ws-feat-003-shelter-warmth-bias/rust/crates/sim-core/src/config.rs](/Users/rexxa/github/new-world-wt/codex-ws-feat-003-shelter-warmth-bias/rust/crates/sim-core/src/config.rs) for:
  - minimum cold pressure before room bias activates
  - room warmth multiplier
  - minimum score delta for switching toward a better shelter tile
  - shelter bias weight
- Extended [/Users/rexxa/github/new-world-wt/codex-ws-feat-003-shelter-warmth-bias/rust/crates/sim-systems/src/runtime/steering.rs](/Users/rexxa/github/new-world-wt/codex-ws-feat-003-shelter-warmth-bias/rust/crates/sim-systems/src/runtime/steering.rs) with a room-aware warmth preference layer:
  - `room_shelter_force(...)` compares the current tile against local orthogonal neighbors
  - `shelter_tile_score(...)` combines sampled `ChannelId::Warmth` with `TileGrid.room_id`
  - the shelter term is added to the existing `food + warmth - danger` movement force
  - shelter bias only activates when Warmth need is low enough to indicate real cold pressure
- Reused existing structural foundations instead of inventing a new shelter search:
  - room detection in [/Users/rexxa/github/new-world-wt/codex-ws-feat-003-shelter-warmth-bias/rust/crates/sim-core/src/room.rs](/Users/rexxa/github/new-world-wt/codex-ws-feat-003-shelter-warmth-bias/rust/crates/sim-core/src/room.rs)
  - wall/path attenuation in [/Users/rexxa/github/new-world-wt/codex-ws-feat-003-shelter-warmth-bias/rust/crates/sim-core/src/influence_grid.rs](/Users/rexxa/github/new-world-wt/codex-ws-feat-003-shelter-warmth-bias/rust/crates/sim-core/src/influence_grid.rs)
  - warmth emitters and room refresh in [/Users/rexxa/github/new-world-wt/codex-ws-feat-003-shelter-warmth-bias/rust/crates/sim-systems/src/runtime/influence.rs](/Users/rexxa/github/new-world-wt/codex-ws-feat-003-shelter-warmth-bias/rust/crates/sim-systems/src/runtime/influence.rs)
  - warmth recovery in [/Users/rexxa/github/new-world-wt/codex-ws-feat-003-shelter-warmth-bias/rust/crates/sim-systems/src/runtime/needs.rs](/Users/rexxa/github/new-world-wt/codex-ws-feat-003-shelter-warmth-bias/rust/crates/sim-systems/src/runtime/needs.rs)
- Added focused steering tests covering:
  - room-aware shelter preference
  - no shelter bias when warmth need is already satisfied
  - shelter causal classification
  - regression coverage for existing food and danger steering paths

## What Feature It Adds

This adds the first real shelter instinct.

- cold agents begin preferring warmer local tiles
- enclosed room tiles become more attractive than equally warm outdoor tiles
- campfires and shelters can now act as combined structural attractors
- small clusters around warm interiors become possible without direct shelter lookup

## Verification After Implementation

Verification commands run:

- `cd rust && cargo test -p sim-systems steering -- --nocapture`
- `cd rust && cargo check --workspace`
- `cd rust && cargo test --workspace`
- `cd rust && cargo clippy --workspace -- -D warnings`

Results:

- `cargo test -p sim-systems steering -- --nocapture`: PASS
- `cargo check --workspace`: PASS
- `cargo test --workspace`: PASS
- `cargo clippy --workspace -- -D warnings`: PASS

Key behavioral evidence:

- cold-only shelter helper test confirms a neighboring room tile is preferred
- no-bias test confirms shelter preference does not trigger when Warmth need is already satisfied
- shelter cause test confirms room-driven movement is causally logged as `shelter_gradient`
- focused regression tests confirm Food attraction and Danger avoidance still behave

## Remaining Risks

- this slice only adds a local room-aware warmth bias, not a full shelter planning system
- movement still combines heuristic terms instead of a full steering framework
- direct action completion rewards such as explicit sit-by-fire or shelter actions still exist elsewhere in runtime
- room preference currently uses `room_id` presence as the indoor signal, not a richer enclosure/ventilation model

## In-Game Checks (한국어)

- 추운 환경에서 에이전트가 따뜻한 장소로 이동하는지 확인한다.
- 캠프파이어 주변에 에이전트가 모이는지 확인한다.
- 실내가 실외보다 따뜻하게 느껴져서 실내 쪽으로 더 끌리는지 확인한다.
- 벽이 warmth influence를 막아서 실내/실외 차이가 보이는지 확인한다.
- 음식 행동이 깨지지 않았는지 확인한다.
- 위험 회피 행동이 깨지지 않았는지 확인한다.
- 시뮬레이션이 정상적으로 계속 진행되는지 확인한다.
