# Commit 084 - 정규화 batch output-buffer 재사용

## 커밋 요약
- 정규화 batch 조회 결과 배열을 caller가 재사용할 수 있도록 API를 확장하고 stress tick에 적용.

## 상세 변경
- `scripts/core/stats/stat_query.gd`
  - 신규 API `get_normalized_batch_into(entity, stat_ids, out_values)` 추가.
  - `get_normalized_batch()`는 내부에서 `get_normalized_batch_into()`를 호출하도록 변경(호환 유지).
  - `entity == null` 케이스도 out buffer를 0으로 채워 재사용 가능하게 처리.
- `scripts/systems/psychology/stress_system.gd`
  - scratch buffer `_tick_norm_values` 추가.
  - `_update_entity_stress()`가 `StatQuery.get_normalized_batch_into(...)`를 사용해 norm 결과 배열을 재사용.

## 기능 영향
- 정규화 값 및 stress 계산 결과는 기존과 동일.
- stress tick 엔티티 루프에서 `PackedFloat32Array` 임시 할당을 줄여 GC/메모리 churn을 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=458.2`, `checksum=13761358.00000`
