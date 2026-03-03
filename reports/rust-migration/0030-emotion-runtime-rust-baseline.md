# 0030 - Rust runtime baseline port (emotion_system)

## Summary
`emotion_system`을 Rust runtime 지원 시스템으로 추가했다.  
감정 fast/slow/memory-trace 상태와 appraisal 이벤트 큐가 아직 Rust 데이터 코어로 이관되지 않았기 때문에, 이번 단계는 side-effect-free baseline 실행으로 스케줄러/레지스트리 이관 범위를 확장했다.

## Files Changed
- `rust/crates/sim-systems/src/runtime.rs`
  - `EmotionRuntimeSystem` 추가 (`SimSystem` 구현)
  - `Emotion`/`Stress`/`Personality` 입력으로 감정 break threshold/probability/type 계산 경로를 baseline 실행
  - 현 단계에서 상태 mutation 없음
  - 테스트 추가:
    - `emotion_runtime_system_baseline_runs_without_side_effects`
- `rust/crates/sim-bridge/src/lib.rs`
  - 런타임 지원 시스템 목록에 `emotion_system` 추가
  - `register_supported_rust_system(...)`에 `EmotionRuntimeSystem` 등록 분기 추가
  - 테스트 갱신:
    - `runtime_supports_expected_ported_systems`

## Tracking Data Updated
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `emotion_system`의 `rust_runtime_impl=yes` 반영
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule` 확장:
    - `runtime_registry_match_v1(stats_recorder|resource_regen_system|stat_sync_system|upper_needs_system|needs_system|stress_system|emotion_system)`

## Verification
- `cd rust && cargo check -p sim-systems -p sim-bridge` : PASS
- `cd rust && cargo test -p sim-systems runtime::tests::emotion_runtime_system_baseline_runs_without_side_effects -- --nocapture` : PASS
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
- Previous: `6 / 46 = 13.04%`
- Current: `7 / 46 = 15.22%`
- Remaining: `84.78%`

## Notes
- emotion full parity는 event preset/appraisal/habituation/memory trace 구조를 Rust 소유로 이관해야 한다.
- 다음 포팅 후보는 `stat_threshold_system` 또는 `child_stress_processor`다.
