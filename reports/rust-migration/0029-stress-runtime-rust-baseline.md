# 0029 - Rust runtime baseline port (stress_system)

## Summary
`stress_system`을 Rust runtime 지원 시스템으로 추가했다.  
현재 단계는 스트레스 이벤트/트레이스 큐 데이터 코어 이관 전이므로, side-effect-free baseline 실행으로 스케줄러/레지스트리 포팅 범위를 확장했다.

## Files Changed
- `rust/crates/sim-systems/src/runtime.rs`
  - `StressRuntimeSystem` 추가 (`SimSystem` 구현)
  - `Needs`/`Stress`/`Emotion` 기반 입력을 읽어 Rust stress kernel(`needs_critical_severity_step`) 경로를 baseline 실행
  - 현재는 의도적으로 상태 mutation 없음 (phase D 준비 단계)
  - 테스트 추가:
    - `stress_runtime_system_baseline_runs_without_side_effects`
- `rust/crates/sim-bridge/src/lib.rs`
  - 런타임 지원 시스템 목록에 `stress_system` 추가
  - `register_supported_rust_system(...)`에 `StressRuntimeSystem` 등록 분기 추가
  - 테스트 갱신:
    - `runtime_supports_expected_ported_systems`

## Tracking Data Updated
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `stress_system`의 `rust_runtime_impl=yes` 반영
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule` 확장:
    - `runtime_registry_match_v1(stats_recorder|resource_regen_system|stat_sync_system|upper_needs_system|needs_system|stress_system)`

## Verification
- `cd rust && cargo check -p sim-systems -p sim-bridge` : PASS
- `cd rust && cargo test -p sim-systems runtime::tests::stress_runtime_system_baseline_runs_without_side_effects -- --nocapture` : PASS
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
- Previous: `5 / 46 = 10.87%`
- Current: `6 / 46 = 13.04%`
- Remaining: `86.96%`

## Notes
- stress full parity는 stress trace/rebound/event source 데이터 구조를 Rust 소유로 옮겨야 가능하다.
- 다음 포팅 후보는 `emotion_system` 또는 `stat_threshold_system`이다.
