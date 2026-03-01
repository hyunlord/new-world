# Commit 047 - full stress tick step 단일 Rust 호출화

## 커밋 요약
- stress 틱 핵심 수학 경로를 `stress_tick_step` 단일 Rust step으로 통합.
- `StressSystem`은 기존 다단계 helper 호출(Primary, Trace+Emotion+Recovery+Delta, Post+Resilience)을 1회 호출로 축소.

## 상세 변경
- `rust/crates/sim-systems/src/stat_curve.rs`
  - 신규 구조체:
    - `StressTickStep`
  - 신규 함수:
    - `stress_tick_step(...) -> StressTickStep`
  - 포함 범위:
    - `stress_primary_step` (appraisal + continuous needs)
    - `stress_trace_emotion_recovery_delta_step`
    - stress 업데이트(`stress + delta`, clamp 0..2000)
    - `stress_post_update_resilience_step`
  - 단위 테스트 1개 추가:
    - `stress_tick_step_matches_composed_steps`
    - 기존 조합 호출 결과와 동치 검증
- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 Godot 함수:
    - `stat_stress_tick_step(...) -> Dictionary`
  - 인자 형식:
    - `per_tick: PackedFloat32Array`
    - `decay_rate: PackedFloat32Array`
    - `min_keep: f32`
    - `scalar_inputs: PackedFloat32Array` (40개 스칼라 입력)
    - `flags: PackedByteArray` (`is_sleeping`, `is_safe`, `denial_active`)
  - 반환:
    - continuous/trace/emotion/recovery/delta
    - hidden accumulator, stress
    - reserve/gas/allostatic/state snapshot/resilience
- `scripts/core/simulation/sim_bridge.gd`
  - 신규 프록시:
    - `stat_stress_tick_step(...)`
- `scripts/core/stats/stat_curve.gd`
  - 신규 함수:
    - `stress_tick_step(...)`
  - Rust 우선 + fallback 제공
    - fallback은 기존 결합 step들을 순차 조합
- `scripts/systems/psychology/stress_system.gd`
  - `_update_entity_stress`를 단일 tick-step 호출 기반으로 재구성:
    - scalar/flag 입력 패킹
    - trace 배열 갱신(active_mask + updated_per_tick) 반영
    - breakdown(`hunger`, `energy_deficit`, `social_isolation`, `trace_*`, `emo_*`, `va_composite`, `recovery`) 유지
    - `ed.stress`, `ed.stress_delta_last`, `ed.reserve`, `ed.gas_stage`, `ed.allostatic`, `ed.resilience` 반영
  - 더 이상 필요 없는 내부 helper 제거:
    - `_calc_primary_inputs`
    - `_calc_trace_emotion_recovery_delta`
- `rust/crates/sim-test/src/main.rs`
  - stress 벤치에 `stress_tick_step` 호출 추가

## 기능 영향
- stress 틱의 핵심 수학 경로가 사실상 1회 Rust 브리지 호출로 수렴.
- breakdown/trace 유지/hidden accumulator 의미는 기존과 동일하게 유지.
- 브리지 인자 제한은 PackedArray 인코딩으로 해결.

## 검증
- `cd rust && cargo fmt -p sim-systems -p sim-bridge -p sim-test` 통과
- `cd rust && cargo test -q -p sim-systems` 통과 (35 tests)
- `cd rust && cargo test -q -p sim-bridge` 통과 (8 tests)
- `cd rust && cargo test -q -p sim-test` 통과
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000` 통과
  - 예시 출력: `ns_per_iter=243.7`
- `tools/migration_verify.sh` 통과
  - strict audit: inline localized fields 0 유지
