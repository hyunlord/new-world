# 0004 - Phase H shadow reporter and command pipeline

## Summary
Shadow 비교를 지속 기록하는 리포터를 추가하고, Bus v2 명령 큐를 Rust runtime command API까지 연결했다.

## Files Changed
- `scripts/core/simulation/runtime_shadow_reporter.gd` (new)
  - GD vs Rust shadow 비교 프레임 누적 통계 기록.
  - 주기적 JSON 리포트 출력 (`user://reports/rust_shadow/latest.json`).
- `scripts/core/simulation/game_config.gd`
  - shadow 리포트 경로/주기 상수 추가.
- `scripts/core/simulation/simulation_bus_v2.gd`
  - runtime command queue 추가.
  - `queue_runtime_command()` / `drain_runtime_commands()` 추가.
- `scripts/core/simulation/simulation_engine.gd`
  - shadow 모드에서 mismatch/event 통계 기록 연동.
  - runtime command drain/apply 연동.
  - speed 변경 시 runtime command enqueue 연동.
- `rust/crates/sim-bridge/src/lib.rs`
  - `runtime_apply_commands_v2()` 구현(no-op 제거).
  - 지원 명령: `set_speed_index`, `reset_accumulator`.

## API / Signal / Schema Changes
### Bus v2 command API
- Added: `SimulationBusV2.queue_runtime_command(command_id, payload)`
- Added: `SimulationBusV2.drain_runtime_commands() -> Array`

### Shadow report output
- Path: `user://reports/rust_shadow/latest.json`
- Fields:
  - `current_tick`
  - `frames`, `mismatch_frames`
  - `max_tick_delta`, `avg_tick_delta`
  - `max_event_delta`, `avg_event_delta`
  - `generated_at_unix_ms`

## Verification
- `cd rust && cargo check -p sim-bridge` : PASS
- `cd rust && cargo test -p sim-bridge --lib` : PASS (24 passed)
- Godot headless check: 미실행 (`godot` binary 없음)

## Rust Migration Progress
- Previous: 50% complete / 50% remaining
- Current: 57% complete / 43% remaining
- Delta: +7%

## Notes
- command pipeline은 구조를 연결했고, 실제 command set은 단계적으로 확장 예정.
- shadow report는 현재 tick/event count 정합성 중심이며 full state diff는 후속 단계에서 추가.
