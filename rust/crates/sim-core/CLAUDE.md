# rust/crates/sim-core/ — CLAUDE.md

> ECS components, world data, config constants, shared types.
> **This is the foundation crate.** It depends on nothing internal.
> Changes here affect every other crate. Treat this as schema.

---

## Ownership

This crate owns:
- All ECS component structs (entity data decomposed into components)
- World data (tiles, resources, climate)
- Settlement and building data
- Config constants (all tuning values)
- Shared enums and ID types
- Scale/curve utility functions

---

## Module Map

```
sim-core/src/
  lib.rs                    — Re-exports
  config.rs                 — ALL simulation constants
  enums.rs                  — Shared enums (AgeStage, Job, Gender, etc.)
  ids.rs                    — EntityId, SettlementId, BuildingId newtypes
  scales.rs                 — Utility: clamp, lerp, sigmoid, normalize
  building.rs               — Building struct
  settlement.rs             — Settlement struct
  calendar.rs               — Tick↔date conversion
  components/
    mod.rs                  — Re-exports all components
    identity.rs             — Identity (name, species, gender)
    age.rs                  — Age, birth_tick, age_stage
    personality.rs          — HEXACO 6-axis + 24 facets
    emotion.rs              — Plutchik 8 emotions + composites
    body.rs                 — BodyAttributes, health, body_parts
    intelligence.rs         — Gardner 8 intelligences
    needs.rs                — 13 needs (hunger, thirst, sleep, ...)
    stress.rs               — 4-phase stress (GAS model)
    coping.rs               — Coping strategies
    values.rs               — 33 values (-1.0~+1.0)
    social.rs               — Relationships, reputation, social class
    economic.rs             — Wealth, saving, risk appetite
    skills.rs               — Skill levels
    memory.rs               — Short-term memory, trauma records
    traits.rs               — Active trait list + salience
    behavior.rs             — Current action, target, path
    position.rs             — World position (x, y)
    faith.rs                — Religious beliefs
  world/
    mod.rs                  — World struct
    tile.rs                 — Tile data (biome, elevation, moisture)
    resource_map.rs         — Per-tile resource amounts
```

---

## Component Design Rules

1. **Components are plain data.** No methods with side effects. No references to other components.
2. **`#[derive(Clone, Debug, Default)]`** minimum on every component.
3. **f64 for all simulation values.** No f32.
4. **Use enums (not strings)** for categorical data: `AgeStage::Adult`, not `"adult"`.
5. **Flat structure preferred.** Minimize nested structs — ECS queries are fastest on flat data.

### Adding a Component
1. Create file in `components/`
2. Add `pub mod` to `components/mod.rs`
3. If UI needs it → tell lead to add SimBridge getter
4. Document here in the Component Registry below

---

## Component Registry

| Component | File | Key Fields | Updated By |
|-----------|------|------------|------------|
| Identity | identity.rs | name, species_id, gender | spawn only |
| Age | age.rs | birth_tick, age_years, age_stage | age_system |
| Personality | personality.rs | hexaco: [f64; 6], facets: [f64; 24] | maturation_system |
| Emotion | emotion.rs | intensities: [f64; 8] | emotion_system |
| Body | body.rs | attributes (str/agi/end/...), health | mortality, combat |
| Intelligence | intelligence.rs | gardner: [f64; 8] | learning_system |
| Needs | needs.rs | values: [f64; 13] | needs_system |
| Stress | stress.rs | level, phase, allostatic_load | stress_system |
| Coping | coping.rs | strategies, active_coping | coping_system |
| Values | values.rs | values: [f64; 33] | value_system |
| Social | social.rs | relationships, reputation | social_system |
| Economic | economic.rs | wealth, inventory | economy systems |
| Skills | skills.rs | skill_levels: HashMap | learning_system |
| Memory | memory.rs | short_term, trauma_records | various |
| Traits | traits.rs | active_traits, salience | trait_system |
| Behavior | behavior.rs | current_action, target, path | behavior_system |
| Position | position.rs | x: i32, y: i32 | movement_system |
| Faith | faith.rs | beliefs, piety | faith_system |

---

## Config Constants (`config.rs`)

All tuning values live here. **No magic numbers anywhere else.**

### Naming Convention
```rust
pub const SYSTEM_NAME_PARAMETER: f64 = value;
// Examples:
pub const HUNGER_DECAY_RATE: f64 = 0.005;
pub const STRESS_TRAUMA_THRESHOLD: f64 = 0.8;
pub const MORTALITY_GOMPERTZ_B: f64 = 0.0001;
```

### Adding a Constant
1. Add to `config.rs` with `///` doc comment
2. Reference from system: `config::CONSTANT_NAME`
3. Search existing constants first — never duplicate

---

## Shared Enums (`enums.rs`)

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AgeStage { Infant, Toddler, Child, Teen, Adult, Elder }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Job { None, Gatherer, Lumberjack, Builder, Miner, Hunter, ... }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Gender { Male, Female }

// ... more enums
```

**Rule**: Use `#[derive(Copy, Clone)]` for small enums. Use these in systems instead of string matching.

---

## Do NOT

- Add methods with side effects to component structs
- Use `String` where an enum would suffice
- Import from sim-systems, sim-engine, or sim-bridge (dependency violation)
- Use f32 for any simulation value
- Add Godot-specific types
- Change field types without updating all consumers