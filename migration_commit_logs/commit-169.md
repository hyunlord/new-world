# Commit 169 - pathfinding backend 옵션 제어 API 연결

## 커밋 요약
- `ComputeBackend`의 모드(`cpu/gpu_auto/gpu_force`)가 pathfinding bridge backend(`cpu/auto/gpu`)와 즉시 동기화되도록 연결하고, `SimBridge`에 backend 제어/조회 공개 API를 추가.

## 상세 변경
- `scripts/core/simulation/sim_bridge.gd`
  - 공개 API 추가:
    - `set_pathfinding_backend(mode: String) -> bool`
    - `get_pathfinding_backend() -> String`
    - `resolve_pathfinding_backend() -> String`
    - `has_gpu_pathfinding() -> bool`
  - 기존 내부 method-discovery 캐시(`_set/_get/_resolve_pathfind_backend_method_name`)를 재사용해 bridge method 호출.
  - bridge 미존재 시 안전 fallback(`false`, `auto`, `cpu`) 유지.

- `scripts/core/simulation/compute_backend.gd`
  - `_ready()`에서 `_sync_pathfinding_backend_mode()` 호출 추가.
  - `set_mode()`에서 모드 변경 저장 직후 `_sync_pathfinding_backend_mode()` 호출 추가.
  - helper 추가:
    - `_resolve_pathfinding_backend_mode()` (`cpu`, `gpu_force`, `gpu_auto` -> `cpu`, `gpu`, `auto` 매핑)
    - `_sync_pathfinding_backend_mode()` (`SimBridge.set_pathfinding_backend(...)` 호출)

## 기능 영향
- 사용자/설정이 compute 모드를 바꾸면 pathfinding backend 선호 모드도 즉시 bridge에 반영.
- GPU 구현이 없는 환경에서는 기존처럼 CPU fallback 동작 유지.
- 경로 탐색 backend 상태를 스크립트에서 명시적으로 조회/진단 가능.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 98 tests)
  - localization compile `filled=0`, `updated=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=643.2`, `checksum=24032652.00000`
- `cd rust && cargo run -q -p sim-test -- --bench-needs-math --iters 10000`
  - `ns_per_iter=312.3`, `checksum=38457848.00000`
