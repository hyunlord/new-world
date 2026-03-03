# rust/crates/sim-engine/ — CLAUDE.md

> Tick loop, EventBus, system scheduling, command queue, snapshots.
> This crate orchestrates the simulation — it calls systems in order and distributes events.

---

## Module Map

```
sim-engine/src/
  lib.rs              — Re-exports
  engine.rs           — SimulationEngine: tick loop, system scheduling
  event_bus.rs        — EventBus: inter-system communication
  events.rs           — SimEvent enum: all possible simulation events
  command.rs          — CommandQueue: player input processing
  snapshot.rs         — FrameSnapshot: data sent to Godot each frame
  system_trait.rs     — SystemFn type alias, registration
```

---

## SimulationEngine

```rust
pub struct SimulationEngine {
    world: hecs::World,
    event_bus: EventBus,
    command_queue: CommandQueue,
    systems: Vec<RegisteredSystem>,  // sorted by priority
    current_tick: u64,
    tick_accumulator: f64,
    rng: StdRng,                     // seeded, deterministic
}

impl SimulationEngine {
    pub fn tick(&mut self) {
        self.current_tick += 1;
        self.command_queue.process(&mut self.world);
        for system in &self.systems {
            if self.current_tick % system.interval == 0 {
                (system.func)(&mut self.world, &mut self.event_bus, self.current_tick);
            }
        }
        self.event_bus.flush_to_bridge();  // relay events to GDScript
    }
}
```

---

## EventBus

All inter-system communication goes through EventBus. **Direct system calls are forbidden.**

### Usage
```rust
// Emitting (from a system)
events.emit(SimEvent::EntityDied { entity, cause: DeathCause::Starvation });

// Reading (in same tick, later system)
for event in events.iter() {
    match event {
        SimEvent::EntityDied { entity, cause } => { ... }
        _ => {}
    }
}
```

### Event Naming Convention
- PascalCase enum variants: `EntityDied`, `TraumaRecorded`, `TechDiscovered`
- Include relevant entity/IDs as fields
- Minimal fields — listeners can query the World for details

---

## SimEvent Registry (events.rs)

```rust
#[derive(Clone, Debug)]
pub enum SimEvent {
    // Entity lifecycle
    EntitySpawned { entity: Entity },
    EntityDied { entity: Entity, cause: DeathCause },

    // Needs
    NeedChanged { entity: Entity, need: NeedType, old: f64, new: f64 },
    NeedCritical { entity: Entity, need: NeedType },

    // Emotions
    EmotionChanged { entity: Entity, emotion: EmotionType, intensity: f64 },

    // Stress / Mental
    StressChanged { entity: Entity, stress: f64 },
    MentalBreakTriggered { entity: Entity, break_type: MentalBreakType },
    TraumaRecorded { entity: Entity, trauma_type: String, severity: f64 },

    // Social
    RelationshipChanged { a: Entity, b: Entity, affinity: f64 },
    FamilyFormed { a: Entity, b: Entity },
    BirthOccurred { parent_a: Entity, parent_b: Entity, child: Entity },

    // Economy / Jobs
    JobAssigned { entity: Entity, job: Job },
    BuildingConstructed { building_id: u64, building_type: String },
    ResourceGathered { entity: Entity, resource: String, amount: f64 },

    // Settlement
    SettlementFounded { settlement_id: u64 },
    MigrationOccurred { entity: Entity, from: u64, to: u64 },

    // Tech
    TechDiscovered { settlement_id: u64, tech_id: String },
    TechRegressed { settlement_id: u64, tech_id: String },
}
```

### Adding an Event
1. Add variant to `SimEvent` in `events.rs`
2. Update SimBridge event relay (if UI needs to see it)
3. Document in this registry
4. Update `sim-bridge` GDScript signal mapping

---

## CommandQueue

Player commands from Godot enter through the command queue:
```rust
pub enum Command {
    SetSpeed(f64),
    SpawnEntity { x: i32, y: i32 },
    PauseSimulation,
    ResumeSimulation,
    DiscoverTech { settlement_id: u64, tech_id: String },
    // ... future commands
}
```

Commands are processed at the START of each tick, before systems run.

---

## Snapshots

FrameSnapshot is the data packet sent to Godot each frame:
```rust
pub struct FrameSnapshot {
    pub tick: u64,
    pub entity_positions: Vec<(EntityId, i32, i32)>,
    pub entity_states: Vec<EntityBriefState>,
    pub events_this_tick: Vec<SimEvent>,
    pub population: u32,
    pub resources: ResourceSummary,
}
```

GDScript reads this through `SimBridge.get_frame_snapshot()`.

---

## Do NOT

- Run systems out of priority order
- Allow non-deterministic operations (unseeded random, system time)
- Import from sim-bridge (dependency inversion)
- Let events leak across tick boundaries (flush every tick)
- Process commands in the middle of system execution