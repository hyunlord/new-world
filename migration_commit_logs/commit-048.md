# Commit 048 - stress work efficiency Rust 이관

## 커밋 요약
- `StressSystem.get_work_efficiency`의 Yerkes-Dodson 수식을 Rust로 이관.
- stress tick 단일화 이후 남아 있던 stress 관련 GDScript 수식 경로를 추가 축소.

## 상세 변경
- `rust/crates/sim-systems/src/stat_curve.rs`
  - 신규 함수:
    - `stress_work_efficiency(stress, shaken_penalty) -> f32`
  - 수식:
    - 기존 piecewise(150/350 구간) + shaken penalty + clamp(0.35~1.10) 유지
  - 단위 테스트 2개 추가:
    - 구간별 출력값 검증
    - penalty/클램프 동작 검증
- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 Godot 함수:
    - `stat_stress_work_efficiency(stress, shaken_penalty) -> f32`
- `scripts/core/simulation/sim_bridge.gd`
  - 신규 프록시:
    - `stat_stress_work_efficiency(...)`
- `scripts/core/stats/stat_curve.gd`
  - 신규 함수:
    - `stress_work_efficiency(...)`
  - Rust 우선 + 기존 GDScript fallback 수식 유지
- `scripts/systems/psychology/stress_system.gd`
  - `get_work_efficiency(ed)`가 `StatCurveScript.stress_work_efficiency(...)` 호출로 전환
  - 더 이상 사용되지 않는 `EUSTRESS_OPTIMAL` 상수 제거
- `rust/crates/sim-test/src/main.rs`
  - stress 벤치 루프에 `stress_work_efficiency` 호출 추가

## 기능 영향
- work efficiency 계산 경로가 네이티브 수학으로 이동.
- 기존 값 구간/penalty/clamp 의미는 그대로 유지.
- stress 관련 GDScript 순수 수식 경로를 추가로 축소.

## 검증
- `cd rust && cargo fmt -p sim-systems -p sim-bridge -p sim-test` 통과
- `cd rust && cargo test -q -p sim-systems` 통과 (37 tests)
- `cd rust && cargo test -q -p sim-bridge` 통과 (8 tests)
- `cd rust && cargo test -q -p sim-test` 통과
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000` 통과
  - 예시 출력: `ns_per_iter=246.0`
- `tools/migration_verify.sh` 통과
  - strict audit: inline localized fields 0 유지
