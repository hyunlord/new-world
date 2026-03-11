# GDScript Authority Map

## Category Definitions
- `ui_only`: presentation, layout, input routing, or static data formatting only
- `render_only`: draws world/entity/building state without owning gameplay mutation
- `bridge_only`: forwards commands or snapshots to/from `sim-bridge`
- `mixed_authority`: active path still instantiates or carries legacy shadow/bootstrap state alongside UI
- `debug_only`: read-only debug/archive/history surface
- `dead_legacy`: no active boot/tick ownership; retained only through deprecated fallback paths or not referenced at all

## Active Boot / Runtime Path
- `scenes/main/main.gd` → `scripts/core/simulation/simulation_engine.gd`
- `simulation_engine.gd` → `scripts/core/simulation/sim_bridge.gd`
- `sim_bridge.gd` → `rust/crates/sim-bridge`
- `sim-bridge` → Rust ECS runtime / typed runtime registry / scheduler

## Classification

| Path | Category | Why |
|---|---|---|
| `scenes/main/main.gd` | `mixed_authority` | Active shell/bootstrap entry. Rust owns tick authority, but this file still instantiates legacy shadow managers for UI/setup compatibility. |
| `scripts/core/simulation/simulation_engine.gd` | `bridge_only` | Calls `runtime_init`, `runtime_register_default_systems`, `runtime_tick_frame`, runtime getters, and command relay only. |
| `scripts/core/simulation/sim_bridge.gd` | `bridge_only` | Godot wrapper over native bridge methods. |
| `scripts/core/simulation/simulation_bus.gd` | `bridge_only` | Legacy signal relay/event fanout only. |
| `scripts/core/simulation/simulation_bus_v2.gd` | `bridge_only` | Runtime command/event relay only. |
| `scripts/core/simulation/compute_backend.gd` | `bridge_only` | Sends runtime compute-mode commands through approved command path. |
| `scripts/ui/**` | `ui_only` or `render_only` | UI/render tree. Reads snapshots/getters or legacy shadow managers; does not own simulation tick. |
| `scripts/debug/**` | `debug_only` | Read-only runtime inspection surfaces. |
| `scripts/rendering/**` | `render_only` | Snapshot decode and rendering helpers only. |
| `scripts/core/entity/entity_manager.gd` | `mixed_authority` | Legacy shadow entity store plus deprecated local spawn path. Active runtime does not call `spawn_entity()`, but UI fallback still reads it. |
| `scripts/core/settlement/building_manager.gd` | `mixed_authority` | Legacy building shadow manager still passed into UI/render fallbacks. |
| `scripts/core/settlement/settlement_manager.gd` | `mixed_authority` | Legacy settlement shadow/cache path still present for UI fallback. |
| `scripts/core/social/relationship_manager.gd` | `mixed_authority` | Legacy relationship shadow data used by legacy UI panels. |
| `scripts/core/social/reputation_manager.gd` | `mixed_authority` | Legacy reputation shadow data used by legacy UI panels. |
| `scripts/core/world/resource_map.gd` | `mixed_authority` | Pre-runtime setup/editor resource authority plus render fallback after bootstrap. Not the live Rust tick owner. |
| `scripts/core/social/name_generator.gd` | `debug_only` | Naming culture loader/autoload; no active runtime spawn path uses it after this cutover pass. |
| `scripts/systems/record/chronicle_system.gd` | `debug_only` | Autoloaded observer/archive system; stores history for UI, not gameplay tick ownership. |
| `scripts/systems/record/memory_system.gd` | `debug_only` | Read-only chronicle support path; not active gameplay scheduler authority. |
| `scripts/systems/biology/personality_generator.gd` | `dead_legacy` | Only reachable through deprecated `EntityManager.spawn_entity()` path; not used by active boot/runtime. |
| `scripts/systems/cognition/intelligence_generator.gd` | `dead_legacy` | Only reachable through deprecated `EntityManager.spawn_entity()` path. |
| `scripts/systems/social/value_system.gd` | `dead_legacy` | Only used by deprecated local spawn/value bootstrap path. |
| `scripts/systems/social/settlement_culture.gd` | `dead_legacy` | Only referenced by deprecated `value_system.gd`. |
| `scripts/systems/psychology/trait_system.gd` | `mixed_authority` | Not scheduler-owned, but still referenced by legacy tooltip/entity shadow code. |
| `scripts/systems/psychology/trait_effect_cache.gd` | `mixed_authority` | Supports legacy stat-query / trait tooltip path. |

## Active Simulation Authority Leaks
- No active GDScript file owns the authoritative simulation tick.
- No active boot path registers runtime systems from GDScript.
- No active boot path clears/rebuilds the runtime registry in GDScript.

## Remaining Residue
- Mixed-authority shadow managers still exist for UI/setup/save fallback paths.
- Dead-legacy spawn/value generator scripts still exist on disk, but the active runtime no longer initializes them during boot.
