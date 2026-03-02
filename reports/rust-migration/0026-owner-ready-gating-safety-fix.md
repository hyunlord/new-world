# 0026 - Owner-ready gating safety fix

## Summary
`rust_primary` 하이브리드 경로에서 Rust/GDScript 데이터 코어가 아직 분리된 상태임을 반영해, owner-ready 화이트리스트가 비어 있으면 모든 시스템을 GDScript fallback으로 계속 실행하도록 안전장치를 추가했다.  
즉, 실행 소유권 전환은 Phase D(데이터 코어 Rust 소유) 완료 전까지 보수적으로 유지한다.

## Files Changed
- `scripts/core/simulation/simulation_engine.gd`
  - `_RUST_OWNER_READY_SYSTEM_KEYS` 상수 추가 (초기값 빈 배열)
  - `_is_rust_registered_system(...)`에 owner-ready 화이트리스트 조건 추가
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - 3개 시스템 `exec_owner`를 `gdscript`로 복원
- `reports/rust-migration/data/tracking-metadata.json`
  - `exec_owner_rule`을 owner-ready 기반 규칙으로 갱신

## Verification
- `cd rust && cargo check -p sim-systems -p sim-bridge` : PASS
- `Godot --headless --check-only --quit-after 1` : SKIP (`Godot` binary not found in current PATH)

## Migration Progress (Dual Track)
1. Infra Migration Index
- Previous: `100% complete / 0% remaining`
- Current: `100% complete / 0% remaining`
- Delta: `+0%`

2. Runtime Logic Port Index (active execution owner)
- 정의: `rust_exec_owner_systems / registered_systems`
- Previous: `3 / 46 = 6.52%`
- Current: `0 / 46 = 0.0%`
- Remaining: `100.0%`

3. Runtime Logic Implementation Index
- 정의: `rust_runtime_impl_systems / registered_systems`
- Previous: `3 / 46 = 6.52%`
- Current: `3 / 46 = 6.52%`
- Remaining: `93.48%`

## Notes
- 이 수정은 실행 안정성 우선 정책에 따른 보정이다.
- 다음 owner 전환은 실제 Rust 데이터 코어 상태 동기화가 확보된 시스템부터 단계적으로 allowlist에 추가해야 한다.
