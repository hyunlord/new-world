# Commit 192 - sim-test pathfinding bridge 벤치 모드 추가

## 커밋 요약
- `sim-test`에 Rust bridge pathfinding 전용 벤치 모드(`--bench-pathfind-bridge`)를 추가해 경로탐색 batch 성능을 직접 측정할 수 있도록 확장.

## 상세 변경
- `rust/crates/sim-test/Cargo.toml`
  - `sim-bridge` 의존성 추가.
- `rust/crates/sim-test/src/main.rs`
  - CLI 분기에 `--bench-pathfind-bridge` 모드 추가.
  - `run_pathfind_bridge_bench(args)` 추가:
    - 64x64 grid 입력(`walkable`, `move_cost`) 생성.
    - tuple batch API(`pathfind_grid_batch_bytes`)와 packed XY batch API(`pathfind_grid_batch_xy_bytes`)를 동일 입력으로 반복 호출.
    - 결과 path 길이 합을 checksum으로 누적해 회귀 감지 지표 확보.
  - 장거리 탐색이 빈 경로로 끝나지 않도록 `max_steps`를 맵 셀 수(`width*height`)로 상향.
  - 기본 반복 수를 `20,000 -> 1,000`으로 조정해 기본 실행 시간을 실사용 가능한 수준으로 조정.

## 기능 영향
- `sim-test` 실행 경로에 선택적 pathfinding 벤치 모드가 추가됨.
- 기존 기본 실행/needs/stress 벤치 동작에는 영향 없음.

## 검증
- `cd rust && cargo test -p sim-test --release` 통과.
- `tools/migration_verify.sh` 통과.
- `cd rust && cargo run -p sim-test --release -- --bench-pathfind-bridge`
  - `iterations=1000`, `checksum=708000.00000`
- `cd rust && cargo run -p sim-test --release -- --bench-pathfind-bridge --iters 10000`
  - `iterations=10000`, `checksum=7080000.00000`
- `cd rust && cargo run -p sim-test --release -- --bench-stress-math --iters 10000`
  - `checksum=24032652.00000`
- `cd rust && cargo run -p sim-test --release -- --bench-needs-math --iters 10000`
  - `checksum=38457848.00000`
