# 0008 - Phase F runtime compute sync

## Summary
ComputeBackend의 도메인별 모드 변경이 Bus v2 command를 통해 Rust runtime 상태로 동기화되도록 연결했다. 또한 Rust primary 모드에서 pause 상태일 때도 runtime command가 누락되지 않도록 엔진 update 흐름을 수정했다.

## Files Changed
- `scripts/core/simulation/compute_backend.gd`
  - runtime command enqueue 추가 (`set_compute_mode_all`, `set_compute_domain_mode`).
  - startup 시 runtime mode 동기화 루틴 추가.
- `scripts/core/simulation/simulation_engine.gd`
  - Rust primary에서 pause 상태여도 `runtime_tick_frame(..., paused=true)` 호출.
  - shadow 모드 paused 경로 추가.
- `scripts/core/simulation/sim_bridge.gd`
  - `runtime_get_compute_domain_modes()` 래퍼 추가.
- `rust/crates/sim-bridge/src/lib.rs`
  - runtime state에 `compute_domain_modes` 저장 추가.
  - `runtime_get_compute_domain_modes()` API 추가.
  - `runtime_apply_commands_v2()` 명령 처리 확장:
    - `set_compute_domain_mode`
    - `set_compute_mode_all`

## API / Signal / Schema Changes
### Runtime bridge API
- Added: `runtime_get_compute_domain_modes() -> Dictionary`

### Runtime commands
- Added command: `set_compute_domain_mode` payload
  - `domain: String`
  - `mode: String(cpu|gpu_auto|gpu_force)`
- Added command: `set_compute_mode_all` payload
  - `mode: String(cpu|gpu_auto|gpu_force)`

## Verification
- `cd rust && cargo check -p sim-bridge` : PASS
- `cd rust && cargo test -p sim-bridge --lib` : PASS (24 passed)
- Godot headless check: 미실행 (`godot` binary 없음)

## Rust Migration Progress
- Previous: 72% complete / 28% remaining
- Current: 76% complete / 24% remaining
- Delta: +4%

## Notes
- 이 단계는 compute mode 상태 동기화 기반이며, domain별 실제 GPU kernel 적용 범위 확장은 후속 단계.
