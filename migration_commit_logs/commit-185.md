# Commit 185 - sim_bridge resolved backend cache hit 시 sync 스킵

## 커밋 요약
- pathfinding backend resolved cache가 유효할 때 `_sync_pathfinding_backend_mode` 호출을 생략해 `_prefer_gpu` hot path 분기 비용을 줄임.

## 상세 변경
- `scripts/core/simulation/sim_bridge.gd`
  - `_resolve_pathfinding_backend_cached(bridge)` 순서 조정:
    - 기존: 항상 `_sync_pathfinding_backend_mode` 호출 후 cache 확인
    - 변경: cache hit면 즉시 반환, miss일 때만 sync 수행
  - sync 직후 cache가 채워졌을 가능성을 고려해 2차 cache 확인 추가.

## 기능 영향
- backend 해석 결과(`cpu`/`gpu`) 의미는 동일.
- cache hit 구간에서 불필요한 `ComputeBackend.get_mode`/sync 분기를 줄여 pathfinding 호출당 오버헤드 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 98 tests)
  - localization compile `filled=0`, `updated=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -p sim-test --release -- --bench-stress-math --iters 10000`
  - `ns_per_iter=381.6`, `checksum=24032652.00000`
- `cd rust && cargo run -p sim-test --release -- --bench-needs-math --iters 10000`
  - `ns_per_iter=143.4`, `checksum=38457848.00000`
