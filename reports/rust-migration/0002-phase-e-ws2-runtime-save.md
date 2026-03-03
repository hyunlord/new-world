# 0002 - Phase E ws2 runtime save pipeline

## Summary
Rust 런타임 기반 `.ws2` 저장/로드 파이프라인을 추가했다. 포맷은 `magic+version+checksum+payload_len+zstd(bincode(snapshot))` 구조를 사용한다.

## Files Changed
- `rust/Cargo.toml`
  - workspace dependency 추가: `bincode`, `zstd`, `crc32fast`.
- `rust/crates/sim-bridge/Cargo.toml`
  - sim-bridge에 ws2 직렬화/압축/체크섬 dependency 연결.
- `rust/crates/sim-engine/src/engine.rs`
  - `restore_from_snapshot(&EngineSnapshot)` 추가.
  - timeline 복원 테스트 추가.
- `rust/crates/sim-bridge/src/lib.rs`
  - ws2 codec(`encode_ws2_blob`, `decode_ws2_blob`) 추가.
  - `WorldSimRuntime.runtime_apply_snapshot()` 추가.
  - `WorldSimRuntime.runtime_save_ws2(path)` 추가.
  - `WorldSimRuntime.runtime_load_ws2(path)` 추가.
  - ws2 roundtrip/invalid magic 테스트 추가.
- `scripts/core/simulation/sim_bridge.gd`
  - `runtime_apply_snapshot`, `runtime_save_ws2`, `runtime_load_ws2` 래퍼 추가.
- `scripts/core/simulation/save_manager.gd`
  - Rust primary 모드에서 ws2 저장/로드 분기 추가.
  - 메타에 `save_backend` 추가 (`legacy_v8` / `rust_ws2`).

## API / Signal / Schema Changes
### Rust runtime API
- Added: `runtime_apply_snapshot(snapshot_bytes: PackedByteArray) -> bool`
- Added: `runtime_save_ws2(path: String) -> bool`
- Added: `runtime_load_ws2(path: String) -> bool`

### Save schema
- Added ws2 binary file: `sim.ws2`
- ws2 header layout:
  - `magic[4] = "WS2\0"`
  - `version(u16, LE)`
  - `flags(u16, LE)`
  - `checksum(u32, LE, crc32 of compressed payload)`
  - `payload_len(u32, LE)`
  - `payload = zstd(bincode(EngineSnapshot))`
- `meta.json` now includes:
  - `save_backend: "legacy_v8" | "rust_ws2"`

## Verification
- `cd rust && cargo check -p sim-bridge -p sim-engine` : PASS
- `cd rust && cargo test -p sim-engine --lib` : PASS (20 passed)
- `cd rust && cargo test -p sim-bridge --lib` : PASS (24 passed)
- Godot headless check: 미실행 (`godot` binary 없음)

## Rust Migration Progress
- Previous: 34% complete / 66% remaining
- Current: 42% complete / 58% remaining
- Delta: +8%

## Notes
- 현재 ws2는 timeline(snapshot) 중심 복원이며, 전체 ECS/entity/world full-state restore는 후속 단계에서 확장한다.
- Legacy 저장 경로는 Rust primary 외 모드에서 계속 동작한다.
