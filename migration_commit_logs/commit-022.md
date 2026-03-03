# Commit 022 - Single pathfinding도 Int32 packed 좌표 경로 추가

## 커밋 요약
- 단일 pathfinding에도 `PackedInt32Array(x,y,...)` 경로를 추가해 single/batch 모두 int-packed 직렬화로 통일.
- `Pathfinder`는 single 경로도 XY packed API를 우선 시도하고, 실패 시 기존 vec2 경로로 fallback.

## 상세 변경
- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 헬퍼 `encode_path_xy(Vec<GridPos>) -> PackedInt32Array`
  - 신규 Godot 함수:
    - `pathfind_grid_xy(...) -> PackedInt32Array`
    - `pathfind_grid_gpu_xy(...) -> PackedInt32Array` (GPU 미구현, CPU fallback)
- `scripts/core/simulation/sim_bridge.gd`
  - 후보군 추가:
    - `_PATHFIND_XY_METHOD_CANDIDATES`
    - `_PATHFIND_XY_GPU_METHOD_CANDIDATES`
  - 신규 프록시 함수 `pathfind_grid_xy(...)` 추가.
- `scripts/core/world/pathfinder.gd`
  - single path: `pathfind_grid_xy` 우선 호출, null 시 기존 `pathfind_grid` fallback.
  - batch path: xy 우선 시도 후 null일 경우 vec2 fallback하도록 보강(브리지 래퍼 환경 호환).
  - int-packed 결과 파싱 로직 재사용.

## 기능 영향
- single/batch 모두 좌표 직렬화 오버헤드 감소 경로 확보.
- 기존 상위 반환 타입(`Array[Vector2i]`) 유지.
- 래퍼가 xy 메서드를 노출하지만 네이티브가 미지원인 경우에도 vec2 경로로 안전 fallback.

## 검증
- `cd rust && cargo fmt -p sim-bridge` 통과
- `cd rust && cargo test -q -p sim-bridge` 통과
- `cd rust && cargo test -q` 통과
- `tools/migration_verify.sh` 통과
