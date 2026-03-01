# Commit 186 - world terrain revision 기반 path cache invalidation

## 커밋 요약
- world tile 변경 시 pathfinder의 walkable/move_cost 캐시가 자동 갱신되도록 `terrain_revision` 추적을 추가.

## 상세 변경
- `scripts/core/world/world_data.gd`
  - `terrain_revision: int` 필드 추가.
  - `init_world(...)` 완료 시 revision 증가.
  - `set_tile(...)`에서 기존 값과 비교해 실제 변경이 있을 때만 revision 증가.
- `scripts/core/world/pathfinder.gd`
  - `_cached_world_revision` 캐시 필드 추가.
  - `_ensure_world_cache(...)`가 `world_data.terrain_revision`을 읽어
    revision 변경 시 캐시 재빌드하도록 조건 확장.
  - 캐시 재빌드 시 `_cached_world_revision` 갱신.

## 기능 영향
- path 의미는 동일.
- 동적 지형/타일 변경 이후에도 stale path cache를 재사용하지 않아 경로 정합성이 개선.
- 변경 없는 경우 기존 캐시 재사용 동작은 유지.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 98 tests)
  - localization compile `filled=0`, `updated=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -p sim-test --release -- --bench-stress-math --iters 10000`
  - `ns_per_iter=379.4`, `checksum=24032652.00000`
- `cd rust && cargo run -p sim-test --release -- --bench-needs-math --iters 10000`
  - `ns_per_iter=147.3`, `checksum=38457848.00000`
