# Commit 168 - Locale.ltr key-id 캐시 최적화

## 커밋 요약
- `Locale.ltr()`에 key-id 캐시를 추가해 반복 키 조회 시 딕셔너리 탐색 오버헤드를 줄이고, id 기반 문자열 조회 경로를 통일.

## 상세 변경
- `scripts/core/simulation/locale.gd`
  - `_ltr_key_id_cache` 추가.
  - `load_locale()`에서 `_ltr_key_id_cache.clear()` 수행.
  - `ltr(key)` 구현 변경:
    - 기존: `_flat_strings.has(key)` + `_flat_strings[key]`
    - 변경: key-id 캐시 조회/갱신 후 `ltr_id(id)` 반환
    - 미존재 key는 기존처럼 key 자체 반환 유지

## 기능 영향
- 번역 결과(정상 키/미존재 키 fallback)는 유지.
- 반복되는 `Locale.ltr("SOME_KEY")` 호출 경로의 조회 비용을 줄여 UI/로그 텍스트 조회 효율 개선.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 98 tests)
  - localization compile `filled=0`, `updated=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=474.9`, `checksum=24032652.00000`
- `cd rust && cargo run -q -p sim-test -- --bench-needs-math --iters 10000`
  - `ns_per_iter=206.1`, `checksum=38457848.00000`
