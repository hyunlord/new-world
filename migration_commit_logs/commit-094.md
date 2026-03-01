# Commit 094 - ListPanel 포맷 호출 경량 경로 적용

## 커밋 요약
- `ListPanel`의 반복 포맷 호출 2곳을 `Locale.trf`에서 `Locale.trf1`로 전환.

## 상세 변경
- `scripts/ui/panels/list_panel.gd`
  - deceased row 상태 문자열 생성:
    - `Locale.trf("UI_DECEASED_STATUS_FMT", {"cause": cause_loc})`
    - → `Locale.trf1("UI_DECEASED_STATUS_FMT", "cause", cause_loc)`
  - 하단 엔티티 수 표시:
    - `Locale.trf("UI_ENTITIES_COUNT_FMT", {"n": rows.size()})`
    - → `Locale.trf1("UI_ENTITIES_COUNT_FMT", "n", rows.size())`

## 기능 영향
- 표시 텍스트는 기존과 동일.
- 리스트 패널 draw 경로의 포맷 호출에서 임시 params Dictionary 생성을 줄여 미세 오버헤드를 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=465.0`, `checksum=13761358.00000`
