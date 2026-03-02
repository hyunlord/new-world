# 0069 - owner transfer allowlist phase 5

## Commit
- `[rust-r0-169] Expand owner-ready allowlist with network and occupation systems`

## 변경 파일
- `scripts/core/simulation/simulation_engine.gd`
  - `_RUST_OWNER_READY_SYSTEM_KEYS`에 `network_system`, `occupation_system` 추가.
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `network_system`, `occupation_system`의 `exec_owner=gdscript -> rust` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - `exec_owner_rule`에 `network_system|occupation_system` 추가.
- `reports/rust-migration/README.md`
  - 0069 항목 추가 및 누적 전환률 갱신.

## 추가/삭제 시스템 키
- owner transfer 추가: `network_system`, `occupation_system`
- 삭제: 없음

## 변경 API / 시그널 / 스키마
- 공개 GDExtension 시그니처 변경 없음.
- Runtime owner-ready policy 변경:
  - Rust primary 경로에서 `network_system`, `occupation_system` fallback skip 가능.
- tracking 스키마 변경 없음(값만 갱신).

## 검증 결과
- `cd rust && cargo check -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅
- `cd rust && cargo test -p sim-bridge` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `13 / 46 = 28.26%`
- Owner transfer 완료율 (`exec_owner=rust`): `12 / 46 = 26.09%`
- State-write 잔여율: `71.74%`
- Owner transfer 잔여율: `73.91%`

## 메모
- 이번 단계로 network/occupation이 Rust primary owner로 승격되었다.
- 최신 active-write로 추가된 `contagion_system`은 parity 안정화 확인 후 owner transfer 후보로 이어간다.
