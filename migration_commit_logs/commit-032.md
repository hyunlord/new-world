# Commit 032 - Stress appraisal 수식 Rust 이관

## 커밋 요약
- `StressSystem`의 Lazarus appraisal 스케일 수식을 Rust로 이관.
- GDScript는 입력값 수집(need/support/personality/emotion)만 담당하고, 최종 스케일 계산은 Rust 경로로 수행.

## 상세 변경
- `rust/crates/sim-systems/src/stat_curve.rs`
  - 신규 함수:
    - `stress_appraisal_scale(...) -> f32`
  - 기존 appraisal 수식(위협/자원 평가, imbalance, clamp 0.7~1.9) 1:1 구현
  - 단위 테스트 2개 추가:
    - clamp 범위 검증
    - 악화된 맥락에서 스케일 증가 검증
- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 Godot 함수:
    - `stat_stress_appraisal_scale(...) -> f32`
- `scripts/core/simulation/sim_bridge.gd`
  - 신규 프록시:
    - `stat_stress_appraisal_scale(...)`
- `scripts/core/stats/stat_curve.gd`
  - 신규 함수:
    - `stress_appraisal_scale(...)`
  - Rust 우선 호출 + 기존 GDScript 수식 fallback 제공
- `scripts/systems/psychology/stress_system.gd`
  - `_calc_appraisal_scale(...)` 내부 수식을 `StatCurveScript.stress_appraisal_scale(...)` 호출로 교체

## 기능 영향
- appraisal 계산 hot path가 네이티브 경로로 이동해 CPU 부담을 낮춤.
- 기존 결과 범위/의미(0.7~1.9 clamp)는 유지.

## 검증
- `cd rust && cargo fmt -p sim-systems -p sim-bridge` 통과
- `cd rust && cargo test -q -p sim-systems` 통과 (13 tests)
- `cd rust && cargo test -q -p sim-bridge` 통과 (8 tests)
- `tools/migration_verify.sh` 통과
  - strict audit: inline localized fields 0 유지
