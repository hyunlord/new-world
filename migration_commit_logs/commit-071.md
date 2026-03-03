# Commit 071 - tick당 debug 플래그 단일 조회

## 커밋 요약
- stress tick에서 `DEBUG_STRESS_LOG`를 엔티티마다 조회하지 않고 tick당 1회 조회하도록 최적화.

## 상세 변경
- `scripts/systems/psychology/stress_system.gd`
  - `execute_tick(...)`
    - `collect_breakdown` 값을 루프 진입 전에 1회 계산.
    - `_update_entity_stress(...)` 호출 시 해당 값을 인자로 전달.
  - `_update_entity_stress(...)`
    - 시그니처를 `collect_breakdown` 인자 기반으로 변경.
    - 함수 내부 `GameConfig.DEBUG_STRESS_LOG` 직접 조회 제거.

## 기능 영향
- tick 내부의 debug on/off 동작 의미는 동일.
- 엔티티 수가 많을수록 반복 `GameConfig` 조회 비용을 줄여 tick 핫패스 미세 최적화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=480.2`, `checksum=13761358.00000`
