# Commit 052 - stress relationship/context scale 수식 Rust 이관

## 커밋 요약
- 이벤트 기반 stress 주입 경로의 관계 스케일/상황 스케일 계산을 Rust helper로 이관.
- `inject_event`의 scale 계산이 personality + relationship + context 모두 네이티브 수식 경로를 사용하도록 정리.

## 상세 변경
- `rust/crates/sim-systems/src/stat_curve.rs`
  - 신규 함수:
    - `stress_relationship_scale(method, bond_strength, min_mult, max_mult) -> f32`
    - `stress_context_scale(active_multipliers) -> f32`
  - 단위 테스트 2개 추가:
    - 관계 스케일(bond 공식/none/unknown) 검증
    - 컨텍스트 스케일 곱셈 + clamp(0.1..5.0) 검증
- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 Godot 함수:
    - `stat_stress_relationship_scale(...) -> f32`
    - `stat_stress_context_scale(...) -> f32`
- `scripts/core/simulation/sim_bridge.gd`
  - 신규 프록시:
    - `stat_stress_relationship_scale(...)`
    - `stat_stress_context_scale(...)`
- `scripts/core/stats/stat_curve.gd`
  - 신규 함수:
    - `stress_relationship_scale(...)`
    - `stress_context_scale(...)`
  - Rust 우선 + 기존 GDScript fallback 제공
- `scripts/systems/psychology/stress_system.gd`
  - `_calc_relationship_scale(...)`를 `StatCurveScript.stress_relationship_scale(...)` 호출 기반으로 변경
  - `_calc_context_scale(...)`를 활성 multiplier 배열 구성 + `StatCurveScript.stress_context_scale(...)` 호출 기반으로 변경
- `rust/crates/sim-test/src/main.rs`
  - stress 벤치에 `stress_relationship_scale`/`stress_context_scale` 호출 추가

## 기능 영향
- 이벤트 stress 스케일링 경로에서 관계/상황 배수 계산이 Rust 수식과 일치하도록 통일.
- 기존 method 처리(`none`, `bond_strength`, unknown)와 clamp 범위 의미를 유지.

## 검증
- `cd rust && cargo fmt -p sim-systems -p sim-bridge -p sim-test` 통과
- `cd rust && cargo test -q -p sim-systems` 통과 (43 tests)
- `cd rust && cargo test -q -p sim-bridge` 통과 (8 tests)
- `cd rust && cargo test -q -p sim-test` 통과
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000` 통과
  - 예시 출력: `ns_per_iter=265.2`
- `tools/migration_verify.sh` 통과
  - strict audit: inline localized fields 0 유지
