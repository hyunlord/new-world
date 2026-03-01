# Commit 034 - Stress recovery(decay) 계산 Rust 이관

## 커밋 요약
- `StressSystem._calc_recovery`의 decay 계산 수식을 Rust로 이관.
- GDScript는 support/resilience 등 입력 수집 후 단일 호출만 수행.

## 상세 변경
- `rust/crates/sim-systems/src/stat_curve.rs`
  - 신규 함수:
    - `stress_recovery_value(stress, support_score, resilience, reserve, is_sleeping, is_safe)`
  - 기존 수식 요소 반영:
    - base + stress 비례
    - safe/sleep 보너스
    - support 배율
    - resilience 배율
    - low reserve 페널티
  - 단위 테스트 2개 추가:
    - `sleep+safe`일 때 recovery 증가
    - `low reserve`일 때 recovery 감소
- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 Godot 함수:
    - `stat_stress_recovery_value(...) -> f32`
- `scripts/core/simulation/sim_bridge.gd`
  - 신규 프록시:
    - `stat_stress_recovery_value(...)`
- `scripts/core/stats/stat_curve.gd`
  - 신규 함수:
    - `stress_recovery_value(...)`
  - Rust 우선 + 기존 GDScript 수식 fallback 제공
- `scripts/systems/psychology/stress_system.gd`
  - `_calc_recovery(...)`에서 수식 직접 계산 제거
  - `StatCurveScript.stress_recovery_value(...)` 호출로 전환
  - 사용 종료된 recovery 상수 제거

## 기능 영향
- stress 회복 수학 경로가 네이티브화되어 tick당 연산 부담 감소.
- breakdown(`recovery`) 및 결과 의미는 기존과 동일하게 유지.

## 검증
- `cd rust && cargo fmt -p sim-systems -p sim-bridge` 통과
- `cd rust && cargo test -q -p sim-systems` 통과 (17 tests)
- `cd rust && cargo test -q -p sim-bridge` 통과 (8 tests)
- `tools/migration_verify.sh` 통과
  - strict audit: inline localized fields 0 유지
