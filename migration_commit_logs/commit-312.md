# Commit 312 - TraumaScar 획득/민감도 수식 Rust 브리지 이관

## 커밋 요약
- `trauma_scar_system`의 흉터 획득 확률/스트레스 민감도 배수 수식을 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `trauma_scar_acquire_chance(...) -> f32`
    - `trauma_scar_sensitivity_factor(...) -> f32`
  - 단위 테스트 추가:
    - `trauma_scar_acquire_chance_scales_and_clamps`
    - `trauma_scar_sensitivity_factor_applies_diminishing_stack_bonus`

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_trauma_scar_acquire_chance(...)`
    - `body_trauma_scar_sensitivity_factor(...)`

- `scripts/systems/psychology/trauma_scar_system.gd`
  - SimBridge 캐시/조회 로직(`_get_sim_bridge`) 추가.
  - `try_acquire_scar`:
    - 흉터 획득 확률 계산을 Rust-first 호출로 전환.
  - `get_scar_stress_sensitivity`:
    - 스택 감쇠 민감도 factor 계산을 Rust-first 호출로 전환.
  - 브리지 실패 시 기존 GDScript 계산 fallback 유지.

## 기능 영향
- TraumaScar의 반복 수식(획득 확률, 민감도 감쇠 factor)이 Rust 경로로 이동.
- scar 스택 처리/이벤트 emit/감정 드리프트 등 기존 흐름은 유지.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(bridged 대상 56개 기준)**: `45/56` 적용, 잔여 `11/56`.
- **잔여 주요 파일(11)**:
  - `scripts/systems/biology/population_system.gd`
  - `scripts/systems/development/ace_tracker.gd`
  - `scripts/systems/development/attachment_system.gd`
  - `scripts/systems/development/childcare_system.gd`
  - `scripts/systems/development/intergenerational_system.gd`
  - `scripts/systems/development/parenting_system.gd`
  - `scripts/systems/psychology/coping_system.gd`
  - `scripts/systems/psychology/emotion_system.gd`
  - `scripts/systems/psychology/psychology_coordinator.gd`
  - `scripts/systems/record/chronicle_system.gd`
  - `scripts/systems/record/memory_system.gd`
