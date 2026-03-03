# Commit 314 - Attachment 핵심 수식 Rust 브리지 이관

## 커밋 요약
- `attachment_system`의 유형 판정/부모양육 품질/보호계수 계산을 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `attachment_type_code(...) -> i32`
    - `attachment_raw_parenting_quality(...) -> f32`
    - `attachment_coping_quality_step(...) -> [f32; 3]`
    - `attachment_protective_factor(...) -> f32`
  - 단위 테스트 추가:
    - `attachment_type_code_follows_threshold_ordering`
    - `attachment_raw_parenting_quality_decreases_with_burden`
    - `attachment_coping_quality_step_returns_adjusted_triplet`
    - `attachment_protective_factor_is_clamped`

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_attachment_type_code(...)`
    - `body_attachment_raw_parenting_quality(...)`
    - `body_attachment_coping_quality_step(...)`
    - `body_attachment_protective_factor(...)`

- `scripts/systems/development/attachment_system.gd`
  - SimBridge 캐시/조회 로직(`_get_sim_bridge`) 추가.
  - `determine_attachment_type`:
    - 유형 판정 로직을 Rust-first 호출로 전환(code→string 매핑).
  - `_compute_raw_quality`:
    - 양육 품질 핵심 수식을 Rust-first 호출로 전환.
  - `_apply_coping_modifiers_to_quality`:
    - substance coping 품질/부작용 누적 step을 Rust-first 호출로 전환.
  - `calculate_protective_factor`:
    - 보호요인 계산을 Rust-first 호출로 전환.
  - 브리지 실패 시 기존 GDScript 계산 fallback 유지.

## 기능 영향
- Attachment 관련 반복 수식이 Rust 경로로 이동.
- 기존 메타키 업데이트/chronicle 기록/성인 효과 적용 흐름은 유지.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(bridged 대상 56개 기준)**: `47/56` 적용, 잔여 `9/56`.
- **잔여 주요 파일(9)**:
  - `scripts/systems/biology/population_system.gd`
  - `scripts/systems/development/ace_tracker.gd`
  - `scripts/systems/development/childcare_system.gd`
  - `scripts/systems/development/intergenerational_system.gd`
  - `scripts/systems/development/parenting_system.gd`
  - `scripts/systems/psychology/coping_system.gd`
  - `scripts/systems/psychology/emotion_system.gd`
  - `scripts/systems/psychology/psychology_coordinator.gd`
  - `scripts/systems/record/chronicle_system.gd`
