# Godot Authority Scan

## Scan Scope
- `scripts/core`
- `scripts/systems`
- `scripts/agents` — not present in this repo
- `scripts/world` — not present in this repo
- [scenes/main/main.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scenes/main/main.gd)

## Classification

### bridge_only
- [scripts/core/simulation/sim_bridge.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/core/simulation/sim_bridge.gd)
- [scripts/core/simulation/simulation_engine.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/core/simulation/simulation_engine.gd)
- [scripts/core/simulation/simulation_bus.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/core/simulation/simulation_bus.gd)
- [scripts/core/simulation/simulation_bus_v2.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/core/simulation/simulation_bus_v2.gd)
- [scripts/core/simulation/runtime_shadow_reporter.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/core/simulation/runtime_shadow_reporter.gd)

Evidence:
- These files initialize the native runtime, relay commands/events, export snapshots, or compare shadow/runtime state.
- They do not own the active simulation scheduler.

### ui_only
- [scenes/main/main.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scenes/main/main.gd)
- [scripts/core/simulation/event_logger.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/core/simulation/event_logger.gd)
- [scripts/systems/record/chronicle_system.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/systems/record/chronicle_system.gd)
- [scripts/systems/record/memory_system.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/systems/record/memory_system.gd)

Evidence:
- These files observe or log runtime data.
- They do not drive per-frame simulation updates.

### render_only
- `scripts/ui/**`
- `scripts/rendering/**`

Evidence:
- These paths draw world snapshots, panels, overlays, and camera state sourced from Rust runtime outputs.

### simulation_authority_leak
Residual mutable shadow/bootstrap helpers still on disk:
- [scripts/core/entity/entity_manager.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/core/entity/entity_manager.gd)
- [scripts/core/settlement/building_manager.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/core/settlement/building_manager.gd)
- [scripts/core/settlement/settlement_manager.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/core/settlement/settlement_manager.gd)
- [scripts/core/social/relationship_manager.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/core/social/relationship_manager.gd)
- [scripts/systems/biology/personality_generator.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/systems/biology/personality_generator.gd)
- [scripts/systems/cognition/intelligence_generator.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/systems/cognition/intelligence_generator.gd)
- [scripts/systems/psychology/trait_system.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/systems/psychology/trait_system.gd)
- [scripts/systems/social/value_system.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/systems/social/value_system.gd)

Evidence:
- These files can construct or mutate simulation-shaped shadow objects.
- They are still instantiated by shell/bootstrap managers such as `main.gd` and `entity_manager.gd`.

Current severity:
- They are residual leakage risks, but not active runtime scheduler authority.
- No active boot path registers these as simulation systems.
- No active frame tick path runs them as the source of truth.

### dead_legacy
Already removed from the active codebase by prior authority-cutover work:
- `scripts/ai/behavior_system.gd`
- most legacy runtime systems formerly under `scripts/systems/**`

Evidence:
- `main.gd` no longer preloads or registers them
- they are deleted from disk on `lead/main`

## Active Truth
- Active simulation tick authority: Rust only
- Active boot registry authority: Rust only
- Residual Godot shadow state: still present, documented, and not part of the active scheduler
