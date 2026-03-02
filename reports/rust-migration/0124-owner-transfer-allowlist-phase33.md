# 0124 - owner transfer allowlist phase 33

## Commit
- `[rust-r0-224] Expand owner-ready allowlist with family system`

## 변경 파일
- `scripts/core/simulation/simulation_engine.gd`
  - `_RUST_OWNER_READY_SYSTEM_KEYS`에 `family_system` 추가.
  - Rust primary/hybrid 모드에서 `family_system`의 GDScript fallback 실행을 건너뛸 수 있도록 owner-ready 상태 반영.
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `family_system`의 `exec_owner=gdscript -> rust` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - `exec_owner_rule`에 `family_system` 추가.
  - `generated_at` 갱신.
- `reports/rust-migration/README.md`
  - 0124 항목 및 누적 전환률 반영.

## 추가/삭제 시스템 키
- owner-ready allowlist 추가: `family_system`
- 삭제: 없음

## 변경 API / 시그널 / 스키마
- 공개 API/시그널/세이브 스키마 변경 없음.
- 런타임 owner routing만 변경 (`family_system` 실행 소유권을 Rust 경로로 전환).

## 검증 결과
- `cd rust && cargo check -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅
- `cd rust && cargo test -p sim-bridge` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `40 / 46 = 86.96%`
- Owner transfer 완료율 (`exec_owner=rust`): `40 / 46 = 86.96%`
- State-write 잔여율: `13.04%`
- Owner transfer 잔여율: `13.04%`

## 메모
- `family_system`은 active-write 포팅(`r0-223`) 이후 owner transfer까지 완료되어 Rust 단일 실행 경로로 집계된다.
