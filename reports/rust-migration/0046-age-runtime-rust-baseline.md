# 0046 - Rust runtime baseline port (age_system)

## Summary
`age_system`을 Rust runtime 지원 시스템으로 추가했다.  
성장 단계 전환 이벤트/연간 성숙화/body realized 값 갱신의 full parity 이관 전 단계로, 이번 커밋에서는 연령 기반 계산 경로를 Rust scheduler에서 baseline 실행하도록 포팅했다.

## Files Changed
- `rust/crates/sim-systems/src/runtime.rs`
  - `AgeRuntimeSystem` 추가 (`SimSystem` 구현)
  - baseline 경로 실행:
    - `body::title_is_elder(...)`
    - `body::compute_age_curves(...)`
    - `body::age_body_speed(...)`
    - `body::age_body_strength(...)`
  - 현재 단계는 side-effect-free baseline (age stage/body mutation 및 이벤트 발행 없음)
  - 테스트 추가:
    - `age_runtime_system_baseline_runs_without_side_effects`
- `rust/crates/sim-bridge/src/lib.rs`
  - 런타임 지원 시스템 키에 `age_system` 추가
  - `runtime_supports_rust_system(...)` allowlist에 `age_system` 추가
  - `register_supported_rust_system(...)`에 `AgeRuntimeSystem` 등록 분기 추가
  - 테스트 갱신:
    - `runtime_supports_expected_ported_systems`

## Tracking Data Updated
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `age_system`의 `rust_runtime_impl=yes` 반영
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule` 확장:
    - `runtime_registry_match_v1(stats_recorder|resource_regen_system|stat_sync_system|building_effect_system|child_stress_processor|mental_break_system|occupation_system|trauma_scar_system|title_system|value_system|network_system|social_event_system|family_system|leader_system|age_system|population_system|migration_system|job_assignment_system|stat_threshold_system|upper_needs_system|needs_system|stress_system|emotion_system)`

## Verification
- `cd rust && cargo check -p sim-systems -p sim-bridge` : PASS
- `cd rust && cargo test -p sim-systems runtime::tests::age_runtime_system_baseline_runs_without_side_effects -- --nocapture` : PASS
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
- Previous: `22 / 46 = 47.83%`
- Current: `23 / 46 = 50.00%`
- Remaining: `50.00%`

## Notes
- full parity 이관은 성장 단계 전환, 성숙화 적용, 신체 스탯 갱신 및 관련 이벤트 발행을 Rust 소유 상태로 이전해야 완료된다.
- 다음 후보는 `trait_violation_system` 또는 `mortality_system` baseline 포팅이다.
