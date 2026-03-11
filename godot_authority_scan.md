# Godot Authority Scan

## Scan Scope

- `scripts/core`
- `scripts/systems`
- `scripts/agents` — not present
- `scripts/world` — not present
- [scenes/main/main.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scenes/main/main.gd)

## Classification

### `bridge_only`

- [scripts/core/simulation/sim_bridge.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/core/simulation/sim_bridge.gd)
- [scripts/core/simulation/simulation_engine.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/core/simulation/simulation_engine.gd)
- [scripts/core/simulation/simulation_bus_v2.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/core/simulation/simulation_bus_v2.gd)

Evidence:
- runtime init, runtime tick relay, runtime bootstrap, command relay, snapshot/detail getters
- no active per-frame gameplay mutation owned locally

### `render_only`

- `scripts/ui/**`
- `scripts/rendering/**`

Evidence:
- draw runtime snapshots, panels, overlays, camera state, and debug visuals

### `ui_only`

- [scripts/core/simulation/event_logger.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/core/simulation/event_logger.gd)
- [scripts/systems/record/chronicle_system.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/systems/record/chronicle_system.gd)
- [scripts/systems/record/memory_system.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/systems/record/memory_system.gd)

Evidence:
- observer/log/presentation behavior only
- not in the active boot scheduler path, but `main.gd` still hooks `ChronicleSystem` and `SimulationBus` for observer compatibility during bootstrap

### `simulation_authority_leak`

- [scenes/main/main.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scenes/main/main.gd)
  - still instantiates shadow/bootstrap managers and map/resource helpers during startup
- [scripts/core/entity/entity_manager.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/core/entity/entity_manager.gd)
- [scripts/core/settlement/building_manager.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/core/settlement/building_manager.gd)
- [scripts/core/settlement/settlement_manager.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/core/settlement/settlement_manager.gd)
- [scripts/core/world/resource_map.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/core/world/resource_map.gd)
- [scripts/core/social/relationship_manager.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/core/social/relationship_manager.gd)
- [scripts/core/social/reputation_manager.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/core/social/reputation_manager.gd)
- residual legacy logic under `scripts/systems/**`

Evidence:
- these files create or mutate simulation-shaped shadow state
- they are not the active Rust scheduler, but they remain leakage risks and hybrid-architecture debt

## Active Runtime Truth

- active scheduler owner: Rust
- active frame tick owner: Rust
- active registry owner: Rust
- active Godot role: shell, bridge, setup, UI, rendering

## Remaining Leak Summary

The repository no longer gives active simulation tick ownership to Godot during boot, but Godot still creates mutable shadow/bootstrap objects. That means:

- `simulation_tick_owner = rust_only`
- `boot_shell = still hybrid`
- `observer_compatibility_hooks = still present`

This ticket secures boot authority, but it does not finish the full shadow-manager deletion program.
