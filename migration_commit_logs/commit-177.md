# Commit 177 - movement periodic recalc tick 판정 재사용

## 커밋 요약
- movement 루프에서 엔티티마다 반복 계산하던 `tick % 50` 판정을 tick 단위 1회 계산으로 재사용해 경로 재계산 판정 비용을 줄임.

## 상세 변경
- `scripts/systems/world/movement_system.gd`
  - `execute_tick(...)`에 `periodic_recalc_tick` 로컬 추가:
    - `var periodic_recalc_tick: bool = (tick % 50) == 0`
  - `_needs_path_recalc` 호출부를 tick 값 대신 precomputed bool 전달로 변경.
  - `_needs_path_recalc` 시그니처 변경:
    - `tick: int` → `periodic_recalc_tick: bool`
    - 내부 `% 50` 연산 제거, 전달된 bool로 판정.
  - `_move_with_pathfinding`의 `allow_recalc` 경로에서도 동일 시그니처에 맞춰 bool 전달.

## 기능 영향
- path recalc 조건 의미(빈 경로/경로 소진/50 tick 주기)는 동일.
- 다수 엔티티 이동 tick에서 반복 modulo 연산을 줄여 movement hot path를 미세 최적화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 98 tests)
  - localization compile `filled=0`, `updated=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -p sim-test --release -- --bench-stress-math --iters 10000`
  - `ns_per_iter=383.8`, `checksum=24032652.00000`
- `cd rust && cargo run -p sim-test --release -- --bench-needs-math --iters 10000`
  - `ns_per_iter=146.8`, `checksum=38457848.00000`
