# Commit 210 - pathfind_from_flat 소유 버퍼 직소비 경로 추가

## 커밋 요약
- `PathfindInput`이 가진 소유 버퍼를 그대로 소비하는 `GridCostMap` 생성 경로를 추가해 bridge flat 입력 경로의 불필요한 복사를 제거.

## 상세 변경
- `rust/crates/sim-systems/src/pathfinding.rs`
  - `GridCostMap::from_flat_owned_unchecked(width, height, walkable, move_cost)` 추가.
    - `move_cost`는 in-place clamp(`max(0.0)`) 처리.
    - `walkable`/`move_cost`를 추가 복사 없이 구조체에 소유 이전.
  - 테스트 `builds_grid_from_owned_flat_vectors_with_clamped_costs` 추가.
- `rust/crates/sim-bridge/src/lib.rs`
  - `pathfind_from_flat`에서 `PathfindInput`을 구조 분해하여
    `GridCostMap::from_flat_owned_unchecked(...)` 호출로 전환.

## 기능 영향
- path 결과/체크섬 의미 유지.
- bridge flat 경로에서 `Vec<bool>/Vec<f32>` 재복사를 제거해 메모리 오버헤드 완화.

## 검증
- `cd rust && cargo test -p sim-systems --release` 통과 (101 tests).
- `cd rust && cargo test -p sim-bridge --release` 통과 (15 tests).
- `tools/migration_verify.sh --with-benches` 통과.
  - `pathfind-bridge checksum=70800.00000`(@100)
  - `stress checksum=24032652.00000`(@10k)
  - `needs checksum=38457848.00000`(@10k)
