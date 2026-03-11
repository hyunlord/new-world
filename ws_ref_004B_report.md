# WS-REF-004B Report

## Implementation Intent
This ticket removes the remaining string-key dependency from the runtime system registry itself. The runtime was already mostly typed, but registry ordering and bridge/debug surfaces still leaned on old `*_system` names strongly enough to blur the authority boundary. The goal of this pass was to make `RuntimeSystemId` the only scheduler/registry identity and demote remaining strings to non-authoritative compatibility labels.

## How It Was Implemented
- [rust/crates/sim-bridge/src/runtime_system.rs](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004b/rust/crates/sim-bridge/src/runtime_system.rs)
  - split registry/display concerns:
    - `display_label()` for debug/UI readability
    - `perf_label()` for internal engine perf lookup only
- [rust/crates/sim-bridge/src/runtime_registry.rs](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004b/rust/crates/sim-bridge/src/runtime_registry.rs)
  - changed deterministic tie-break from string name ordering to `RuntimeSystemId`
- [rust/crates/sim-bridge/src/runtime_commands.rs](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004b/rust/crates/sim-bridge/src/runtime_commands.rs)
  - registry snapshot now exposes display names layered on top of typed `system_id`, not legacy registry key strings
- [rust/crates/sim-bridge/src/debug_api.rs](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004b/rust/crates/sim-bridge/src/debug_api.rs)
  - perf/debug output now iterates typed registered systems first and only uses `perf_label()` as an internal compatibility lookup
- Added/refreshed:
  - [runtime_system_registry_map.md](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004b/runtime_system_registry_map.md)
  - [runtime_system_classification.md](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004b/runtime_system_classification.md)
  - [verify_runtime_registry.md](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004b/verify_runtime_registry.md)

## What Feature It Adds
The runtime registry now has a cleaner authority boundary:

- scheduler identity = `RuntimeSystemId`
- boot manifest identity = `RuntimeSystemId`
- registry sorting = `RuntimeSystemId`
- bridge/debug readability = display labels only

That means legacy `*_system` strings are no longer acting as registry keys even indirectly.

## Verification After Implementation
- Static legacy-key scan:
  - no `system_key`
  - no `register_system`
  - no `runtime_system_key_from_name`
  - no `runtime_supports_rust_system`
  - no `clear_registry`
  - no `registry_name()`
- `cargo build -p sim-bridge`
- `cargo test -p sim-bridge`
- `cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
- Godot headless boot

## In-Game Checks (한국어)
- 게임 시작 후 부트가 정상적으로 올라오고 runtime registry mismatch 경고가 없는지 본다.
- Probe/Sandbox 어느 쪽이든 시스템 등록 때문에 시작이 실패하지 않는지 본다.
- 디버그/검증 경로에서 registry snapshot을 읽을 때 시스템 이름은 표시용으로만 보이고, 실제 식별은 `system_id` 기준으로 읽히는지 확인한다.
- 속도 변경, 일시정지, compute domain 모드 변경 같은 runtime 명령은 계속 동작하는지 본다.
- 이상하면 부팅 직후 registry count mismatch, all_rust 실패, 또는 runtime command 처리 실패 경고가 보일 수 있다.
