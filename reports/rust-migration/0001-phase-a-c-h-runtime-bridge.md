# 0001 - Phase A/C/H runtime bridge scaffold

## Summary
Rust 런타임 진입점(`WorldSimRuntime`)과 GDScript 연동 래퍼를 추가하고, Bus v2 병행 계층 및 shadow 비교 기반을 연결했다.

## Files Changed
- `rust/crates/sim-bridge/src/lib.rs`
  - `WorldSimRuntime` Godot class 추가.
  - `runtime_init(seed, config_json)` 구현.
  - `runtime_tick_frame(delta_sec, speed_index, paused)` 구현.
  - `runtime_get_snapshot()` 구현.
  - `runtime_export_events_v2()` 구현.
  - `runtime_apply_commands_v2(commands)` 시그니처 구현(초기 no-op).
  - SimEngine + EventBus 기반 tick 처리/이벤트 캡처 추가.
- `scripts/core/simulation/sim_bridge.gd`
  - Rust runtime 클래스 인스턴스 해석 로직 추가.
  - runtime API 래퍼(`runtime_*`) 추가.
  - 기존 `_get_native_bridge()` 들여쓰기 오류 구간 수정.
- `scripts/core/simulation/simulation_engine.gd`
  - 런타임 모드(gdscript/rust_shadow/rust_primary) 분기 추가.
  - Rust primary 업데이트 경로 추가.
  - Rust shadow 비교 경로 추가(틱 mismatch 카운트/경고).
  - Bus v2 이벤트 전달 경로 추가.
- `scripts/core/simulation/simulation_bus_v2.gd` (new)
  - `event_emitted(event_type_id, payload, tick)` 신호 추가.
  - `ui_command(command_id, payload)` 신호 추가.
  - 런타임 이벤트 emit helper 추가.
- `scripts/core/simulation/simulation_bus.gd`
  - Bus v2 이벤트 수신 및 v1 신호 재방출 어댑터 추가.
- `scripts/core/simulation/game_config.gd`
  - `SIM_RUNTIME_MODE_*` 상수 추가 및 기본 모드(`rust_shadow`) 설정.
- `project.godot`
  - `SimulationBusV2` autoload 등록.

## API / Signal / Schema Changes
### Rust bridge API
- Added: `WorldSimRuntime.runtime_init(seed: int, config_json: String) -> bool`
- Added: `WorldSimRuntime.runtime_tick_frame(delta_sec: float, speed_index: int, paused: bool) -> Dictionary`
- Added: `WorldSimRuntime.runtime_get_snapshot() -> PackedByteArray`
- Added: `WorldSimRuntime.runtime_export_events_v2() -> Array[Dictionary]`
- Added: `WorldSimRuntime.runtime_apply_commands_v2(commands: Array[Dictionary]) -> void`

### Bus signals
- Added v2 signals in `simulation_bus_v2.gd`:
  - `event_emitted(event_type_id: int, payload: Dictionary, tick: int)`
  - `ui_command(command_id: StringName, payload: Dictionary)`
- Modified v1 behavior in `simulation_bus.gd`:
  - v2 tick/pause/resume 이벤트를 v1 `tick_completed`/`pause_changed`로 재방출.

### Save schema
- 변경 없음 (이번 커밋에서는 Save v2 미적용).

## Verification
- `cd rust && cargo check -p sim-bridge` : PASS
- `cd rust && cargo test -p sim-bridge --lib` : PASS (22 passed)
- `godot --headless --check-only --path ...` : FAIL (`godot` 실행 파일 미설치)

## Rust Migration Progress
- Previous: 18% complete / 82% remaining
- Current: 34% complete / 66% remaining
- Delta: +16%

## Notes
- 이번 단계는 Phase A/C/H의 기반 구현이다.
- Runtime command 처리, 시스템 레지스트리 Rust 전량 이관, Save v2(.ws2), GPU 광범위 커널, Fluent/ICU 전환은 후속 커밋에서 확장한다.
