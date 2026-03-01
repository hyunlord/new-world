# Commit 190 - sim-bridge Vec2 batch 좌표 변환 할당 제거

## 커밋 요약
- `pathfind_grid_batch`(PackedVector2Array 입력) 경로에서 tuple 벡터 생성 할당을 제거하고 벡터 슬라이스 직접 순회 경로를 추가.

## 상세 변경
- `rust/crates/sim-bridge/src/lib.rs`
  - `pathfind_grid_batch_vec2_bytes(...)` 신규 추가:
    - `from_points`/`to_points` 길이 검증
    - 배치당 grid 1회 생성(`build_grid_cost_map`)
    - `Vector2` 슬라이스를 직접 순회하며 좌표 반올림 후 `find_path` 수행
  - `WorldSimBridge.pathfind_grid_batch(...)`
    - 기존 `from_pairs`/`to_pairs` tuple 벡터 생성 + `pathfind_grid_batch_bytes` 호출 제거
    - `pathfind_grid_batch_vec2_bytes` 직접 호출로 전환

## 기능 영향
- batch path 결과/실패 의미는 동일.
- Vec2 batch 경로의 중간 tuple 변환 할당을 줄여 CPU/메모리 오버헤드 완화.

## 검증
- `cd rust && cargo test -p sim-bridge --release` 통과 (8 tests)
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 98 tests)
  - localization compile `filled=0`, `updated=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -p sim-test --release -- --bench-stress-math --iters 10000`
  - `ns_per_iter=374.9`, `checksum=24032652.00000`
- `cd rust && cargo run -p sim-test --release -- --bench-needs-math --iters 10000`
  - `ns_per_iter=158.3`, `checksum=38457848.00000`
