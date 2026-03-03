# Commit 036 - Allostatic load 업데이트 Rust 이관

## 커밋 요약
- `StressSystem._update_allostatic` 수식을 Rust로 이관.
- avoidant attachment 배율은 GDScript에서 계산 후 Rust step 함수에 입력.

## 상세 변경
- `rust/crates/sim-systems/src/stat_curve.rs`
  - 신규 함수:
    - `stress_allostatic_step(allostatic, stress, avoidant_allostatic_mult) -> f32`
  - 기존 규칙 반영:
    - 고스트레스 구간 누적 증가(상한 0.05/tick)
    - 저스트레스 구간 자연 회복
  - 단위 테스트 2개 추가:
    - 고스트레스 증가/avoidant 배율 영향
    - 저스트레스 회복
- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 Godot 함수:
    - `stat_stress_allostatic_step(...) -> f32`
- `scripts/core/simulation/sim_bridge.gd`
  - 신규 프록시:
    - `stat_stress_allostatic_step(...)`
- `scripts/core/stats/stat_curve.gd`
  - 신규 함수:
    - `stress_allostatic_step(...)`
  - Rust 우선 + 기존 GDScript fallback 수식 제공
- `scripts/systems/psychology/stress_system.gd`
  - `_update_allostatic(...)`를 `StatCurveScript.stress_allostatic_step(...)` 호출 기반으로 교체
  - 사용 종료된 ALLO 상수 제거

## 기능 영향
- stress 파이프라인의 allostatic 업데이트가 네이티브 경로로 이동.
- attachment avoidant 증폭 규칙과 기존 clamp 동작은 유지.

## 검증
- `cd rust && cargo fmt -p sim-systems -p sim-bridge` 통과
- `cd rust && cargo test -q -p sim-systems` 통과 (21 tests)
- `cd rust && cargo test -q -p sim-bridge` 통과 (8 tests)
- `tools/migration_verify.sh` 통과
  - strict audit: inline localized fields 0 유지
