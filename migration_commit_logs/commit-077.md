# Commit 077 - tr_id 경로 key-id 캐시 적용

## 커밋 요약
- `Locale.tr_id()`를 key-id 캐시 기반 조회로 최적화해 UI 반복 호출 비용을 줄임.

## 상세 변경
- `scripts/core/simulation/locale.gd`
  - `_tr_id_key_id_cache` 추가.
  - locale 로드 시 `_tr_id_key_id_cache.clear()`로 캐시 초기화.
  - `tr_id(prefix, id)`에서:
    - 조합 키(`PREFIX_ID`)별 key id를 1회 해석 후 캐시.
    - key id가 있으면 `ltr_id()` 경로 우선 사용.
    - 미지원 상황은 기존 `ltr()` + `id` fallback 동작 유지.

## 기능 영향
- `Locale.tr_id()` 반환 문자열 의미는 기존과 동일.
- HUD/패널 등 `tr_id` 다중 호출 구간의 key lookup 오버헤드를 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=462.4`, `checksum=13761358.00000`
