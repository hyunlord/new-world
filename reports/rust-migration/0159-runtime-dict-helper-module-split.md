# 0159 - runtime dict helper module split

## Commit
- `[rust-r0-259] Extract runtime dictionary parsing helpers`

## 변경 파일
- `rust/crates/sim-bridge/src/runtime_dict.rs`
  - runtime payload 파싱 유틸 추가:
    - `dict_get_string`
    - `dict_get_i32`
    - `dict_get_bool`
- `rust/crates/sim-bridge/src/runtime_commands.rs`
  - command payload 파싱이 `runtime_dict` 모듈 헬퍼를 사용하도록 변경.
- `rust/crates/sim-bridge/src/runtime_events.rs`
  - event type id 상수와 event 직렬화 책임만 유지하도록 정리.
- `rust/crates/sim-bridge/src/lib.rs`
  - event type id 상수 인라인 정의 제거.
  - 테스트가 `runtime_events::EVENT_TYPE_ID_*`를 직접 참조하도록 정리.

## 추가/삭제 시스템 키
- 없음

## 변경 API / 시그널 / 스키마
- 외부 GDExtension API/시그널/세이브 스키마 변경 없음.
- 내부 파싱 유틸 책임만 모듈 분리(동작 동일).

## 검증 결과
- `cargo test -p sim-bridge` ✅
- `bash tools/migration_verify.sh` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `46 / 46 = 100.00%`
- Owner transfer 완료율 (`exec_owner=rust`): `46 / 46 = 100.00%`
- State-write 잔여율: `0.00%`
- Owner transfer 잔여율: `0.00%`

## 메모
- runtime 계층의 모듈 결합도 추가 완화.
