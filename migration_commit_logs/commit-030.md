# Commit 030 - Stress 연속 입력 수학 Rust 이관(Needs deficit path)

## 커밋 요약
- `StressSystem`의 연속 스트레스 입력(배고픔/에너지 결핍/사회적 고립) 수학식을 Rust로 이관.
- GDScript는 `StatCurveScript`를 통해 단일 브리지 호출로 값을 받아 breakdown 구성만 수행.

## 상세 변경
- `rust/crates/sim-systems/src/stat_curve.rs`
  - 신규 타입:
    - `ContinuousStressInputs { hunger, energy_deficit, social_isolation, total }`
  - 신규 함수:
    - `stress_continuous_inputs(hunger, energy, social, appraisal_scale)`
  - 내부 헬퍼:
    - `need_deficit(value, threshold)`
  - 단위 테스트 2개 추가:
    - high-need(충족 상태)에서 0 반환
    - 결핍 심화 시 출력 증가 검증
- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 Godot 함수:
    - `stat_stress_continuous_inputs(...) -> VarDictionary`
  - 반환 키:
    - `hunger`, `energy_deficit`, `social_isolation`, `total`
- `scripts/core/simulation/sim_bridge.gd`
  - 신규 프록시:
    - `stat_stress_continuous_inputs(...)`
- `scripts/core/stats/stat_curve.gd`
  - 신규 함수:
    - `stress_continuous_inputs(...)`
  - Rust 우선 호출 + 기존 GDScript 수식 fallback 제공
- `scripts/systems/psychology/stress_system.gd`
  - `_calc_continuous_stressors(...)`를 `StatCurveScript.stress_continuous_inputs(...)` 기반으로 전환
  - 기존 breakdown 키(`hunger`, `energy_deficit`, `social_isolation`)는 유지

## 기능 영향
- 스트레스 시스템 틱당 hot path 일부(연속 스트레스 계산)가 Rust 경로로 이전.
- 수식/게임플레이 결과는 기존과 동일하게 유지.

## 검증
- `cd rust && cargo fmt -p sim-systems -p sim-bridge` 통과
- `cd rust && cargo test -q -p sim-systems` 통과 (11 tests)
- `cd rust && cargo test -q -p sim-bridge` 통과 (8 tests)
- `tools/migration_verify.sh` 통과
  - strict audit: inline localized fields 0 유지
