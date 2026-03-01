# Commit 170 - pathfinder batch packed 버퍼 재사용 최적화

## 커밋 요약
- `pathfinder.gd`의 배치 Rust 경로탐색에서 호출마다 새로 생성하던 PackedArray를 재사용 버퍼로 전환해 할당/GC 오버헤드를 줄임.

## 상세 변경
- `scripts/core/world/pathfinder.gd`
  - 재사용 scratch 버퍼 필드 추가:
    - `_batch_from_xy`, `_batch_to_xy` (`PackedInt32Array`)
    - `_batch_from_points`, `_batch_to_points` (`PackedVector2Array`)
  - `_find_paths_rust_batch(...)` 변경:
    - XY batch 경로에서 지역 배열 생성 대신 scratch 배열 `resize + index write` 사용.
    - Vector2 batch 경로에서도 `push_back` 신규 배열 생성 대신 scratch 배열 재사용.

## 기능 영향
- path 결과/의미는 변경 없음.
- 요청 수가 많은 배치 pathfinding 루프에서 메모리 할당 churn 감소.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 98 tests)
  - localization compile `filled=0`, `updated=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=1413.3`, `checksum=24032652.00000`
- `cd rust && cargo run -q -p sim-test -- --bench-needs-math --iters 10000`
  - `ns_per_iter=320.3`, `checksum=38457848.00000`
