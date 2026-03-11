# WS-REF-004A Report

## Implementation Intent

This ticket focused on one narrow question: does boot still leak simulation authority into Godot?

The highest-value ambiguity to remove was not gameplay logic in general. It was boot-time truth:

- who loads data first
- who builds the runtime first
- who owns scheduler registration
- who owns the first active simulation tick

This stayed small on purpose. It did not attempt a full deletion of every shadow manager. It only tightened the active boot path and refreshed the audit artifacts so they match the current repository truth.

## How It Was Implemented

- Re-scanned the boot entry points:
  - [project.godot](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/project.godot)
  - [scenes/main/main.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scenes/main/main.gd)
  - [scripts/core/simulation/simulation_engine.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/core/simulation/simulation_engine.gd)
  - [scripts/core/simulation/sim_bridge.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/core/simulation/sim_bridge.gd)
- Refreshed the boot truth docs:
  - [boot_pipeline_map.md](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/boot_pipeline_map.md)
  - [godot_authority_scan.md](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/godot_authority_scan.md)
  - [simulation_ownership_map.md](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/simulation_ownership_map.md)
  - [verify_boot_authority.md](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/verify_boot_authority.md)
- Tightened [main.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scenes/main/main.gd) so Rust runtime registry validation happens immediately after `SimulationEngine` startup and aborts boot before shadow/bootstrap managers are created if authority validation fails.

## What Feature It Adds

This ticket adds a safer and more legible boot contract.

After this change:

- Rust authority is checked before Godot builds the rest of the shell/bootstrap graph
- startup fails fast when the runtime registry is not fully Rust-backed
- boot ownership is documented with explicit responsibility maps instead of assumption

That makes it much easier to trust that the active simulation boot path is Rust-first, even though residual hybrid helpers still remain on disk.

## Verification After Implementation

- `git diff --check`
- `cargo check --workspace`
- `cargo build -p sim-bridge`
- `cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
- Godot headless boot:
  - `"/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot" --headless --path /Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a --quit`

Result:

- Rust build/test/lint passed
- Godot headless boot passed
- static boot-authority scans passed

## Remaining Authority Risks

- `main.gd` still instantiates mutable shadow/bootstrap managers after Rust validation succeeds
- `main.gd` still wires `ChronicleSystem` and `SimulationBus` observer compatibility hooks after bootstrap
- legacy helper scripts under `scripts/core/**` and `scripts/systems/**` still exist on disk
- some UI/panel paths still depend on legacy managers for fallback or compatibility behavior
- boot now fails fast on registry mismatch, but there is no richer fallback/error UI yet

So the accurate state is:

- active boot authority: Rust
- active tick authority: Rust
- full shadow deletion: not complete

## In-Game Checks (한국어)

- 게임이 정상적으로 부팅되는지 확인한다.
- 시작 직후 에이전트가 정상적으로 생성되고 움직이는지 확인한다.
- 시뮬레이션 tick이 Rust에서만 실행되는지 확인한다.
- Rust runtime registry가 비정상일 때는 부팅이 계속 진행되지 않아야 한다.
- UI와 렌더링은 정상 동작하지만, Godot이 simulation state의 source of truth가 아닌지 확인한다.
- 여전히 일부 legacy manager 기반 패널이 남아 있어도, 그것이 active tick authority를 다시 가져오지 않는지 확인한다.
