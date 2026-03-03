# 0160 - test re-export cleanup for runtime events

## Commit
- `[rust-r0-260] Remove test-only runtime event re-export from sim-bridge lib`

## 변경 파일
- `rust/crates/sim-bridge/src/lib.rs`
  - `game_event_type_id` test-only re-export 제거.
  - 테스트가 `runtime_events::game_event_type_id`를 직접 참조하도록 정리.

## 추가/삭제 시스템 키
- 없음

## 변경 API / 시그널 / 스키마
- 외부 API/시그널/세이브 스키마 변경 없음.
- 테스트 참조 경로 정리만 수행(동작 동일).

## 검증 결과
- `cargo test -p sim-bridge` ✅
- `bash tools/migration_verify.sh` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `46 / 46 = 100.00%`
- Owner transfer 완료율 (`exec_owner=rust`): `46 / 46 = 100.00%`
- State-write 잔여율: `0.00%`
- Owner transfer 잔여율: `0.00%`

## 메모
- 모듈 경계 명확화와 test surface 최소화를 위한 정리 커밋.
