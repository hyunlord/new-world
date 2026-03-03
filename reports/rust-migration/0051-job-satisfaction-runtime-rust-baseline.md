# 0051 - Rust runtime baseline port (job_satisfaction_system)

## Summary
`job_satisfaction_system`을 Rust runtime 지원 시스템으로 추가했다.  
직무 프로파일 로딩/drift 의사결정/작업속도 modifier 반영의 full parity 이관 전 단계로, 이번 커밋에서는 job satisfaction 수식 경로를 Rust scheduler에서 baseline 실행하도록 포팅했다.

## Files Changed
- `rust/crates/sim-systems/src/runtime.rs`
  - `JobSatisfactionRuntimeSystem` 추가 (`SimSystem` 구현)
  - baseline 경로 실행:
    - `body::job_satisfaction_score(...)`
  - 현재 단계는 side-effect-free baseline (behavior.job_satisfaction/work-speed mutation 없음)
  - 테스트 추가:
    - `job_satisfaction_runtime_system_baseline_runs_without_side_effects`
- `rust/crates/sim-bridge/src/lib.rs`
  - 런타임 지원 시스템 키에 `job_satisfaction_system` 추가
  - `runtime_supports_rust_system(...)` allowlist에 `job_satisfaction_system` 추가
  - `register_supported_rust_system(...)`에 `JobSatisfactionRuntimeSystem` 등록 분기 추가
  - 테스트 갱신:
    - `runtime_supports_expected_ported_systems`

## Tracking Data Updated
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `job_satisfaction_system`의 `rust_runtime_impl=yes` 반영
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule` 확장:
    - `runtime_registry_match_v1(stats_recorder|resource_regen_system|stat_sync_system|building_effect_system|child_stress_processor|mental_break_system|occupation_system|trauma_scar_system|title_system|trait_violation_system|reputation_system|contagion_system|job_satisfaction_system|value_system|network_system|social_event_system|family_system|leader_system|age_system|mortality_system|population_system|migration_system|job_assignment_system|stat_threshold_system|upper_needs_system|needs_system|stress_system|emotion_system)`

## Verification
- `cd rust && cargo check -p sim-systems -p sim-bridge` : PASS
- `cd rust && cargo test -p sim-systems runtime::tests::job_satisfaction_runtime_system_baseline_runs_without_side_effects -- --nocapture` : PASS
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
- Previous: `27 / 46 = 58.70%`
- Current: `28 / 46 = 60.87%`
- Remaining: `39.13%`

## Notes
- full parity 이관은 프로파일 데이터 로딩, 후보 직무 배치 계산, drift 적용 및 메타 플래그 갱신을 Rust 소유 상태로 이전해야 완료된다.
- 다음 후보는 `morale_system` 또는 `economic_tendency_system` baseline 포팅이다.
