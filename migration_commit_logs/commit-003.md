# Commit 003 - 경로탐색 배치 API 추가 + MovementSystem 배치 재계산 연동

## 커밋 요약
- Rust bridge에 배치 pathfinding API를 추가.
- GDScript `Pathfinder`에 배치 요청 경로(`find_paths_batch`)를 추가.
- `MovementSystem`이 경로 재계산이 필요한 엔티티를 모아 배치 호출하도록 변경.

## 상세 변경

### 1) Rust bridge 배치 경로탐색 API
- `rust/crates/sim-bridge/src/lib.rs`
  - `PathfindError::MismatchedBatchLength` 추가.
  - `pathfind_grid_batch_bytes(...) -> Result<Vec<Vec<GridPos>>, PathfindError>` 추가.
  - `WorldSimBridge`에 `#[func] pathfind_grid_batch(...) -> Array<PackedVector2Array>` 추가.
  - 배치 테스트 2개 추가:
    - 다중 쿼리 정상 처리
    - 입력 길이 불일치 에러 처리

### 2) SimBridge shim 배치 프록시
- `scripts/core/simulation/sim_bridge.gd`
  - `pathfind_grid_batch(...)` 메서드 추가.
  - 네이티브 bridge 부재 또는 메서드 부재 시 `null` 반환.

### 3) Pathfinder 배치 메서드 추가
- `scripts/core/world/pathfinder.gd`
  - `find_paths_batch(world_data, requests, max_steps)` 추가.
  - Rust batch 메서드(`pathfind_grid_batch`)가 있으면 우선 사용.
  - Rust batch 미지원/미연결 시 기존 `find_path()`를 순차 fallback.

### 4) MovementSystem 재계산 흐름 최적화
- `scripts/systems/world/movement_system.gd`
  - `execute_tick()`를 2단계로 조정:
    1. 이동 대상 중 재계산 필요 엔티티 수집
    2. `find_paths_batch()` 한 번 호출 후 개별 엔티티에 경로 반영
  - `_needs_path_recalc()` 헬퍼 추가.
  - `_apply_recalculated_path()` 헬퍼 추가.
  - `_move_with_pathfinding(entity, tick, allow_recalc)` 시그니처 확장.

## 기능 영향
- 경로 재계산 호출 수가 많은 틱에서 bridge 호출 오버헤드 감소.
- batch API가 없거나 실패해도 기존 개별 경로탐색 fallback으로 동작 유지.

## 검증
- `cargo test -q` 통과 (workspace 전체)
