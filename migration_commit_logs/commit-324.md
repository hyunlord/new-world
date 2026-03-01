# Commit 324 - Emotion temporal/contagion 보조 수식 Tier-2 Rust 이관

## 커밋 요약
- `emotion_system`의 반복 계산 경로(half-life 조정, baseline 계산, habituation, contagion 보조 exp 수식)를 Rust-first로 추가 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `emotion_adjusted_half_life(...) -> f32`
    - `emotion_baseline_value(...) -> f32`
    - `emotion_habituation_factor(...) -> f32`
    - `emotion_contagion_susceptibility(...) -> f32`
    - `emotion_contagion_distance_factor(...) -> f32`
  - 단위 테스트 추가:
    - `emotion_adjusted_half_life_and_baseline_follow_formula`
    - `emotion_habituation_and_contagion_factors_are_sensible`

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_emotion_adjusted_half_life(...)`
    - `body_emotion_baseline_value(...)`
    - `body_emotion_habituation_factor(...)`
    - `body_emotion_contagion_susceptibility(...)`
    - `body_emotion_contagion_distance_factor(...)`

- `scripts/systems/psychology/emotion_system.gd`
  - SimBridge 메서드 상수 추가.
  - `_get_sim_bridge` 검증 목록 확장.
  - `_get_adjusted_half_life`를 Rust-first로 전환.
  - `_get_baseline`을 Rust-first로 전환.
  - `_get_habituation`을 Rust-first로 전환.
  - `_apply_contagion_settlement`의 susceptibility/distance factor 계산을 Rust-first로 전환.
  - 브리지 실패 시 기존 GDScript 계산 fallback 유지.

## 기능 영향
- 감정 갱신 루프에서 반복되는 지수/클램프 계산 일부가 Rust 경로로 이동.
- 기존 감정 상태 전이/행동 의미는 유지.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **코어 전환 축(bridged 대상 56개 기준)**: `56/56` 완료, 잔여 `0/56`.
- **심화(Tier-2) 전환**: 진행 중 (추가 미세 수식 이관 단위 계속 가능).
