# wildlife-entity-and-spawn-v1 — Implementation Prompt

## Feature Summary

Implement Phase A1 of the animal-attack system: Wildlife ECS component (Wolf/Bear/Boar),
spawn system that places wildlife far from settlements, and a wander behavior system.

This is **Phase A1 only** — no combat, no threat detection, no sprites. Just entity
existence, spawn placement, and idle wandering movement.

---

## Section 1: Implementation Intent

Wildlife entities give the simulation ecological presence and set the foundation for
predator-prey dynamics (Phase A2+). Three species in Phase A1: Wolf (pack predator),
Bear (solitary high-HP), Boar (aggressive mid-range). The `Wildlife` component stores
all species data. `WildlifeRuntimeSystem` (priority 115, interval 60) handles both
spawn (once at init) and per-tick wander.

Design: wildlife spawns at simulation startup (tick 0–1), ≥20 tiles from any settlement,
on passable terrain. Wander = random ±1 tile walk each system tick if passable. Home
tile anchors; wander_radius limits straying.

---

## Section 2: What to Build

### 2A. New file: `rust/crates/sim-core/src/components/wildlife.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WildlifeKind {
    Wolf,
    Bear,
    Boar,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wildlife {
    pub kind: WildlifeKind,
    pub max_hp: f32,
    pub current_hp: f32,
    pub move_speed: f32,
    pub home_tile: (i32, i32),
    pub wander_radius: i32,
}

