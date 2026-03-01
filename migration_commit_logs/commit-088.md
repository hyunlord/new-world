# Commit 088 - entity detail panel 정규화 조회 batch화

## 커밋 요약
- `EntityDetailPanel`의 needs/personality/derived 렌더 구간에서 다수 `StatQuery.get_normalized()` 호출을 batch 조회로 통합.

## 상세 변경
- `scripts/ui/panels/entity_detail_panel.gd`
  - batch 대상 stat id 상수 추가:
    - `_NEEDS_BASIC_STAT_IDS`
    - `_NEEDS_HIGHER_STAT_IDS`
    - `_PERSONALITY_AXIS_STAT_IDS` + `_PERSONALITY_AXIS_INDEX`
    - `_DERIVED_STAT_IDS`
  - scratch buffer 추가:
    - `_needs_basic_norm_values`
    - `_needs_higher_norm_values`
    - `_personality_axis_norm_values`
    - `_derived_norm_values`
  - needs 기본/상위 섹션:
    - `get_normalized_batch_into(..., true)` 1회 호출 결과를 인덱스로 사용하도록 전환.
  - personality 축(HEXACO):
    - 축별 개별 조회 대신 6축 batch 조회 결과를 사용.
  - derived stats:
    - 파생 스탯 개별 조회 8회를 batch 조회 1회로 전환.
    - social capital raw stat 조회 경로는 기존 유지.

## 기능 영향
- 패널 표시값/색상/순서는 기존과 동일.
- 패널 draw 루프에서 정규화 조회 호출 수를 줄여 렌더 경로 오버헤드를 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=465.0`, `checksum=13761358.00000`
