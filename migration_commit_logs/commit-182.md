# Commit 182 - pathfinder 내부 rust 결과 래퍼 딕셔너리 제거

## 커밋 요약
- pathfinder 내부 Rust 경로 반환값을 `{used, path(s)}` 딕셔너리에서 `null/Array` 패턴으로 전환해 hot path 딕셔너리 할당을 줄임.

## 상세 변경
- `scripts/core/world/pathfinder.gd`
  - `find_path(...)`
    - `_find_path_rust(...)` 결과를 `Variant`로 받아 `null` 여부로 분기.
  - `find_paths_batch(...)`, `find_paths_batch_xy(...)`
    - `_find_paths_rust_batch*` 결과를 `Variant`로 받아 `null` 여부로 분기.
  - `_find_path_rust(...) -> Variant`
    - 기존 `{"used": bool, "path": Array}` 반환 제거.
    - 성공 시 정규화된 path Array 직접 반환, 실패 시 `null` 반환.
  - `_find_paths_rust_batch(...) -> Variant`
    - 기존 `{"used": bool, "paths": Array}` 반환 제거.
    - 성공 시 paths Array 직접 반환, 실패 시 `null` 반환.
  - `_find_paths_rust_batch_xy(...) -> Variant`
    - 동일하게 성공 Array / 실패 `null` 패턴으로 정리.

## 기능 영향
- pathfinding 성공/실패 의미와 fallback 동작은 동일.
- 내부 호출당 딕셔너리 생성/키 조회를 제거해 pathfinder hot path 메모리 churn 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 98 tests)
  - localization compile `filled=0`, `updated=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -p sim-test --release -- --bench-stress-math --iters 10000`
  - `ns_per_iter=408.1`, `checksum=24032652.00000`
- `cd rust && cargo run -p sim-test --release -- --bench-needs-math --iters 10000`
  - `ns_per_iter=147.6`, `checksum=38457848.00000`
