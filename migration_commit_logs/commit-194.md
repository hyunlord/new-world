# Commit 194 - sim-bridge pathfinding batch API 동치성 테스트 강화

## 커밋 요약
- `sim-bridge` pathfinding batch API(tuple/packed XY/Vec2) 간 결과 동치성을 테스트로 고정하고 XY 입력 길이 검증 케이스를 추가.

## 상세 변경
- `rust/crates/sim-bridge/src/lib.rs`
  - 테스트 import에 `pathfind_grid_batch_xy_bytes`, `pathfind_grid_batch_vec2_bytes`, `Vector2` 추가.
  - 신규 테스트 추가:
    - `pathfind_grid_batch_xy_matches_tuple_results`
      - 동일 질의 세트에서 tuple batch 결과와 packed XY batch 결과가 완전히 동일함을 검증.
    - `pathfind_grid_batch_vec2_matches_tuple_results`
      - 동일 질의 세트에서 tuple batch 결과와 Vec2 batch 결과가 완전히 동일함을 검증.
    - `pathfind_grid_batch_xy_rejects_odd_length_inputs`
      - XY 배열이 홀수 길이일 때 `MismatchedBatchLength` 오류를 반환하는지 검증.

## 기능 영향
- 런타임 동작 변경 없음.
- 배치 API 변환 경로(최근 최적화 대상)의 회귀 감지력이 향상됨.

## 검증
- `cd rust && cargo test -p sim-bridge --release` 통과 (11 tests).
- `tools/migration_verify.sh --with-benches` 통과.
  - `pathfind-bridge checksum ok: 70800.00000`
  - `stress-math checksum ok: 24032652.00000`
  - `needs-math checksum ok: 38457848.00000`
