# Commit 179 - path 결과 정규화 resize/index write 전환

## 커밋 요약
- pathfinder 결과 정규화에서 `append` 기반 확장을 `resize + index write` 방식으로 전환해 경로 배열 생성 오버헤드를 줄임.

## 상세 변경
- `scripts/core/world/pathfinder.gd`
  - `_normalize_path_xy_result(...)`
    - `PackedInt32Array` 입력 경로: pair 수만큼 `path.resize(...)` 후 인덱스 직접 기록.
    - `Array` 입력 경로: 최대 크기로 선할당 후 `write_idx`로 유효 항목만 채우고 마지막에 shrink.
  - `_normalize_path_result(...)`
    - `PackedVector2Array` 입력 경로: `path.resize(...)` 후 인덱스 직접 기록.
    - `Array` 입력 경로: `write_idx` 기반 compaction으로 `Vector2i/Vector2/Dictionary{x,y}` 항목만 반영.

## 기능 영향
- path 좌표 의미/순서는 동일.
- batch/single path 정규화 시 동적 `append` 증가를 줄여 pathfinder hot path 배열 생성 비용을 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 98 tests)
  - localization compile `filled=0`, `updated=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -p sim-test --release -- --bench-stress-math --iters 10000`
  - `ns_per_iter=392.8`, `checksum=24032652.00000`
- `cd rust && cargo run -p sim-test --release -- --bench-needs-math --iters 10000`
  - `ns_per_iter=141.1`, `checksum=38457848.00000`
