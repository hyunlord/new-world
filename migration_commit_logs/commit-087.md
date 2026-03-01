# Commit 087 - list panel draw 루프 번역 호출 축소

## 커밋 요약
- `ListPanel`의 entity/building draw 루프에서 반복 번역 호출을 줄이도록 row 생성/함수 스코프 캐시로 이관.

## 상세 변경
- `scripts/ui/panels/list_panel.gd`
  - entity row 생성 시 `job_display`를 미리 계산해 저장.
  - deceased row도 `job_display`를 미리 계산해 저장.
  - draw 루프의 job 렌더링은 `job_display`를 사용하도록 변경.
  - building list 렌더링에 `building_type_cache` 추가:
    - `BUILDING_TYPE_*` 번역 문자열을 타입별 1회 계산 후 재사용.
  - building status의 built 라벨(`UI_BUILT_LABEL`)을 루프 밖에서 1회 조회 후 재사용.

## 기능 영향
- 목록 표시에 보이는 문자열 결과는 기존과 동일.
- `_draw()` 반복 호출 경로에서 `Locale` 조회 횟수를 줄여 UI 패널 렌더 오버헤드를 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=474.3`, `checksum=13761358.00000`
