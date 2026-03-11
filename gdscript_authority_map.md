# GDScript Authority Map

## Scope
- Scanned:
  - `project.godot`
  - `scenes/main/main.gd`
  - `scripts/core/**`
  - `scripts/systems/**`
  - `scripts/ai/**`
  - `scripts/agents/**`
  - `scripts/world/**`
- Missing directories in this repo:
  - `scripts/agents`
  - `scripts/world`

## Active Authority Boundary
- Active runtime authority path:
  - `Godot shell/UI`
  - `scripts/core/simulation/sim_bridge.gd`
  - `rust/crates/sim-bridge`
  - `Rust ECS runtime + scheduler`
- Active boot/tick path:
  - `scenes/main/main.gd` creates the shell and setup flow
  - `scripts/core/simulation/simulation_engine.gd` calls `runtime_init()`
  - `simulation_engine.gd` then calls `runtime_register_default_systems()`
  - every frame tick goes through `runtime_tick_frame()`
  - UI/render paths pull state through runtime getters and snapshots

## Classification

### bridge_only
- `scripts/core/simulation/sim_bridge.gd`
- `scripts/core/simulation/simulation_engine.gd`
- `scripts/core/simulation/simulation_bus.gd`
- `scripts/core/simulation/simulation_bus_v2.gd`
- `scripts/core/simulation/compute_backend.gd`

Evidence:
- These files initialize the native runtime, queue runtime commands, relay runtime events, or expose bridge getters.
- They do not register per-system GDScript schedulers or own simulation ticks.

### render_only
- `scripts/ui/**`
- `scripts/rendering/**`

Evidence:
- Renderers and panels read runtime snapshots/detail queries and draw them.
- Probe overlays, HUD, minimap, and camera logic do not mutate ECS state directly.

### ui_only
- `scenes/main/main.gd`
- `scripts/core/simulation/event_logger.gd`
- `scripts/core/entity/deceased_registry.gd`
- `scripts/systems/record/chronicle_system.gd`
- `scripts/systems/record/memory_system.gd`
- `scripts/core/tech/tech_tree_manager.gd`
- `scripts/core/social/name_generator.gd`

Evidence:
- These files bootstrap the shell, log/format events, keep UI-facing archives, or load static definition data.
- They may store observer-facing history or fallback metadata, but they do not drive the active ECS scheduler.

### simulation_authority
- Active boot/tick authority leaks: **none found**

Evidence:
- `scenes/main/main.gd` has no `register_system(...)` calls.
- `scripts/core/simulation/simulation_engine.gd` has no legacy `runtime_clear_registry` or GDScript scheduler rebuild path.
- Active frame ticks are issued only through `runtime_tick_frame()`.
- Runtime registry population happens through Rust default registration.

Residual shadow/bootstrap helpers with simulation-shaped logic but no active scheduler authority:
- `scripts/core/entity/entity_manager.gd`
- `scripts/core/settlement/building_manager.gd`
- `scripts/core/social/relationship_manager.gd`
- `scripts/core/social/reputation_manager.gd`
- `scripts/core/world/resource_map.gd`
- `scripts/systems/biology/personality_generator.gd`
- `scripts/systems/cognition/intelligence_generator.gd`
- `scripts/systems/psychology/trait_system.gd`
- `scripts/systems/psychology/trait_effect_cache.gd`
- `scripts/systems/social/value_system.gd`
- `scripts/systems/social/settlement_culture.gd`

Status:
- These files remain as shadow/bootstrap/reference helpers and legacy UI fallbacks.
- They are not registered into the active runtime scheduler and do not own the authoritative tick.
- They remain cleanup targets, but they are not active authority blockers for the current cutover.

### dead_legacy
Deleted or removed from active authority:
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
- `scripts/systems/record/stats_recorder.gd`
- `scripts/systems/record/stat_threshold_system.gd`
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
- `scripts/core/combat/combat_resolver.gd`
- `scripts/core/simulation/runtime_shadow_reporter.gd`

## Final Authority Statement
- `simulation_authority = rust_only` for the active runtime boot and tick path.
- Godot boots the shell, forwards commands, renders state, and keeps observer/debug archives.
- Remaining GDScript shadow helpers are non-authoritative residuals, not active simulation schedulers.
