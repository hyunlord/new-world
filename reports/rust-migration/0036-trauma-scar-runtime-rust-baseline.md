# 0036 - Rust runtime baseline port (trauma_scar_system)

## Summary
`trauma_scar_system`을 Rust runtime 지원 시스템으로 추가했다.  
scar 정의 데이터 로딩/stack 획득 갱신/reactivation 이벤트/감정 baseline drift의 full parity 이관 전 단계로, 이번 커밋에서는 scar 획득 확률과 stress 민감도 계수 수식을 Rust scheduler에서 baseline 실행하도록 포팅했다.

## Files Changed
- `rust/crates/sim-systems/src/runtime.rs`
  - `TraumaScarRuntimeSystem` 추가 (`SimSystem` 구현)
  - `Memory.trauma_scars`를 순회하며 baseline 경로 실행:
    - `body::trauma_scar_acquire_chance(...)`
    - `body::trauma_scar_sensitivity_factor(...)`
  - 현 단계는 side-effect-free baseline (scar stack mutation / reactivation / emotion drift 없음)
  - 테스트 추가:
    - `trauma_scar_runtime_system_baseline_runs_without_side_effects`
- `rust/crates/sim-bridge/src/lib.rs`
  - 런타임 지원 시스템 목록에 `trauma_scar_system` 추가
  - `register_supported_rust_system(...)`에 `TraumaScarRuntimeSystem` 등록 분기 추가
  - 테스트 갱신:
    - `runtime_supports_expected_ported_systems`

## Tracking Data Updated
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `trauma_scar_system`의 `rust_runtime_impl=yes` 반영
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule` 확장:
    - `runtime_registry_match_v1(stats_recorder|resource_regen_system|stat_sync_system|child_stress_processor|mental_break_system|occupation_system|trauma_scar_system|job_assignment_system|stat_threshold_system|upper_needs_system|needs_system|stress_system|emotion_system)`

## Verification
- `cd rust && cargo check -p sim-systems -p sim-bridge` : PASS
- `cd rust && cargo test -p sim-systems runtime::tests::trauma_scar_runtime_system_baseline_runs_without_side_effects -- --nocapture` : PASS
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
- Previous: `12 / 46 = 26.09%`
- Current: `13 / 46 = 28.26%`
- Remaining: `71.74%`

## Notes
- full parity 이관은 scar 정의 데이터셋 Rust 소유화, scar stack/획득 로직 상태 변경, reactivation 트리거 이벤트, emotion baseline drift 적용을 Rust로 이전해야 완료된다.
- 다음 후보는 `title_system` 또는 `building_effect_system` baseline 포팅이다.
