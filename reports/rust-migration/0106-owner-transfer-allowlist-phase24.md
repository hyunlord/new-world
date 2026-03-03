# 0106 - owner transfer allowlist phase 24

## Commit
- `[rust-r0-206] Expand owner-ready allowlist with building-effect system`

## 변경 파일
- `scripts/core/simulation/simulation_engine.gd`
  - `_RUST_OWNER_READY_SYSTEM_KEYS`에 `building_effect_system` 추가.
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `building_effect_system`의 `exec_owner=gdscript -> rust` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - `exec_owner_rule`에 `building_effect_system` 추가.
  - `generated_at` 갱신.
- `reports/rust-migration/README.md`
  - 0106 항목 추가 및 누적 전환률 갱신.

## 추가/삭제 시스템 키
- owner transfer 추가: `building_effect_system`
- 삭제: 없음

## 변경 API / 시그널 / 스키마
- 공개 GDExtension 시그니처 변경 없음.
- Runtime owner-ready policy 변경:
  - Rust primary 경로에서 `building_effect_system` fallback skip 가능.
- tracking 스키마 구조 변경 없음(값만 갱신).

## 검증 결과
- `cd rust && cargo check -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅
- `cd rust && cargo test -p sim-bridge` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `31 / 46 = 67.39%`
- Owner transfer 완료율 (`exec_owner=rust`): `31 / 46 = 67.39%`
- State-write 잔여율: `32.61%`
- Owner transfer 잔여율: `32.61%`

## 메모
- 이번 단계로 `building_effect_system` 실행 소유권이 Rust로 승격됐다.
- 다음 실포팅 우선순위 후보는 `migration_system` 또는 `tech_discovery_system`이다.
