# Commit 203 - sim-test pathfinding bridge split 벤치 모드 추가

## 커밋 요약
- `sim-test`에 pathfinding bridge API별 성능을 분리 관측하는 `--bench-pathfind-bridge-split` 모드를 추가.

## 상세 변경
- `rust/crates/sim-test/src/main.rs`
  - CLI 분기에 `--bench-pathfind-bridge-split` 추가.
  - 공통 입력 생성 헬퍼 `pathfind_bench_inputs()` 추가.
  - 기존 `run_pathfind_bridge_bench()`가 공통 입력 헬퍼를 재사용하도록 정리.
  - 신규 `run_pathfind_bridge_split_bench()` 추가:
    - tuple batch API(`pathfind_grid_batch_bytes`) 측정/출력
    - packed XY batch API(`pathfind_grid_batch_xy_bytes`) 측정/출력
    - 각 API별 `elapsed_ms`, `ns_per_iter`, `checksum` 독립 출력

## 기능 영향
- 기존 `--bench-pathfind-bridge` 동작/체크섬은 유지.
- bridge 내부 최적화 후 tuple vs XY 경로의 상대 성능/회귀를 별도 추적 가능.

## 검증
- `cd rust && cargo test -p sim-test --release` 통과.
- `cd rust && cargo run -p sim-test --release -- --bench-pathfind-bridge-split` 실행 확인.
  - tuple checksum=`354000.00000` (@1000)
  - xy checksum=`354000.00000` (@1000)
- `tools/migration_verify.sh --with-benches` 통과.
  - 기존 `pathfind-bridge checksum=70800.00000`(@100) 유지.
