# Commit 083 - StatQuery 정규화 range 캐시

## 커밋 요약
- `StatQuery`의 정규화 경로에서 stat range 조회를 캐시해 반복 호출 오버헤드를 줄임.

## 상세 변경
- `scripts/core/stats/stat_query.gd`
  - `_normalized_range_cache` 추가 및 `_ready()`에서 초기화.
  - 신규 내부 헬퍼 `_get_normalized_range_cached(stat_id)` 추가.
  - `get_normalized()`와 `get_normalized_batch()`가 `StatDefinitionScript.get_range()` 직접 호출 대신 캐시된 range를 사용하도록 변경.

## 기능 영향
- 정규화 결과 값은 기존과 동일.
- stress 등 반복 정규화 조회 경로에서 stat range 해석 비용이 감소.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=483.6`, `checksum=13761358.00000`
