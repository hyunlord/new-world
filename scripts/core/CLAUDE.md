# scripts/core/ — CLAUDE.md

> Core data structures, shared interfaces, and simulation infrastructure.
> **Changes here affect every system.** Treat this layer as schema — modify with extreme care.

---

## Ownership

This directory owns:
- Entity data model and lifecycle
- Simulation engine and tick loop
- SimulationBus (global signal hub)
- GameConfig (all constants)
- Stat calculation pipeline
- World data and generation
- Settlement and building data
- Social relationships and values
- Localization (Locale Autoload)
- Save/Load, Event logging

---

## Subdirectory Map

```
core/
  entity/       — EntityData, EntityManager, PersonalityData, PersonalitySystem,
                  BodyAttributes, EmotionData, DeceasedRegistry
  stats/        — StatQuery, StatCache, StatCurve, StatDefinition,
                  StatEvaluatorRegistry, StatGraph, StatModifier
  world/        — WorldData, WorldGenerator, ResourceMap, ChunkIndex, Pathfinder
  settlement/   — SettlementData, SettlementManager, BuildingData, BuildingManager
  social/       — RelationshipData, RelationshipManager, ValueDefs,
                  NameGenerator, SpeciesManager
  simulation/   — SimulationEngine, SimulationSystem (base class),
                  SimulationBus, GameConfig, GameCalendar
  locale.gd, save_manager.gd, event_logger.gd, deceased_registry.gd
```

---

## EntityData Schema

EntityData is the central data object for every agent. All systems read/write through it.

### Key Fields (Partial — check entity_data.gd for full list)

```gdscript
# Identity
var id: int
var name: String
var species_id: String = "human"
var age: float
var sex: String  # "male" / "female"
var alive: bool = true

# Layer 1: Personality (HEXACO 6-axis, 0.0~1.0)
var hexaco: Dictionary  # {H, E, X, A, C, O}
var attachment_type: String  # "secure"/"anxious"/"avoidant"/"fearful"

# Layer 1.5: Body
var body_attributes: BodyAttributes  # strength, agility, endurance, toughness, recuperation, disease_resistance
var health: float  # 0.0~1.0
var body_parts: Dictionary
var attractiveness: float
var height: float

# Layer 1.7: Intelligence (Gardner 8, 0.0~1.0)
var intelligences: Dictionary  # {linguistic, logical, spatial, musical, kinesthetic, interpersonal, intrapersonal, naturalistic}

# Layer 2: Needs (13, 0.0~1.0)
var needs: Dictionary  # {hunger, thirst, sleep, warmth, safety, belonging, intimacy, recognition, autonomy, competence, self_actualization, meaning, transcendence}

# Layer 3: Emotions (Plutchik 8 + composites)
var emotions: Dictionary  # {joy, trust, fear, surprise, sadness, disgust, anger, anticipation}

# Layer 4: Values (33, -1.0~+1.0)
var values: Dictionary  # {LAW, LOYALTY, FAMILY, ... PEACE}

# Layer 4.5: Social Identity
var occupation: String
var occupation_satisfaction: float
var reputation: Dictionary  # {local, regional, tags[]}
var social_class: String
var faction_id: String

# Layer 4.7: Economic
var saving_tendency: float
var risk_appetite: float
var generosity: float
var materialism: float
var wealth: float

# Layer 5: Skills
var skills: Dictionary  # {skill_id: level}

# Layer 6: Memory
var memories: Array  # short-term, auto-compressed
var history: Array   # permanent, intensity > 0.5
var trauma_records: Array

# Layer 7: Flavor
var blood_type: String
var speech_style: Dictionary  # {tone, verbosity, humor}
var preferences: Dictionary   # {favorite_food, favorite_color, ...}

# Derived Stats (auto-calculated, read-only)
var derived_stats: Dictionary  # {charisma, intimidation, allure, trustworthiness, creativity, wisdom, popularity, risk_tolerance}

# Location
var position: Vector2i
var settlement_id: int
```

### Rules for EntityData Changes
- **Adding a field**: Add to EntityData + update this CLAUDE.md + add to SaveManager serialization
- **Removing a field**: grep ALL systems that reference it first. Migration required.
- **Changing type**: Migration required. Update SaveManager + all readers.

---

## SimulationBus Signal Registry

All inter-system communication goes through SimulationBus. **Direct system-to-system references are forbidden.**

### Current Signals (Keep Updated)

