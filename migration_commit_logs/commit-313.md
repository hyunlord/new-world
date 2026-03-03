# Commit 313 - Memory 감쇠/요약 강도 수식 Rust 브리지 이관

## 커밋 요약
- `memory_system`의 working memory 감쇠 배치 계산과 요약 강도 계산을 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `memory_decay_intensity(...) -> f32`
    - `memory_decay_batch(...) -> Vec<f32>`
    - `memory_summary_intensity(...) -> f32`
  - 단위 테스트 추가:
    - `memory_decay_intensity_matches_exponential_formula`
    - `memory_decay_batch_uses_pairwise_min_length`
    - `memory_summary_intensity_scales_max_value`

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_memory_decay_batch(...)`
    - `body_memory_summary_intensity(...)`

- `scripts/systems/record/memory_system.gd`
  - SimBridge 캐시/조회 로직(`_get_sim_bridge`) 추가.
  - `_decay_working_memory`:
    - intensity/rate를 PackedFloat32Array로 구성해 Rust 배치 감쇠를 우선 호출.
    - 브리지 실패/길이 불일치 시 기존 GDScript 루프 fallback 유지.
  - `_compress_old_entries`:
    - summary intensity(`max_int * 0.7`) 계산을 Rust helper 우선 호출로 전환.
    - 브리지 실패 시 기존 계산 fallback 유지.

## 기능 영향
- Memory annual tick 경로의 핵심 감쇠 계산(exp)과 요약 강도 산식이 Rust 경로로 이동.
- eviction/compression grouping/permanent_history 승격 규칙은 기존 유지.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(bridged 대상 56개 기준)**: `46/56` 적용, 잔여 `10/56`.
- **잔여 주요 파일(10)**:
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
