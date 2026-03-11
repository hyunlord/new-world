# Verify Simulation Authority Cutover

## Static Scans
- `rg -n "register_system\\(|runtime_clear_registry|execute_tick\\(|spawn_entity\\(" scenes/main scripts/core/simulation scripts/systems -g '*.gd'`
  - Expect: no active boot/runtime string registration paths; `spawn_entity` remains only as deprecated local helper definition
- `rg -n "runtime_tick_frame|runtime_register_default_systems|runtime_bootstrap_world|runtime_apply_commands_v2" scenes/main scripts/core -g '*.gd'`
  - Expect: Rust bridge/tick path is present
- `rg -n "NameGenerator\\.init|NameGenerator\\.save_registry|NameGenerator\\.load_registry" scenes/main/main.gd scripts/core -g '*.gd'`
  - Expect: no active gameplay boot/save hooks remain

## Build / Test
- `cd rust && cargo check --workspace`
- `cd rust && cargo test --workspace`
- `cd rust && cargo clippy --workspace -- -D warnings`

## Godot Validation
- `Godot --headless --path /Users/rexxa/github/new-world-wt/codex-refactor-sim-authority-cutover --quit`
  - Expect: exit code 0
- `Godot --headless --path /Users/rexxa/github/new-world-wt/codex-refactor-sim-authority-cutover --script res://tests/test_stage1.gd`
  - Expect: headless runtime harness passes

## Acceptance Criteria
- active runtime tick occurs only through Rust bridge calls
- active boot path validates Rust registry before legacy shadow composition
- active boot path does not initialize deprecated local spawn helpers
- active camera/building renderer boot path prefers runtime-backed state
- Godot contains no active runtime system registration or registry-clearing logic
