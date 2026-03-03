# Commit 045 - trace+emotion/recovery/delta batch step Rust 이관

## 커밋 요약
- stress 틱의 trace 처리와 emotion/recovery/delta 계산을 단일 Rust step으로 통합.
- `StressSystem`에서 trace 처리 함수와 emotion/recovery/delta 함수를 분리 호출하던 경로를 결합 경로로 치환.

## 상세 변경
- `rust/crates/sim-systems/src/stat_curve.rs`
  - 신규 구조체:
    - `StressTraceEmotionRecoveryDeltaStep`
  - 신규 함수:
    - `stress_trace_emotion_recovery_delta_step(...) -> StressTraceEmotionRecoveryDeltaStep`
  - 내부 구성:
    - `stress_trace_batch_step`
    - `stress_emotion_recovery_delta_step`
    - trace total을 delta 계산의 trace input으로 연결
  - 단위 테스트 1개 추가:
    - `stress_trace_emotion_recovery_delta_step_matches_component_steps`
    - 기존 분리 호출과 결과 동치 검증
- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 Godot 함수:
    - `stat_stress_trace_emotion_recovery_delta_step(...) -> Dictionary`
  - 인자 인코딩:
    - `per_tick`, `decay_rate`, `min_keep`
    - `emotion_inputs: PackedFloat32Array`
    - `scalar_inputs: PackedFloat32Array`
    - `flags: PackedByteArray`
  - 반환:
    - trace(`total_trace_contribution`, `updated_per_tick`, `active_mask`)
    - emotion/recovery/delta payload
- `scripts/core/simulation/sim_bridge.gd`
  - 신규 프록시:
    - `stat_stress_trace_emotion_recovery_delta_step(...)`
  - 다수 scalar/bool 입력을 PackedArray로 인코딩해 네이티브에 전달
- `scripts/core/stats/stat_curve.gd`
  - 신규 함수:
    - `stress_trace_emotion_recovery_delta_step(...)`
  - Rust 우선 + fallback 제공
    - fallback은 `stress_trace_batch_step` + `stress_emotion_recovery_delta_step` 조합
- `scripts/systems/psychology/stress_system.gd`
  - `_update_entity_stress`에서:
    - `_calc_trace_emotion_recovery_delta(...)` 단일 호출 사용
  - 기존 `_process_stress_traces`, `_calc_emotion_recovery_delta` 제거
  - 새 helper `_calc_trace_emotion_recovery_delta` 추가:
    - trace 배열 구성/갱신
    - trace breakdown 반영
    - emotion/recovery breakdown 반영
    - 최종 delta/hidden accumulator 반환
- `rust/crates/sim-test/src/main.rs`
  - stress 벤치에 `stress_trace_emotion_recovery_delta_step` 호출을 추가해 신규 결합 경로 측정 포함

## 기능 영향
- stress 틱 핫패스에서 trace + emotion/recovery/delta 구간이 결합되어 브리지 round-trip이 추가로 감소.
- trace 유지/제거(active_mask), breakdown 키(`trace_*`, `emo_*`, `va_composite`, `recovery`), denial hidden accumulator 의미는 유지.
- sim-bridge의 다인자 수치 API는 PackedArray 인코딩 패턴으로 확장됨.

## 검증
- `cd rust && cargo fmt -p sim-systems -p sim-bridge -p sim-test` 통과
- `cd rust && cargo test -q -p sim-systems` 통과 (33 tests)
- `cd rust && cargo test -q -p sim-bridge` 통과 (8 tests)
- `cd rust && cargo test -q -p sim-test` 통과
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000` 통과
  - 예시 출력: `ns_per_iter=157.2`
- `tools/migration_verify.sh` 통과
  - strict audit: inline localized fields 0 유지
