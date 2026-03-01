# Commit 092 - 건물 ID 조회 O(1) 경로 추가

## 커밋 요약
- `BuildingManager`에 건물 ID 직접 조회 API를 추가하고 `HUD`가 선형 탐색 대신 해당 경로를 사용하도록 전환.

## 상세 변경
- `scripts/core/settlement/building_manager.gd`
  - `get_building(id: int) -> RefCounted` 추가:
    - 내부 `_buildings` dictionary에서 ID로 즉시 조회.
- `scripts/ui/hud.gd`
  - `_get_building_by_id(bid)` 변경:
    - 기존 `get_all_buildings()` 순회 기반 검색 제거.
    - `BuildingManager.get_building(bid)` 직접 호출로 치환.

## 기능 영향
- 선택 건물 패널 표시 동작은 기존과 동일.
- 선택된 건물 갱신 시 전체 건물 순회를 제거해 HUD 건물 상세 갱신 경로의 탐색 비용을 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=452.3`, `checksum=13761358.00000`
