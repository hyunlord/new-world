# Commit 183 - movement recalc/path apply size 체크 단순화

## 커밋 요약
- movement 경로 재계산 판정/적용 함수의 중복 size 조회를 줄여 hot path 분기 비용을 미세 최적화.

## 상세 변경
- `scripts/systems/world/movement_system.gd`
  - `_needs_path_recalc(...)`
    - `entity.cached_path.size()`를 지역 변수 `cached_size`로 1회 조회 후 재사용.
    - 기존 `is_empty() + size()` 이중 조회를 단일 size 기반 판정으로 정리.
  - `_apply_recalculated_path(...)`
    - `path.size()`를 지역 변수 `path_size`로 1회 조회 후 재사용.
    - non-empty 보장 이후 `entity.cached_path[0]` 비교에서 추가 size 분기 제거.

## 기능 영향
- 경로 재계산 조건/적용 의미는 동일.
- entity당 반복되는 크기 조회와 불필요 분기 감소로 movement hot path를 미세 최적화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 98 tests)
  - localization compile `filled=0`, `updated=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -p sim-test --release -- --bench-stress-math --iters 10000`
  - `ns_per_iter=399.8`, `checksum=24032652.00000`
- `cd rust && cargo run -p sim-test --release -- --bench-needs-math --iters 10000`
  - `ns_per_iter=150.7`, `checksum=38457848.00000`
