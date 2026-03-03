# 0158 - runtime event constants module cutover

## Commit
- `[rust-r0-258] Move runtime event type constants into runtime_events module`

## 변경 파일
- `rust/crates/sim-bridge/src/runtime_events.rs`
  - event type id 상수를 `runtime_events` 모듈 내부로 이동.
  - `game_event_type_id` 매핑이 모듈 내부 상수를 직접 사용하도록 정리.
- `rust/crates/sim-bridge/src/lib.rs`
  - event type id 상수 중복 정의 제거.
  - 테스트가 `runtime_events::EVENT_TYPE_ID_*`를 참조하도록 변경.

## 추가/삭제 시스템 키
- 없음

## 변경 API / 시그널 / 스키마
- 외부 API/시그널/세이브 스키마 변경 없음.
- 내부 상수 소유권만 `runtime_events` 모듈로 이동(동작 동일).

## 검증 결과
- `cargo test -p sim-bridge` ✅
- `bash tools/migration_verify.sh` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `46 / 46 = 100.00%`
- Owner transfer 완료율 (`exec_owner=rust`): `46 / 46 = 100.00%`
- State-write 잔여율: `0.00%`
- Owner transfer 잔여율: `0.00%`

## 메모
- runtime event 계층의 결합도를 낮춰 이후 `lib.rs` 정리 리스크를 줄였다.
