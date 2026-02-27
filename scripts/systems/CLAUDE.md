# scripts/systems/ — CLAUDE.md

> All tick-based simulation systems. Each system extends SimulationSystem.
> Systems communicate ONLY through SimulationBus. No direct references between systems.

---

## Architecture Rule

```
SimulationSystem (base class)
  ├─ system_name: String
  ├─ priority: int        # lower = runs first
  ├─ tick_interval: int   # 1 = every tick, 5 = every 5th tick
  └─ process_tick(tick: int)  # override this
```

**Every system MUST:**
1. Extend `SimulationSystem`
2. Set `system_name`, `priority`, `tick_interval` in `_init()`
3. Communicate via `SimulationBus.emit_signal()` — never import another system
4. Read entity data from `EntityManager` — never cache entity references
5. Use `GameConfig` for all tuning constants — no magic numbers

---

## Subdirectory Map

```
systems/
  lifecycle/     — Birth, death, aging, family formation
    population_system.gd      (prio=100, interval=10)
    mortality_system.gd       (prio=110, interval=5)
    age_system.gd             (prio=105, interval=1)
    family_system.gd          (prio=120, interval=10)

  psychology/    — Emotions, stress, mental breaks, personality growth
    emotion_system.gd         (prio=200, interval=1)
    stress_system.gd          (prio=210, interval=1)
    mental_break_system.gd    (prio=220, interval=5)
    personality_maturation_system.gd (prio=230, interval=50)

  economy/       — Needs, gathering, building, jobs
    needs_system.gd           (prio=10, interval=1)
    gathering_system.gd       (prio=40, interval=1)
    construction_system.gd    (prio=50, interval=1)
    job_assignment_system.gd  (prio=60, interval=10)

  social/        — Events, chronicle, reputation
    social_event_system.gd    (prio=300, interval=5)
    chronicle_system.gd       (prio=310, interval=1)
    reputation_system.gd      (prio=320, interval=10)

  world/         — Resources, buildings, movement, migration
    resource_regen_system.gd  (prio=5, interval=10)
    building_effect_system.gd (prio=15, interval=5)
    movement_system.gd        (prio=30, interval=1)
    migration_system.gd       (prio=130, interval=20)

  stats/
    stats_recorder_system.gd  (prio=400, interval=10)
```

---

## Priority Ordering (Execution Sequence)

```
5    ResourceRegenSystem       — replenish world resources
10   NeedsSystem               — decay hunger/energy/social, starvation check
15   BuildingEffectSystem      — apply building bonuses
30   MovementSystem            — move entities toward targets
40   GatheringSystem           — harvest resources
50   ConstructionSystem        — build structures
60   JobAssignmentSystem       — assign/reassign jobs
100  PopulationSystem          — birth/reproduction
105  AgeSystem                 — age all entities
110  MortalitySystem           — death checks
120  FamilySystem              — family bonds
130  MigrationSystem           — settlement transfers
200  EmotionSystem             — emotion decay/generation
210  StressSystem              — stress accumulation/decay
220  MentalBreakSystem         — mental break checks
230  PersonalityMaturation     — slow personality drift
300  SocialEventSystem         — trigger social interactions
310  ChronicleSystem           — record important events
320  ReputationSystem          — reputation calculations
400  StatsRecorderSystem       — aggregate statistics
```

**Rule:** Economy before lifecycle before psychology before social before recording.
If adding a new system, pick a priority that respects this ordering.

---

## System Template

```gdscript
class_name MyNewSystem
extends SimulationSystem

## [Brief description of what this system does]
## Academic basis: [theory/paper if applicable]

func _init() -> void:
    system_name = "my_new"
    priority = XXX
    tick_interval = X

func process_tick(tick: int) -> void:
    var entities = EntityManager.get_alive_entities()
    for entity_id in entities:
        var ed = EntityManager.get_entity(entity_id)
        if ed == null:
            continue
        _process_entity(ed, tick)

func _process_entity(ed: EntityData, tick: int) -> void:
    # Core logic here
    # Use GameConfig.CONSTANT_NAME for all values
    # Emit results via SimulationBus
    pass
```

---

## Key Formulas Reference

### Needs Decay (NeedsSystem)
```
new_value = current - GameConfig.NEED_DECAY_RATES[need_type] * tick_interval
if new_value <= 0.0: → emit need_critical
```

### Stress Accumulation (StressSystem, Lazarus Appraisal Theory)
```
primary_appraisal = stressor.severity × (1.0 - coping_resources)
secondary_appraisal = available_coping / stressor.demand
stress_delta = primary_appraisal × (1.0 - secondary_appraisal)
total_stress = clamp(current_stress + stress_delta - recovery_rate, 0.0, 1.0)
```

### Mortality (MortalitySystem, Gompertz-Makeham)
```
hazard_rate = GameConfig.MORTALITY_MAKEHAM_A
            + GameConfig.MORTALITY_GOMPERTZ_B
            * exp(GameConfig.MORTALITY_GOMPERTZ_C * age)
death_probability = 1.0 - exp(-hazard_rate)
```

### Emotion Decay (EmotionSystem, Plutchik)
```
new_intensity = current × (1.0 - GameConfig.EMOTION_DECAY_RATE)
if new_intensity < GameConfig.EMOTION_MIN_THRESHOLD: → set to 0.0
```

---

## Adding a New System

1. Create file in appropriate subdirectory
2. Extend `SimulationSystem`, set priority/interval
3. Register with SimulationEngine (auto if in correct directory)
4. Add any new signals to `SimulationBus` + update `scripts/core/CLAUDE.md`
5. Add any new constants to `GameConfig`
6. Update priority table in this file
7. Localize all user-visible text via `Locale.ltr()`

---

## Rust Migration Notes

Systems themselves stay in GDScript (they're Godot nodes with signal connections).
**What migrates to Rust is the heavy computation inside systems:**

| Pattern | GDScript | Rust |
|---------|----------|------|
| Per-entity math loop | `for e in entities: calculate(e)` | Rust function takes PackedArray, returns PackedArray |
| Pathfinding | `Pathfinder.find_path()` | `RustPathfinder.find_path()` via GDExtension |
| Stat evaluation | `StatQuery.get()` | Internal evaluators in Rust |

**Rule:** The system `.gd` file stays. The expensive inner function gets extracted to a static helper that can later become a Rust FFI call.

```gdscript
# Current (GDScript)
func _calculate_stress(ed: EntityData) -> float:
    return StressCalculator.compute(ed.hexaco, ed.needs, ed.emotions)

# Future (Rust behind the scenes)
# StressCalculator.compute() → calls Rust GDExtension internally
# This system file doesn't change at all
```

---

## Testing Expectations

- Every system should have a smoke test: spawn N entities, run M ticks, verify expected signal emissions
- Edge cases: entity dies mid-tick, entity has no settlement, need at exactly 0.0
- Performance: measure tick time with `OS.get_ticks_usec()` before and after changes

---

## Do NOT

- Import another system directly (`var other_sys = get_node("...")` → FORBIDDEN)
- Cache EntityData references across ticks (entity may die)
- Skip SimulationBus for state changes
- Use `await` in `process_tick()` — tick processing must be synchronous
- Add UI logic to systems — systems emit events, UI listens