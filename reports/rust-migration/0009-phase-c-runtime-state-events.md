# 0009 - Phase C runtime state events

## Summary
Rust primary 경로에서 pause/resume/speed 상태 변화를 Rust runtime 이벤트로 표준화하고, Bus v2/v1 어댑터를 통해 재방출되도록 정리했다. `SimulationEngine`는 Rust primary일 때 v1 신호를 직접 emit하지 않고 runtime 이벤트 경로를 우선 사용한다.

## Files Changed
- `rust/crates/sim-engine/src/events.rs`
  - `GameEvent::SpeedChanged { speed_index }` 이벤트 타입 추가.
  - 이벤트 이름 매핑(`speed_changed`) 추가.
  - 안정성 테스트(`speed_changed_name_is_stable`) 추가.
- `rust/crates/sim-bridge/src/lib.rs`
  - Bus v2 이벤트 타입 ID `4`(speed_changed) 추가.
  - runtime state에 `paused` 상태 필드 추가.
  - `runtime_tick_frame()`에서 상태 전이 감지 시 이벤트 emit:
    - pause 상태 변경 -> `SimulationPaused` / `SimulationResumed`
    - speed index 변경 -> `SpeedChanged`
  - v2 payload 변환에 `speed_index` 포함.
- `scripts/core/simulation/simulation_bus_v2.gd`
  - `EVENT_SPEED_CHANGED` 상수 추가.
- `scripts/core/simulation/simulation_bus.gd`
  - v2 이벤트 ID `4` 수신 시 v1 `speed_changed` 신호 재방출 추가.
- `scripts/core/simulation/simulation_engine.gd`
  - Rust primary 모드에서 `toggle_pause()` / `set_speed()`의 v1 direct emit 제거.
  - 동일 상태 변경은 runtime v2 이벤트 경로를 통해 전달되도록 조정.

## API / Signal / Schema Changes
### Runtime event type IDs
- Added: `event_type_id = 4` (`speed_changed`)

### Runtime event payload
- `speed_changed` payload
  - `speed_index: int`

### Bus compatibility
- `SimulationBusV2.event_emitted(4, payload, tick)` -> `SimulationBus.speed_changed(speed_index)` 재방출

## Verification
- `cd rust && cargo check -p sim-bridge` : PASS
- `cd rust && cargo test -p sim-engine --lib` : PASS (21 passed)
- `cd rust && cargo test -p sim-bridge --lib` : PASS (24 passed)
- Godot headless check: 미실행 (`godot` binary 없음)

## Rust Migration Progress
- Previous: 76% complete / 24% remaining
- Current: 80% complete / 20% remaining
- Delta: +4%

## Notes
- 이 단계는 Rust primary 오케스트레이션 신호 경로를 Bus v2 기준으로 정렬하는 작업이다.
- GDScript shadow/gdscript 모드의 기존 direct 신호 경로는 회귀 방지를 위해 유지했다.
