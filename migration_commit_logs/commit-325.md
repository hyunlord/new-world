# Commit 325 - Emotion 이벤트 impulse 수식 Tier-2 Rust 이관

## 커밋 요약
- `emotion_system`의 이벤트→8감정 impulse 변환 수식을 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `emotion_event_impulse_from_appraisal(...) -> [f32; 8]`
  - 단위 테스트 추가:
    - `emotion_event_impulse_from_appraisal_matches_expected_shape`

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_emotion_event_impulse_from_appraisal(...)`

- `scripts/systems/psychology/emotion_system.gd`
  - SimBridge 메서드 상수 추가:
    - `_SIM_BRIDGE_EVENT_IMPULSE_METHOD`
  - `_get_sim_bridge` 검증 목록 확장.
  - `_calculate_event_impulse`:
    - appraisal+personality sensitivity 입력을 `PackedFloat32Array`로 Rust에 전달해 8감정 impulse를 일괄 계산.
    - Rust 실패 시 기존 GDScript 식 fallback 유지.

## 기능 영향
- emotion 이벤트 처리 루프의 핵심 수식 계산이 Rust로 이전되어 per-event 계산 부담 감소.
- 기존 habituation, trace 생성, 누적 로직은 유지.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **코어 전환 축(bridged 대상 56개 기준)**: `56/56` 완료, 잔여 `0/56`.
- **심화(Tier-2) 전환**: 진행 중.
