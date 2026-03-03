# Commit 199 - sim-bridge batch pathfinding stationary fast-path 확장

## 커밋 요약
- `sim-bridge` batch pathfinding API들(tuple/XY/Vec2)에 stationary(`from==to`) fast-path를 추가해 불필요한 A* 호출과 일부 케이스의 그리드 빌드를 줄임.

## 상세 변경
- `rust/crates/sim-bridge/src/lib.rs`
  - `pathfind_grid_batch_bytes`
    - 전체 질의가 stationary면 입력 검증 후 singleton path 묶음을 즉시 반환(그리드 빌드 스킵).
    - 혼합 질의에서도 stationary 항목은 `find_path` 호출 없이 singleton 반환.
  - `pathfind_grid_batch_xy_bytes`
    - packed XY 전체 stationary 감지 경로 추가(검증 후 즉시 반환).
    - 혼합 질의 stationary 항목 short-circuit 추가.
  - `pathfind_grid_batch_vec2_bytes`
    - rounded 좌표 기준 전체 stationary 감지/즉시 반환 추가.
    - 혼합 질의 stationary 항목 short-circuit 추가.

## 테스트 추가
- `pathfind_grid_batch_returns_singletons_for_stationary_queries`
  - tuple/XY/Vec2 세 경로 모두 stationary 배치에서 singleton path를 반환하는지 검증.
  - walkable=0(막힌 타일)에서도 기존 `find_path` 의미와 동일하게 singleton 반환을 확인.

## 기능 영향
- path 의미는 유지.
- stationary 배치 질의 비중이 높을수록 batch 경로탐색 CPU 오버헤드 완화.

## 검증
- `cd rust && cargo test -p sim-bridge --release` 통과 (15 tests).
- `tools/migration_verify.sh --with-benches` 통과.
  - `pathfind-bridge checksum=70800.00000`(@100)
  - `stress checksum=24032652.00000`(@10k)
  - `needs checksum=38457848.00000`(@10k)
