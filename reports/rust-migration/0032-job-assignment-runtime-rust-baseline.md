# 0032 - Rust runtime baseline port (job_assignment_system)

## Summary
`job_assignment_system`을 Rust runtime 지원 시스템으로 추가했다.  
실제 직업 재배정(엔티티 mutation)과 job assignment 이벤트 발행을 Rust 데이터 코어로 이관하기 전 단계로, 이번 커밋에서는 비율/결손 계산 경로를 Rust scheduler에서 baseline 실행하도록 포팅했다.

## Files Changed
- `rust/crates/sim-systems/src/runtime.rs`
  - `JobAssignmentRuntimeSystem` 추가 (`SimSystem` 구현)
  - Rust baseline 계산 유틸 추가:
    - `job_code_from_name(...)`
    - `baseline_job_ratios(...)`
  - 런타임 실행 시 성인/아동 성장단계와 현재 직업 카운트를 집계하고:
    - `body::job_assignment_best_job_code(...)`
    - `body::job_assignment_rebalance_codes(...)`
    경로를 실행
  - 현 단계는 side-effect-free baseline (직업 변경/이벤트 발행 없음)
  - 테스트 추가:
    - `job_assignment_runtime_system_baseline_runs_without_side_effects`
- `rust/crates/sim-bridge/src/lib.rs`
  - 런타임 지원 시스템 목록에 `job_assignment_system` 추가
  - `register_supported_rust_system(...)`에 `JobAssignmentRuntimeSystem` 등록 분기 추가
  - 테스트 갱신:
    - `runtime_supports_expected_ported_systems`

## Tracking Data Updated
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `job_assignment_system`의 `rust_runtime_impl=yes` 반영
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule` 확장:
    - `runtime_registry_match_v1(stats_recorder|resource_regen_system|stat_sync_system|job_assignment_system|stat_threshold_system|upper_needs_system|needs_system|stress_system|emotion_system)`

## Verification
- `cd rust && cargo check -p sim-systems -p sim-bridge` : PASS
- `cd rust && cargo test -p sim-systems runtime::tests::job_assignment_runtime_system_baseline_runs_without_side_effects -- --nocapture` : PASS
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
- Previous: `8 / 46 = 17.39%`
- Current: `9 / 46 = 19.57%`
- Remaining: `80.43%`

## Notes
- full parity 이관은 실제 reassign 조건 평가/직업 변경 적용/통계 반영/이벤트 발행 경로까지 Rust 소유로 옮겨야 완료된다.
- 다음 후보는 `child_stress_processor` 또는 `building_effect_system` baseline 포팅이다.
