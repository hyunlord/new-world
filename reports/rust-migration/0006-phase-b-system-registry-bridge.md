# 0006 - Phase B system registry bridge

## Summary
GDScript에서 등록하는 시스템 메타데이터(이름/우선순위/tick_interval/활성여부)를 Bus v2 command 파이프를 통해 Rust runtime 레지스트리로 전달하고 조회할 수 있게 만들었다.

## Files Changed
- `scripts/core/simulation/simulation_engine.gd`
  - `register_system()`에서 Rust runtime 사용 가능 시 `register_system` command enqueue.
  - runtime init 성공 시 registry 초기화(`runtime_clear_registry`) 추가.
  - 시스템 payload 빌더 추가(`_build_runtime_system_payload`).
- `scripts/core/simulation/sim_bridge.gd`
  - runtime registry API 래퍼 추가:
    - `runtime_get_registry_snapshot()`
    - `runtime_clear_registry()`
- `rust/crates/sim-bridge/src/lib.rs`
  - `RuntimeState`에 `registered_systems` 저장 추가.
  - `runtime_get_registry_snapshot()` / `runtime_clear_registry()` 추가.
  - `runtime_apply_commands_v2()`에 command 처리 확장:
    - `register_system`
    - `clear_registry`
    - 기존 `set_speed_index`, `reset_accumulator` 유지.
  - Dictionary 파싱 helper 추가.

## API / Signal / Schema Changes
### Runtime bridge API
- Added: `runtime_get_registry_snapshot() -> Array[Dictionary]`
- Added: `runtime_clear_registry() -> void`

### Runtime commands (Bus v2)
- Added command: `register_system` payload
  - `name: String`
  - `priority: int`
  - `tick_interval: int`
  - `active: bool`
- Added command: `clear_registry`

## Verification
- `cd rust && cargo check -p sim-bridge` : PASS
- `cd rust && cargo test -p sim-bridge --lib` : PASS (24 passed)
- Godot headless check: 미실행 (`godot` binary 없음)

## Rust Migration Progress
- Previous: 62% complete / 38% remaining
- Current: 68% complete / 32% remaining
- Delta: +6%

## Notes
- 현재 단계는 레지스트리 메타데이터 브릿지이며, 실제 시스템 로직의 Rust 실행 전환은 후속 단계에서 진행한다.
