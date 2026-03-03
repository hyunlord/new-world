# 0024 - Rust runtime baseline port (stat_sync_system)

## Summary
`stat_sync_system`을 Rust 런타임 지원 시스템에 추가했다.  
현재 단계는 데이터 코어 Rust 소유 이전(Phase D) 전의 baseline 구현으로, 스케줄러/레지스트리 관점의 Rust 실행 소유권 준비를 확장했다.

## Files Changed
- `rust/crates/sim-systems/src/runtime.rs`
  - `StatSyncSystem` 추가 (`SimSystem` 구현)
  - 현 단계에서는 side-effect-free baseline 실행으로 제한
- `rust/crates/sim-bridge/src/lib.rs`
  - Rust 런타임 지원 시스템 목록에 `stat_sync_system` 추가
  - `register_supported_rust_system(...)`에 `StatSyncSystem` 등록 분기 추가
  - 테스트 갱신:
    - `runtime_supports_expected_ported_systems`

## Tracking Data Updated
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `stat_sync_system`의 `rust_runtime_impl=yes` 반영
- `reports/rust-migration/data/tracking-metadata.json`
  - `rust_runtime_impl_rule` 확장:
    - `runtime_registry_match_v1(stats_recorder|resource_regen_system|stat_sync_system)`

## Verification
- `cd rust && cargo check -p sim-systems -p sim-bridge` : PASS
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
- Previous: `2 / 46 = 4.35%`
- Current: `3 / 46 = 6.52%`
- Remaining: `93.48%`

## Notes
- 이 커밋은 구현 커버리지 확대 단계이며, gameplay parity는 데이터 코어 이관 완료 후 진행된다.
- 다음 우선순위는 실제 실행 owner 전환이 가능한 하이브리드 실행 게이트 도입이다.
