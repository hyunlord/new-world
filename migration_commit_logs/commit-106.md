# Commit 106 - Settlement Tech 탭 포맷 호출 경량화

## 커밋 요약
- `settlement_tech_tab`의 반복 포맷 호출을 `Locale.trf1/trf2/trf3` 경량 경로로 전환.

## 상세 변경
- `scripts/ui/panels/settlement_tabs/settlement_tech_tab.gd`
  - `UI_PRACTITIONERS_FMT`:
    - `trf`(3 params) → `trf3`
  - `UI_NEEDS_MORE_FMT`:
    - `trf`(1 param) → `trf1`
  - `UI_DISCOVERER_FMT` (known/forgotten 경로 2곳):
    - `trf`(2 params) → `trf2`
  - `UI_AND_N_MORE`:
    - `trf`(1 param) → `trf1`
  - `UI_STAT_POP_FMT` (required population line):
    - `trf`(1 param) → `trf1`

## 기능 영향
- Technology 탭의 상태/경고/발견자/요약 텍스트는 기존과 동일.
- draw 경로 반복 포맷 호출에서 임시 params Dictionary 생성을 줄여 렌더 미세 오버헤드를 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=463.2`, `checksum=13761358.00000`
