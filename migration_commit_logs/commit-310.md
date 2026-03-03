# Commit 310 - MentalBreak 역치/확률 수식 Rust 브리지 이관

## 커밋 요약
- `mental_break_system`의 역치 계산과 발동 확률 계산을 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `mental_break_threshold(...) -> f32`
    - `mental_break_chance(...) -> f32`
  - 단위 테스트 추가:
    - `mental_break_threshold_applies_reserve_and_scar_reductions`
    - `mental_break_chance_respects_threshold_and_modifiers`

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_mental_break_threshold(...)`
    - `body_mental_break_chance(...)`

- `scripts/systems/psychology/mental_break_system.gd`
  - SimBridge 캐시/조회 로직(`_get_sim_bridge`) 추가.
  - `_calc_threshold`:
    - Rust-first 호출로 역치 계산 수행.
    - 브리지 실패 시 기존 GDScript 계산 fallback 유지.
  - `_calc_break_chance`:
    - Rust-first 호출로 발동 확률 계산 수행.
    - 브리지 실패 시 기존 GDScript 계산 fallback 유지.

## 기능 영향
- 멘탈 브레이크 판정 핵심 수식(역치/확률)이 Rust 경로로 이동.
- 브레이크 타입 선택/발동/종료 흐름 및 신호/로그 동작은 기존 유지.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(bridged 대상 56개 기준)**: `43/56` 적용, 잔여 `13/56`.
- **잔여 주요 파일(13)**:
  - `scripts/systems/biology/population_system.gd`
  - `scripts/systems/development/ace_tracker.gd`
  - `scripts/systems/development/attachment_system.gd`
  - `scripts/systems/development/childcare_system.gd`
  - `scripts/systems/development/intergenerational_system.gd`
  - `scripts/systems/development/parenting_system.gd`
  - `scripts/systems/psychology/coping_system.gd`
  - `scripts/systems/psychology/emotion_system.gd`
  - `scripts/systems/psychology/psychology_coordinator.gd`
  - `scripts/systems/psychology/trait_violation_system.gd`
  - `scripts/systems/psychology/trauma_scar_system.gd`
  - `scripts/systems/record/chronicle_system.gd`
  - `scripts/systems/record/memory_system.gd`
