# Commit 100 - World Stats Tech 탭 2-파라미터 포맷 경량화

## 커밋 요약
- `world_stats_tech_tab`의 기술 카운트 포맷 호출을 `Locale.trf2`로 전환.

## 상세 변경
- `scripts/ui/panels/world_stats_tabs/world_stats_tech_tab.gd`
  - `UI_TECH_COUNT_FMT` 호출:
    - `Locale.trf("UI_TECH_COUNT_FMT", {"known": known_count, "forgotten": forgotten_count})`
    - → `Locale.trf2("UI_TECH_COUNT_FMT", "known", known_count, "forgotten", forgotten_count)`

## 기능 영향
- 기술 요약 텍스트 출력은 기존과 동일.
- draw 경로의 2-파라미터 포맷 호출에서 임시 params Dictionary 생성을 줄여 미세 오버헤드를 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=451.7`, `checksum=13761358.00000`
