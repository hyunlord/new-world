# Commit 202 - from_flat_bytes_unchecked 중간 복사 제거

## 커밋 요약
- `GridCostMap::from_flat_bytes_unchecked`가 중간 `Vec<bool>`를 다시 복사하던 경로를 제거하고, 단일 패스로 최종 `GridCostMap`을 직접 구성하도록 최적화.

## 상세 변경
- `rust/crates/sim-systems/src/pathfinding.rs`
  - `from_flat_bytes_unchecked`가 기존 `from_flat_unchecked` 재호출(중간 벡터 재복사) 대신:
    - `walkable` byte -> `Vec<bool>` 변환
    - `move_cost` clamp(`max(0.0)`) 벡터 생성
    - `GridCostMap` 직접 구성
  - 의미는 동일하며 할당/복사 경로만 축소.

## 기능 영향
- pathfinding 결과 의미 변화 없음.
- byte 기반 grid 빌드 경로에서 중간 복사 비용 감소.

## 검증
- `cd rust && cargo test -p sim-systems --release` 통과 (99 tests).
- `tools/migration_verify.sh --with-benches` 통과.
  - `pathfind-bridge checksum=70800.00000`(@100)
  - `stress checksum=24032652.00000`(@10k)
  - `needs checksum=38457848.00000`(@10k)
