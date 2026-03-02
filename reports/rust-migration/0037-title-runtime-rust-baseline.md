# 0037 - Rust runtime baseline port (title_system)

## Summary
`title_system`을 Rust runtime 지원 시스템으로 추가했다.  
title grant/revoke 상태 변경과 settlement 리더십 연결(Chief/Former Chief), 이벤트 발행의 full parity 이관 전 단계로, 이번 커밋에서는 연령 기반 Elder 판정과 스킬 기반 tier 판정 수식을 Rust scheduler에서 baseline 실행하도록 포팅했다.

## Files Changed
- `rust/crates/sim-systems/src/runtime.rs`
  - `TitleRuntimeSystem` 추가 (`SimSystem` 구현)
  - baseline 경로에서 아래 Rust body 수식을 실행:
    - `body::title_is_elder(...)`
    - `body::title_skill_tier(...)`
  - 현 단계는 side-effect-free baseline (title 목록 mutation 및 이벤트 발행 없음)
  - 테스트 추가:
    - `title_runtime_system_baseline_runs_without_side_effects`
- `rust/crates/sim-bridge/src/lib.rs`
  - 런타임 지원 시스템 목록에 `title_system` 추가
  - `register_supported_rust_system(...)`에 `TitleRuntimeSystem` 등록 분기 추가
  - 테스트 갱신:
    - `runtime_supports_expected_ported_systems`

## Tracking Data Updated
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `title_system`의 `rust_runtime_impl=yes` 반영
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule` 확장:
    - `runtime_registry_match_v1(stats_recorder|resource_regen_system|stat_sync_system|child_stress_processor|mental_break_system|occupation_system|trauma_scar_system|title_system|job_assignment_system|stat_threshold_system|upper_needs_system|needs_system|stress_system|emotion_system)`

## Verification
- `cd rust && cargo check -p sim-systems -p sim-bridge` : PASS
- `cd rust && cargo test -p sim-systems runtime::tests::title_runtime_system_baseline_runs_without_side_effects -- --nocapture` : PASS
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
- Previous: `13 / 46 = 28.26%`
- Current: `14 / 46 = 30.43%`
- Remaining: `69.57%`

## Notes
- full parity 이관은 title 부여/회수 상태 저장, settlement leader 연동(`TITLE_CHIEF`/`TITLE_FORMER_CHIEF`), `title_granted`/`title_revoked` 이벤트 발행을 Rust 소유로 이전해야 완료된다.
- 다음 후보는 `value_system` 또는 `building_effect_system` baseline 포팅이다.
