# 0153 - sim-bridge pathfinding core module split

## Commit
- `[rust-r0-253] Extract pathfinding core block from sim-bridge lib`

## 변경 파일
- `rust/crates/sim-bridge/src/lib.rs`
  - pathfinding 코어 블록(입력/검증/배치/디스패치)을 제거하고 `pathfinding_core` 모듈을 re-export로 연결.
  - 기존 호출 지점은 동일 함수명 유지(`dispatch_*`, `pathfind_*`).
  - 테스트 전용 import(`parse_pathfind_backend`)를 `#[cfg(test)]`로 정리.
- `rust/crates/sim-bridge/src/pathfinding_core.rs`
  - 기존 `lib.rs`의 pathfinding 타입/함수 집합을 모듈로 이동.
  - `PathfindInput`, `PathfindError`, `pathfind_*`, backend dispatch 함수들을 구조화.

## 추가/삭제 시스템 키
- 없음

## 변경 API / 시그널 / 스키마
- 외부 GDExtension API/시그널/세이브 스키마 변경 없음.
- 내부 모듈 구조만 분리(동작 동일).

## 검증 결과
- `cargo test -p sim-bridge` ✅
- `bash tools/migration_verify.sh` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `46 / 46 = 100.00%`
- Owner transfer 완료율 (`exec_owner=rust`): `46 / 46 = 100.00%`
- State-write 잔여율: `0.00%`
- Owner transfer 잔여율: `0.00%`

## 메모
- `sim-bridge/lib.rs` 단일 파일 분해 2차 단계 완료.
- 다음 분리 후보는 runtime binding/registry 블록이다.
