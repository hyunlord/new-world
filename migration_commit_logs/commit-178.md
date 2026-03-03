# Commit 178 - movement cached_path clear 재사용 경로

## 커밋 요약
- movement 시스템에서 빈 경로 처리 시 `cached_path = []` 반복 할당을 제거하고 clear 재사용 경로를 도입.

## 상세 변경
- `scripts/systems/world/movement_system.gd`
  - `_clear_cached_path(entity)` 헬퍼 추가:
    - 기존 `cached_path`가 Array면 `clear()`로 재사용
    - 비정상 타입일 때만 `[]` 재할당
    - `path_index`를 0으로 동기화
  - action 완료 시 경로 초기화를 `_clear_cached_path`로 통일.
  - batch recalc 결과 부족 구간에서 `[]` 전달 대신 `_clear_cached_path` 직접 호출.
  - `_move_with_pathfinding` 재계산 분기에서 중간 할당(`entity.cached_path = ...`)을 제거하고 지역 변수로 받아 `_apply_recalculated_path`에 전달.
  - path blocked fallback에서도 `_clear_cached_path`를 사용.
  - `_apply_recalculated_path`가 empty path 입력 시 `_clear_cached_path`로 위임하도록 보강.

## 기능 영향
- 이동/경로 재계산 의미는 동일.
- 빈 경로 처리 구간의 임시 Array 재할당을 줄여 movement hot path 메모리 churn 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 98 tests)
  - localization compile `filled=0`, `updated=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -p sim-test --release -- --bench-stress-math --iters 10000`
  - `ns_per_iter=387.6`, `checksum=24032652.00000`
- `cd rust && cargo run -p sim-test --release -- --bench-needs-math --iters 10000`
  - `ns_per_iter=140.8`, `checksum=38457848.00000`
