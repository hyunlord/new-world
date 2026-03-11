# GDScript Full Authority Scan

## Scope
- `project.godot`
- `scenes/main/main.gd`
- `scripts/core/**`
- `scripts/systems/**`
- `scripts/debug/**`
- `scripts/ui/**`

## Active Authority Findings

| Path | Current Purpose | Reads Simulation State | Mutates Simulation State | Schedules Simulation Logic | Can Remove Now | Notes |
|---|---|---:|---:|---:|---:|---|
| `scenes/main/main.gd` | Boot shell, setup flow, render/UI composition | Yes | Pre-runtime setup state only | No | No | Still instantiates shadow managers for compatibility. |
| `scripts/core/simulation/simulation_engine.gd` | Runtime bootstrap/tick/command relay | Yes | No direct ECS mutation | No | No | Rust owns `runtime_tick_frame`. |
| `scripts/core/simulation/sim_bridge.gd` | Native bridge wrapper | Yes | Through approved bridge commands only | No | No | Bridge-only. |
| `scripts/core/entity/entity_manager.gd` | Legacy shadow entity store | Yes | Yes, local shadow only | No | Not yet | Deprecated local spawn path remains but is unused by active runtime. |
| `scripts/core/settlement/building_manager.gd` | Legacy building shadow/cache | Yes | Yes, local shadow only | No | Not yet | UI/render fallback. |
| `scripts/core/settlement/settlement_manager.gd` | Legacy settlement shadow/cache | Yes | Yes, local shadow only | No | Not yet | UI/render fallback. |
| `scripts/core/world/resource_map.gd` | Pre-runtime map editor/setup resource state | Yes | Yes | No | Not yet | Setup-only authority before bootstrap. |
| `scripts/core/social/relationship_manager.gd` | Legacy relationship shadow store | Yes | Yes, local shadow only | No | Not yet | Legacy UI/detail fallback. |
| `scripts/core/social/reputation_manager.gd` | Legacy reputation shadow store | Yes | Yes, local shadow only | No | Not yet | Legacy UI/detail fallback. |
| `scripts/systems/record/chronicle_system.gd` | Observer/archive log | Yes | Yes, archive-only | No | Not yet | Does not feed back into gameplay tick. |
| `scripts/systems/record/memory_system.gd` | Chronicle support | Yes | Yes, archive-only | No | Not yet | Not scheduler-owned in active runtime. |
| `scripts/systems/biology/personality_generator.gd` | Legacy spawn helper | No active path | Deprecated local spawn only | No | Yes, after spawn fallback removal | Not booted in active runtime anymore. |
| `scripts/systems/cognition/intelligence_generator.gd` | Legacy spawn helper | No active path | Deprecated local spawn only | No | Yes, after spawn fallback removal | Not booted in active runtime anymore. |
| `scripts/systems/social/value_system.gd` | Legacy value bootstrap | No active path | Deprecated local spawn only | No | Yes, after spawn fallback removal | Not part of active scheduler. |
| `scripts/systems/social/settlement_culture.gd` | Legacy value support | No active path | Deprecated local spawn only | No | Yes, after value fallback removal | Only referenced by dead-legacy value path. |
| `scripts/ui/**` | Presentation | Yes | No | No | No | Read-only presentation. |
| `scripts/debug/**` | Debug overlay/panels | Yes | Approved config commands only | No | No | Read-only unless using approved command path. |

## Evidence
- `SimulationEngine.update()` calls `runtime_tick_frame()`.
- No active `register_system(...)` or `runtime_clear_registry` usage remains on the boot path.
- `EntityManager.spawn_entity()` has no repository callers.
- `main.gd` now validates Rust registry authority before shadow/bootstrap managers are initialized.

## Conclusion
- Active gameplay authority is Rust-owned.
- Remaining Godot-side mutation exists in pre-runtime setup state and legacy shadow/archive helpers, not in the authoritative ECS tick.
