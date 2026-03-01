# Commit 042 - stress post-update batch step Rust 이관

## 커밋 요약
- stress 틱 후반부의 `reserve + allostatic + state snapshot` 계산을 단일 Rust step으로 통합.
- GDScript에서 분리 호출하던 3개 브리지 호출을 1개 호출로 축소.

## 상세 변경
- `rust/crates/sim-systems/src/stat_curve.rs`
  - 신규 구조체:
    - `StressPostUpdateStep`
  - 신규 함수:
    - `stress_post_update_step(...) -> StressPostUpdateStep`
  - 내부 구성:
    - 기존 `stress_reserve_step` 호출
    - 기존 `stress_allostatic_step` 호출
    - 기존 `stress_state_snapshot` 호출
    - 결과를 단일 payload로 결합
  - 단위 테스트 1개 추가:
    - `stress_post_update_step_matches_component_steps`
    - 분리 호출 결과와 batch step 결과 동치 검증
- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 Godot 함수:
    - `stat_stress_post_update_step(...) -> Dictionary`
  - 반환 키:
    - `reserve`, `gas_stage`, `allostatic`
    - `stress_state`, `stress_mu_*`, `stress_neg_gain_mult`, `stress_pos_gain_mult`, `stress_blunt_mult`
- `scripts/core/simulation/sim_bridge.gd`
  - 신규 프록시:
    - `stat_stress_post_update_step(...)`
- `scripts/core/stats/stat_curve.gd`
  - 신규 함수:
    - `stress_post_update_step(...)`
  - Rust 우선 + fallback 경로 제공
    - fallback은 기존 `stress_reserve_step`, `stress_allostatic_step`, `stress_state_snapshot`를 조합
- `scripts/systems/psychology/stress_system.gd`
  - `_update_entity_stress`에서:
    - 기존 `_update_reserve`, `_update_allostatic`, `stress_state_snapshot` 분리 호출 제거
    - `StatCurveScript.stress_post_update_step(...)` 단일 호출로 치환
    - 반환값으로 `ed.reserve`, `ed.gas_stage`, `ed.allostatic` 동기화
  - 더 이상 사용되지 않는 `_update_reserve`, `_update_allostatic` 함수 제거
- `rust/crates/sim-test/src/main.rs`
  - `--bench-stress-math` 루프에 `stress_post_update_step` 호출을 추가해 신규 batch step도 측정 범위에 포함

## 기능 영향
- stress 틱 hot path에서 post-update 수학 처리의 FFI round-trip이 감소.
- reserve/GAS/allostatic/state snapshot 수식은 기존 함수 재사용으로 의미 동일성 유지.
- 벤치 지표가 batch step 포함 경로로 확장됨.

## 검증
- `cd rust && cargo fmt -p sim-systems -p sim-bridge -p sim-test` 통과
- `cd rust && cargo test -q -p sim-systems` 통과 (30 tests)
- `cd rust && cargo test -q -p sim-bridge` 통과 (8 tests)
- `cd rust && cargo test -q -p sim-test` 통과
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000` 통과
  - 예시 출력: `ns_per_iter=90.2`
- `tools/migration_verify.sh` 통과
  - strict audit: inline localized fields 0 유지