impl Wildlife {
    pub fn wolf(home: (i32, i32)) -> Self {
        Self { kind: WildlifeKind::Wolf,  max_hp: 60.0, current_hp: 60.0, move_speed: 1.4, home_tile: home, wander_radius: 15 }
    }
    pub fn bear(home: (i32, i32)) -> Self {
        Self { kind: WildlifeKind::Bear,  max_hp: 120.0, current_hp: 120.0, move_speed: 0.9, home_tile: home, wander_radius: 10 }
    }
    pub fn boar(home: (i32, i32)) -> Self {
        Self { kind: WildlifeKind::Boar,  max_hp: 80.0, current_hp: 80.0, move_speed: 1.1, home_tile: home, wander_radius: 12 }
    }
}
```

### 2B. Edit: `rust/crates/sim-core/src/components/mod.rs`

Add `pub mod wildlife;` and re-export `Wildlife`, `WildlifeKind`.

### 2C. New file: `rust/crates/sim-systems/src/wildlife_system.rs`

`WildlifeRuntimeSystem` — handles both spawn-on-first-tick and per-tick wander.

Spawn logic (runs once, tracked via a resource flag or tick == 1):
- Spawn 3 wolves, 2 bears, 2 boars = 7 entities total
- Each entity gets: `Identity` (name="Wolf/Bear/Boar", species_id="wolf"/"bear"/"boar"),
  `Position` (x, y), `Wildlife` (constructed via factory method)
- Placement: iterate `resources.map` tiles, find passable tiles ≥20 tiles (Chebyshev)
  from every settlement center (`s.x`, `s.y`). Pick via `resources.rng`.
- **Map API**: `resources.map.get(x as u32, y as u32).passable` — boolean field.
  Bounds check: `resources.map.in_bounds(x, y)` returns bool.
  Map size: `resources.map.width`, `resources.map.height` (u32).

Wander logic (every tick when system fires):
- For each `(entity, (wildlife, position))` in `world.query::<(&mut Wildlife, &mut Position)>()`:
  - Pick random dx, dy ∈ {-1, 0, 1}
  - Candidate = (position.x + dx, position.y + dy)
  - Accept if: `map.in_bounds(cx, cy)` AND `map.get(cx as u32, cy as u32).passable`
    AND Chebyshev distance from `wildlife.home_tile` ≤ `wildlife.wander_radius`
  - If accepted: update position

### 2D. Edit: `rust/crates/sim-systems/src/lib.rs`

Add `pub mod wildlife_system;`.

### 2E. Edit: `rust/crates/sim-bridge/src/runtime_system.rs`

Add `WildlifeRuntimeSystem` to `DEFAULT_RUNTIME_SYSTEMS`.
Current count: **67** entries. Wildlife → 68.
Insert near `HealthRuntimeSystem` (priority 110):

```rust
RuntimeSystemEntry {
    name: "WildlifeRuntimeSystem",
    priority: 115,
    tick_interval: 60,
    enabled: true,
    ..Default::default()
},
```

### 2F. Config constants in `rust/crates/sim-core/src/config.rs`

```rust
pub const WILDLIFE_SPAWN_MIN_DIST_FROM_SETTLEMENT: i32 = 20;
pub const WILDLIFE_WOLF_COUNT: usize = 3;
pub const WILDLIFE_BEAR_COUNT: usize = 2;
pub const WILDLIFE_BOAR_COUNT: usize = 2;
```

### 2G. Harness tests in `rust/crates/sim-test/src/main.rs`

Add 7 tests (W1–W7). Use `make_stage1_engine(42, 20)` which creates a 256×256 map
with settlement at (128, 128).

**W1** `harness_wildlife_entities_spawned`:
```rust
let mut engine = make_stage1_engine(42, 20);
engine.run_ticks(2);
let world = engine.world();
let count = world.query::<&Wildlife>().iter().count();
assert_eq!(count, 7, "expected 7 wildlife entities (3 wolf + 2 bear + 2 boar)");
```

**W2** `harness_wildlife_kinds_correct`:
```rust
engine.run_ticks(2);
let mut wolves = 0; let mut bears = 0; let mut boars = 0;
for (_, w) in world.query::<&Wildlife>().iter() {
    match w.kind { Wolf => wolves += 1, Bear => bears += 1, Boar => boars += 1 }
}
assert_eq!(wolves, 3); assert_eq!(bears, 2); assert_eq!(boars, 2);
```

**W3** `harness_wildlife_hp_valid`:
```rust
for (_, w) in world.query::<&Wildlife>().iter() {
    assert!(w.current_hp > 0.0);
    assert!(w.current_hp <= w.max_hp);
}
```

**W4** `harness_wildlife_spawn_far_from_settlement`:
```rust
// settlement is at (128,128); wildlife must be ≥20 tiles away (Chebyshev)
let resources = engine.resources();
for (_, (w, pos)) in world.query::<(&Wildlife, &Position)>().iter() {
    for (_, s) in resources.settlements.iter() {
        let dx = (pos.x - s.x).abs();
        let dy = (pos.y - s.y).abs();
        let chebyshev = dx.max(dy);
        assert!(chebyshev >= 20,
            "{:?} spawned at ({},{}) only {} tiles from settlement ({},{})",
            w.kind, pos.x, pos.y, chebyshev, s.x, s.y);
    }
}
```

**W5** `harness_wildlife_on_passable_tile`:
```rust
let resources = engine.resources();
for (_, pos) in world.query::<&Position>().iter() {  // filter by wildlife via join
    // Only wildlife entities — use query::<(&Wildlife, &Position)>
}
// Correct version:
for (_, (_, pos)) in world.query::<(&Wildlife, &Position)>().iter() {
    assert!(resources.map.in_bounds(pos.x, pos.y));
    assert!(resources.map.get(pos.x as u32, pos.y as u32).passable,
        "wildlife on non-passable tile ({}, {})", pos.x, pos.y);
}
```

**W6** `harness_wildlife_wanders_over_ticks`:
```rust
let mut engine = make_stage1_engine(42, 20);
engine.run_ticks(2);
let initial_positions: Vec<(i32, i32)> = {
    let world = engine.world();
    world.query::<(&Wildlife, &Position)>().iter()
        .map(|(_, (_, p))| (p.x, p.y)).collect()
};
engine.run_ticks(120);  // 2 system fires at interval=60
let final_positions: Vec<(i32, i32)> = {
    let world = engine.world();
    world.query::<(&Wildlife, &Position)>().iter()
        .map(|(_, (_, p))| (p.x, p.y)).collect()
};
let moved = initial_positions.iter().zip(final_positions.iter())
    .filter(|(a, b)| a != b).count();
