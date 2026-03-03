# 0035 - Rust runtime baseline port (occupation_system)

## Summary
`occupation_system`을 Rust runtime 지원 시스템으로 추가했다.  
occupation 카테고리 매핑 데이터 로드/직업 변경 mutation/`occupation_changed` 이벤트 발행은 아직 Rust 데이터 코어로 이관되지 않았기 때문에, 이번 단계는 best-skill 탐색과 hysteresis 판정 수식을 Rust scheduler에서 baseline 실행하도록 포팅했다.

## Files Changed
- `rust/crates/sim-systems/src/runtime.rs`
  - `OccupationRuntimeSystem` 추가 (`SimSystem` 구현)
  - baseline 유틸 추가:
    - `occupation_to_skill_id(...)`
  - baseline 경로에서 아래 Rust body 수식 실행:
    - `body::occupation_best_skill_index(...)`
    - `body::occupation_should_switch(...)`
  - 성장 단계(`Infant/Toddler`) 스킵 규칙 반영
  - 현 단계는 side-effect-free baseline (occupation/job mutation 및 이벤트 발행 없음)
  - 테스트 추가:
    - `occupation_runtime_system_baseline_runs_without_side_effects`
- `rust/crates/sim-bridge/src/lib.rs`
  - 런타임 지원 시스템 목록에 `occupation_system` 추가
  - `register_supported_rust_system(...)`에 `OccupationRuntimeSystem` 등록 분기 추가
  - 테스트 갱신:
    - `runtime_supports_expected_ported_systems`

## Tracking Data Updated
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `occupation_system`의 `rust_runtime_impl=yes` 반영
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule` 확장:
    - `runtime_registry_match_v1(stats_recorder|resource_regen_system|stat_sync_system|child_stress_processor|mental_break_system|occupation_system|job_assignment_system|stat_threshold_system|upper_needs_system|needs_system|stress_system|emotion_system)`

## Verification
- `cd rust && cargo check -p sim-systems -p sim-bridge` : PASS
- `cd rust && cargo test -p sim-systems runtime::tests::occupation_runtime_system_baseline_runs_without_side_effects -- --nocapture` : PASS
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
- Previous: `11 / 46 = 23.91%`
- Current: `12 / 46 = 26.09%`
- Remaining: `73.91%`

## Notes
- full parity 이관은 occupation category 데이터셋 로딩, 실제 occupation/job 변경 적용, legacy job 매핑, `occupation_changed` 이벤트 발행을 Rust 소유로 이전해야 완료된다.
- 다음 후보는 `building_effect_system` 또는 `trauma_scar_system` baseline 포팅이다.
