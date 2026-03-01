# Commit 101 - World Stats 패널 정착지 수 포맷 재사용

## 커밋 요약
- `world_stats_panel`에서 헤더/푸터의 정착지 수 포맷 문자열을 1회 생성 후 재사용하도록 변경.

## 상세 변경
- `scripts/ui/panels/world_stats_panel.gd`
  - `settlement_count_text` 추가:
    - `Locale.trf1("UI_SETTLEMENT_COUNT_FMT", "n", settlement_count)`를 1회 생성.
  - 헤더 표시:
    - 기존 `Locale.trf(...)` 직접 호출 대신 `settlement_count_text` 사용.
  - 푸터 표시:
    - 별도 `Locale.trf(...)` 호출 제거, 같은 `settlement_count_text` 재사용.

## 기능 영향
- 헤더/푸터의 정착지 수 텍스트는 기존과 동일.
- draw 루프에서 동일 포맷 문자열 중복 계산과 임시 params Dictionary 생성을 줄여 UI 오버헤드를 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=480.9`, `checksum=13761358.00000`
