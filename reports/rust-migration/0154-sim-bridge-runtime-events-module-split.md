# 0154 - sim-bridge runtime events module split

## Commit
- `[rust-r0-254] Extract runtime event mapping helpers from sim-bridge lib`

## 변경 파일
- `rust/crates/sim-bridge/src/lib.rs`
  - runtime event helper(`game_event_*`, `dict_get_*`) 인라인 구현 제거.
  - `runtime_events` 모듈 import/re-export로 치환.
  - 테스트 전용 `game_event_type_id` 노출을 `#[cfg(test)]`로 제한.
- `rust/crates/sim-bridge/src/runtime_events.rs`
  - 이벤트 타입 ID 매핑, payload 직렬화, runtime event v2 Dictionary 변환 로직 이동.
  - runtime command payload 파싱 보조 함수(`dict_get_*`) 이동.

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
- `sim-bridge/lib.rs` 모듈 분해 3차 단계 완료.
- 남은 대형 블록은 runtime registry/command 처리 파트다.
