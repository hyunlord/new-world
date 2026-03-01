# Commit 103 - Building Detail 건물 조회 O(1) 전환

## 커밋 요약
- `building_detail_panel`의 건물 ID 조회를 전체 배열 선형 탐색에서 direct lookup으로 전환.

## 상세 변경
- `scripts/ui/panels/building_detail_panel.gd`
  - `_draw()` 초반 조회 경로:
    - 기존 `get_all_buildings()` + loop 기반 ID 탐색 제거.
    - `BuildingManager.get_building(_building_id)` 직접 호출로 치환.

## 기능 영향
- Building Detail 패널 표시 동작은 기존과 동일.
- 패널 draw 경로에서 프레임당 건물 전체 순회를 제거해 조회 비용을 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=466.9`, `checksum=13761358.00000`
