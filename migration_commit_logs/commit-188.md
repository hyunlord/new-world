# Commit 188 - sim-bridge batch pathfinding grid 재사용

## 커밋 요약
- Rust bridge의 batch pathfinding이 요청마다 그리드를 재구성하던 경로를 개선해, 배치 전체에서 그리드를 1회만 구성 후 재사용하도록 최적화.

## 상세 변경
- `rust/crates/sim-bridge/src/lib.rs`
  - `build_grid_cost_map(...) -> Result<GridCostMap, PathfindError>` 헬퍼 추가:
    - walkable/move_cost 길이 검증 공통화
    - `GridCostMap` 생성/채우기 로직 공통화
  - `pathfind_grid_bytes(...)`
    - 기존 `PathfindInput` 생성 + `pathfind_from_flat` 경로 대신
      `build_grid_cost_map` + `find_path` 직접 호출로 단순화.
  - `pathfind_grid_batch_bytes(...)`
    - 루프마다 `pathfind_grid_bytes(...)`를 호출하던 구조 제거.
    - 배치 시작 시 `build_grid_cost_map` 1회 호출 후, 각 요청에 대해 `find_path(&grid, ...)`만 수행.

## 기능 영향
- path 결과/에러 조건은 동일.
- 배치 경로 탐색 시 동일 월드 그리드 재구성 비용을 제거해 CPU/할당 오버헤드를 크게 완화.

## 검증
- `cd rust && cargo test -p sim-bridge --release` 통과 (8 tests)
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 98 tests)
  - localization compile `filled=0`, `updated=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -p sim-test --release -- --bench-stress-math --iters 10000`
  - `ns_per_iter=385.5`, `checksum=24032652.00000`
- `cd rust && cargo run -p sim-test --release -- --bench-needs-math --iters 10000`
  - `ns_per_iter=142.8`, `checksum=38457848.00000`
