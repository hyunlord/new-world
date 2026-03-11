# WS-REF-004B Report

## Implementation Intent
This ticket removes the remaining legacy string-key authority from the runtime system registry. The registry now uses typed Rust identities only, which makes scheduler registration deterministic and eliminates alias layers that were still normalizing script names into runtime system keys.

## How It Was Implemented
- Added `RuntimeSystemId` and the typed default manifest in [rust/crates/sim-bridge/src/runtime_system.rs](/Users/rexxa/github/new-world-wt/codex-refactor-runtime-registry/rust/crates/sim-bridge/src/runtime_system.rs).
- Refactored [rust/crates/sim-bridge/src/runtime_registry.rs](/Users/rexxa/github/new-world-wt/codex-refactor-runtime-registry/rust/crates/sim-bridge/src/runtime_registry.rs) so registry entries store `RuntimeSystemId` instead of `name/system_key` pairs.
- Refactored [rust/crates/sim-bridge/src/runtime_commands.rs](/Users/rexxa/github/new-world-wt/codex-refactor-runtime-registry/rust/crates/sim-bridge/src/runtime_commands.rs) to remove the `register_system` and `clear_registry` command paths. The command layer now only handles speed/accumulator/compute-domain controls plus typed registry snapshot export.
- Updated [rust/crates/sim-bridge/src/debug_api.rs](/Users/rexxa/github/new-world-wt/codex-refactor-runtime-registry/rust/crates/sim-bridge/src/debug_api.rs) to map perf/debug metadata through typed registry names.
- Removed the unused `runtime_clear_registry` wrapper from [scripts/core/simulation/sim_bridge.gd](/Users/rexxa/github/new-world-wt/codex-refactor-runtime-registry/scripts/core/simulation/sim_bridge.gd).

## What Feature It Adds
The runtime registry now has a single typed authority path. Boot and scheduler setup no longer depend on legacy string keys, script-path normalization, or GDScript-side registry mutation. Debug and validation consumers still get readable names, but those names are display metadata, not scheduler identity.

## Verification After Implementation
- Static legacy-key scan: no remaining `system_key`, `register_system`, `runtime_system_key_from_name`, `runtime_supports_rust_system`, or `clear_registry` paths in active bridge/core simulation code.
- `cargo build -p sim-bridge`
- `cargo test -p sim-bridge`
- `cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
- Godot headless boot

## In-Game Checks (한국어)
- 게임 시작 후 부트가 정상적으로 올라오고 runtime registry mismatch 경고가 없는지 본다.
- Probe/Sandbox 어느 쪽이든 시스템 등록 때문에 시작이 실패하지 않는지 본다.
- 디버그/검증 경로에서 registry snapshot을 읽을 때 시스템 이름은 보이지만, 더 이상 문자열 키 기반 등록이 필요하지 않은지 확인한다.
- 속도 변경, 일시정지, compute domain 모드 변경 같은 runtime 명령은 계속 동작하는지 본다.
- 이상하면 부팅 직후 registry count mismatch, all_rust 실패, 또는 runtime command 처리 실패 경고가 보일 수 있다.
