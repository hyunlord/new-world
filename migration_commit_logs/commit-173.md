# Commit 173 - movement batch path 재계산 1회 처리로 정렬

## 커밋 요약
- movement 시스템이 엔티티 스캔 도중 반복 실행하던 배치 경로 재계산을 스캔 이후 1회 처리로 정렬해 경로 탐색 호출 중복을 제거.

## 상세 변경
- `scripts/systems/world/movement_system.gd`
  - `execute_tick(...)`에서 A* 경로 수집 루프를 정리:
    - `_pathfinder != null` 분기 들여쓰기 오류를 수정해 정상 조건 분기로 고정.
    - `recalc_entities`/`_recalc_from_xy`/`_recalc_to_xy` 수집은 엔티티 루프 내에서만 수행.
  - 배치 경로 계산 호출을 루프 밖으로 이동:
    - `_pathfinder.find_paths_batch_xy(...)`를 tick당 최대 1회만 호출.
    - 결과 적용 시 `recalc_count`를 사용해 경로 존재/미존재 케이스를 명시적으로 처리.

## 기능 영향
- 이동 및 경로 갱신 결과 의미는 유지.
- recalc 대상이 있는 tick에서 pathfinding batch 호출 횟수를 줄여 movement hot path CPU 사용량을 완화.
- `count` 스코프 의존 문제를 제거해 재계산 결과 적용 경로를 안정화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 98 tests)
  - localization compile `filled=0`, `updated=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -p sim-test --release -- --bench-stress-math --iters 10000`
  - `ns_per_iter=387.1`, `checksum=24032652.00000`
- `cd rust && cargo run -p sim-test --release -- --bench-needs-math --iters 10000`
  - `ns_per_iter=154.6`, `checksum=38457848.00000`
