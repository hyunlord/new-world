# 0071 - owner transfer allowlist phase 6

## Commit
- `[rust-r0-171] Expand owner-ready allowlist with age system`

## 변경 파일
- `scripts/core/simulation/simulation_engine.gd`
  - `_RUST_OWNER_READY_SYSTEM_KEYS`에 `age_system` 추가.
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `age_system`의 `exec_owner=gdscript -> rust` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - `exec_owner_rule`에 `age_system` 추가.
- `reports/rust-migration/README.md`
  - 0071 항목 추가 및 누적 전환률 갱신.

## 추가/삭제 시스템 키
- owner transfer 추가: `age_system`
- 삭제: 없음

## 변경 API / 시그널 / 스키마
- 공개 GDExtension 시그니처 변경 없음.
- Runtime owner-ready policy 변경:
  - Rust primary 경로에서 `age_system` fallback skip 가능.
- tracking 스키마 변경 없음(값만 갱신).

## 검증 결과
- `cd rust && cargo check -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅
- `cd rust && cargo test -p sim-bridge` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `14 / 46 = 30.43%`
- Owner transfer 완료율 (`exec_owner=rust`): `13 / 46 = 28.26%`
- State-write 잔여율: `69.57%`
- Owner transfer 잔여율: `71.74%`

## 메모
- 이번 단계로 age 경로를 Rust primary owner로 승격했다.
- 다음 owner transfer 후보는 최신 active-write 상태의 `contagion_system`이다.
