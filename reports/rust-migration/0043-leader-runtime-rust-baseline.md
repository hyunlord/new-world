# 0043 - Rust runtime baseline port (leader_system)

## Summary
`leader_system`을 Rust runtime 지원 시스템으로 추가했다.  
정착지 단위 후보 선별/리더 선출 상태 변경/선출 이벤트 발행의 full parity 이관 전 단계로, 이번 커밋에서는 리더 연령 존중도와 리더 점수 수식 경로를 Rust scheduler에서 baseline 실행하도록 포팅했다.

## Files Changed
- `rust/crates/sim-systems/src/runtime.rs`
  - `LeaderRuntimeSystem` 추가 (`SimSystem` 구현)
  - 성인 이상 엔티티에 대해 baseline 경로 실행:
    - `body::leader_age_respect(...)`
    - `body::leader_score(...)`
  - Personality/Social/Values 기반 입력을 조합해 leader score 계산
  - 현 단계는 side-effect-free baseline (leader assignment mutation 및 이벤트 발행 없음)
  - 테스트 추가:
    - `leader_runtime_system_baseline_runs_without_side_effects`
- `rust/crates/sim-bridge/src/lib.rs`
  - 런타임 지원 시스템 목록에 `leader_system` 추가
  - `register_supported_rust_system(...)`에 `LeaderRuntimeSystem` 등록 분기 추가
  - 테스트 갱신:
    - `runtime_supports_expected_ported_systems`

## Tracking Data Updated
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `leader_system`의 `rust_runtime_impl=yes` 반영
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule` 확장:
    - `runtime_registry_match_v1(stats_recorder|resource_regen_system|stat_sync_system|building_effect_system|child_stress_processor|mental_break_system|occupation_system|trauma_scar_system|title_system|value_system|network_system|social_event_system|family_system|leader_system|job_assignment_system|stat_threshold_system|upper_needs_system|needs_system|stress_system|emotion_system)`

## Verification
- `cd rust && cargo check -p sim-systems -p sim-bridge` : PASS
- `cd rust && cargo test -p sim-systems runtime::tests::leader_runtime_system_baseline_runs_without_side_effects -- --nocapture` : PASS
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
- Previous: `19 / 46 = 41.30%`
- Current: `20 / 46 = 43.48%`
- Remaining: `56.52%`

## Notes
- full parity 이관은 settlement별 후보 집합 구성, 선출/재선출 상태 갱신, 리더 상실/선출 이벤트 발행을 Rust 소유로 이전해야 완료된다.
- 다음 후보는 `population_system` 또는 `migration_system` baseline 포팅이다.
