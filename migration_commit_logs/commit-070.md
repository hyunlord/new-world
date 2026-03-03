# Commit 070 - debug off 경로의 빈 할당/호출 제거

## 커밋 요약
- `DEBUG_STRESS_LOG` 비활성 tick에서 불필요한 빈 `Dictionary` 생성과 `_debug_log` 함수 호출을 제거.

## 상세 변경
- `scripts/systems/psychology/stress_system.gd`
  - `_update_entity_stress(...)`
    - `breakdown` 초기화를 `collect_breakdown` 활성 시점으로 지연해 debug off 틱의 빈 dictionary 할당을 제거.
    - `_debug_log(entity, ed, delta)` 호출을 `collect_breakdown` 조건 내부로 이동해 debug off 틱에서 함수 호출 자체를 생략.

## 기능 영향
- debug on/off의 스트레스 수치 계산 결과는 기존과 동일.
- debug off 환경에서 tick 핫패스의 미세 할당/호출 오버헤드가 감소.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=510.4`, `checksum=13761358.00000`
