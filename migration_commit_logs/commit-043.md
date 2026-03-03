# Commit 043 - stress primary step(appraisal+continuous) Rust 통합

## 커밋 요약
- stress 틱 초반의 appraisal 계산과 continuous need-deficit 계산을 단일 Rust step으로 통합.
- `StressSystem`의 초기 stress 입력 계산 브리지 호출 수를 추가로 축소.

## 상세 변경
- `rust/crates/sim-systems/src/stat_curve.rs`
  - 신규 구조체:
    - `StressPrimaryStep`
  - 신규 함수:
    - `stress_primary_step(...) -> StressPrimaryStep`
  - 내부 구성:
    - `stress_appraisal_scale` 계산
    - appraisal 결과를 입력으로 `stress_continuous_inputs` 계산
    - 결과를 단일 payload(`appraisal_scale`, `hunger`, `energy_deficit`, `social_isolation`, `total`)로 반환
  - 단위 테스트 1개 추가:
    - `stress_primary_step_matches_appraisal_plus_continuous`
    - 기존 분리 계산과 결과 동치 검증
- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 Godot 함수:
    - `stat_stress_primary_step(...) -> Dictionary`
- `scripts/core/simulation/sim_bridge.gd`
  - 신규 프록시:
    - `stat_stress_primary_step(...)`
- `scripts/core/stats/stat_curve.gd`
  - 신규 함수:
    - `stress_primary_step(...)`
  - Rust 우선 + fallback 경로 제공
    - fallback은 기존 `stress_appraisal_scale` + `stress_continuous_inputs` 조합
- `scripts/systems/psychology/stress_system.gd`
  - `_update_entity_stress`에서:
    - `_calc_appraisal_scale` + `_calc_continuous_stressors` 분리 호출 제거
    - `_calc_primary_inputs`(결합 step) 호출로 대체
  - 기존 `_calc_appraisal_scale`, `_calc_continuous_stressors` 함수 제거
  - breakdown 키(`hunger`, `energy_deficit`, `social_isolation`) 유지
- `rust/crates/sim-test/src/main.rs`
  - stress 벤치에서 분리 호출 대신 `stress_primary_step`을 사용하도록 갱신

## 기능 영향
- stress 틱 초반 math path가 단일 Rust 호출로 정리되어 FFI round-trip 감소.
- appraisal 및 continuous 입력 수식은 기존 함수 재사용으로 의미 동일성 유지.
- breakdown 동작/키 체계는 기존과 동일.

## 검증
- `cd rust && cargo fmt -p sim-systems -p sim-bridge -p sim-test` 통과
- `cd rust && cargo test -q -p sim-systems` 통과 (31 tests)
- `cd rust && cargo test -q -p sim-bridge` 통과 (8 tests)
- `cd rust && cargo test -q -p sim-test` 통과
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000` 통과
  - 예시 출력: `ns_per_iter=93.2`
- `tools/migration_verify.sh` 통과
  - strict audit: inline localized fields 0 유지
