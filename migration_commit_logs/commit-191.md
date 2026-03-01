# Commit 191 - sim-bridge Vec2 path 인코딩 중간 Vec 제거

## 커밋 요약
- Rust bridge path 결과를 `Vec<Vector2>`로 변환하던 중간 할당을 제거하고 `PackedVector2Array` 직접 인코딩으로 변경.

## 상세 변경
- `rust/crates/sim-bridge/src/lib.rs`
  - `encode_path_vec2(path)` 추가:
    - `Vec<GridPos>`를 `PackedVector2Array`로 직접 `resize + index write` 인코딩.
  - `encode_path_groups_vec2(path_groups)` 추가:
    - 다중 path 그룹을 `PackedVector2Array` 배열로 직접 구성.
  - `pathfind_grid(...)`
    - 기존 `Vec<Vector2>` collect 제거, `encode_path_vec2` 사용.
  - `pathfind_grid_batch(...)`
    - 기존 그룹별 `Vec<Vector2>` collect 제거, `encode_path_groups_vec2` 사용.

## 기능 영향
- path 출력 의미/순서는 동일.
- path 결과 인코딩 단계의 중간 벡터 할당을 제거해 bridge 반환 경로 오버헤드 완화.

## 검증
- `cd rust && cargo test -p sim-bridge --release` 통과 (8 tests)
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 98 tests)
  - localization compile `filled=0`, `updated=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -p sim-test --release -- --bench-stress-math --iters 10000`
  - `ns_per_iter=383.0`, `checksum=24032652.00000`
- `cd rust && cargo run -p sim-test --release -- --bench-needs-math --iters 10000`
  - `ns_per_iter=138.0`, `checksum=38457848.00000`
