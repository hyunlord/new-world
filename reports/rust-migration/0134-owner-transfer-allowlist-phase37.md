# 0134 - owner transfer allowlist phase 37

## Commit
- `[rust-r0-234] Expand owner-ready allowlist with behavior system`

## 변경 파일
- `scripts/core/simulation/simulation_engine.gd`
  - `_RUST_OWNER_READY_SYSTEM_KEYS`에 `behavior_system` 추가.
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `behavior_system`의 `exec_owner=gdscript -> rust` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - `exec_owner_rule`에 `behavior_system` 추가.
  - `generated_at` 갱신.
- `reports/rust-migration/README.md`
  - 0134 항목 및 누적 전환률 반영.

## 추가/삭제 시스템 키
- owner transfer 추가: `behavior_system`
- 삭제: 없음

## 변경 API / 시그널 / 스키마
- 공개 API/시그널/세이브 스키마 변경 없음.
- Runtime 실행 소유권만 변경 (`exec_owner=rust`).

## 검증 결과
- `cd rust && cargo check -p sim-engine -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-engine` ✅
- `cd rust && cargo test -p sim-systems` ✅
- `cd rust && cargo test -p sim-bridge` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `46 / 46 = 100.00%`
- Owner transfer 완료율 (`exec_owner=rust`): `44 / 46 = 95.65%`
- State-write 잔여율: `0.00%`
- Owner transfer 잔여율: `4.35%`

## 메모
- 잔여 owner transfer 대상: `stat_sync_system`, `stat_threshold_system`.
