# 0050 - Rust runtime baseline port (contagion_system)

## Summary
`contagion_system`을 Rust runtime 지원 시스템으로 추가했다.  
AoE/네트워크 전파 대상 선택, refractory 상태 갱신, 감염 결과 상태 반영의 full parity 이관 전 단계로, 이번 커밋에서는 contagion 수식 경로를 Rust scheduler에서 baseline 실행하도록 포팅했다.

## Files Changed
- `rust/crates/sim-systems/src/runtime.rs`
  - `ContagionRuntimeSystem` 추가 (`SimSystem` 구현)
  - baseline 경로 실행:
    - `body::contagion_aoe_total_susceptibility(...)`
    - `body::contagion_stress_delta(...)`
    - `body::contagion_network_delta(...)`
    - `body::contagion_spiral_increment(...)`
  - 현재 단계는 side-effect-free baseline (emotion/stress mutation 없음)
  - 테스트 추가:
    - `contagion_runtime_system_baseline_runs_without_side_effects`
- `rust/crates/sim-bridge/src/lib.rs`
  - 런타임 지원 시스템 키에 `contagion_system` 추가
  - `runtime_supports_rust_system(...)` allowlist에 `contagion_system` 추가
  - `register_supported_rust_system(...)`에 `ContagionRuntimeSystem` 등록 분기 추가
  - 테스트 갱신:
    - `runtime_supports_expected_ported_systems`

## Tracking Data Updated
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `contagion_system`의 `rust_runtime_impl=yes` 반영
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule` 확장:
    - `runtime_registry_match_v1(stats_recorder|resource_regen_system|stat_sync_system|building_effect_system|child_stress_processor|mental_break_system|occupation_system|trauma_scar_system|title_system|trait_violation_system|reputation_system|contagion_system|value_system|network_system|social_event_system|family_system|leader_system|age_system|mortality_system|population_system|migration_system|job_assignment_system|stat_threshold_system|upper_needs_system|needs_system|stress_system|emotion_system)`

## Verification
- `cd rust && cargo check -p sim-systems -p sim-bridge` : PASS
- `cd rust && cargo test -p sim-systems runtime::tests::contagion_runtime_system_baseline_runs_without_side_effects -- --nocapture` : PASS
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
- Previous: `26 / 46 = 56.52%`
- Current: `27 / 46 = 58.70%`
- Remaining: `41.30%`

## Notes
- full parity 이관은 AoE/네트워크 donor selection, refractory 갱신, contagion 결과 반영(감정/스트레스/스파이럴 플래그)을 Rust 소유 상태로 이전해야 완료된다.
- 다음 후보는 `job_satisfaction_system` 또는 `morale_system` baseline 포팅이다.
