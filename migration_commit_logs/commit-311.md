# Commit 311 - TraitViolation 보조 수식 Rust 브리지 이관

## 커밋 요약
- `trait_violation_system`의 맥락 배수/Facet 스케일/Intrusive chance 계산을 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `trait_violation_context_modifier(...) -> f32`
    - `trait_violation_facet_scale(...) -> f32`
    - `trait_violation_intrusive_chance(...) -> f32`
  - 단위 테스트 추가:
    - `trait_violation_context_modifier_applies_expected_multipliers`
    - `trait_violation_facet_scale_increases_above_threshold`
    - `trait_violation_intrusive_chance_requires_ptsd_and_decays_over_time`

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_trait_violation_context_modifier(...)`
    - `body_trait_violation_facet_scale(...)`
    - `body_trait_violation_intrusive_chance(...)`

- `scripts/systems/psychology/trait_violation_system.gd`
  - SimBridge 캐시/조회 로직(`_get_sim_bridge`) 추가.
  - `_process_intrusive_thoughts`:
    - intrusive chance 계산을 Rust-first 호출로 전환.
  - `_calc_context_modifier`:
    - 맥락 배수 계산을 Rust-first 호출로 전환.
  - `_calc_facet_scale`:
    - facet 스케일 계산을 Rust-first 호출로 전환.
  - 브리지 실패 시 기존 GDScript 계산 fallback 유지.

## 기능 영향
- TraitViolation의 반복 보조 수식(맥락/Facet/intrusive chance)이 Rust 경로로 이동.
- violation history/PTSD/PTG/스트레스 주입 이벤트 플로우는 기존 유지.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(bridged 대상 56개 기준)**: `44/56` 적용, 잔여 `12/56`.
- **잔여 주요 파일(12)**:
  - `scripts/systems/biology/population_system.gd`
  - `scripts/systems/development/ace_tracker.gd`
  - `scripts/systems/development/attachment_system.gd`
  - `scripts/systems/development/childcare_system.gd`
  - `scripts/systems/development/intergenerational_system.gd`
  - `scripts/systems/development/parenting_system.gd`
  - `scripts/systems/psychology/coping_system.gd`
  - `scripts/systems/psychology/emotion_system.gd`
  - `scripts/systems/psychology/psychology_coordinator.gd`
  - `scripts/systems/psychology/trauma_scar_system.gd`
  - `scripts/systems/record/chronicle_system.gd`
  - `scripts/systems/record/memory_system.gd`
