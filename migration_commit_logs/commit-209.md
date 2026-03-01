# Commit 209 - A* 휴리스틱 좌표 직접 계산 경로 정리

## 커밋 요약
- `find_path`에서 휴리스틱 계산 시 반복 `GridPos` 생성 대신 좌표 직접 계산(`chebyshev_xy`)을 사용하도록 정리하고, 미사용 함수 경고를 제거.

## 상세 변경
- `rust/crates/sim-systems/src/pathfinding.rs`
  - `find_path` 내에서 목표 좌표(`to_x`, `to_y`)를 캐시.
  - `f_score[from]`, neighbor `f_score` 계산을 `chebyshev_xy(...)` 직접 호출로 변경.
  - 미사용 `chebyshev(GridPos, GridPos)` 함수 제거.

## 기능 영향
- path 결과/체크섬 의미는 동일.
- hot loop의 임시 `GridPos` 생성과 호출 오버헤드를 미세 감소.
- `sim-systems` 빌드 경고(`dead_code`) 제거.

## 검증
- `cd rust && cargo test -p sim-systems --release` 통과 (100 tests, 경고 없음).
- `tools/migration_verify.sh --with-benches` 통과.
  - `pathfind-bridge checksum=70800.00000`(@100) 유지.
