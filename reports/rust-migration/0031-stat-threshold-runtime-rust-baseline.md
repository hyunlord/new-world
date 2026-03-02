# 0031 - Rust runtime baseline port (stat_threshold_system)

## Summary
`stat_threshold_system`을 Rust runtime 지원 시스템으로 추가했다.  
modifier/effect 적용 및 threshold enter/exit 이벤트의 Rust 데이터 코어가 아직 준비되지 않아, 이번 단계는 threshold predicate 실행을 Rust scheduler에서 baseline으로 수행하는 포팅이다.

## Files Changed
- `rust/crates/sim-systems/src/runtime.rs`
  - `StatThresholdRuntimeSystem` 추가 (`SimSystem` 구현)
  - `Need` 기반 정규화 값을 stat scale(0~1000)로 변환해 `body::stat_threshold_is_active` 경로 실행
  - 현재는 side-effect-free baseline (상태 mutation/이벤트 없음)
  - 테스트 추가:
    - `stat_threshold_runtime_system_baseline_runs_without_side_effects`
- `rust/crates/sim-bridge/src/lib.rs`
  - 런타임 지원 시스템 목록에 `stat_threshold_system` 추가
  - `register_supported_rust_system(...)`에 `StatThresholdRuntimeSystem` 등록 분기 추가
  - 테스트 갱신:
    - `runtime_supports_expected_ported_systems`

## Tracking Data Updated
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `stat_threshold_system`의 `rust_runtime_impl=yes` 반영
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule` 확장:
    - `runtime_registry_match_v1(stats_recorder|resource_regen_system|stat_sync_system|stat_threshold_system|upper_needs_system|needs_system|stress_system|emotion_system)`

## Verification
- `cd rust && cargo check -p sim-systems -p sim-bridge` : PASS
- `cd rust && cargo test -p sim-systems runtime::tests::stat_threshold_runtime_system_baseline_runs_without_side_effects -- --nocapture` : PASS
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
- Previous: `7 / 46 = 15.22%`
- Current: `8 / 46 = 17.39%`
- Remaining: `82.61%`

## Notes
- threshold full parity 이관은 effect apply/remove, hysteresis active-state 저장, v1/v2 이벤트 발행까지 Rust로 옮겨야 완료된다.
- 다음 후보는 `child_stress_processor` 또는 `job_assignment_system` baseline 포팅이다.
