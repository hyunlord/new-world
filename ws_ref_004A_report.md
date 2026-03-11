# ws_ref_004A_report

## Implementation Intent

This ticket focused on one architectural ambiguity: Godot boot still looked like the source of simulation truth because it instantiated and registered legacy GDScript systems during startup.

That was the wrong ownership model for the current WorldSim architecture. The correct boundary is:

`Godot shell -> sim-bridge -> Rust runtime registry -> Rust ECS tick`

So the implementation targeted the boot path only:

- remove boot-time simulation authority leakage from `main.gd`
- stop `simulation_engine.gd` from acting like a GDScript registry owner
- make Rust register the default startup manifest itself

This stayed intentionally narrow. It did not try to delete every legacy GDScript system file on disk. It only removed their role in active boot authority.

## How It Was Implemented

The refactor had three parts.

### 1. Rust default runtime manifest

[runtime_registry.rs](/Users/rexxa/github/new-world-wt/codex-refactor-boot-authority/rust/crates/sim-bridge/src/runtime_registry.rs) now exposes:

- a default runtime manifest
- a shared `upsert_runtime_system_entry(...)`
- `register_default_runtime_systems(...)`

This makes Rust the authoritative startup source for simulation-system registration.

### 2. Bridge exposure

[lib.rs](/Users/rexxa/github/new-world-wt/codex-refactor-boot-authority/rust/crates/sim-bridge/src/lib.rs) now exposes `runtime_register_default_systems()`, and [sim_bridge.gd](/Users/rexxa/github/new-world-wt/codex-refactor-boot-authority/scripts/core/simulation/sim_bridge.gd) forwards it to GDScript.

### 3. Godot boot cleanup

[simulation_engine.gd](/Users/rexxa/github/new-world-wt/codex-refactor-boot-authority/scripts/core/simulation/simulation_engine.gd) now:

- calls `runtime_init(...)`
- calls `runtime_register_default_systems()`
- validates the Rust registry snapshot
- ticks via `runtime_tick_frame(...)`

It no longer:

- clears the Rust registry after init
- keeps GDScript-side startup registry payload lists
- builds the authoritative boot registry from local system instances

[main.gd](/Users/rexxa/github/new-world-wt/codex-refactor-boot-authority/scenes/main/main.gd) no longer instantiates or registers the legacy GDScript simulation systems during startup. It remains a shell/bootstrap scene for:

- setup flow
- camera/UI/rendering init
- save/load plumbing
- bootstrap payload handoff to Rust

## What Feature It Adds

This refactor adds a concrete architectural guarantee:

- startup simulation-system registration is Rust-owned
- startup simulation ticking is Rust-owned
- Godot boot no longer pretends to be the simulation registry source of truth

For developers, that means the boot path is now much easier to reason about:

- there is one authoritative runtime-system manifest
- registry truth lives in Rust
- Godot boot is reduced to shell/bootstrap responsibilities

## Verification After Implementation

The following checks were used after implementation:

- `git diff --check`
- `cd rust && cargo test -p sim-bridge`
- `cd rust && cargo test --workspace`
- `cd rust && cargo clippy --workspace -- -D warnings`
- `cd rust && cargo build -p sim-bridge`
- `/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot --headless --path /Users/rexxa/github/new-world-wt/codex-refactor-boot-authority --quit`

Static verification also checked:

- no `register_system(...)` calls remain in [main.gd](/Users/rexxa/github/new-world-wt/codex-refactor-boot-authority/scenes/main/main.gd)
- [simulation_engine.gd](/Users/rexxa/github/new-world-wt/codex-refactor-boot-authority/scripts/core/simulation/simulation_engine.gd) no longer calls `runtime_clear_registry`
- startup now calls `runtime_register_default_systems()`
- the Rust default manifest test passes and reports Rust-backed registry entries
- Godot headless boot exits with code `0` after the debug `sim-bridge` library is built; existing UID fallback and shutdown leak warnings remain pre-existing project warnings

## In-Game Checks (한국어)

- 게임 시작 직후 시뮬레이션이 정상적으로 계속 진행되는지 본다.
- Probe/Sandbox 어느 쪽이든 시작 후 에이전트가 실제로 움직이고 행동하는지 확인한다.
- 시작 로그에 GDScript 시스템을 다시 등록했다는 경고가 없는지 본다.
- 부트 직후 디버그/검증 오버레이가 있다면 행동, need 변화, 건설 상태가 계속 들어오는지 확인한다.
- UI는 뜨는데 시뮬레이션만 멈추면 Rust runtime registry 초기화 경로를 다시 봐야 한다.
- 반대로 legacy GDScript 시스템이 다시 authority를 잡은 흔적이 보이면 `main.gd`나 `simulation_engine.gd`에서 boot registration 경로가 되살아난 것이다.
