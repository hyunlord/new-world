# Commit 116 - 경량 포맷 사전 문자열 변환 제거

## 커밋 요약
- `Locale.trf*` 호출 전 `str(...)`를 미리 생성하던 경로를 정리해 중복 문자열 변환을 제거.

## 상세 변경
- `scripts/ui/panels/chronicle_panel.gd`
  - `UI_SHORT_DATE` 포맷 인자로 `str(evt.month/day)` 대신 정수 값을 직접 전달.

- `scripts/ui/panels/stats_detail_panel_legacy.gd`
  - `UI_TECH_COUNT_FMT` 포맷 인자로 `str(known/forgotten)` 대신 정수 값을 직접 전달.

## 기능 영향
- 표시 문자열 의미는 기존과 동일.
- 포맷 호출 전 사전 문자열 변환을 줄여 이벤트/통계 렌더 경로의 미세 할당 오버헤드를 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=452.1`, `checksum=13761358.00000`
