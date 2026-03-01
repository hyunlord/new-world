# Commit 189 - sim-bridge XY batch 좌표 디코딩 할당 제거

## 커밋 요약
- `pathfind_grid_batch_xy` 경로에서 packed XY 좌표를 tuple 벡터로 복사하던 단계(추가 할당)를 제거하고, 슬라이스 직접 순회 경로를 추가.

## 상세 변경
- `rust/crates/sim-bridge/src/lib.rs`
  - `pathfind_grid_batch_xy_bytes(...)` 신규 추가:
    - `from_xy`/`to_xy` 슬라이스 길이 검증(동일 길이 + 짝수 길이)
    - 배치당 grid 1회 생성(`build_grid_cost_map`)
    - `[x,y,x,y,...]` 슬라이스를 2칸씩 직접 순회하며 `find_path` 수행
  - `WorldSimBridge.pathfind_grid_batch_xy(...)`
    - 기존 `decode_xy_pairs` + `pathfind_grid_batch_bytes` 경로 제거
    - `pathfind_grid_batch_xy_bytes` 직접 호출로 전환
  - 불필요해진 `decode_xy_pairs(...)` 제거

## 기능 영향
- path 결과/실패 처리 의미는 동일.
- XY batch 호출 시 중간 tuple 벡터 2개(`from_pairs`, `to_pairs`) 생성이 사라져 메모리/CPU 오버헤드 완화.

## 검증
- `cd rust && cargo test -p sim-bridge --release` 통과 (8 tests)
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 98 tests)
  - localization compile `filled=0`, `updated=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -p sim-test --release -- --bench-stress-math --iters 10000`
  - `ns_per_iter=393.2`, `checksum=24032652.00000`
- `cd rust && cargo run -p sim-test --release -- --bench-needs-math --iters 10000`
  - `ns_per_iter=155.8`, `checksum=38457848.00000`
