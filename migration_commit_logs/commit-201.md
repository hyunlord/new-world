# Commit 201 - GridCostMap flat 버퍼 직생성 경로 도입

## 커밋 요약
- `GridCostMap`에 flat 버퍼 직생성 API를 추가하고 `sim-bridge`가 이를 사용하도록 전환해 grid 구성 시 per-cell setter 호출 오버헤드를 제거.

## 상세 변경
- `rust/crates/sim-systems/src/pathfinding.rs`
  - `GridCostMap::from_flat_unchecked(width, height, walkable, move_cost)` 추가.
  - `GridCostMap::from_flat_bytes_unchecked(width, height, walkable, move_cost)` 추가.
  - 두 경로 모두 move_cost를 기존 setter 의미와 동일하게 `max(0.0)`으로 clamp.
  - 신규 테스트 `builds_grid_from_flat_bytes_with_clamped_costs` 추가.
- `rust/crates/sim-bridge/src/lib.rs`
  - `pathfind_from_flat`이 새 `GridCostMap::from_flat_unchecked` 경로 사용.
  - `build_grid_cost_map_unchecked`가 새 `GridCostMap::from_flat_bytes_unchecked` 경로 사용.

## 기능 영향
- path 의미/검증 의미는 유지.
- bridge pathfinding에서 grid 생성 시 반복 `set_walkable/set_move_cost` + 인덱스 체크 호출을 제거해 batch/단건 공통 경로의 초기화 오버헤드 완화.

## 검증
- `cd rust && cargo test -p sim-systems --release` 통과 (99 tests).
- `cd rust && cargo test -p sim-bridge --release` 통과 (15 tests).
- `tools/migration_verify.sh --with-benches` 통과.
  - `pathfind-bridge checksum=70800.00000`(@100)
  - `stress checksum=24032652.00000`(@10k)
  - `needs checksum=38457848.00000`(@10k)
