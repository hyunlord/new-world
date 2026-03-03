# Commit 215 - PathfindWorkspace 세대 스탬프 리셋 최적화

## 커밋 요약
- `PathfindWorkspace`를 쿼리마다 전체 배열을 `fill` 하던 구조에서 `generation stamp` 기반 상태 추적으로 전환해 반복 경로탐색 리셋 비용을 줄임.

## 상세 변경
- `rust/crates/sim-systems/src/pathfinding.rs`
  - `PathfindWorkspace` 구조 변경:
    - 추가: `query_id`, `seen_gen`, `closed_gen`
    - 변경: `came_from: Vec<Option<usize>> -> Vec<usize>`(`usize::MAX` sentinel)
    - 제거: 쿼리마다 전체 초기화하던 `reset()`
  - `begin_query()` 추가:
    - 쿼리 시작 시 `query_id` 증가
    - `u32::MAX` 도달 시 generation 배열을 0으로 정리 후 `query_id=1`로 래핑
  - A* 본문(`find_path_with_workspace`)을 generation 기반 판별로 전환:
    - 현재 쿼리에서 본 노드만 유효하게 취급
    - closed 판별도 `closed_gen[idx] == query_id`로 처리
    - neighbor relax 조건을 `unseen || better_g`로 변경
  - 경로 재구성 함수(`reconstruct_path`)를 시작 인덱스 기반 체인 복원으로 변경.
  - 테스트 추가: `wraps_workspace_generation_counter_without_state_leak`
    - generation 카운터 래핑 상황에서도 상태 누수 없이 동일 경로가 산출되는지 검증.

## 기능 영향
- batch/반복 pathfinding에서 쿼리당 O(N) 초기화 비용을 제거(세대값 비교로 대체)해 대형 그리드에서 재사용 효율 개선.
- 경로 의미/체크섬은 기존과 동일하게 유지.

## 검증
- `cd rust && cargo test -p sim-systems --release` 통과.
- `cd rust && cargo test -p sim-bridge --release` 통과.
- `tools/migration_verify.sh --with-benches` 통과.
  - pathfind checksum: `70800.00000` (@100)
  - stress checksum: `24032652.00000` (@10000)
  - needs checksum: `38457848.00000` (@10000)
- `cd rust && cargo run -p sim-test --release -- --bench-pathfind-bridge-split` 확인.
  - tuple checksum: `354000.00000` (@1000)
  - xy checksum: `354000.00000` (@1000)
