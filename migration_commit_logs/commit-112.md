# Commit 112 - Calendar/Save Slot 포맷 경량화 확장

## 커밋 요약
- `Locale`에 `trf5`를 추가하고, `GameCalendar`/`PauseMenu`의 고정 placeholder 포맷을 경량 API로 전환.

## 상세 변경
- `scripts/core/simulation/locale.gd`
  - `trf5(key, a,b,c,d,e)` 추가.
  - 기존 `trf1~trf4` 패턴과 동일하게 key-id 캐시 + 순차 replace 경로 사용.

- `scripts/core/simulation/game_calendar.gd`
  - `DATE_FORMAT` → `trf5`
  - `UI_SHORT_DATE` → `trf2`
  - `UI_SHORT_DATE_WITH_YEAR` → `trf3`
  - `UI_FULL_DATETIME` → `trf5`
  - `UI_SHORT_DATETIME` → `trf4`
  - `UI_SHORT_DATETIME_WITH_YEAR` → `trf5`
  - `BIRTH_DATE_FORMAT` → `trf3`
  - `UI_AGE_TOTAL_DAYS_FMT` → `trf1`
  - `UI_AGE_SHORT_FORMAT` → `trf3`

- `scripts/ui/panels/pause_menu.gd`
  - `UI_SLOT_FORMAT` 호출을 `trf` + Dictionary에서 `trf5`로 전환.

## 기능 영향
- 날짜/시간/나이/세이브 슬롯 텍스트 출력 의미는 기존과 동일.
- UI 및 캘린더 포맷 경로에서 임시 params Dictionary 생성 비용을 줄여 미세 오버헤드를 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=453.2`, `checksum=13761358.00000`
