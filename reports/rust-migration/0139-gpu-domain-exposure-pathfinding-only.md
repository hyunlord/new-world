# 0139 - gpu domain exposure pathfinding only

## Commit
- `[rust-r0-239] Restrict compute domain exposure to pathfinding only`

## 변경 파일
- `scripts/core/simulation/compute_backend.gd`
  - `COMPUTE_DOMAINS`를 `["pathfinding"]` 단일 도메인으로 축소.
  - UI/설정/명령 큐에서 미연결 도메인(`needs/stress/emotion/orchestration`) 노출 제거.
- `rust/crates/sim-bridge/src/lib.rs`
  - `RUNTIME_COMPUTE_DOMAINS`를 `["pathfinding"]` 단일 도메인으로 축소.
  - runtime command(`set_compute_mode_all`, `set_compute_domain_mode`)의 유효 도메인 집합을 실제 GPU 지원 범위와 일치시킴.
- `reports/rust-migration/README.md`
  - 0139 항목 및 누적 전환률 반영.

## 추가/삭제 시스템 키
- 없음

## 변경 API / 시그널 / 스키마
- 공개 메서드 시그니처 변경 없음.
- 동작 변경:
  - compute domain 모드 조회/설정 경로가 pathfinding 도메인만 대상으로 동작.
  - pathfinding 외 도메인 GPU placeholder 노출 제거.

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
- 향후 needs/stress/emotion/orchestration 커널이 실제 GPU 경로로 연결될 때 도메인을 재노출하면 된다.
