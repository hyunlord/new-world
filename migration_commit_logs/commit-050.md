# Commit 050 - stress event scaling 수식 Rust 공통화

## 커밋 요약
- 이벤트 기반 stress 주입 경로(`inject_stress_event`, `inject_event`)의 최종 스케일/손실배수 계산을 Rust helper로 공통화.
- 이벤트 주입 수식 중복을 제거하고 두 경로의 계산 일관성을 강화.

## 상세 변경
- `rust/crates/sim-systems/src/stat_curve.rs`
  - 신규 구조체:
    - `StressEventScaled`
  - 신규 함수:
    - `stress_event_scaled(base_instant, base_per_tick, is_loss, personality_scale, relationship_scale, context_scale, appraisal_scale)`
  - 반환:
    - `total_scale`, `loss_mult`, `final_instant`, `final_per_tick`
  - 단위 테스트 2개 추가:
    - 전체 배수 곱 적용 검증
    - 손실 혐오(2.5배) 적용 검증
- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 Godot 함수:
    - `stat_stress_event_scaled(...) -> Dictionary`
- `scripts/core/simulation/sim_bridge.gd`
  - 신규 프록시:
    - `stat_stress_event_scaled(...)`
- `scripts/core/stats/stat_curve.gd`
  - 신규 함수:
    - `stress_event_scaled(...)`
  - Rust 우선 + fallback 제공
- `scripts/systems/psychology/stress_system.gd`
  - `inject_stress_event(...)`:
    - 직접 `loss_mult` 계산 대신 `StatCurveScript.stress_event_scaled(...)` 결과 사용
  - `inject_event(...)`:
    - final scale/instant/per_tick 계산을 Rust helper 결과로 대체
    - `total_scale`은 감정 주입(`_inject_emotions`)에서 그대로 사용
- `rust/crates/sim-test/src/main.rs`
  - stress 벤치에 `stress_event_scaled` 호출 추가

## 기능 영향
- 이벤트 기반 stress 주입 경로의 최종 수식이 네이티브 경로로 통일.
- loss multiplier/total scale/최종 instant·per_tick 의미는 기존과 동일하게 유지.
- 수식 중복 감소로 유지보수 안정성 향상.

## 검증
- `cd rust && cargo fmt -p sim-systems -p sim-bridge -p sim-test` 통과
- `cd rust && cargo test -q -p sim-systems` 통과 (39 tests)
- `cd rust && cargo test -q -p sim-bridge` 통과 (8 tests)
- `cd rust && cargo test -q -p sim-test` 통과
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000` 통과
  - 예시 출력: `ns_per_iter=243.0`
- `tools/migration_verify.sh` 통과
  - strict audit: inline localized fields 0 유지
