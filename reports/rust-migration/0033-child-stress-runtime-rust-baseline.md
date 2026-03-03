# 0033 - Rust runtime baseline port (child_stress_processor)

## Summary
`child_stress_processor`를 Rust runtime 지원 시스템으로 추가했다.  
아동 스트레스의 full parity(발달 단계 데이터 소유, stressor/event 큐, meta 기반 상태 변이)가 아직 Rust 데이터 코어로 이관되지 않아, 이번 단계는 child-stage appraise/apply 수식을 Rust scheduler에서 baseline 실행하도록 포팅했다.

## Files Changed
- `rust/crates/sim-systems/src/runtime.rs`
  - `ChildStressProcessorRuntimeSystem` 추가 (`SimSystem` 구현)
  - baseline 유틸 추가:
    - `child_stage_code_from_growth_stage(...)`
    - `child_stage_baseline_params(...)`
  - 아동 성장 단계(`Infant/Toddler/Child/Teen`) 엔티티 대상 baseline 경로 실행:
    - `body::child_social_buffered_intensity(...)`
    - `body::child_shrp_step(...)`
    - `body::child_stress_type_code(...)`
    - `body::child_stress_apply_step(...)`
  - 현 단계는 side-effect-free baseline (Stress/Need mutation 및 이벤트 발행 없음)
  - 테스트 추가:
    - `child_stress_processor_runtime_system_baseline_runs_without_side_effects`
- `rust/crates/sim-bridge/src/lib.rs`
  - 런타임 지원 시스템 목록에 `child_stress_processor` 추가
  - `register_supported_rust_system(...)`에 `ChildStressProcessorRuntimeSystem` 등록 분기 추가
  - 테스트 갱신:
    - `runtime_supports_expected_ported_systems`

## Tracking Data Updated
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `child_stress_processor`의 `rust_runtime_impl=yes` 반영
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule` 확장:
    - `runtime_registry_match_v1(stats_recorder|resource_regen_system|stat_sync_system|child_stress_processor|job_assignment_system|stat_threshold_system|upper_needs_system|needs_system|stress_system|emotion_system)`

## Verification
- `cd rust && cargo check -p sim-systems -p sim-bridge` : PASS
- `cd rust && cargo test -p sim-systems runtime::tests::child_stress_processor_runtime_system_baseline_runs_without_side_effects -- --nocapture` : PASS
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
- Previous: `9 / 46 = 19.57%`
- Current: `10 / 46 = 21.74%`
- Remaining: `78.26%`

## Notes
- full parity 이관은 stage 데이터(발달/attachment/caregiver), deprivation 누적 상태, child_stress_processed 이벤트 발행을 Rust 소유로 이전해야 완료된다.
- 다음 후보는 `building_effect_system` 또는 `mental_break_system` baseline 포팅이다.
