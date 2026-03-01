# Commit 225 - Dispatch 경유 batch API 공개 및 sim-test backend 계측 출력 연동

## 커밋 요약
- `sim-bridge`에 backend dispatch를 경유하는 공개 batch API를 추가하고, `sim-test` pathfinding 벤치가 해당 경로를 사용하도록 연결.
- 벤치 출력에 backend dispatch 카운터를 함께 표시해 CPU/GPU 경로 사용량을 즉시 관측 가능하게 개선.

## 상세 변경
- `rust/crates/sim-bridge/src/pathfinding_gpu.rs`
  - tuple 배치용 GPU placeholder 함수 추가:
    - `pathfind_grid_batch_tuple_gpu_bytes(...)`
- `rust/crates/sim-bridge/src/lib.rs`
  - tuple 배치 dispatch 함수 추가:
    - `dispatch_pathfind_grid_batch_bytes(...)`
  - 공개 dispatch 배치 API 추가:
    - `pathfind_grid_batch_dispatch_bytes(...)`
    - `pathfind_grid_batch_xy_dispatch_bytes(...)`
  - 공개 계측 API 추가:
    - `pathfind_backend_dispatch_counts()`
    - `reset_pathfind_backend_dispatch_counts()`
- `rust/crates/sim-test/src/main.rs`
  - pathfinding 벤치가 기존 직접 batch API 대신 dispatch 공개 API를 사용하도록 변경.
  - 벤치 시작 전 카운터 리셋, 종료 후 `cpu/gpu/total dispatch` 출력 추가.

## 기능 영향
- headless 벤치에서도 실제 backend dispatch 계층을 통과하게 되어, 운영 환경과 더 가까운 경로를 측정 가능.
- checksum 호환성은 유지하면서 backend 사용 분포를 숫자로 확인 가능.
  - 예: `pathfind-bridge @100`에서 `cpu=200 gpu=0 total=200` 출력.

## 검증
- `cd rust && cargo test -p sim-bridge --release` 통과.
- `cd rust && cargo run -q -p sim-test --release -- --bench-pathfind-bridge --iters 10` 확인.
  - checksum `7080.00000`
  - backend dispatches `cpu=20 gpu=0 total=20`
- `tools/migration_verify.sh --with-benches` 통과.
  - pathfind checksum: `70800.00000` (@100)
  - stress checksum: `24032652.00000` (@10000)
  - needs checksum: `38457848.00000` (@10000)
