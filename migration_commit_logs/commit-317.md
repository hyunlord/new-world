# Commit 317 - ACE Tracker 핵심 수식 Rust 브리지 이관

## 커밋 요약
- `ace_tracker`의 핵심 반복 산식(부분 점수 누적, 위협/박탈 합산, 성인 modifier 보정, 성인 백필, 총점 합산)을 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `ace_partial_score_next(...) -> f32`
    - `ace_score_total_from_partials(...) -> f32`
    - `ace_threat_deprivation_totals(...) -> [f32; 2]`
    - `ace_adult_modifiers_adjusted(...) -> [f32; 3]`
    - `ace_backfill_score(...) -> f32`
  - 단위 테스트 추가:
    - `ace_partial_score_next_clamps_to_unit_interval`
    - `ace_score_total_from_partials_clamps_to_ten`
    - `ace_threat_deprivation_totals_routes_by_type_code`
    - `ace_adult_modifiers_adjusted_applies_floor_and_protective_factor`
    - `ace_backfill_score_accounts_for_attachment_and_scars`

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_ace_partial_score_next(...)`
    - `body_ace_score_total_from_partials(...)`
    - `body_ace_threat_deprivation_totals(...)`
    - `body_ace_adult_modifiers_adjusted(...)`
    - `body_ace_backfill_score(...)`

- `scripts/systems/development/ace_tracker.gd`
  - SimBridge 캐시/조회 로직(`_get_sim_bridge`) 추가.
  - `record_ace_event`:
    - partial score 누적을 Rust-first 호출로 전환.
  - `get_threat_deprivation_scores`:
    - `PackedFloat32Array`/`PackedInt32Array`로 Rust 합산 호출 경로 추가.
  - `calculate_adult_modifiers`:
    - break floor + protective factor 보정 산식을 Rust-first 호출로 전환.
  - `backfill_ace_for_adult`:
    - 성인 ACE 추정 산식을 Rust-first 호출로 전환.
    - attachment 문자열을 code로 변환하는 `_attachment_code` 헬퍼 추가.
  - `_recalculate_total_score`:
    - partials 기반 총점 계산을 Rust-first 호출로 전환.
  - 브리지 실패 시 기존 GDScript 계산 fallback 유지.

## 기능 영향
- ACE tracker의 고빈도 수식 경로가 Rust로 이동해 계산 비용을 낮춤.
- 기존 이벤트/로그/메타 저장 동작은 유지하며, 브리지 미사용 환경에서도 동일 fallback으로 동작.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(bridged 대상 56개 기준)**: `50/56` 적용, 잔여 `6/56`.
- **잔여 주요 파일(6)**:
  - `scripts/systems/biology/population_system.gd`
  - `scripts/systems/development/childcare_system.gd`
  - `scripts/systems/psychology/coping_system.gd`
  - `scripts/systems/psychology/emotion_system.gd`
  - `scripts/systems/psychology/psychology_coordinator.gd`
  - `scripts/systems/record/chronicle_system.gd`
