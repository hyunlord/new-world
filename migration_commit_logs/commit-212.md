# Commit 212 - sim-bridge out-of-bounds 시작점 동작 테스트 고정

## 커밋 요약
- pathfinding 시작점이 맵 범위를 벗어나는 경우 빈 경로를 반환하는 동작을 `sim-bridge` 테스트에 명시적으로 고정.

## 상세 변경
- `rust/crates/sim-bridge/src/lib.rs` 테스트 추가:
  - `pathfind_grid_returns_empty_when_start_is_out_of_bounds`
    - 단건 API에서 시작점 OOB 시 빈 경로 반환 검증.
  - `pathfind_grid_batch_returns_empty_for_out_of_bounds_start`
    - batch(tuple/xy)에서 OOB 시작 질의는 빈 경로, 정상 질의는 유효 경로 유지 검증.

## 기능 영향
- 런타임 로직 변화 없음.
- 최근 pathfinding 코어 최적화 이후 경계 의미가 테스트로 명시되어 회귀 탐지력 향상.

## 검증
- `cd rust && cargo test -p sim-bridge --release` 통과 (17 tests).
- `tools/migration_verify.sh --with-benches` 통과.
  - `pathfind-bridge checksum=70800.00000`(@100)
  - `stress checksum=24032652.00000`(@10k)
  - `needs checksum=38457848.00000`(@10k)
