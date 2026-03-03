# 0155 - sim-bridge runtime registry module split

## Commit
- `[rust-r0-255] Extract runtime registry/config block from sim-bridge lib`

## 변경 파일
- `rust/crates/sim-bridge/src/lib.rs`
  - Runtime registry/config/state 관련 인라인 블록 제거.
  - `runtime_registry` 모듈 import로 치환.
  - 기존 `WorldSimRuntime` 동작 경로는 유지.
- `rust/crates/sim-bridge/src/runtime_registry.rs`
  - 아래 구성요소를 모듈로 이동:
    - `RuntimeConfig`, `RuntimeState`, `RuntimeSystemEntry`
    - `parse_runtime_config`, `clamp_speed_index`, `runtime_speed_multiplier`
    - `runtime_system_key_from_name`, `runtime_supports_rust_system`
    - `register_supported_rust_system`
    - runtime system key 상수/속도 옵션/compute domain 상수

## 추가/삭제 시스템 키
- 없음 (키 목록/매핑 유지)

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
- `sim-bridge/lib.rs` 책임 분해 4차 단계 완료.
- 다음 분리 후보는 body/stat bridge helper 구간이다.
