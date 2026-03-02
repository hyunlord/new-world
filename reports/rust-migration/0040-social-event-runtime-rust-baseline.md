# 0040 - Rust runtime baseline port (social_event_system)

## Summary
`social_event_system`을 Rust runtime 지원 시스템으로 추가했다.  
관계 이벤트 선택/상태 변경/메모리 기록/UI/버스 이벤트 발행의 full parity 이관 전 단계로, 이번 커밋에서는 attachment 친화도 배수와 proposal 수락 확률 계산 경로를 Rust scheduler에서 baseline 실행하도록 포팅했다.

## Files Changed
- `rust/crates/sim-systems/src/runtime.rs`
  - `SocialEventRuntimeSystem` 추가 (`SimSystem` 구현)
  - attachment 사회화 계수 매핑 유틸 추가:
    - `attachment_socialize_mult(...)`
  - baseline 경로에서 아래 Rust body 수식 실행:
    - `body::social_attachment_affinity_multiplier(...)`
    - `body::social_proposal_accept_prob(...)`
  - 현 단계는 side-effect-free baseline (관계값 mutation/이벤트 발행 없음)
  - 테스트 추가:
    - `social_event_runtime_system_baseline_runs_without_side_effects`
- `rust/crates/sim-bridge/src/lib.rs`
  - 런타임 지원 시스템 목록에 `social_event_system` 추가
  - `register_supported_rust_system(...)`에 `SocialEventRuntimeSystem` 등록 분기 추가
  - 테스트 갱신:
    - `runtime_supports_expected_ported_systems`

## Tracking Data Updated
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `social_event_system`의 `rust_runtime_impl=yes` 반영
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule` 확장:
    - `runtime_registry_match_v1(stats_recorder|resource_regen_system|stat_sync_system|child_stress_processor|mental_break_system|occupation_system|trauma_scar_system|title_system|value_system|network_system|social_event_system|job_assignment_system|stat_threshold_system|upper_needs_system|needs_system|stress_system|emotion_system)`

## Verification
- `cd rust && cargo check -p sim-systems -p sim-bridge` : PASS
- `cd rust && cargo test -p sim-systems runtime::tests::social_event_runtime_system_baseline_runs_without_side_effects -- --nocapture` : PASS
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
- Previous: `16 / 46 = 34.78%`
- Current: `17 / 46 = 36.96%`
- Remaining: `63.04%`

## Notes
- full parity 이관은 social event 선택/적용, 관계 상태 갱신, 메모리 기록, proposal 결과 처리, 관련 시그널/버스 이벤트 발행을 Rust 소유로 이전해야 완료된다.
- 다음 후보는 `building_effect_system` 또는 `family_system` baseline 포팅이다.
