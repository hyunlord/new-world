# Commit 074 - game calendar locale key-id 캐시 적용

## 커밋 요약
- `GameCalendar`의 반복 age 라벨 조회(`UI_AGE_*`)를 문자열 key lookup에서 key-id 캐시 경로로 전환.

## 상세 변경
- `scripts/core/simulation/game_calendar.gd`
  - 정적 key id 캐시 추가:
    - `_ui_age_years_id`
    - `_ui_age_months_id`
    - `_ui_age_days_id`
  - 신규 헬퍼 추가:
    - `_age_years_label()`
    - `_age_months_label()`
    - `_age_days_label()`
  - `format_age_detailed`, `format_age_short`에서 `Locale.ltr("UI_AGE_*")` 직접 호출 대신 캐시 헬퍼 사용.
  - 캐시 miss/미지원 시 기존 `Locale.ltr()` fallback 유지.

## 기능 영향
- age 문자열 결과는 기존과 동일.
- 반복 포맷 경로에서 문자열 dictionary lookup 대신 `ltr_id` 배열 조회 경로를 재사용해 UI 호출 오버헤드를 줄임.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=489.9`, `checksum=13761358.00000`
