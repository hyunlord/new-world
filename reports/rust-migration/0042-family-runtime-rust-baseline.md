# 0042 - Rust runtime baseline port (family_system)

## Summary
`family_system`을 Rust runtime 지원 시스템으로 추가했다.  
커플 형성/임신 진행/출산 스폰/사망 및 연쇄 이벤트의 full parity 이관 전 단계로, 이번 커밋에서는 가임 연령/파트너 조건 기반 newborn health 계산 경로를 Rust scheduler에서 baseline 실행하도록 포팅했다.

## Files Changed
- `rust/crates/sim-systems/src/runtime.rs`
  - `FamilyRuntimeSystem` 추가 (`SimSystem` 구현)
  - 가임 조건(여성, 15~45세, spouse 존재) baseline 필터링 후:
    - `body::family_newborn_health(...)` 실행
  - 현 단계는 side-effect-free baseline (임신 상태/출산/자식 스폰/이벤트 발행 없음)
  - 테스트 추가:
    - `family_runtime_system_baseline_runs_without_side_effects`
- `rust/crates/sim-bridge/src/lib.rs`
  - 런타임 지원 시스템 목록에 `family_system` 추가
  - `register_supported_rust_system(...)`에 `FamilyRuntimeSystem` 등록 분기 추가
  - 테스트 갱신:
    - `runtime_supports_expected_ported_systems`

## Tracking Data Updated
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `family_system`의 `rust_runtime_impl=yes` 반영
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule` 확장:
    - `runtime_registry_match_v1(stats_recorder|resource_regen_system|stat_sync_system|building_effect_system|child_stress_processor|mental_break_system|occupation_system|trauma_scar_system|title_system|value_system|network_system|social_event_system|family_system|job_assignment_system|stat_threshold_system|upper_needs_system|needs_system|stress_system|emotion_system)`

## Verification
- `cd rust && cargo check -p sim-systems -p sim-bridge` : PASS
- `cd rust && cargo test -p sim-systems runtime::tests::family_runtime_system_baseline_runs_without_side_effects -- --nocapture` : PASS
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
- Previous: `18 / 46 = 39.13%`
- Current: `19 / 46 = 41.30%`
- Remaining: `58.70%`

## Notes
- full parity 이관은 couple 형성/임신 지속/출산 엔티티 생성/산모 사망 처리/가족 관계 및 이벤트 발행 경로를 Rust 소유로 이전해야 완료된다.
- 다음 후보는 `leader_system` 또는 `population_system` baseline 포팅이다.