```gdscript
# Entity lifecycle
signal entity_spawned(entity_id: int)
signal entity_died(entity_id: int, cause: String)
signal entity_removed(entity_id: int)

# Tick
signal tick_completed(tick: int)
signal simulation_paused()
signal simulation_resumed()

# Needs
signal need_changed(entity_id: int, need: String, old_value: float, new_value: float)
signal need_critical(entity_id: int, need: String)

# Emotions
signal emotion_changed(entity_id: int, emotion: String, intensity: float)

# Stress / Mental
signal stress_changed(entity_id: int, stress: float)
signal mental_break_triggered(entity_id: int, break_type: String)
signal trauma_recorded(entity_id: int, trauma_type: String)

# Social
signal relationship_changed(entity_a: int, entity_b: int, affinity: float)
signal social_event_occurred(event_type: String, participants: Array)

# Economy / Jobs
signal job_assigned(entity_id: int, job: String)
signal building_constructed(building_id: int, type: String)
signal resource_gathered(entity_id: int, resource: String, amount: float)

# Settlement
signal settlement_founded(settlement_id: int)
signal migration_occurred(entity_id: int, from_settlement: int, to_settlement: int)

# Population
signal birth_occurred(parent_a: int, parent_b: int, child_id: int)
signal family_formed(entity_a: int, entity_b: int)
```

### Signal Naming Convention
- Past tense: `entity_spawned`, NOT `spawn_entity`
- Include relevant IDs as parameters
- Minimal parameters — listeners can look up EntityData for details

### Adding a Signal
1. Add to `simulation_bus.gd`
2. Update this registry in this CLAUDE.md
3. Document which system emits it and which systems listen

---

## GameConfig Constants

All tuning values live in GameConfig. **No magic numbers in system code.**

### Naming Convention
```
SYSTEM_NAME_PARAMETER_NAME
# Examples:
HUNGER_DECAY_RATE = 0.005
STRESS_TRAUMA_THRESHOLD = 0.8
POPULATION_FERTILITY_MIN_AGE = 15
MORTALITY_BASE_RATE = 0.001
```

### Adding a Constant
1. Add to `game_config.gd` with `##` doc comment
2. Reference from system code: `GameConfig.CONSTANT_NAME`
3. Never duplicate — search GameConfig first

---

## Stat System Architecture

```
StatQuery.get(entity_id, stat_name) → float
  │
  ├─ StatCache: check if cached & valid
  │    └─ if miss → StatEvaluatorRegistry
  │
  ├─ StatEvaluatorRegistry: find evaluator for stat_name
  │    └─ evaluator.evaluate(entity) → base_value
  │
  ├─ StatModifier: apply active modifiers (buffs, debuffs, conditions)
  │
  ├─ StatCurve: apply non-linear scaling
  │
  └─ return final value (clamped)
```

### Rust Migration Note
StatCurve, StatGraph, and StatEvaluatorRegistry are prime candidates for Rust migration.
The API boundary is `StatQuery.get()` — callers never change. Only internal implementation swaps.

---

## SimulationEngine Tick Loop

```
SimulationEngine.update(delta):
  accumulate delta → tick_accumulator
  while tick_accumulator >= TICK_DURATION:
    current_tick += 1
    for system in registered_systems (sorted by priority):
      if current_tick % system.tick_interval == 0:
        system.process_tick(current_tick)
    SimulationBus.emit_signal("tick_completed", current_tick)
    tick_accumulator -= TICK_DURATION
```

### System Registration
- Systems self-register with SimulationEngine
- Priority: lower number = runs first
- tick_interval: 1 = every tick, 5 = every 5th tick, etc.

---

## Rust Migration Targets in core/

| File | Migration Priority | Reason |
|------|--------------------|--------|
| `world/pathfinder.gd` | 🔴 HIGH | A* is O(n²), biggest hot path |
| `stats/stat_curve.gd` | 🔴 HIGH | Called 100s of times per tick per entity |
| `stats/stat_graph.gd` | 🔴 HIGH | Dependency resolution, parallelizable |
| `stats/stat_evaluator_registry.gd` | 🟡 MEDIUM | Dispatch overhead |
| `world/world_generator.gd` | 🟡 MEDIUM | One-time but heavy, parallelizable |
| `entity/entity_data.gd` | 🟢 LOW | Data class, fast enough in GDScript |
| `simulation/simulation_bus.gd` | ⚫ NEVER | Must stay GDScript (Godot signal system) |

### Preparing for Rust (Do Now)
- Keep computation in static/pure functions separate from Node lifecycle
- Use PackedFloat64Array/PackedInt32Array for bulk data
- Avoid GDScript-specific patterns in hot paths (no string match, no Variant boxing)
- One concern per file — maps 1:1 to Rust modules

---

## Do NOT Touch Without Explicit Ticket

- SimulationBus signal definitions
- GameConfig constant values (affects all balance)
- EntityData field types/names
- SimulationEngine tick loop logic
- Locale.gd public API