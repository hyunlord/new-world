# Commit 326 - Emotion impulse 배치 계산 Tier-2 Rust 이관

## 커밋 요약
- `emotion_system`의 이벤트 impulse 계산을 단건 호출에서 Rust 배치 호출로 확장해 브리지 호출 오버헤드를 줄임.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `emotion_event_impulse_batch(...) -> Vec<f32>`
  - 단위 테스트 추가:
    - `emotion_event_impulse_batch_emits_expected_flat_length`

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_emotion_event_impulse_batch(...)`

- `scripts/systems/psychology/emotion_system.gd`
  - SimBridge 메서드 상수 추가:
    - `_SIM_BRIDGE_EVENT_IMPULSE_BATCH_METHOD`
  - `_get_sim_bridge` 검증 목록 확장.
  - `_calculate_event_impulse`:
    - 이벤트 전체를 flat 입력으로 묶어 Rust batch 메서드를 1회 호출.
    - batch 결과를 이벤트별 impulse로 복원해 habituation/누적/trace 생성 적용.
    - batch 실패 시 기존 per-event Rust 단건 호출/순수 GDScript 계산 fallback 유지.
  - 반복 루프 내 personality sensitivity 계산을 1회 계산으로 정리.

## 기능 영향
- 감정 이벤트 처리에서 브리지 호출 횟수를 이벤트 수 N회에서 1회로 줄여 처리 효율 개선.
- 기존 결과 의미는 유지.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **코어 전환 축(bridged 대상 56개 기준)**: `56/56` 완료, 잔여 `0/56`.
- **심화(Tier-2) 전환**: 진행 중.
