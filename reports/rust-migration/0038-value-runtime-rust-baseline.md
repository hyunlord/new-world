# 0038 - Rust runtime baseline port (value_system)

## Summary
`value_system`을 Rust runtime 지원 시스템으로 추가했다.  
peer influence 선택, settlement 문화 동조, rationalization/experience event 기반 실제 값 변경의 full parity 이관 전 단계로, 이번 커밋에서는 연령 기반 plasticity와 가치 상태 파생 지표 계산을 Rust scheduler에서 baseline 실행하도록 포팅했다.

## Files Changed
- `rust/crates/sim-systems/src/runtime.rs`
  - `ValueRuntimeSystem` 추가 (`SimSystem` 구현)
  - baseline 경로에서 `body::value_plasticity(...)` 실행
  - `Values` + `Personality`에서 파생되는 수용성 지표를 계산하되 상태 변경 없음
  - 현 단계는 side-effect-free baseline (values mutation 및 이벤트 발행 없음)
  - 테스트 추가:
    - `value_runtime_system_baseline_runs_without_side_effects`
- `rust/crates/sim-bridge/src/lib.rs`
  - 런타임 지원 시스템 목록에 `value_system` 추가
  - `register_supported_rust_system(...)`에 `ValueRuntimeSystem` 등록 분기 추가
  - 테스트 갱신:
    - `runtime_supports_expected_ported_systems`

## Tracking Data Updated
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `value_system`의 `rust_runtime_impl=yes` 반영
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule` 확장:
    - `runtime_registry_match_v1(stats_recorder|resource_regen_system|stat_sync_system|child_stress_processor|mental_break_system|occupation_system|trauma_scar_system|title_system|value_system|job_assignment_system|stat_threshold_system|upper_needs_system|needs_system|stress_system|emotion_system)`

## Verification
- `cd rust && cargo check -p sim-systems -p sim-bridge` : PASS
- `cd rust && cargo test -p sim-systems runtime::tests::value_runtime_system_baseline_runs_without_side_effects -- --nocapture` : PASS
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
- Previous: `14 / 46 = 30.43%`
- Current: `15 / 46 = 32.61%`
- Remaining: `67.39%`

## Notes
- full parity 이관은 value initialization/peer influence/culture conformity/rationalization/event 적용 및 관련 이벤트 발행 흐름을 Rust 소유로 이전해야 완료된다.
- 다음 후보는 `network_system` 또는 `building_effect_system` baseline 포팅이다.
