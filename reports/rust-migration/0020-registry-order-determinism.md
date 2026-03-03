# 0020 - Runtime registry order determinism

## Summary
Rust/GDScript 레지스트리 비교에서 간헐적으로 발생하던 `order_match=false` 경고를 제거하기 위해, 시스템 등록 정렬 키를 `priority + registration_index`로 고정했다.

## Files Changed
- `scripts/core/simulation/simulation_engine.gd`
  - 시스템 payload에 `registration_index` 추가
  - 기대 레지스트리 정렬 로직을 `priority` 우선, 동순위 시 `registration_index` 보조키로 변경
- `rust/crates/sim-bridge/src/lib.rs`
  - `RuntimeSystemEntry`에 `registration_index` 필드 추가
  - `runtime_apply_commands_v2(register_system)`에서 `registration_index` 수신/저장
  - Rust 레지스트리 정렬을 `priority -> registration_index -> name` 순으로 고정
  - `runtime_get_registry_snapshot()`에 `registration_index` 포함

## API / Signal / Schema Changes
- Runtime registry snapshot row 필드 확장
  - 추가: `registration_index: int`

## Verification
- `cd rust && cargo build -p sim-bridge && cargo build --release -p sim-bridge` : PASS
- `Godot --headless --check-only --quit-after 1` : PASS
  - 기존 `[SimulationEngine] Runtime registry mismatch ... order_match=false` 경고 제거 확인
- `Godot --headless --script tools/rust_shadow_smoke.gd` : PASS
- `python3 tools/rust_shadow_cutover_check.py --report <latest.json>` : PASS
  - `approved_for_cutover=True`

## Rust Migration Progress
- Previous: 100% complete / 0% remaining
- Current: 100% complete / 0% remaining
- Delta: +0%

## Notes
- 본 변경은 기능 확장보다 검증 신뢰성 강화를 위한 결정성 보강이다.
