# Commit 075 - month name lookup key-id 캐시화

## 커밋 요약
- `Locale.get_month_name()` 경로를 key-id 캐시 기반으로 최적화.

## 상세 변경
- `scripts/core/simulation/locale.gd`
  - `_month_key_ids: PackedInt32Array` 추가.
  - locale 로드 완료 시 `_refresh_month_key_ids()`를 호출해 `MONTH_1..12` key id를 미리 계산.
  - `get_month_name(month)`가 key id가 있으면 `ltr_id()` 배열 조회를 우선 사용.
  - key id 미구축/미지원 시 기존 `ltr("MONTH_n")` fallback 유지.

## 기능 영향
- 월 이름 문자열 결과는 기존과 동일.
- 날짜 포맷에서 반복되는 월 이름 조회가 문자열 key lookup 대신 id 캐시 경로를 사용해 미세 오버헤드를 줄임.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=452.0`, `checksum=13761358.00000`
