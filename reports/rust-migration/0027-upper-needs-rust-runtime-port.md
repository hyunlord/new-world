# 0027 - Rust runtime system port (upper_needs_system)

## Summary
`upper_needs_system`을 Rust 런타임 시스템으로 포팅했다.  
이번 단계는 GDScript의 상위욕구(competence/autonomy/self-actualization/meaning/transcendence/recognition/belonging/intimacy) decay+fulfillment 스텝을 Rust `SimSystem`으로 직접 실행하는 구현 이관이다.

## Files Changed
- `rust/crates/sim-systems/src/runtime.rs`
  - `UpperNeedsRuntimeSystem` 추가 (`SimSystem` 구현)
  - ECS query 기반 입력 매핑:
    - `Needs`, `Skills`, `Values`, `Behavior`, `Identity`, `Social`
  - GDScript 경로와 동일한 수식 경로 사용:
    - `body::upper_needs_best_skill_normalized`
    - `body::upper_needs_job_alignment`
    - `body::upper_needs_step`
  - 테스트 추가:
    - `upper_needs_runtime_system_updates_upper_need_buckets`
- `rust/crates/sim-bridge/src/lib.rs`
  - 런타임 지원 시스템 목록에 `upper_needs_system` 추가
  - `register_supported_rust_system(...)`에 `UpperNeedsRuntimeSystem` 등록 분기 추가
  - 테스트 갱신:
    - `runtime_supports_expected_ported_systems`

## Tracking Data Updated
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `upper_needs_system`의 `rust_runtime_impl=yes` 반영
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule` 확장:
    - `runtime_registry_match_v1(stats_recorder|resource_regen_system|stat_sync_system|upper_needs_system)`

## Verification
- `cd rust && cargo check -p sim-systems -p sim-bridge` : PASS
- `cd rust && cargo test -p sim-systems runtime::tests::upper_needs_runtime_system_updates_upper_need_buckets -- --nocapture` : PASS
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
- Previous: `3 / 46 = 6.52%`
- Current: `4 / 46 = 8.70%`
- Remaining: `91.30%`

## Notes
- 실행 owner 전환은 owner-ready allowlist가 비어 있으므로 아직 보류 상태다.
- 다음 단계는 `needs_system` 또는 `stress_system`의 ECS 입력 매핑을 Rust로 확장해 구현 커버리지를 계속 올리는 것이다.
