# Commit 207 - sim-systems A* 경로탐색 배열 기반 코어로 전환

## 커밋 요약
- `sim-systems::find_path`의 내부 상태 구조를 `HashMap/HashSet` 기반에서 인덱스 배열(`Vec`) 기반으로 전환해 hot path 오버헤드를 크게 절감.

## 상세 변경
- `rust/crates/sim-systems/src/pathfinding.rs`
  - `find_path` 내부 자료구조 전환:
    - `open_set: Vec<usize>` (노드 인덱스)
    - `in_open: Vec<bool>`
    - `closed_set: Vec<bool>`
    - `came_from: Vec<Option<usize>>`
    - `g_score/f_score: Vec<f32>`
  - neighbor 순회 시 `GridPos` 해시 키 대신 인덱스 직접 접근으로 변경.
  - path 재구성 로직을 `reconstruct_path(came_from, current_idx, width)`로 교체.
  - 시작/목표 좌표 in-bounds 검증 경로를 명시적으로 추가.
  - 신규 테스트 `returns_empty_when_start_is_out_of_bounds` 추가.

## 기능 영향
- 기존 path 의미/체크섬은 유지.
- 경로탐색 핵심 루프의 해시 조회/할당 비용이 제거되어 성능이 크게 개선됨.

## 검증
- `cd rust && cargo test -p sim-systems --release` 통과 (100 tests).
- `cd rust && cargo test -p sim-bridge --release` 통과 (15 tests).
- `tools/migration_verify.sh --with-benches` 통과.
  - `pathfind-bridge checksum=70800.00000`(@100) 유지
  - `stress checksum=24032652.00000`(@10k) 유지
  - `needs checksum=38457848.00000`(@10k) 유지
- 동일 조건 pathfind 벤치 성능 관측:
  - 이전: `ns_per_iter ≈ 20147583.8`
  - 이후: `ns_per_iter ≈ 2907532.5`