assert!(moved >= 1, "at least 1 wildlife entity should have moved after 120 ticks");
```

**W7** `harness_wildlife_stays_within_wander_radius`:
```rust
let mut engine = make_stage1_engine(42, 20);
engine.run_ticks(600);  // 10 system fires
let world = engine.world();
for (_, (w, pos)) in world.query::<(&Wildlife, &Position)>().iter() {
    let dx = (pos.x - w.home_tile.0).abs();
    let dy = (pos.y - w.home_tile.1).abs();
    let dist = dx.max(dy);  // Chebyshev
    assert!(dist <= w.wander_radius,
        "{:?} wandered {} tiles from home (radius={})", w.kind, dist, w.wander_radius);
}
```

---

## Section 3: How to Implement

### Step 1: `sim-core/src/components/wildlife.rs`
Create the file as specified in Section 2A. Add `Default` derive where needed.

### Step 2: `sim-core/src/components/mod.rs`
Add:
```rust
pub mod wildlife;
pub use wildlife::{Wildlife, WildlifeKind};
```

### Step 3: `sim-systems/src/wildlife_system.rs`

```rust
use sim_core::components::{Wildlife, WildlifeKind, Identity, Position};
use sim_core::config;
use sim_engine::{SimResources, EventBus};

pub struct WildlifeRuntimeSystem {
    spawned: bool,
}

impl WildlifeRuntimeSystem {
    pub fn new() -> Self { Self { spawned: false } }
}
```

Spawn block (inside tick method, runs once):
```rust
if !self.spawned {
    self.spawned = true;
    let spawn_plan = vec![
        (WildlifeKind::Wolf, config::WILDLIFE_WOLF_COUNT),
        (WildlifeKind::Bear, config::WILDLIFE_BEAR_COUNT),
        (WildlifeKind::Boar, config::WILDLIFE_BOAR_COUNT),
    ];
    // Collect settlement centers
    let settlement_centers: Vec<(i32, i32)> = resources.settlements.values()
        .map(|s| (s.x, s.y)).collect();
    // Find all valid spawn tiles
    let mut candidates = Vec::new();
    for gy in 0..resources.map.height {
        for gx in 0..resources.map.width {
            let tile = resources.map.get(gx, gy);
            if !tile.passable { continue; }
            let x = gx as i32; let y = gy as i32;
            let far_enough = settlement_centers.iter().all(|&(sx, sy)| {
                (x - sx).abs().max((y - sy).abs()) >= config::WILDLIFE_SPAWN_MIN_DIST_FROM_SETTLEMENT
            });
            if far_enough { candidates.push((x, y)); }
        }
    }
    for (kind, count) in spawn_plan {
        for _ in 0..count {
            if candidates.is_empty() { break; }
            let idx = (resources.rng.next_u64() as usize) % candidates.len();
            let (x, y) = candidates.swap_remove(idx);
            let species = match kind { Wolf => "wolf", Bear => "bear", Boar => "boar" };
            let name = match kind { Wolf => "Wolf", Bear => "Bear", Boar => "Boar" };
            let w = match kind {
                Wolf => Wildlife::wolf((x, y)),
                Bear => Wildlife::bear((x, y)),
                Boar => Wildlife::boar((x, y)),
            };
            world.spawn((
                Identity { name: name.into(), species_id: species.into(), ..Default::default() },
                Position { x, y },
                w,
            ));
        }
    }
}
```

Wander block (runs every tick when system fires):
```rust
let map = &resources.map;
for (_, (wildlife, pos)) in world.query::<(&mut Wildlife, &mut Position)>().iter() {
    let dx = (resources.rng.next_u64() as i32 % 3) - 1;  // -1, 0, 1
    let dy = (resources.rng.next_u64() as i32 % 3) - 1;
    let cx = pos.x + dx; let cy = pos.y + dy;
    if !map.in_bounds(cx, cy) { continue; }
    if !map.get(cx as u32, cy as u32).passable { continue; }
    let hdx = (cx - wildlife.home_tile.0).abs();
    let hdy = (cy - wildlife.home_tile.1).abs();
    if hdx.max(hdy) > wildlife.wander_radius { continue; }
    pos.x = cx; pos.y = cy;
}
```

### Step 4: `sim-systems/src/lib.rs`
Add: `pub mod wildlife_system;`

### Step 5: `sim-bridge/src/runtime_system.rs`
Insert `WildlifeRuntimeSystem` at priority 115 in `DEFAULT_RUNTIME_SYSTEMS`.
Add use import: `use sim_systems::wildlife_system::WildlifeRuntimeSystem;`

### Step 6: `sim-core/src/config.rs`
Add the 4 constants listed in 2F.

### Step 7: Tests in `sim-test/src/main.rs`
Add W1–W7 as `#[test]` functions. Import `Wildlife`, `WildlifeKind`, `Position` at top.
Use `make_stage1_engine(42, 20)`.

