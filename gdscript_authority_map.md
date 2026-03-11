# GDScript Authority Map

## Scope
- Scanned: `scripts/core`, `scripts/systems`, `scripts/agents`, `scripts/world`, `scenes/main/main.gd`, `project.godot`
- Missing directories in this repo: `scripts/agents`, `scripts/world`

## Authoritative Runtime Boundary
- Target and active runtime path:
  - `Godot shell/UI`
  - `scripts/core/simulation/sim_bridge.gd`
  - `rust/crates/sim-bridge`
  - `Rust ECS runtime tick`
- Boot/tick authority now lives in Rust:
  - `scripts/core/simulation/simulation_engine.gd` calls `runtime_init()`, `runtime_register_default_systems()`, and `runtime_tick_frame()`
  - `scenes/main/main.gd` no longer preloads or registers legacy GDScript simulation systems

## Classification

### bridge_only
- `scripts/core/simulation/sim_bridge.gd`
- `scripts/core/simulation/simulation_engine.gd`
- `scripts/core/simulation/simulation_bus.gd`
- `scripts/core/simulation/simulation_bus_v2.gd`
- `scripts/core/simulation/runtime_shadow_reporter.gd`

Evidence:
- These scripts initialize the native runtime, forward commands/events, export snapshots, or report parity drift.
- They do not own the simulation tick or register per-system GDScript schedulers anymore.

### ui_only
- `scenes/main/main.gd`
- `scripts/core/simulation/event_logger.gd`
- `scripts/core/entity/deceased_registry.gd`
- `scripts/systems/record/chronicle_system.gd`
- `scripts/systems/record/memory_system.gd`

Evidence:
- `main.gd` builds the shell, renderers, setup flow, and validation warnings.
- `event_logger.gd`, `chronicle_system.gd`, and `memory_system.gd` consume events or record history; they do not drive ticks.

### render_only
- `scripts/ui/**`
- `scripts/rendering/**`

Evidence:
- These paths render world snapshots, entity sprites, overlays, and HUD state sourced from Rust runtime snapshots/detail queries.

### simulation_authority
- Active boot/tick authority leaks: **none found**

Evidence:
- `scenes/main/main.gd` has no `register_system(...)` usage.
- `scripts/core/simulation/simulation_engine.gd` has no legacy registry clear/register flow.
- Runtime ticks are issued only through `runtime_tick_frame()`.

Residual shadow/bootstrap helpers that still contain simulation-shaped code but are not in the active boot/tick authority path:
- `scripts/core/entity/entity_manager.gd`
- `scripts/core/settlement/building_manager.gd`
- `scripts/systems/biology/personality_generator.gd`
- `scripts/systems/cognition/intelligence_generator.gd`
- `scripts/systems/psychology/trait_system.gd`
- `scripts/systems/psychology/trait_effect_cache.gd`
- `scripts/systems/social/value_system.gd`
- `scripts/systems/social/settlement_culture.gd`

Status:
- These files remain as shadow/bootstrap/reference helpers.
- They are not registered into the runtime scheduler and are not used to drive simulation ticks.
- They remain a cleanup target, but not an active authority blocker for this cutover.

### dead_legacy
Deleted runtime-unused authority files:
- `scripts/ai/behavior_system.gd`
- `scripts/systems/biology/age_system.gd`
- `scripts/systems/biology/mortality_system.gd`
- `scripts/systems/biology/population_system.gd`
- `scripts/systems/cognition/intelligence_curves.gd`
- `scripts/systems/cognition/intelligence_system.gd`
- `scripts/systems/development/ace_tracker.gd`
- `scripts/systems/development/attachment_system.gd`
- `scripts/systems/development/child_stress_processor.gd`
- `scripts/systems/development/childcare_system.gd`
- `scripts/systems/development/intergenerational_system.gd`
- `scripts/systems/development/parenting_system.gd`
- `scripts/systems/psychology/contagion_system.gd`
- `scripts/systems/psychology/coping_system.gd`
- `scripts/systems/psychology/emotion_system.gd`
- `scripts/systems/psychology/mental_break_system.gd`
- `scripts/systems/psychology/morale_system.gd`
- `scripts/systems/psychology/needs_system.gd`
- `scripts/systems/psychology/personality_maturation.gd`
- `scripts/systems/psychology/personality_maturation_system.gd`
- `scripts/systems/psychology/psychology_coordinator.gd`
- `scripts/systems/psychology/stress_system.gd`
- `scripts/systems/psychology/trait_violation_system.gd`
- `scripts/systems/psychology/trauma_scar_system.gd`
- `scripts/systems/psychology/upper_needs_system.gd`
- `scripts/systems/record/stat_sync_system.gd`
- `scripts/systems/record/stat_threshold_system.gd`
- `scripts/systems/record/stats_recorder.gd`
- `scripts/systems/social/economic_tendency_system.gd`
- `scripts/systems/social/family_system.gd`
- `scripts/systems/social/job_satisfaction_system.gd`
- `scripts/systems/social/leader_system.gd`
- `scripts/systems/social/network_system.gd`
- `scripts/systems/social/occupation_system.gd`
- `scripts/systems/social/reputation_system.gd`
- `scripts/systems/social/social_event_system.gd`
- `scripts/systems/social/stratification_monitor.gd`
- `scripts/systems/social/title_system.gd`
- `scripts/systems/work/building_effect_system.gd`
- `scripts/systems/work/construction_system.gd`
- `scripts/systems/work/gathering_system.gd`
- `scripts/systems/work/job_assignment_system.gd`
- `scripts/systems/world/migration_system.gd`
- `scripts/systems/world/movement_system.gd`
- `scripts/systems/world/resource_regen_system.gd`
- `scripts/systems/world/steering_system.gd`
- `scripts/systems/world/tech_discovery_system.gd`
- `scripts/systems/world/tech_maintenance_system.gd`
- `scripts/systems/world/tech_propagation_system.gd`
- `scripts/systems/world/tech_utilization_system.gd`
- `scripts/systems/world/tension_system.gd`

## Final Authority Statement
- `simulation_authority = rust_only` for active runtime boot and tick execution.
- Godot now boots, renders, relays commands/events, and displays state.
- Remaining GDScript shadow helpers are non-authoritative residuals, not active simulation schedulers.
