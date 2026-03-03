# Commit 044 - emotion/recovery/delta 통합 step Rust 이관

## 커밋 요약
- stress 틱 중반부의 `emotion contribution + recovery + delta/denial` 계산을 단일 Rust step으로 통합.
- Godot method 인자 개수 제한을 피하기 위해 해당 브리지 API는 PackedArray 인코딩 방식으로 연결.

## 상세 변경
- `rust/crates/sim-systems/src/stat_curve.rs`
  - 신규 구조체:
    - `StressEmotionRecoveryDeltaStep`
  - 신규 함수:
    - `stress_emotion_recovery_delta_step(...) -> StressEmotionRecoveryDeltaStep`
  - 내부 구성:
    - `stress_emotion_contribution`
    - `stress_recovery_value`
    - `stress_delta_step`
    - 위 3개를 결합해 단일 출력 payload 반환
  - 단위 테스트 1개 추가:
    - `stress_emotion_recovery_delta_step_matches_component_steps`
    - 기존 분리 계산 결과와 동치 검증
- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 Godot 함수:
    - `stat_stress_emotion_recovery_delta_step(...) -> Dictionary`
  - 구현 방식:
    - `emotion_inputs: PackedFloat32Array`
    - `scalar_inputs: PackedFloat32Array`
    - `flags: PackedByteArray`
    - 배열 인덱스 기반으로 Rust 함수 입력 복원 후 결과 반환
  - 배경:
    - `godot-rust` 메서드 파라미터 개수 제한(ParamTuple) 회피
- `scripts/core/simulation/sim_bridge.gd`
  - `stat_stress_emotion_recovery_delta_step(...)` 추가
  - 다수 scalar/bool 인자를 PackedArray로 인코딩해 네이티브 메서드 3-arg 형태로 전달
- `scripts/core/stats/stat_curve.gd`
  - 신규 함수:
    - `stress_emotion_recovery_delta_step(...)`
  - Rust 우선 + fallback 제공
    - fallback은 기존 `stress_emotion_contribution` + `stress_recovery_value` + `stress_delta_step` 조합
- `scripts/systems/psychology/stress_system.gd`
  - `_update_entity_stress`에서:
    - 기존 `_calc_emotion_contribution` + `_calc_recovery` + `stress_delta_step` 분리 경로 제거
    - `_calc_emotion_recovery_delta(...)` 단일 경로로 치환
  - 기존 `_calc_emotion_contribution`, `_calc_recovery` 함수 제거
  - breakdown 키(`emo_*`, `va_composite`, `recovery`) 유지
- `rust/crates/sim-test/src/main.rs`
  - stress 벤치 루프에 `stress_emotion_recovery_delta_step` 호출을 추가해 신규 결합 경로 측정 포함

## 기능 영향
- stress 틱 중반 연산이 결합되어 브리지 round-trip 추가 감소.
- denial hidden accumulator 갱신 의미와 recovery/감정 breakdown 의미는 기존과 동일하게 유지.
- 다인자 Godot 바인딩 한계를 우회하는 PackedArray 패턴이 stress 결합 API에 적용됨.

## 검증
- `cd rust && cargo fmt -p sim-systems -p sim-bridge -p sim-test` 통과
- `cd rust && cargo test -q -p sim-systems` 통과 (32 tests)
- `cd rust && cargo test -q -p sim-bridge` 통과 (8 tests)
- `cd rust && cargo test -q -p sim-test` 통과
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000` 통과
  - 예시 출력: `ns_per_iter=94.4`
- `tools/migration_verify.sh` 통과
  - strict audit: inline localized fields 0 유지
