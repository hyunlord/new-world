# rust/ — CLAUDE.md

> Rust simulation core workspace. ALL simulation logic lives here.
> GDScript is UI/rendering only — it never contains tick logic.

---

## Workspace Structure

```
rust/
  Cargo.toml                  ← Workspace manifest
  crates/
    sim-core/                 ← ECS components, world data, config, enums
    sim-data/                 ← JSON data loaders (serde)
    sim-systems/              ← All tick-based simulation systems
    sim-engine/               ← Tick loop, EventBus, system scheduling, commands
    sim-bridge/               ← GDExtension FFI (gdext crate)
    sim-test/                 ← Headless test/validation binary
  tests/                      ← Integration tests
  worldsim.gdextension        ← Godot registration file
```

## Crate Dependency Graph

```
sim-bridge  →  sim-engine  →  sim-systems  →  sim-core
    │              │               │              │
    │              │               └──→ sim-data ─┘
    │              └──→ sim-core
    └──→ sim-engine, sim-core, godot (gdext)

sim-test  →  sim-engine, sim-systems, sim-core, sim-data
```

**Rule**: Dependencies flow downward. `sim-core` depends on nothing internal. `sim-bridge` depends on everything.

---

## Build Commands

```bash
# Full workspace build
cd rust && cargo build --workspace

# Release build (for Godot)
cd rust && cargo build --release -p sim-bridge

# Test everything
cd rust && cargo test --workspace

# Lint
cd rust && cargo clippy --workspace -- -D warnings

# Run headless simulation test
cd rust && cargo run -p sim-test

# Build dylib for Godot
cd rust && cargo build --release -p sim-bridge
# Output: rust/target/release/libsim_bridge.dylib (macOS)
#         rust/target/release/libsim_bridge.so (Linux)
```

---

## Coding Standards

### Must-Follow Rules
1. **f64 everywhere** for simulation math (determinism across platforms)
2. **No `unwrap()` in production** — use `Result`, `unwrap_or_default()`, or `expect("msg")`
3. **`///` doc comments** on all `pub` items
4. **`#[cfg(test)]` module** in every source file
5. **No Godot types outside sim-bridge** — sim-core/sim-systems/sim-engine are pure Rust
6. **Enums over strings** for all categorical data in hot paths
7. **Constants in `sim-core/src/config.rs`** — no magic numbers

### Style
- Format: `cargo fmt` before commit
- Clippy clean: `cargo clippy -- -D warnings`
- Modules: one concern per file, mirror GDScript system structure
- Tests: at minimum one unit test per public function

---

## Adding a New System

1. Create `rust/crates/sim-systems/src/my_system.rs`
2. Add `pub mod my_system;` to `sim-systems/src/lib.rs`
3. Register in `sim-engine/src/engine.rs` with priority and tick interval
4. Add any new events to `sim-engine/src/events.rs`
5. Add any new components to `sim-core/src/components/`
6. Add any new config constants to `sim-core/src/config.rs`
7. Write `#[cfg(test)]` unit tests
8. If UI needs to see results: add `#[func]` getter in `sim-bridge/`
9. Update the relevant `CLAUDE.md` file

---

## Adding a New ECS Component

1. Create/modify file in `sim-core/src/components/`
2. Add to `sim-core/src/components/mod.rs` re-exports
3. If UI needs it: add to SimBridge snapshot serialization
4. Add unit test for default/construction
5. Update `sim-core/CLAUDE.md` component registry

---

## GDExtension Integration

The `sim-bridge` crate exposes Rust functionality to Godot via `#[func]` methods:

```rust
#[godot_api]
impl SimBridge {
    #[func]
    fn get_frame_snapshot(&self) -> Dictionary { ... }

    #[func]
    fn get_entity_detail(&self, entity_id: i64) -> Dictionary { ... }

    #[func]
    fn push_command(&mut self, cmd: GString, args: Dictionary) { ... }

    #[func]
    fn tick(&mut self) { ... }
}
```

**Rule**: SimBridge converts between Godot types and Rust types. No Godot types leak into sim-core/sim-systems/sim-engine.

---

## Do NOT

- Put simulation logic in GDScript
- Use Godot types (Variant, GString, Dictionary) in sim-core or sim-systems
- Use f32 for simulation math
- Use `unwrap()` without a very good reason + comment
- Skip `#[cfg(test)]` modules
- Import sim-bridge from sim-core or sim-systems (dependency inversion violation)