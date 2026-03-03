# Commit 184 - sim_bridge GPU capability probe 캐시

## 커밋 요약
- pathfinding GPU capability 조회를 캐시해 `_prefer_gpu`/`has_gpu_pathfinding`에서 반복 probe 비용을 줄임.

## 상세 변경
- `scripts/core/simulation/sim_bridge.gd`
  - 캐시 필드 추가:
    - `_gpu_pathfinding_capability_cached`
    - `_gpu_pathfinding_capability`
  - `has_gpu_pathfinding()`이 direct method check/call 대신 `_resolve_gpu_pathfinding_capability(bridge)`를 사용.
  - `_prefer_gpu()` fallback 최종 판정도 동일 capability 캐시 경로를 사용.
  - native bridge 인스턴스 결정 시(`_get_native_bridge`) GPU capability 캐시를 초기화.
  - `_resolve_gpu_pathfinding_capability(bridge)` 헬퍼 추가:
    - 캐시 hit 시 즉시 반환
    - miss 시 `has_gpu_pathfinding` 메서드 또는 GPU 메서드 존재 여부 probe 후 캐시 저장

## 기능 영향
- GPU 사용 가능 여부 판정 의미는 동일.
- pathfinding 호출이 잦은 구간에서 capability probe 반복 비용을 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 98 tests)
  - localization compile `filled=0`, `updated=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -p sim-test --release -- --bench-stress-math --iters 10000`
  - `ns_per_iter=405.2`, `checksum=24032652.00000`
- `cd rust && cargo run -p sim-test --release -- --bench-needs-math --iters 10000`
  - `ns_per_iter=151.0`, `checksum=38457848.00000`
