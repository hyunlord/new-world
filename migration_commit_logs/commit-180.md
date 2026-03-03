# Commit 180 - path batch 그룹 정규화 배열 선할당

## 커밋 요약
- pathfinder batch 결과 그룹 정규화에서 `append` 대신 선할당 배열에 인덱스 기록을 사용해 포장 단계 오버헤드를 줄임.

## 상세 변경
- `scripts/core/world/pathfinder.gd`
  - `_find_paths_rust_batch(...)`
    - `used_batch_xy` 경로의 `xy_normalized`를 `resize(xy_groups.size())` 후 인덱스 기록으로 전환.
    - vec2 fallback 경로의 `normalized`도 `resize(groups.size())` 후 인덱스 기록으로 전환.
  - `_find_paths_rust_batch_xy(...)`
    - 동일하게 `xy_normalized`/`normalized`를 선할당 + 인덱스 기록으로 전환.

## 기능 영향
- batch path 결과 순서/값은 동일.
- 결과 포장 단계의 동적 확장(`append`)을 줄여 pathfinding batch hot path의 메모리 churn을 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 98 tests)
  - localization compile `filled=0`, `updated=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -p sim-test --release -- --bench-stress-math --iters 10000`
  - `ns_per_iter=393.2`, `checksum=24032652.00000`
- `cd rust && cargo run -p sim-test --release -- --bench-needs-math --iters 10000`
  - `ns_per_iter=143.9`, `checksum=38457848.00000`
