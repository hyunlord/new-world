# WS-REF-004A Report

## Implementation Intent
The goal of WS-REF-004A is to make the Godot boot pipeline tell the truth about authority:

- Rust ECS owns simulation runtime init and ticking
- Godot owns shell, UI, rendering, and bootstrap handoff only

This pass stayed focused on boot authority. It did not attempt a full shadow-state deletion across every legacy helper. Instead, it verified that boot no longer leaks scheduler ownership into GDScript and documented the remaining residual helpers honestly.

## How It Was Implemented
- Re-audited the current boot path through:
  - [project.godot](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/project.godot)
  - [scenes/main/main.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scenes/main/main.gd)
  - [scripts/core/simulation/simulation_engine.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/core/simulation/simulation_engine.gd)
  - [scripts/core/simulation/sim_bridge.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/core/simulation/sim_bridge.gd)
- Updated [boot_pipeline_map.md](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/boot_pipeline_map.md) to describe the live boot order and Rust-owned registry path.
- Added [godot_authority_scan.md](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/godot_authority_scan.md) to classify current Godot scripts into `simulation_authority_leak`, `ui_only`, `bridge_only`, and `render_only`.
- Added [authority_boundary_spec.md](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/authority_boundary_spec.md) to lock the allowed Godot <-> sim-bridge <-> Rust boundary.
- Refreshed [verify_boot_authority.md](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/verify_boot_authority.md) so verification matches the current repo rather than an old worktree snapshot.

No new boot code path was required in this branch because the Rust-owned boot cutover is already present on top of `lead/main`.

## What Feature It Adds
This ticket adds a reliable authority audit surface for startup:

- a developer can trace the boot order clearly
- active vs residual Godot responsibilities are explicit
- the expected boundary is written down in one place
- verification commands directly prove that boot authority lives in Rust

That removes ambiguity around whether Godot is still secretly owning simulation startup.

## Verification After Implementation
- `git diff --check`
- `rg -n "register_system\\(" scenes/main/main.gd scripts/core/simulation/simulation_engine.gd`
- `rg -n "runtime_clear_registry" scripts/core/simulation`
- `rg -n 'const .*preload\\("res://scripts/(systems|ai)/' scenes/main/main.gd`
- `cd rust && cargo build -p sim-bridge`
- `cd rust && cargo test -p sim-bridge`
- `cd rust && cargo test --workspace`
- `cd rust && cargo clippy --workspace -- -D warnings`
- `"/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot" --headless --path /Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a --quit`

Result:
- static boot-authority scans passed
- Rust build/tests/clippy passed
- Godot headless boot exited with code `0`

Residual note:
- Some shadow/bootstrap managers still exist and are instantiated by the shell.
- They are documented as residual leakage risks, but they are not part of the active boot scheduler authority path.

## In-Game Checks (한국어)
- 게임 시작 직후 시뮬레이션이 정상적으로 진행되는지 본다.
- 시작 시점에 GDScript 시스템 등록 실패나 registry rebuild 경고가 새로 뜨지 않는지 확인한다.
- Probe/Sandbox 시작 후 에이전트가 실제로 움직이고 tick이 진행되는지 본다.
- UI는 뜨는데 시뮬레이션만 멈추면 Rust runtime init 또는 boot registry 경로를 다시 봐야 한다.
- 반대로 legacy GDScript 시스템이 다시 authority를 잡은 흔적이 보이면 `main.gd` 또는 `simulation_engine.gd`에서 boot registration 경로가 되살아난 것이다.
