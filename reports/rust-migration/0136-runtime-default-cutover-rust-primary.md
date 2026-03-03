# 0136 - runtime default cutover to rust primary

## Commit
- `[rust-r0-236] Switch default simulation runtime mode to rust primary`

## 변경 파일
- `scripts/core/simulation/game_config.gd`
  - `SIM_RUNTIME_MODE` 기본값을 `SIM_RUNTIME_MODE_RUST_SHADOW`에서 `SIM_RUNTIME_MODE_RUST_PRIMARY`로 변경.
- `reports/rust-migration/README.md`
  - 0136 항목 및 누적 전환률 반영.

## 추가/삭제 시스템 키
- 없음

## 변경 API / 시그널 / 스키마
- 공개 API/시그널/세이브 스키마 변경 없음.
- 런타임 기본 동작 변경:
  - 기본 부팅 모드가 shadow 검증 경로가 아닌 Rust primary 실행 경로를 사용.

## 검증 결과
- `cd rust && cargo check -p sim-engine -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-engine` ✅
- `cd rust && cargo test -p sim-systems` ✅
- `cd rust && cargo test -p sim-bridge` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `46 / 46 = 100.00%`
- Owner transfer 완료율 (`exec_owner=rust`): `46 / 46 = 100.00%`
- State-write 잔여율: `0.00%`
- Owner transfer 잔여율: `0.00%`

## 메모
- shadow 모드는 설정값으로 여전히 선택 가능하며, 기본값만 primary로 컷오버했다.
