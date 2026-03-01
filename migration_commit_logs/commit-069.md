# Commit 069 - debug off 경로의 stress breakdown 계산 생략

## 커밋 요약
- `DEBUG_STRESS_LOG` 비활성 시 tick 루프의 stress breakdown 조립을 건너뛰도록 최적화.

## 상세 변경
- `scripts/systems/psychology/stress_system.gd`
  - `_update_entity_stress(...)`
    - `collect_breakdown` 플래그(`GameConfig.DEBUG_STRESS_LOG`)를 도입.
    - 연속 stressor/hunger/energy/social breakdown 기록을 debug 케이스에서만 수행.
    - trace breakdown 키 생성(`trace_<source_id>`) 및 기록을 debug 케이스에서만 수행.
    - 감정 기여(`emo_*`), `va_composite`, `recovery` breakdown 기록을 debug 케이스에서만 수행.
    - debug 비활성 시 `ed.stress_breakdown`은 비어 있지 않을 때만 1회 clear하도록 조정.

## 기능 영향
- `DEBUG_STRESS_LOG=true` 동작은 기존과 동일하게 상세 breakdown 로그를 유지.
- `DEBUG_STRESS_LOG=false`에서는 시뮬레이션 수치 결과는 동일하고, breakdown 구성/문자열 키 처리 오버헤드를 제거.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=554.6`, `checksum=13761358.00000`
