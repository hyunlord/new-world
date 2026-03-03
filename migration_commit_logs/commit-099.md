# Commit 099 - Settlement Population 탭 단일 포맷 경량화

## 커밋 요약
- `settlement_population_tab`의 총 인구 표시 포맷을 `Locale.trf1`로 전환.

## 상세 변경
- `scripts/ui/panels/settlement_tabs/settlement_population_tab.gd`
  - `UI_TOTAL_POP_FMT` 호출:
    - `Locale.trf("UI_TOTAL_POP_FMT", {"n": population})`
    - → `Locale.trf1("UI_TOTAL_POP_FMT", "n", population)`

## 기능 영향
- Population 탭 총 인구 텍스트 출력은 기존과 동일.
- draw 경로에서 단순 포맷 호출의 임시 params Dictionary 생성을 줄여 미세 오버헤드를 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=471.3`, `checksum=13761358.00000`