---

## Section 4: Dispatch Plan

| Ticket | File | Mode | Depends on |
|--------|------|------|------------|
| T1 | `sim-core/src/components/wildlife.rs` (new) | 🟢 DISPATCH | — |
| T2 | `sim-core/src/components/mod.rs` (edit) | 🔴 DIRECT | T1 |
| T3 | `sim-core/src/config.rs` (4 constants) | 🟢 DISPATCH | — |
| T4 | `sim-systems/src/wildlife_system.rs` (new) | 🟢 DISPATCH | T1, T3 |
| T5 | `sim-systems/src/lib.rs` (edit) | 🔴 DIRECT | T4 |
| T6 | `sim-bridge/src/runtime_system.rs` (edit) | 🟢 DISPATCH | T4 |
| T7 | `sim-test/src/main.rs` (7 tests W1–W7) | 🟢 DISPATCH | T1–T6 |
| T8 | `cargo build --workspace` gate | 🔴 DIRECT | T1–T7 |
| T9 | `cargo clippy --workspace -- -D warnings` gate | 🔴 DIRECT | T8 |

Dispatch ratio: 5/9 = 56%.

---

## Section 5: Localization Checklist

No new localization keys. Wildlife entities are simulation-only; no UI text.

---

## Section 6: Verification & Notion

### Gate commands (must ALL pass before commit):
```bash
cd rust && cargo test --workspace 2>&1 | tail -20
cd rust && cargo clippy --workspace -- -D warnings 2>&1 | tail -10
```

### Smoke test:
```bash
cd rust && cargo test -p sim-test harness_wildlife -- --nocapture 2>&1 | grep -E "test harness_wildlife|FAILED|ok"
```

Expected: 7 tests all `ok`.

### Regression check:
```bash
cd rust && cargo test --workspace 2>&1 | grep -E "FAILED|error" | head -20
```

Expected: no failures in existing tests.

---

## Implementation Notes (from Phase 1 investigation)

**Verified correct APIs** — do NOT use the incorrect variants:

| What | CORRECT | WRONG (do not use) |
|------|---------|-------------------|
| Tile passability | `resources.map.get(x as u32, y as u32).passable` | `resources.terrain.is_passable(x, y)` |
| Settlement center | `s.x`, `s.y` | `s.center_x`, `s.center_y` |
| Map bounds check | `resources.map.in_bounds(x: i32, y: i32) -> bool` | — |
| Map dimensions | `resources.map.width`, `resources.map.height` (u32) | — |
| DEFAULT_RUNTIME_SYSTEMS count | **67** (wildlife → 68) | 64 (stale) |
| Test helper | `make_stage1_engine(seed: u64, agent_count: usize)` | — |
| Settlement in resources | `resources.settlements.iter()` yields `(&SettlementId, &Settlement)` | — |

**Identity struct defaults**: Check if `Identity` has `Default`. If not, set all fields explicitly.

**RNG**: `resources.rng` is `SmallRng`. Use `.next_u64()` for random choice.
`use rand::RngCore;` if needed for `next_u64`.

**WorldMap::get** signature: `fn get(&self, x: u32, y: u32) -> &Tile` — note u32, not i32.
Always cast: `map.get(pos.x as u32, pos.y as u32)` after bounds-checking with `in_bounds(i32, i32)`.
