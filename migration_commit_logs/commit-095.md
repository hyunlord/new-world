# Commit 095 - ChroniclePanel 포맷 경량화 적용

## 커밋 요약
- `ChroniclePanel` draw 경로의 고정 포맷 호출 2개를 `Locale.trf1/trf2`로 전환.

## 상세 변경
- `scripts/ui/panels/chronicle_panel.gd`
  - 이벤트 개수 라벨:
    - `Locale.trf("UI_EVENTS_COUNT", {"n": events_count})`
    - → `Locale.trf1("UI_EVENTS_COUNT", "n", events_count)`
  - 짧은 날짜 fallback:
    - `Locale.trf("UI_SHORT_DATE", {"month": ..., "day": ...})`
    - → `Locale.trf2("UI_SHORT_DATE", "month", ..., "day", ...)`

## 기능 영향
- Chronicle 표시 문자열 의미는 기존과 동일.
- draw 루프의 고정 포맷 호출에서 임시 params Dictionary 생성을 줄여 UI 렌더 미세 오버헤드를 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=459.0`, `checksum=13761358.00000`
