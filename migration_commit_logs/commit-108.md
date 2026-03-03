# Commit 108 - Pause Menu 단일 포맷 경량화

## 커밋 요약
- `pause_menu`의 단일 placeholder 포맷 호출들을 `Locale.trf1`로 전환.

## 상세 변경
- `scripts/ui/panels/pause_menu.gd`
  - `UI_OVERWRITE_CONFIRM` (2곳) `trf` → `trf1`
  - `UI_TIME_AGO_MINUTES` `trf` → `trf1`
  - `UI_TIME_AGO_HOURS` `trf` → `trf1`
  - `UI_TIME_AGO_DAYS` `trf` → `trf1`

## 기능 영향
- 저장 덮어쓰기 확인/시간 경과 표기 텍스트는 기존과 동일.
- 메뉴 갱신 경로에서 단일 placeholder 포맷 호출의 임시 params Dictionary 생성을 줄여 미세 오버헤드를 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=458.0`, `checksum=13761358.00000`
