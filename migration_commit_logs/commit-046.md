# Commit 046 - post-update+resilience batch step Rust 이관

## 커밋 요약
- stress 틱 후반의 `post-update(reserve/allostatic/state)`와 `resilience` 계산을 단일 Rust step으로 통합.
- `StressSystem`에서 별도 resilience 업데이트 함수를 제거하고 결합 step 결과를 직접 반영.

## 상세 변경
- `rust/crates/sim-systems/src/stat_curve.rs`
  - 신규 구조체:
    - `StressPostUpdateResilienceStep`
  - 신규 함수:
    - `stress_post_update_resilience_step(...) -> StressPostUpdateResilienceStep`
  - 내부 구성:
    - `stress_post_update_step` 실행
    - post-step의 `allostatic`를 사용해 `stress_resilience_value` 계산
  - 단위 테스트 1개 추가:
    - `stress_post_update_resilience_step_matches_component_steps`
    - 기존 분리 호출 결과와 동치 검증
- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 Godot 함수:
    - `stat_stress_post_update_resilience_step(...) -> Dictionary`
  - 인자 형식:
    - `scalar_inputs: PackedFloat32Array`
    - `flags: PackedByteArray`
  - 반환:
    - post-update state payload + `resilience`
- `scripts/core/simulation/sim_bridge.gd`
  - 신규 프록시:
    - `stat_stress_post_update_resilience_step(...)`
  - scalar/bool 입력을 PackedArray로 인코딩해 네이티브 호출
- `scripts/core/stats/stat_curve.gd`
  - 신규 함수:
    - `stress_post_update_resilience_step(...)`
  - Rust 우선 + fallback 제공
    - fallback은 `stress_post_update_step` + `stress_resilience_value` 조합
- `scripts/systems/psychology/stress_system.gd`
  - `_update_entity_stress`에서:
    - HEXACO 축/흉터 모디파이어를 수집해 `stress_post_update_resilience_step` 호출
    - `ed.reserve`, `ed.gas_stage`, `ed.allostatic`, `ed.resilience`를 단일 결과로 반영
  - 기존 `_update_resilience` 함수 제거
- `rust/crates/sim-test/src/main.rs`
  - stress 벤치에 `stress_post_update_resilience_step` 호출을 추가해 신규 결합 경로 측정 포함

## 기능 영향
- stress 틱 후반 post-update/resilience 구간의 브리지 round-trip이 추가로 감소.
- resilience 계산이 post-step 결과(allostatic 반영 이후) 기준으로 수행되는 기존 의미를 유지.
- stress 파이프라인의 핵심 math 경로가 결합 step 중심으로 더 정리됨.

## 검증
- `cd rust && cargo fmt -p sim-systems -p sim-bridge -p sim-test` 통과
- `cd rust && cargo test -q -p sim-systems` 통과 (34 tests)
- `cd rust && cargo test -q -p sim-bridge` 통과 (8 tests)
- `cd rust && cargo test -q -p sim-test` 통과
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000` 통과
  - 예시 출력: `ns_per_iter=165.8`
- `tools/migration_verify.sh` 통과
  - strict audit: inline localized fields 0 유지
