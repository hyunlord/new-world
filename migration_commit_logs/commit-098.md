# Commit 098 - Settlement Overview 포맷 경량 호출 적용

## 커밋 요약
- `settlement_overview_tab`의 1-파라미터 포맷 호출 2곳을 `Locale.trf1`로 전환.

## 상세 변경
- `scripts/ui/panels/settlement_tabs/settlement_overview_tab.gd`
  - 리더 라벨 charisma 포맷:
    - `Locale.trf("UI_CHARISMA_FMT", {"value": charisma_fmt})`
    - → `Locale.trf1("UI_CHARISMA_FMT", "value", charisma_fmt)`
  - 총 인구 라벨:
    - `Locale.trf("UI_TOTAL_POP_FMT", {"n": population})`
    - → `Locale.trf1("UI_TOTAL_POP_FMT", "n", population)`

## 기능 영향
- Overview 탭 표시 문자열은 기존과 동일.
- draw 경로의 단순 포맷 호출에서 임시 params Dictionary 생성을 줄여 미세 오버헤드를 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=461.6`, `checksum=13761358.00000`
