# Commit 213 - batch pathfinding에 재사용 워크스페이스 도입

## 커밋 요약
- `sim-systems`에 `PathfindWorkspace`를 도입하고 `sim-bridge` batch pathfinding이 이를 재사용하도록 연결해 배치 질의에서 반복 scratch 할당을 제거.

## 상세 변경
- `rust/crates/sim-systems/src/pathfinding.rs`
  - 신규 `PathfindWorkspace` 추가:
    - `open_set`, `came_from`, `g_score`, `f_score`, `closed_set` 재사용 버퍼 보유.
    - `ensure_node_count`/`reset`로 grid 크기 전환 및 반복 호출 초기화 지원.
  - 신규 API `find_path_with_workspace(...)` 추가.
  - 기존 `find_path(...)`는 내부에서 workspace를 만들어 `find_path_with_workspace`를 호출하는 thin wrapper로 전환.
  - 테스트 `reuses_workspace_without_state_leak` 추가.
- `rust/crates/sim-bridge/src/lib.rs`
  - `pathfind_grid_batch_bytes/xy/vec2_bytes`에서 batch당 `PathfindWorkspace` 1회 생성 후 각 질의에서 재사용.

## 기능 영향
- path 의미/체크섬 유지.
- batch 질의에서 쿼리별 scratch 벡터 재할당이 제거되어 메모리 churn 감소.

## 검증
- `cd rust && cargo test -p sim-systems --release` 통과 (102 tests).
- `cd rust && cargo test -p sim-bridge --release` 통과 (17 tests).
- `tools/migration_verify.sh --with-benches` 통과.
  - `pathfind-bridge checksum=70800.00000`(@100) 유지.
