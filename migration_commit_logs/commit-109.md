# Commit 109 - Stats Detail Legacy 포맷 호출 경량화

## 커밋 요약
- `stats_detail_panel_legacy`의 정적 포맷 호출을 `Locale.trf1/trf2/trf4`로 전환.

## 상세 변경
- `scripts/ui/panels/stats_detail_panel_legacy.gd`
  - `UI_STAT_CURRENT_FMT`(4 params) `trf` → `trf4`
  - `UI_STAT_GENDER_FMT`(2 params) `trf` → `trf2`
  - `UI_STAT_COUPLES_FMT`(2 params) `trf` → `trf2`
  - `UI_STAT_POP_FMT`(1 param) `trf` → `trf1`
  - `UI_TECH_COUNT_FMT`(2 params) `trf` → `trf2`

## 기능 영향
- Legacy stats 패널 텍스트 출력은 기존과 동일.
- draw 경로의 정적 포맷 호출에서 임시 params Dictionary 생성을 줄여 렌더 미세 오버헤드를 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=466.1`, `checksum=13761358.00000`
