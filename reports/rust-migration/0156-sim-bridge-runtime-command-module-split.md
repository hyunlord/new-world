# 0156 - sim-bridge runtime command module split

## Commit
- `[rust-r0-256] Extract runtime command processing from sim-bridge lib`

## 변경 파일
- `rust/crates/sim-bridge/src/lib.rs`
  - `runtime_get_registry_snapshot`, `runtime_clear_registry`, `runtime_apply_commands_v2`에서 인라인 처리 로직 제거.
  - `runtime_commands` 모듈 함수 호출로 치환.
- `rust/crates/sim-bridge/src/runtime_commands.rs`
  - 런타임 command pipeline 분리:
    - `registry_snapshot`
    - `clear_registry`
    - `apply_commands_v2`
  - register/compute-domain command 처리 및 정렬/업서트 로직 이동.

## 추가/삭제 시스템 키
- 없음 (등록/지원 매핑 유지)

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
- `sim-bridge/lib.rs` 모듈 분해 5차 단계 완료.
- 다음 단계는 body/stat bridge 대형 함수군 분할 또는 runtime primary 완전 전환 검증 자동화 CI 연결이다.
