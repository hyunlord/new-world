# Commit 181 - path fallback batch 결과 배열 선할당

## 커밋 요약
- pathfinder의 GDScript fallback batch 경로에서도 결과 배열을 선할당해 `append` 확장 오버헤드를 줄임.

## 상세 변경
- `scripts/core/world/pathfinder.gd`
  - `find_paths_batch(...)`
    - fallback `out` 배열을 `requests.size()`로 `resize` 후 인덱스 기록으로 채움.
  - `find_paths_batch_xy(...)`
    - fallback `out` 배열을 `pair_count`로 `resize` 후 인덱스 기록으로 채움.

## 기능 영향
- fallback batch path 결과 순서/값은 동일.
- Rust 경로 미사용 시에도 fallback 결과 포장 단계의 배열 확장 비용을 미세 최적화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 98 tests)
  - localization compile `filled=0`, `updated=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -p sim-test --release -- --bench-stress-math --iters 10000`
  - `ns_per_iter=395.8`, `checksum=24032652.00000`
- `cd rust && cargo run -p sim-test --release -- --bench-needs-math --iters 10000`
  - `ns_per_iter=143.4`, `checksum=38457848.00000`
