# Commit 086 - 정규화 batch fast-path(정의 검사 생략)

## 커밋 요약
- `get_normalized_batch_into`에 정의 검사 생략 옵션을 추가하고 stress tick에서 fast path를 사용.

## 상세 변경
- `scripts/core/stats/stat_query.gd`
  - `get_normalized_batch_into(..., assume_defined: bool = false)` 시그니처로 확장.
  - `assume_defined=true`일 때 `StatDefinitionScript.has_def()` 체크를 생략.
  - 기본값은 `false`로 유지해 기존 호출 안전성 보존.
- `scripts/systems/psychology/stress_system.gd`
  - `_TICK_NORM_STAT_IDS`는 고정 유효 stat id 집합이므로 batch 호출 시 `assume_defined=true` 적용.

## 기능 영향
- stress 정규화 값과 계산 결과는 기존과 동일.
- stress tick에서 stat 정의 확인 분기 비용을 줄여 입력 수집 fast path를 강화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=511.9`, `checksum=13761358.00000`
