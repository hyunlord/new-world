# 0022 - First Rust runtime system port (stats_recorder)

## Summary
실행 소유권 전환의 첫 단계로, `stats_recorder`를 Rust `SimSystem`으로 실제 등록 가능한 상태로 이식했다.  
아직 기본 실행 소유권은 GDScript지만, Runtime registry에서 Rust 구현/등록 여부를 추적할 수 있게 됐다.

## Files Changed
- `rust/crates/sim-systems/src/runtime.rs` (신규)
  - `StatsRecorderSystem` 추가 (`SimSystem` production impl)
  - 우선 side-effect-free baseline으로 Rust scheduler ownership만 이식
- `rust/crates/sim-systems/src/lib.rs`
  - `pub mod runtime;` 추가
- `rust/crates/sim-engine/src/engine.rs`
  - `clear_systems()` 추가 (runtime 재구성 시 시스템 재등록 지원)
- `rust/crates/sim-bridge/src/lib.rs`
  - runtime 시스템 키 파싱: `runtime_system_key_from_name(...)`
  - Rust 지원 시스템 판별: `runtime_supports_rust_system(...)`
  - Rust 시스템 등록기: `register_supported_rust_system(...)`
  - `register_system` command 처리 시 `stats_recorder`를 Rust engine에 실제 등록
  - registry snapshot 확장:
    - `system_key`
    - `rust_implemented`
    - `rust_registered`
    - `exec_backend`
  - `runtime_clear_registry` / `clear_registry`가 engine 시스템도 함께 clear
  - 테스트 추가:
    - `runtime_system_key_normalizes_script_paths`
    - `runtime_supports_expected_first_ported_system`

## Tracking Data Updated
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `system_key`, `rust_runtime_impl` 컬럼 추가
- `reports/rust-migration/data/tracking-metadata.json`
  - `dataset_version=tracking-v2`
  - `rust_runtime_impl_rule` 추가

## Verification
- `cd rust && cargo check -p sim-systems -p sim-engine -p sim-bridge` : PASS
- `cd rust && cargo test -p sim-bridge runtime_system -- --nocapture` : PASS
- `cd rust && cargo build -p sim-bridge && cargo build --release -p sim-bridge` : PASS
- `Godot --headless --check-only --quit-after 1` : PASS (기존 leak/resource warning만 존재)
- `Godot --headless --script tools/rust_shadow_smoke.gd` : PASS
- `python3 tools/rust_shadow_cutover_check.py --report <latest.json>` : PASS (`approved_for_cutover=True`)

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

3. Runtime Logic Implementation Index (new)
- 정의: `rust_runtime_impl_systems / registered_systems`
- Previous: `0 / 46 = 0.0%`
- Current: `1 / 46 = 2.17%`
- Remaining: `97.83%`

## Notes
- 이번 커밋은 “실행 소유권 전환 준비” 단계다.
- 다음 단계는 `stats_recorder`의 full behavior parity 이식 및 hybrid execution gating 도입이 필요하다.
