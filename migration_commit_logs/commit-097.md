# Commit 097 - World Stats Population 탭 포맷 재사용 최적화

## 커밋 요약
- `world_stats_population_tab`에서 반복 생성되던 인구 포맷 문자열을 재사용하고, 단순 포맷 호출을 `trf1/trf2`로 전환.

## 상세 변경
- `scripts/ui/panels/world_stats_tabs/world_stats_population_tab.gd`
  - 전체 인구 라인:
    - `UI_STAT_POP_FMT` 결과를 `total_pop_text`로 1회 생성 후 본문/폭 계산에 재사용.
  - 성비 라인:
    - `UI_STAT_GENDER_FMT` 호출을 `Locale.trf2`로 전환.
  - 정착지 행 렌더:
    - `UI_STAT_POP_FMT`를 `s_pop_text`로 1회 생성.
    - 행 본문/행 너비/trend x 오프셋 계산에서 같은 `row_prefix` 문자열을 재사용.

## 기능 영향
- Population 탭 표시 내용과 정렬/클릭 동작은 기존과 동일.
- draw 루프에서 동일 locale 포맷/문자열 조립 중복을 줄여 렌더 경로 미세 오버헤드를 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=458.0`, `checksum=13761358.00000`
