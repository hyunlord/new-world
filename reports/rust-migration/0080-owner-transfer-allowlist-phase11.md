# 0080 - owner transfer allowlist phase 11

## Commit
- `[rust-r0-180] Expand owner-ready allowlist with trait-violation system`

## 변경 파일
- `scripts/core/simulation/simulation_engine.gd`
  - `_RUST_OWNER_READY_SYSTEM_KEYS`에 `trait_violation_system` 추가.
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `trait_violation_system`의 `exec_owner=gdscript -> rust` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - `exec_owner_rule`에 `trait_violation_system` 추가.
- `reports/rust-migration/README.md`
  - 0080 항목 추가 및 누적 전환률 갱신.

## 추가/삭제 시스템 키
- owner transfer 추가: `trait_violation_system`
- 삭제: 없음

## 변경 API / 시그널 / 스키마
- 공개 GDExtension 시그니처 변경 없음.
- Runtime owner-ready policy 변경:
  - Rust primary 경로에서 `trait_violation_system` fallback skip 가능.
- tracking 스키마 변경 없음(값만 갱신).

## 검증 결과
- `cd rust && cargo check -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅
- `cd rust && cargo test -p sim-bridge` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `18 / 46 = 39.13%`
- Owner transfer 완료율 (`exec_owner=rust`): `18 / 46 = 39.13%`
- State-write 잔여율: `60.87%`
- Owner transfer 잔여율: `60.87%`

## 메모
- 이번 단계로 trait-violation 경로를 Rust primary owner로 승격했다.
- 다음 실포팅 우선순위 후보: `migration_system`, `leader_system`, `population_system`.
