# 0151 - rust primary fallback cache pruning

## Commit
- `[rust-r0-251] Prune rust-primary fallback loop to non-offloaded systems only`

## 변경 파일
- `scripts/core/simulation/simulation_engine.gd`
  - Rust primary tick 경로에서 GDScript fallback 루프를 항상 돌지 않도록 개선.
  - `_refresh_gdscript_fallback_cache()` 추가:
    - fallback이 필요한 시스템(`owner-ready 아님`, `runtime rust registry 미등록`, `키 미식별`)만 별도 캐시에 유지.
  - `_update_rust_primary()`에서 `ticks_processed > 0`이어도 fallback 캐시가 비어 있으면 GDScript 시스템 실행을 건너뜀.
  - `_run_gdscript_fallback_ticks()`가 전체 `_systems` 스캔 대신 fallback 캐시만 순회하도록 변경.

## 추가/삭제 시스템 키
- 없음 (등록/오프로딩 키 자체는 변경 없음)

## 변경 API / 시그널 / 스키마
- 외부 API/시그널/세이브 스키마 변경 없음.
- 런타임 동작 변경:
  - `rust_primary`에서 fully-offloaded 상태면 GDScript fallback loop 실행 0회.

## 검증 결과
- `bash tools/migration_verify.sh` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `46 / 46 = 100.00%`
- Owner transfer 완료율 (`exec_owner=rust`): `46 / 46 = 100.00%`
- State-write 잔여율: `0.00%`
- Owner transfer 잔여율: `0.00%`

## 메모
- 이 변경은 기능 확장보다 오케스트레이션 축소(불필요한 GDScript 루프 제거)에 초점이 있다.
- Rust primary의 프레임당 오버헤드를 줄여 Desktop 우선 성능 목표에 유리하다.
