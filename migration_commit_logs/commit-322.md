# Commit 322 - Coping 확률/선택 수식 Rust 브리지 이관

## 커밋 요약
- `coping_system`의 핵심 확률 계산(학습 확률 sigmoid, softmax 선택)을 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `coping_learn_probability(...) -> f32`
    - `coping_softmax_index(...) -> i32`
  - 단위 테스트 추가:
    - `coping_learn_probability_rises_with_stress_and_recovery`
    - `coping_softmax_index_selects_valid_index`

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_coping_learn_probability(...)`
    - `body_coping_softmax_index(...)`

- `scripts/systems/psychology/coping_system.gd`
  - SimBridge 캐시/조회 로직(`_get_sim_bridge`) 추가.
  - `_calculate_learn_probability`:
    - stress/allostatic/break_count/owned_count 기반 sigmoid 학습확률을 Rust-first 호출로 전환.
  - `_softmax_select`:
    - utility score 배열의 categorical 선택을 Rust softmax index 호출로 전환.
  - 브리지 실패 시 기존 GDScript 계산 fallback 유지.

## 기능 영향
- coping 획득/업그레이드 분기의 고빈도 확률 계산이 Rust 경로로 이동.
- 기존 coping state 업데이트/이벤트 로깅 흐름은 유지.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(bridged 대상 56개 기준)**: `55/56` 적용, 잔여 `1/56`.
- **잔여 주요 파일(1)**:
  - `scripts/systems/psychology/emotion_system.gd`
