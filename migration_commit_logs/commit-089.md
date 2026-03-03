# Commit 089 - list panel 정적 locale 라벨 캐시

## 커밋 요약
- `ListPanel`에서 탭/헤더/정렬/토글 라벨을 locale 변경 시점에 캐시해 draw 루프의 `Locale.ltr()` 호출을 줄임.

## 상세 변경
- `scripts/ui/panels/list_panel.gd`
  - 캐시 필드 추가:
    - `_cached_tab_labels`
    - `_cached_entity_column_labels`
    - `_cached_building_column_labels`
    - `_cached_deceased_label`
    - `_cached_sort_asc_label`, `_cached_sort_desc_label`
  - `_refresh_locale_cache()` 추가:
    - 위 캐시를 locale 기준으로 갱신.
  - `_ready()`와 `_on_locale_changed()`에서 캐시 갱신 호출.
  - `_get_tab_label()`이 캐시 우선 사용.
  - `_draw()`의 deceased 토글, entity/building 헤더, 정렬 방향 라벨이 캐시 문자열을 사용하도록 전환.

## 기능 영향
- 표시되는 문자열/정렬 표시는 기존과 동일.
- 프레임 기반 draw 루프에서 정적 라벨의 locale lookup 횟수를 줄여 UI 렌더 경로 오버헤드를 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=461.5`, `checksum=13761358.00000`
