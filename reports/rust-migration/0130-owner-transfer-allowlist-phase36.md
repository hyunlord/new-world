# 0130 - owner transfer allowlist phase 36

## Commit
- `[rust-r0-230] Expand owner-ready allowlist with stats recorder`

## 변경 파일
- `scripts/core/simulation/simulation_engine.gd`
  - `_RUST_OWNER_READY_SYSTEM_KEYS`에 `stats_recorder` 추가.
  - Rust primary/hybrid 모드에서 `stats_recorder`의 GDScript fallback 실행을 건너뛸 수 있도록 owner-ready 상태 반영.
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `stats_recorder`의 `exec_owner=gdscript -> rust` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - `exec_owner_rule`에 `stats_recorder` 추가.
  - `generated_at` 갱신.
- `reports/rust-migration/README.md`
  - 0130 항목 및 누적 전환률 반영.

## 추가/삭제 시스템 키
- owner-ready allowlist 추가: `stats_recorder`
- 삭제: 없음

## 변경 API / 시그널 / 스키마
- 공개 API/시그널/세이브 스키마 변경 없음.
- 런타임 owner routing만 변경 (`stats_recorder` 실행 소유권을 Rust 경로로 전환).

## 검증 결과
- `cd rust && cargo check -p sim-engine -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-engine` ✅
- `cd rust && cargo test -p sim-systems` ✅
- `cd rust && cargo test -p sim-bridge` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `43 / 46 = 93.48%`
- Owner transfer 완료율 (`exec_owner=rust`): `43 / 46 = 93.48%`
- State-write 잔여율: `6.52%`
- Owner transfer 잔여율: `6.52%`

## 메모
- `stats_recorder`는 active-write 포팅(`r0-229`) 이후 owner transfer까지 완료되어 Rust 단일 실행 경로로 집계된다.
