# 0023 - Rust runtime system port (resource_regen_system)

## Summary
`resource_regen_system`을 Rust `SimSystem`으로 이식하고 `sim-bridge` 런타임 레지스트리에 등록 가능하도록 확장했다.  
이번 단계는 타일 리소스 재생성 루프를 Rust로 이동한 실제 로직 포트이며, 실행 소유권 전환을 위한 두 번째 런타임 구현 시스템이다.

## Files Changed
- `rust/crates/sim-systems/src/runtime.rs`
  - `ResourceRegenSystem` 추가 (`SimSystem` 구현)
  - 타일별 resource deposit에 대해 `amount = min(amount + regen_rate, max_amount)` 적용
  - 단위 테스트 추가:
    - `resource_regen_system_applies_regen_and_caps_at_max`
- `rust/crates/sim-bridge/src/lib.rs`
  - Rust 런타임 지원 시스템 목록에 `resource_regen_system` 추가
  - `register_supported_rust_system(...)`에서 `ResourceRegenSystem` 등록 분기 추가
  - 테스트 갱신:
    - `runtime_supports_expected_ported_systems`

## Tracking Data Updated
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `resource_regen_system`의 `rust_runtime_impl=yes` 반영
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule` 확장:
    - `runtime_registry_match_v1(stats_recorder|resource_regen_system)`

## Verification
- `cd rust && cargo check -p sim-systems -p sim-bridge` : PASS
- `cd rust && cargo test -p sim-systems runtime::tests::resource_regen_system_applies_regen_and_caps_at_max -- --nocapture` : PASS
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
- Previous: `1 / 46 = 2.17%`
- Current: `2 / 46 = 4.35%`
- Remaining: `95.65%`

## Notes
- 현재는 런타임 구현 커버리지(implementation index) 확장 단계다.
- 다음 단계는 `resource_regen_system` 또는 `stats_recorder` 중 1개를 대상으로
  하이브리드 실행 경로에서 실제 execution owner 전환 지점을 도입해야 한다.
