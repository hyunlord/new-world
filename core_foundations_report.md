# WS-REF-003 Core Foundations Report

## Implementation Intent
This ticket closes the largest remaining shared-core gaps between the v3.1 architecture docs and the actual Rust runtime foundations. The goal was not to implement full gameplay behavior, but to make the required core modules exist, compile, integrate with ECS/resources, and expose typed data hooks so later tickets can build real runtime logic on top of them.

## How It Was Implemented

### 1. Core scaffolds added in `sim-core`
- Added:
  - `effect.rs`
  - `causal_log.rs`
  - `tile_grid.rs`
  - `room.rs`
  - `temperament.rs`
- Reused the existing `InfluenceGrid` implementation instead of duplicating it.
- Re-exported the new shared types through `sim-core/src/lib.rs` and `sim-core/src/components/mod.rs`.

### 2. ECS integration
- Added shared ECS-facing components/types:
  - `InfluenceEmitter`
  - `InfluenceReceiver`
  - `Temperament`
  - `RoomId`
- Updated agent spawning so new entities receive:
  - `Temperament`
  - `InfluenceReceiver`

### 3. Engine/resource integration
- Extended `SimResources` with:
  - `tile_grid`
  - `rooms`
  - `causal_log`
- Initialized those resources alongside the pre-existing `influence_grid`.

### 4. Data hooks
- Extended `DataRegistry` with typed accessors for:
  - material wall blocking hints
  - furniture influence emissions
  - structure completion influence
  - action effects
  - world-rules schema access
  - temperament-rules schema access
- Added a legacy trait lookup hook on `DataBundle` to bridge remaining JSON-backed trait data.

### 5. Verification scaffolds
- Added unit tests for:
  - effect scaffolds
  - causal log ring buffering
  - tile-grid behavior
  - room detection
  - temperament derivation
- Extended spawn and engine tests to confirm ECS/resource integration.

## What Feature It Adds
WorldSim now has the missing shared foundation types that later architecture tickets depend on:
- a typed effect scaffold
- a typed causal log scaffold
- a structural tile grid
- room detection and room ids
- a temperament component derived from personality
- ECS/resource wiring for those foundations

This does not yet add full gameplay logic, but it removes the architectural gap where the design required these systems and the codebase simply did not contain them.

## Verification After Implementation

### Commands
```bash
cd /Users/rexxa/github/new-world-wt/codex-refactor-core-foundations/rust
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

### Result
- `cargo test --workspace`: PASS
- `cargo clippy --workspace -- -D warnings`: PASS

### Key evidence
- new `sim-core` module tests pass
- existing influence-grid tests remain green
- spawn path now carries `Temperament` + `InfluenceReceiver`
- engine resource initialization now includes tile/room/causal scaffolds
- typed `sim-data` hooks resolve against live crate-local RON data

## In-Game Checks (한국어)
- 이번 티켓은 핵심 foundation scaffold 작업이라, 인게임에서 바로 큰 변화가 보이는 단계는 아니다.
- 다만 다음을 확인할 수 있다.
  - 게임이 정상 부팅되는지
  - 기존 에이전트 스폰/이동/기본 루프가 깨지지 않았는지
  - 기존 Influence 관련 기능(화덕 온기 등)이 회귀 없이 동작하는지
- 이상하면 이런 증상이 보일 수 있다.
  - 부팅 직후 크래시: `SimResources` 초기화나 새 모듈 export 문제 가능성
  - 에이전트 스폰 실패: `Temperament` / `InfluenceReceiver` 삽입 경로 문제 가능성
  - 기존 온기/건설 관련 테스트는 통과했는데 인게임만 깨짐: bridge/runtime bootstrap 쪽 별도 문제 가능성
