# 0028 - Rust runtime system port (needs_system)

## Summary
`needs_system`의 핵심 필요도 decay/에너지 소모·회복 계산을 Rust runtime `SimSystem`으로 포팅했다.  
이번 단계는 GDScript hot-path를 Rust `body` 커널 기반으로 이식해 구현 커버리지를 확장한 작업이다.

## Files Changed
- `rust/crates/sim-systems/src/runtime.rs`
  - `NeedsRuntimeSystem` 추가 (`SimSystem` 구현)
  - ECS query 입력 매핑:
    - `Needs`, `Behavior`, `Body`, `Position`
  - Rust 수학 커널 직접 사용:
    - `body::needs_base_decay_step`
    - `body::action_energy_cost`
    - `body::rest_energy_recovery`
  - 업데이트 항목:
    - `Hunger`, `Belonging`, `Thirst`, `Warmth`, `Safety`
    - `energy` + `NeedType::Sleep` 동기화
  - 테스트 추가:
    - `needs_runtime_system_applies_decay_and_action_energy_cost`
- `rust/crates/sim-bridge/src/lib.rs`
  - 런타임 지원 시스템 목록에 `needs_system` 추가
  - `register_supported_rust_system(...)`에 `NeedsRuntimeSystem` 등록 분기 추가
  - 테스트 갱신:
    - `runtime_supports_expected_ported_systems`

## Tracking Data Updated
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `needs_system`의 `rust_runtime_impl=yes` 반영
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule` 확장:
    - `runtime_registry_match_v1(stats_recorder|resource_regen_system|stat_sync_system|upper_needs_system|needs_system)`

## Verification
- `cd rust && cargo check -p sim-systems -p sim-bridge` : PASS
- `cd rust && cargo test -p sim-systems runtime::tests::needs_runtime_system_applies_decay_and_action_energy_cost -- --nocapture` : PASS
- `cd rust && cargo test -p sim-bridge runtime_supports_expected_ported_systems -- --nocapture` : PASS
- `Godot --headless --check-only --quit-after 1` : SKIP (`Godot` binary not found in current PATH)

## Migration Progress (Dual Track)
1. Infra Migration Index
- Previous: `100% complete / 0% remaining`
- Current: `100% complete / 0% remaining`
- Delta: `+0%`

2. Runtime Logic Port Index (active execution owner)
- 정의: `rust_exec_owner_systems / registered_systems`
- Previous: `0 / 46 = 0.0%`
- Current: `0 / 46 = 0.0%`
- Remaining: `100.0%`

3. Runtime Logic Implementation Index
- 정의: `rust_runtime_impl_systems / registered_systems`
- Previous: `4 / 46 = 8.70%`
- Current: `5 / 46 = 10.87%`
- Remaining: `89.13%`

## Notes
- owner-ready allowlist가 비어 있으므로 실행 소유권 전환은 아직 보류다.
- 다음 단계는 `stress_system` 또는 `emotion_system`의 ECS 매핑 포팅이 유효하다.
