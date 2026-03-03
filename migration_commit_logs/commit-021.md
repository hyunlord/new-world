# Commit 021 - Pathfinding batch에 Int32 packed 좌표 경로 추가

## 커밋 요약
- Pathfinding batch 호출에 `PackedInt32Array(x,y,...)` 기반 API를 추가해 `Vector2` 변환/반올림 오버헤드를 줄임.
- `Pathfinder`는 새 API를 우선 사용하고, 미지원 시 기존 `PackedVector2Array` 경로로 fallback.

## 상세 변경
- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 내부 헬퍼:
    - `decode_xy_pairs(PackedInt32Array) -> Option<Vec<(i32, i32)>>`
    - `encode_path_groups_xy(Vec<Vec<GridPos>>) -> Array<PackedInt32Array>`
  - 신규 Godot 함수:
    - `pathfind_grid_batch_xy(...) -> Array<PackedInt32Array>`
    - `pathfind_grid_gpu_batch_xy(...) -> Array<PackedInt32Array>`
      - 현재 GPU 미구현으로 CPU fallback
- `scripts/core/simulation/sim_bridge.gd`
  - native 메서드 후보군에 `pathfind_grid_batch_xy`/`pathfind_grid_gpu_batch_xy` 추가.
  - 신규 프록시 함수 `pathfind_grid_batch_xy(...)` 추가.
- `scripts/core/world/pathfinder.gd`
  - batch 호출 시 `pathfind_grid_batch_xy` 지원 여부를 우선 확인.
  - 지원 시 `PackedInt32Array`로 좌표를 전달하고 결과도 int-packed 형식으로 파싱.
  - 미지원 시 기존 `pathfind_grid_batch` (PackedVector2Array) 경로 사용.

## 기능 영향
- batch pathfinding 호출의 좌표 직렬화 비용 감소.
- 기존 반환 타입(`Array` of path arrays with `Vector2i`) 유지로 상위 시스템 영향 최소화.
- GPU API 명칭도 동일 int-packed 형태로 선반영되어 향후 GPU 구현 연결 용이.

## 검증
- `cd rust && cargo fmt -p sim-bridge` 통과
- `cd rust && cargo test -q -p sim-bridge` 통과
- `cd rust && cargo test -q` 통과
- `tools/migration_verify.sh` 통과
