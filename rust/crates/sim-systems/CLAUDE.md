# rust/crates/sim-systems/ — CLAUDE.md

> All tick-based simulation systems. Each system is a pure function operating on the ECS world.
> Systems communicate ONLY through EventBus. No direct references between systems.

---

## Architecture Rule

Every system is a function with this signature:
```rust
pub fn system_name(
    world: &mut hecs::World,
    config: &Config,       // from sim-core
    events: &mut EventBus, // from sim-engine
    data: &DataStore,      // from sim-data (read-only)
    tick: u64,
)
```

**Every system MUST:**
1. Operate on the hecs World via queries
2. Read config from `sim-core::config`
3. Communicate via `EventBus` events — never call another system directly
4. Be deterministic: same input → same output
5. Have `#[cfg(test)]` unit tests

---

## Module Map

```
sim-systems/src/
  lib.rs                    — Re-exports, system registration list
  pathfinding.rs            — A* pathfinding (pure Rust)
  stat_curve.rs             — Stat curve evaluation
  body.rs                   — Body attribute calculations
  runtime/
    mod.rs                  — Runtime system re-exports
    needs.rs                — 13-need decay/fulfillment
    biology.rs              — Age, mortality (Gompertz-Makeham), reproduction
    psychology.rs           — Emotion decay, stress (GAS), mental breaks, coping
    cognition.rs            — Personality maturation, trait activation, learning
    economy.rs              — Gathering, construction, job assignment, resource regen
    social.rs               — Social events, reputation, family, migration, contagion
    world.rs                — Building effects, resource regeneration
    record.rs               — Statistics recording, chronicle
```

---

## Priority Ordering (Execution Sequence)

```
 5   resource_regen       — replenish world resources
10   needs                — decay hunger/energy/social, starvation check
15   building_effects     — apply building bonuses
20   behavior_ai          — Utility AI action selection
30   movement             — move entities toward targets
40   gathering            — harvest resources
50   construction         — build structures
60   job_assignment       — assign/reassign jobs
100  population           — birth/reproduction
105  age                  — age all entities
110  mortality            — death checks (Gompertz-Makeham)
120  family               — family bonds
130  migration            — settlement transfers
200  emotion              — emotion decay/generation (Plutchik)
210  stress               — stress accumulation/decay (Lazarus/GAS)
215  coping               — coping strategy selection (Softmax)
220  mental_break         — mental break checks (10 types)
230  trait_activation     — trait salience and activation
235  personality_maturation — slow personality drift
240  contagion            — emotion contagion (social network)
300  social_event         — trigger social interactions
310  chronicle            — record important events
320  reputation           — reputation calculations
400  stats_recorder       — aggregate statistics
```

**Rule:** Economy before lifecycle before psychology before social before recording.
If adding a new system, pick a priority that respects this ordering.

---

## System Template

```rust
use hecs::World;
use sim_core::config;
use sim_core::components::{MyComponent, OtherComponent};
use sim_engine::{EventBus, SimEvent};

/// Brief description of what this system does.
/// Academic basis: [theory/paper if applicable]
pub fn my_system(world: &mut World, events: &mut EventBus, tick: u64) {
    if tick % TICK_INTERVAL != 0 {
        return;
    }

    for (entity, (comp_a, comp_b)) in world.query_mut::<(&mut MyComponent, &OtherComponent)>() {
        let new_value = comp_a.field - config::MY_DECAY_RATE;
        comp_a.field = new_value.clamp(0.0, 1.0);

        if new_value <= 0.0 {
            events.emit(SimEvent::MyCritical { entity });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decay_reduces_value() {
        let mut world = World::new();
        let mut events = EventBus::new();
        let e = world.spawn((MyComponent { field: 0.5 }, OtherComponent::default()));
        my_system(&mut world, &mut events, 0);
        let comp = world.get::<&MyComponent>(e).unwrap();
        assert!(comp.field < 0.5);
    }
}
```

---

## Key Formulas Reference

### Needs Decay (needs.rs)
```
new_value = current - config::NEED_DECAY_RATES[need_index]
if new_value <= 0.0 → emit NeedCritical
```

### Mortality (biology.rs, Gompertz-Makeham)
```
hazard = config::MORTALITY_MAKEHAM_A
       + config::MORTALITY_GOMPERTZ_B * exp(config::MORTALITY_GOMPERTZ_C * age)
death_probability = 1.0 - exp(-hazard)
```

### Stress (psychology.rs, Lazarus GAS)
```
primary_appraisal = severity × (1.0 - coping_resources)
secondary_appraisal = available_coping / demand
stress_delta = primary × (1.0 - secondary)
total = clamp(current + delta - recovery, 0.0, 1.0)
```

### Emotion Decay (psychology.rs, Plutchik)
```
new_intensity = current × (1.0 - config::EMOTION_DECAY_RATE)
if new_intensity < config::EMOTION_MIN_THRESHOLD → set to 0.0
```

---

## Adding a New System

1. Create file in `src/` or `src/runtime/`
2. Add `pub mod` to `lib.rs` or `runtime/mod.rs`
3. Register in `sim-engine/src/engine.rs` with priority
4. Add new events to `sim-engine/src/events.rs` if needed
5. Add new components to `sim-core/src/components/` if needed
6. Write `#[cfg(test)]` unit tests
7. Update priority table in this file

---

## Do NOT

- Call one system from another — use EventBus
- Cache entity references across ticks (entity may be despawned)
- Use `String` matching in hot paths — use enums from `sim-core::enums`
- Skip EventBus for state changes
- Use non-deterministic operations (random without seed, time, etc.)
- Import from sim-bridge (dependency violation)
- Use `unwrap()` in production paths