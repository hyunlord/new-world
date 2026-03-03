# Commit 320 - Chronicle prune 규칙 Rust 브리지 이관

## 커밋 요약
- `chronicle_system`의 prune 스케줄/보존 규칙(연도 간격 판정, cutoff tick 계산, world/personal event 유지 판정)을 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `chronicle_should_prune(...) -> bool`
    - `chronicle_cutoff_tick(...) -> i32`
    - `chronicle_keep_world_event(...) -> bool`
    - `chronicle_keep_personal_event(...) -> bool`
  - 단위 테스트 추가:
    - `chronicle_should_prune_respects_interval`
    - `chronicle_cutoff_tick_calculates_age_window`
    - `chronicle_keep_world_event_applies_importance_bands`
    - `chronicle_keep_personal_event_keeps_high_importance_or_valid_world_tick`

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_chronicle_should_prune(...)`
    - `body_chronicle_cutoff_tick(...)`
    - `body_chronicle_keep_world_event(...)`
    - `body_chronicle_keep_personal_event(...)`

- `scripts/systems/record/chronicle_system.gd`
  - SimBridge 캐시/조회 로직(`_get_sim_bridge`) 추가.
  - `prune_old_events`:
    - prune 여부 판정, cutoff tick 산정, world event keep 판정, personal event keep 판정을 Rust-first 호출로 전환.
  - 브리지 실패 시 기존 GDScript 조건식 fallback 유지.

## 기능 영향
- Chronicle 정리(prune) 루프의 분기 계산이 Rust 경로로 이동.
- 이벤트 저장 구조와 기존 pruning 의미/결과는 유지.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(bridged 대상 56개 기준)**: `53/56` 적용, 잔여 `3/56`.
- **잔여 주요 파일(3)**:
  - `scripts/systems/psychology/coping_system.gd`
  - `scripts/systems/psychology/emotion_system.gd`
  - `scripts/systems/psychology/psychology_coordinator.gd`
