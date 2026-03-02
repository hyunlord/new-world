# 0025 - Rust primary hybrid execution gate

## Summary
`rust_primary` 모드에서 Rust 구현 시스템은 Rust 엔진이 실행하고, 미이식 시스템은 GDScript fallback으로 같은 tick 범위에서 실행되도록 하이브리드 실행 경로를 추가했다.  
이로써 `stat_sync_system`, `resource_regen_system`, `stats_recorder` 3개 시스템은 실행 소유권을 Rust 쪽으로 실제 전환 가능 상태가 됐다.

## Files Changed
- `scripts/core/simulation/simulation_engine.gd`
  - `register_system(...)`가 `rust_primary`에서도 로컬 시스템 배열을 유지하도록 변경
  - 런타임 레지스트리 캐시 추가:
    - `_runtime_rust_registered_keys`
    - `_refresh_runtime_registry_cache()`
  - 시스템 키 추적 추가:
    - `_system_key_by_instance_id`
    - `_runtime_system_key_from_name(...)`
  - `rust_primary` 프레임 업데이트에 GDScript fallback 실행 추가:
    - `_run_gdscript_fallback_ticks(start_tick, end_tick)`
    - Rust 등록 시스템은 skip, 미등록 시스템만 fallback 실행
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - 3개 시스템의 `exec_owner`를 `hybrid_rust_primary`로 갱신
    - `stat_sync_system`
    - `resource_regen_system`
    - `stats_recorder`
- `reports/rust-migration/data/tracking-metadata.json`
  - `exec_owner_rule` 추가:
    - `runtime_primary_hybrid_v1(stat_sync_system|resource_regen_system|stats_recorder)`

## Verification
- `cd rust && cargo check -p sim-systems -p sim-bridge` : PASS
- `Godot --headless --check-only --quit-after 1` : SKIP (`Godot` binary not found in current PATH)

## Migration Progress (Dual Track)
1. Infra Migration Index
- Previous: `100% complete / 0% remaining`
- Current: `100% complete / 0% remaining`
- Delta: `+0%`

2. Runtime Logic Port Index (active execution owner)
- 정의: `rust_exec_owner_systems / registered_systems`
- Previous: `0 / 46 = 0.0%`
- Current: `3 / 46 = 6.52%`
- Remaining: `93.48%`

3. Runtime Logic Implementation Index
- 정의: `rust_runtime_impl_systems / registered_systems`
- Previous: `3 / 46 = 6.52%`
- Current: `3 / 46 = 6.52%`
- Remaining: `93.48%`

## Notes
- 이번 단계는 실행 owner 전환 게이트 구축이다.
- 다음 단계는 고비용 시스템(`needs`, `stress`, `emotion`) 중 1개를 Rust 실행 owner로 올려 실제 성능 이득 구간을 확장하는 것이다.
