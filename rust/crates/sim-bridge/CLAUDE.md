# rust/crates/sim-bridge/ — CLAUDE.md

> GDExtension FFI bridge. Converts between Godot types and Rust types.
> This is the ONLY crate that touches `godot::` types.

---

## Purpose

sim-bridge is the boundary between Rust simulation and Godot rendering/UI.
It exposes `#[func]` methods that GDScript calls, and translates:
- Rust `hecs::Entity` → Godot `i64` (entity ID)
- Rust structs → Godot `Dictionary`
- Godot `Dictionary` → Rust `Command`
- Rust `SimEvent` → Godot signals via SimulationBus

---

## Module Map

```
sim-bridge/src/
  lib.rs                    — GDExtension entry, WorldSimRuntime class
  runtime_bindings.rs       — Core runtime #[func] methods (tick, snapshot, entity detail)
  runtime_commands.rs       — Command processing (push_command → CommandQueue)
  runtime_events.rs         — Event relay (SimEvent → GDScript SimulationBus)
  runtime_registry.rs       — System registration helpers
  runtime_dict.rs           — Dictionary serialization helpers
  pathfinding_bindings.rs   — Pathfinding #[func] wrappers
  pathfinding_core.rs       — A* core (may move to sim-systems)
  locale_bindings.rs        — Locale-related bindings
  body_bindings.rs          — Body attribute #[func] wrappers
  ws2_codec.rs              — Binary encoding for large snapshots
```

---

## Key #[func] Methods

```rust
#[godot_api]
impl WorldSimRuntime {
    // === Tick Control ===
    #[func] fn tick(&mut self);
    #[func] fn set_speed(&mut self, speed: f64);
    #[func] fn is_paused(&self) -> bool;

    // === Snapshots (Godot reads these) ===
    #[func] fn get_frame_snapshot(&self) -> Dictionary;
    #[func] fn get_entity_detail(&self, entity_id: i64) -> Dictionary;
    #[func] fn get_settlement_detail(&self, settlement_id: i64) -> Dictionary;
    #[func] fn get_building_detail(&self, building_id: i64) -> Dictionary;

    // === Commands (Godot writes these) ===
    #[func] fn push_command(&mut self, cmd: GString, args: Dictionary);

    // === Bulk data (PackedArrays for renderers) ===
    #[func] fn get_entity_positions(&self) -> PackedFloat64Array;
    #[func] fn get_entity_colors(&self) -> PackedFloat32Array;

    // === Query ===
    #[func] fn get_alive_count(&self) -> i64;
    #[func] fn get_alive_entity_ids(&self) -> PackedInt64Array;
}
```

---

## Type Conversion Rules

| Rust Type | Godot Type | Method |
|-----------|-----------|--------|
| `Entity` (hecs) | `i64` | `entity.id() as i64` |
| `f64` | `f64` | direct |
| `String` | `GString` | `.into()` |
| `Vec<T>` | `Array<Variant>` | manual conversion |
| `HashMap<K,V>` | `Dictionary` | manual conversion |
| `enum` | `GString` | `format!("{:?}", value)` or match |
| Component struct | `Dictionary` | field-by-field via `runtime_dict` helpers |

### Conversion Pattern
```rust
fn component_to_dict(comp: &MyComponent) -> Dictionary {
    let mut dict = Dictionary::new();
    dict.set("field_a", comp.field_a);
    dict.set("field_b", comp.field_b.to_godot());
    dict
}
```

---

## Event Relay

SimBridge relays Rust events to GDScript SimulationBus:

```rust
fn relay_events(events: &[SimEvent], bus: &mut SimulationBus) {
    for event in events {
        match event {
            SimEvent::EntityDied { entity, cause } => {
                bus.emit_signal("entity_died", &[entity_id.to_variant(), cause_str.to_variant()]);
            }
            // ...
        }
    }
}
```

GDScript UI listens to SimulationBus signals, NOT to Rust events directly.

---

## Adding a New Bridge Method

1. Add `#[func]` method to appropriate file
2. Convert Godot input types → Rust types
3. Call sim-engine/sim-core logic
4. Convert Rust result → Godot output types
5. Add GDScript caller in appropriate UI file
6. Test both Rust side (unit test) and Godot side (headless test)

---

## Do NOT

- Put simulation logic in sim-bridge — it's just a translator
- Return Rust-specific types to GDScript (must convert to Dictionary/Array/Variant)
- Cache Godot Nodes or Objects in the bridge (lifetime issues)
- Use `unwrap()` on Godot type conversions (use `.unwrap_or_default()`)
- Import sim-bridge from any other crate (it's the top of the dependency tree)