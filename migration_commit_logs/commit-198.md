# Commit 198 - sim-bridge pathfinding 입력 검증 보강 및 stationary fast-path

## 커밋 요약
- `sim-bridge` pathfinding 입력 검증을 강화해 `width/height <= 0`을 명시적으로 차단하고, 단일 질의의 `from==to`를 그리드 빌드 없이 즉시 반환하도록 개선.

## 상세 변경
- `rust/crates/sim-bridge/src/lib.rs`
  - `validate_grid_inputs(...)` 추가:
    - 차원 유효성(`InvalidDimensions`) 및 버퍼 길이 검증을 공통화.
  - `build_grid_cost_map_unchecked(...)` 분리:
    - 검증 완료 이후 그리드 구성만 수행.
  - `build_grid_cost_map(...)`는 검증 + unchecked 빌드 래퍼로 재구성.
  - `pathfind_grid_bytes(...)`
    - 공통 입력 검증 후 `from==to`면 `vec![from]` 즉시 반환.
    - non-stationary 케이스만 그리드 구성 + A* 수행.

## 테스트 추가
- `pathfind_grid_rejects_invalid_dimensions`
- `pathfind_grid_returns_singleton_for_stationary_query`
- `pathfind_grid_batch_xy_rejects_invalid_dimensions`

## 기능 영향
- 기존 정상 입력 경로 의미는 유지.
- 비정상 차원 입력의 오류 의미를 명확히 고정.
- stationary 단일 질의에서 불필요한 그리드 빌드 비용을 제거.

## 검증
- `cd rust && cargo test -p sim-bridge --release` 통과 (14 tests).
- `tools/migration_verify.sh --with-benches` 통과.
  - `pathfind-bridge checksum=70800.00000`(@100)
  - `stress checksum=24032652.00000`(@10k)
  - `needs checksum=38457848.00000`(@10k)
