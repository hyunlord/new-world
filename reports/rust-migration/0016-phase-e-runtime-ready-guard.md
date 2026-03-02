# 0016 - Phase E ws2 runtime-ready guard

## Summary
ws2 저장/로드 준비 판정을 강화했다. SaveManager가 브리지 메서드 존재 여부만 보는 대신 Rust runtime 실제 초기화 상태까지 확인하도록 변경했다.

## Files Changed
- `rust/crates/sim-bridge/src/lib.rs`
  - Added runtime API: `runtime_is_initialized() -> bool`
- `scripts/core/simulation/sim_bridge.gd`
  - Added wrapper: `runtime_is_initialized() -> bool`
- `scripts/core/simulation/save_manager.gd`
  - `_is_ws2_runtime_ready()`에서 `SimBridge.runtime_is_initialized()` 체크 추가

## API / Signal / Schema Changes
### Runtime bridge API
- Added: `runtime_is_initialized() -> bool`

## Verification
- `cd rust && cargo check -p sim-bridge` : PASS
- `cd rust && cargo test -p sim-bridge --lib` : PASS (26 passed)
- `cd rust && cargo test -p sim-engine --lib` : PASS (21 passed)
- `godot --headless --check-only` : 미실행 (`godot` binary 없음)

## Rust Migration Progress
- Previous: 96% complete / 4% remaining
- Current: 98% complete / 2% remaining
- Delta: +2%

## Notes
- runtime 미초기화 상태에서 ws2 저장/로드 시도를 더 이른 단계에서 차단한다.
