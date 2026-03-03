# Commit 318 - Childcare 급식 수식 Rust 브리지 이관

## 커밋 요약
- `childcare_system`의 급식 핵심 계산(stockpile 취식량 계산, hunger 갱신)을 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `childcare_take_food(...) -> f32`
    - `childcare_hunger_after(...) -> f32`
  - 단위 테스트 추가:
    - `childcare_take_food_clamps_to_remaining_and_zero_bounds`
    - `childcare_hunger_after_applies_restore_and_clamp`

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_childcare_take_food(...)`
    - `body_childcare_hunger_after(...)`

- `scripts/systems/development/childcare_system.gd`
  - SimBridge 캐시/조회 로직(`_get_sim_bridge`) 추가.
  - `execute_tick`:
    - child hunger 갱신식을 Rust-first 호출로 전환.
  - `_withdraw_food`:
    - stockpile별 취식량(`take`) 계산을 Rust-first 호출로 전환.
  - 브리지 실패 시 기존 GDScript 계산 fallback 유지.

## 기능 영향
- Childcare 급식 루프의 반복 산식이 Rust 경로로 이동하여 per-tick 계산 비용을 절감.
- 기존 우선순위, 식량 소모 순서, 이벤트 emission 동작은 유지.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(bridged 대상 56개 기준)**: `51/56` 적용, 잔여 `5/56`.
- **잔여 주요 파일(5)**:
  - `scripts/systems/biology/population_system.gd`
  - `scripts/systems/psychology/coping_system.gd`
  - `scripts/systems/psychology/emotion_system.gd`
  - `scripts/systems/psychology/psychology_coordinator.gd`
  - `scripts/systems/record/chronicle_system.gd`
