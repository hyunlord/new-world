# 0039 - Rust runtime baseline port (network_system)

## Summary
`network_system`을 Rust runtime 지원 시스템으로 추가했다.  
authority type 변경, revolution 트리거 상태 변경, 리더 교체/이벤트 발행의 full parity 이관 전 단계로, 이번 커밋에서는 social capital 및 revolution risk 계산 경로를 Rust scheduler에서 baseline 실행하도록 포팅했다.

## Files Changed
- `rust/crates/sim-systems/src/runtime.rs`
  - `NetworkRuntimeSystem` 추가 (`SimSystem` 구현)
  - 정착지별 집계를 위한 baseline 누적 구조 추가:
    - `NetworkSettlementAccumulator`
  - baseline 경로에서 아래 Rust body 수식 실행:
    - `body::network_social_capital_norm(...)`
    - `body::revolution_risk_score(...)`
  - 현재 ECS 컴포넌트/settlement 데이터 한계로 `inequality`, `leader_unpopularity`는 중립값 baseline(0.5) 사용
  - 현 단계는 side-effect-free baseline (authority/revolution 상태 mutation 및 이벤트 발행 없음)
  - 테스트 추가:
    - `network_runtime_system_baseline_runs_without_side_effects`
- `rust/crates/sim-bridge/src/lib.rs`
  - 런타임 지원 시스템 목록에 `network_system` 추가
  - `register_supported_rust_system(...)`에 `NetworkRuntimeSystem` 등록 분기 추가
  - 테스트 갱신:
    - `runtime_supports_expected_ported_systems`

## Tracking Data Updated
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `network_system`의 `rust_runtime_impl=yes` 반영
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule` 확장:
    - `runtime_registry_match_v1(stats_recorder|resource_regen_system|stat_sync_system|child_stress_processor|mental_break_system|occupation_system|trauma_scar_system|title_system|value_system|network_system|job_assignment_system|stat_threshold_system|upper_needs_system|needs_system|stress_system|emotion_system)`

## Verification
- `cd rust && cargo check -p sim-systems -p sim-bridge` : PASS
- `cd rust && cargo test -p sim-systems runtime::tests::network_runtime_system_baseline_runs_without_side_effects -- --nocapture` : PASS
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
- Previous: `15 / 46 = 32.61%`
- Current: `16 / 46 = 34.78%`
- Remaining: `65.22%`

## Notes
- full parity 이관은 settlement authority type 변경, revolution cooldown/leader 교체 적용, 관련 이벤트(`authority_type_changed`, `revolution`) 발행을 Rust 소유로 이전해야 완료된다.
- 다음 후보는 `social_event_system` 또는 `building_effect_system` baseline 포팅이다.
