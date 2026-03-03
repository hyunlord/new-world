# Commit 171 - movement batch pathfinding packed XY 경로 전환

## 커밋 요약
- movement 시스템의 경로 재계산 요청을 Dictionary 배열에서 packed XY 배열로 전환해 배치 pathfinding 요청 구성 비용을 줄임.

## 상세 변경
- `scripts/core/world/pathfinder.gd`
  - 신규 API 추가: `find_paths_batch_xy(world_data, from_xy, to_xy, max_steps)`
    - 입력: `[x0,y0,x1,y1,...]` 형식의 packed 좌표 배열.
    - Rust 배치 경로(우선) + 기존 GDScript fallback 제공.
  - 신규 내부 함수 추가: `_find_paths_rust_batch_xy(...)`
    - batch-xy bridge를 우선 호출.
    - 필요 시 기존 vec2 batch bridge로 자동 fallback.

- `scripts/systems/world/movement_system.gd`
  - scratch 버퍼 추가:
    - `_recalc_from_xy`, `_recalc_to_xy`
  - tick 루프에서 재계산 요청을 Dictionary 대신 packed XY로 누적.
  - 경로 재계산 호출을 `find_paths_batch(...)` -> `find_paths_batch_xy(...)`로 전환.

## 기능 영향
- path 결과 의미는 유지.
- movement tick에서 batch path request 구성 시 Dictionary 생성/해시 비용 감소.
- Rust batch-xy 경로를 더 직접 사용해 대규모 이동 시 효율 개선.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 98 tests)
  - localization compile `filled=0`, `updated=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=589.1`, `checksum=24032652.00000`
- `cd rust && cargo run -q -p sim-test -- --bench-needs-math --iters 10000`
  - `ns_per_iter=232.9`, `checksum=38457848.00000`
