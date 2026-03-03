# Commit 323 - Emotion mental break 판정 수식 Rust 브리지 이관

## 커밋 요약
- `emotion_system`의 mental break 핵심 판정 수식(역치, 발동 확률, break 타입 결정)을 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `emotion_break_threshold(...) -> f32`
    - `emotion_break_trigger_probability(...) -> f32`
    - `emotion_break_type_code(...) -> i32`
  - 단위 테스트 추가:
    - `emotion_break_threshold_and_probability_are_bounded`
    - `emotion_break_type_code_prioritizes_outrage_then_dominant_emotion`

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_emotion_break_threshold(...)`
    - `body_emotion_break_trigger_probability(...)`
    - `body_emotion_break_type_code(...)`

- `scripts/systems/psychology/emotion_system.gd`
  - SimBridge 캐시/조회 로직(`_get_sim_bridge`) 추가.
  - `_check_mental_break`:
    - threshold 계산을 Rust-first 호출로 전환.
    - tick 단위 발동 확률(sigmoid * tick_prob) 계산을 Rust-first 호출로 전환.
  - `_determine_break_type`:
    - outrage/fear/anger/sadness/disgust 입력 기반 break type code 계산 + label 복원을 Rust-first 호출로 전환.
  - 브리지 실패 시 기존 GDScript 계산 fallback 유지.

## 기능 영향
- emotion 시스템의 고빈도 mental-break 판정 계산이 Rust 경로로 이동.
- break 시작/종료 이벤트, 행동 전환, Chronicle 기록 흐름은 유지.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(bridged 대상 56개 기준)**: `56/56` 적용, 잔여 `0/56`.
- **잔여 주요 파일**: 없음.
