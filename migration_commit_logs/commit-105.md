# Commit 105 - Locale `trf3`/`trf4` 추가 및 draw 경로 적용

## 커밋 요약
- `Locale`에 3/4 placeholder 경량 포맷 API를 추가하고, draw 루프 2곳에 적용.

## 상세 변경
- `scripts/core/simulation/locale.gd`
  - `trf3(...)` 추가: 3 placeholder 치환 전용 경량 경로.
  - `trf4(...)` 추가: 4 placeholder 치환 전용 경량 경로.
  - 기존 `trf1/trf2`와 동일하게 `key_id` 캐시 + fallback(`ltr`) 의미 유지.
- `scripts/ui/panels/world_stats_tabs/world_stats_population_tab.gd`
  - `UI_STAT_CURRENT_FMT` 호출을 `Locale.trf4`로 전환.
- `scripts/ui/panels/settlement_tabs/settlement_overview_tab.gd`
  - `UI_POP_SUMMARY_FMT` 호출을 `Locale.trf3`로 전환.

## 기능 영향
- Population/Settlement Overview 탭 표시 문자열은 기존과 동일.
- draw 경로의 3~4 placeholder 포맷 호출에서 임시 params Dictionary 생성을 줄여 렌더 오버헤드를 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=469.3`, `checksum=13761358.00000`
