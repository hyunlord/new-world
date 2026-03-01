# Commit 174 - pathfinder bridge 메서드 capability 캐시

## 커밋 요약
- pathfinder hot path에서 반복 호출되던 bridge `has_method` 확인을 1회 캐시로 전환해 분기 오버헤드를 줄임.

## 상세 변경
- `scripts/core/world/pathfinder.gd`
  - bridge capability 캐시 필드 추가:
    - `_bridge_methods_cached`
    - `_bridge_has_pathfind`
    - `_bridge_has_pathfind_xy`
    - `_bridge_has_batch`
    - `_bridge_has_batch_xy`
  - `_find_path_rust`, `_find_paths_rust_batch`, `_find_paths_rust_batch_xy`에서
    - 매 호출 `has_method(...)` 대신 캐시된 capability 사용.
  - `_get_rust_bridge()`에서 bridge 객체 획득 시 capability 캐시를 초기화.
  - `_ensure_bridge_method_cache(bridge)` 헬퍼 추가로 메서드 탐색을 단일 지점으로 통합.

## 기능 영향
- pathfinding 결과 및 fallback 동작은 동일.
- pathfinder 호출 빈도가 높은 tick에서 메서드 존재 확인 비용을 줄여 미세 성능 개선.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 98 tests)
  - localization compile `filled=0`, `updated=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -p sim-test --release -- --bench-stress-math --iters 10000`
  - `ns_per_iter=397.5`, `checksum=24032652.00000`
- `cd rust && cargo run -p sim-test --release -- --bench-needs-math --iters 10000`
  - `ns_per_iter=150.2`, `checksum=38457848.00000`
