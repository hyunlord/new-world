# Commit 102 - Building Detail 포맷 호출 경량화

## 커밋 요약
- `building_detail_panel`의 단일 placeholder 포맷 호출 3곳을 `Locale.trf1`로 전환.

## 상세 변경
- `scripts/ui/panels/building_detail_panel.gd`
  - `UI_STATUS_UNDER_CONSTRUCTION_FMT`:
    - `trf` → `trf1("pct", pct)`
  - `UI_DETAIL_CAPACITY_FMT`:
    - `trf` → `trf1("n", 6)`
  - `UI_DETAIL_EFFECT_RADIUS_FMT`:
    - `trf` → `trf1("n", radius)`

## 기능 영향
- Building Detail 패널의 텍스트 출력은 기존과 동일.
- draw 경로에서 단순 포맷 호출 임시 params Dictionary 생성을 줄여 렌더 미세 오버헤드를 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=451.5`, `checksum=13761358.00000`
