# Commit 082 - stress 정규화 스탯 batch 조회

## 커밋 요약
- stress tick에서 다수 `StatQuery.get_normalized()` 호출을 batch API 1회 호출로 통합.

## 상세 변경
- `scripts/core/stats/stat_query.gd`
  - 신규 API `get_normalized_batch(entity, stat_ids)` 추가.
  - 단일 `stat_cache` 조회 경로를 공유해 여러 stat의 정규화 값을 `PackedFloat32Array`로 반환.
  - 기존 `get_normalized` 동작은 유지.
- `scripts/systems/psychology/stress_system.gd`
  - `_TICK_NORM_STAT_IDS` 상수와 인덱스 상수 추가.
  - `_update_entity_stress()`에서 NEED/HEXACO 정규화 값을 batch 결과로 읽도록 전환.
  - 기존 값 의미(배고픔/에너지/사회/HEXACO 각 축)는 동일 유지.

## 기능 영향
- stress 계산 로직과 결과는 기존과 동일.
- tick당 정규화 stat 조회의 함수 호출/캐시 접근 중복이 줄어 stress 입력 수집 경로가 경량화됨.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=458.8`, `checksum=13761358.00000`
