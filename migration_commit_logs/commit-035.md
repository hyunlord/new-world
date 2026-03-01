# Commit 035 - Reserve/GAS 단계 갱신 Rust step 함수 이관

## 커밋 요약
- `StressSystem._update_reserve`의 reserve 갱신 + GAS 단계 전이 로직을 Rust step 함수로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/stat_curve.rs`
  - 신규 타입:
    - `StressReserveStep { reserve, gas_stage }`
  - 신규 함수:
    - `stress_reserve_step(reserve, stress, resilience, stress_delta_last, gas_stage, is_sleeping)`
  - 기존 로직 반영:
    - drain/recover 계산
    - reserve clamp(0~100)
    - GAS 단계 전이(Alarm/Resistance/Exhaustion)
  - 단위 테스트 2개 추가:
    - 높은 스트레스 입력에서 단계 전이 검증
    - 저 reserve에서 exhaustion 전이 검증
- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 Godot 함수:
    - `stat_stress_reserve_step(...) -> VarDictionary`
- `scripts/core/simulation/sim_bridge.gd`
  - 신규 프록시:
    - `stat_stress_reserve_step(...)`
- `scripts/core/stats/stat_curve.gd`
  - 신규 함수:
    - `stress_reserve_step(...)`
  - Rust 우선 + 기존 GDScript step 수식 fallback 제공
- `scripts/systems/psychology/stress_system.gd`
  - `_update_reserve(...)`를 `StatCurveScript.stress_reserve_step(...)` 호출 기반으로 교체
  - 사용 종료된 `RESERVE_MAX` 상수 제거

## 기능 영향
- stress 파이프라인의 reserve/GAS 단계 갱신까지 네이티브 수학 경로로 이동.
- 기존 단계 의미(0~3)와 전이 규칙은 유지.

## 검증
- `cd rust && cargo fmt -p sim-systems -p sim-bridge` 통과
- `cd rust && cargo test -q -p sim-systems` 통과 (19 tests)
- `cd rust && cargo test -q -p sim-bridge` 통과 (8 tests)
- `tools/migration_verify.sh` 통과
  - strict audit: inline localized fields 0 유지
