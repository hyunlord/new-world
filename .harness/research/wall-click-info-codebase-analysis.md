# Wall Click Info — Codebase Analysis

**Date**: 2026-04-12
**Purpose**: Research findings for implementing wall-click-info feature
**Status**: Complete — Ready for implementation

---

## 1. Test Plan Overview

The test plan (`.harness/plans/wall-click-info/plan_final.md`) defines **10 assertions across 3 categories**:

### Type A (Data Integrity Invariants) — 6 assertions
1. Wall HP/material validity checks
2. Floor material validity checks
3. Furniture ID validity checks
4. Room_id referential integrity
5. Room-tile floor consistency
6. Door-wall mutual exclusion

### Type C (Data Existence) — 1 assertion
7. After 1 year: walls ≥8, floors ≥6, furniture ≥2, rooms ≥1 (with runaway upper bounds ≤500/500/100)

### Type E (Boundary Safety) — 2 assertions
8. `in_bounds()` rejects invalid coordinates
9. Remote empty tiles have all-default values

**Key insight**: The test validates the **tile_grid data layer** that `get_tile_info()` reads from—NOT the FFI dictionary packing or GDScript UI chain.

---

## 2. Core Data Structures

### TileGrid (`rust/crates/sim-core/src/tile_grid.rs`)

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TileGrid {
    width: u32,
    height: u32,
    tiles: Vec<StructuralTile>,
}

impl TileGrid {
    pub fn new(width: u32, height: u32) -> Self { ... }
    pub fn dimensions(&self) -> (u32, u32) { ... }
    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.width as i32 && y < self.height as i32
    }
    pub fn get(&self, x: u32, y: u32) -> &StructuralTile { ... }
    pub fn get_mut(&mut self, x: u32, y: u32) -> &mut StructuralTile { ... }
    pub fn get_furniture(&self, x: u32, y: u32) -> Option<&str> { ... }
    pub fn set_wall(&mut self, x: u32, y: u32, material_id: impl Into<String>, wall_hp: f64) { ... }
    pub fn set_floor(&mut self, x: u32, y: u32, material_id: impl Into<String>) { ... }
    pub fn set_door(&mut self, x: u32, y: u32) { ... }
    pub fn set_furniture(&mut self, x: u32, y: u32, furniture_id: impl Into<String>) { ... }
    pub fn assign_room(&mut self, x: u32, y: u32, room_id: RoomId) { ... }
    pub fn orthogonal_neighbors(&self, x: u32, y: u32) -> Vec<(u32, u32)> { ... }
}
```

### StructuralTile (`rust/crates/sim-core/src/tile_grid.rs`)

```rust
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StructuralTile {
    pub wall_material: Option<String>,      // Material ID
    pub floor_material: Option<String>,     // Material ID
    pub roof_material: Option<String>,      // Material ID
    pub wall_hp: f64,                       // Hit points
    pub room_id: Option<RoomId>,            // Room assignment
    pub is_door: bool,                      // Door flag
    pub furniture_id: Option<String>,       // Furniture ID
}

impl StructuralTile {
    pub fn blocks_room_flow(&self) -> bool { ... }
    pub fn is_room_floor(&self) -> bool { ... }
}
```

### Room (`rust/crates/sim-core/src/room.rs`)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoomId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoomRole {
    Unknown,
    Shelter,
    Hearth,
    Storage,
    Crafting,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Room {
    pub id: RoomId,
    pub tiles: Vec<(u32, u32)>,
    pub enclosed: bool,
    pub role: RoomRole,
}
```

---

## 3. SimResources (`rust/crates/sim-engine/src/engine.rs`)

The shared non-component data passed to every system includes:

```rust
pub struct SimResources {
    pub calendar: GameCalendar,
    pub map: WorldMap,
    pub settlements: HashMap<SettlementId, Settlement>,
    pub buildings: HashMap<BuildingId, Building>,
    // ... other fields ...
    pub tile_grid: TileGrid,              // ← THE GRID WE READ FROM
    pub rooms: Vec<Room>,                 // ← ROOM METADATA
    // ... more fields ...
}
```

**Access pattern**: `resources.tile_grid.get(x, y)` returns `&StructuralTile`

---

## 4. SimBridge FFI Pattern

### Example #[func] Method

File: `rust/crates/sim-bridge/src/lib.rs`

```rust
#[godot_api]
impl WorldSimRuntime {
    #[func]
    fn runtime_init(&mut self, seed: i64, config_json: GString) -> bool {
        let Some(state) = self.state.as_mut() else {
            return false;
        };
        // ... implementation ...
        true
    }
}
```

**Key pattern**:
- Check `self.state.as_ref()` to access `RuntimeState`
- Access simulation via `state.engine.resources()` or `state.engine.world()`
- Convert Rust types → Godot types (f64, GString, Dictionary, Array)
- Return converted value to Godot

### Type Conversion Rules
| Rust | Godot | Method |
|------|-------|--------|
| `f64` | `f64` | direct |
| `String` | `GString` | `.into()` |
| `Option<String>` | Check with `.is_some()`, use `.as_deref()` |
| Struct → | `Dictionary` | Create dict, set fields, return |
| `u32`, `i32` | `i64` | Cast and return |

---

## 5. Harness Test Pattern

### Test Structure (`rust/crates/sim-test/src/main.rs`)

```rust
#[test]
fn harness_stone_collected_after_one_year() {
    let mut engine = make_stage1_engine(42, 20);  // seed, agent_count
    engine.run_ticks(4380);  // 1 year = 4380 ticks

    let resources = engine.resources();

    // Type C assertion: existence check
    let total_stone: f64 = resources.map.tiles.iter()
        .filter_map(|t| t.resources.stone.as_ref())
        .map(|r| r.amount)
        .sum();

    assert!(total_stone > 0.0, "No stone collected after one year");
    println!("[harness] harness_stone_collected_after_one_year: PASS");
}
```

### Harness Test Functions
- `make_stage1_engine(seed: u64, agent_count: usize) -> SimEngine`
  - Creates a fresh engine with agents spawned at a test settlement
  - Loads personality distribution from data
  - Seeds tile resources near spawn point
  - Registers all 65 systems

- `engine.run_ticks(N)` — runs N ticks
- `engine.resources()` — accesses SimResources
- `engine.world()` — accesses hecs::World with entities

### Assertion Patterns
- **Type A (Invariants)**: `assert!(condition, "message")` with specific logic
- **Type C (Existence)**: `assert!(count >= threshold, "threshold not met")`
- **Type E (Bounds)**: `assert!(in_bounds(...), "coordinate OOB")`

---

## 6. FFI Call Chain — Existing Pattern

### Example: Building Click (from prompt)

**Rust (sim-bridge)**:
```rust
#[func]
fn get_building_detail(&self, building_id: i64) -> Dictionary {
    let Some(state) = self.state.as_ref() else {
        return Dictionary::new();
    };
    let resources = state.engine.resources();
    let Some(building) = resources.buildings.get(&BuildingId(building_id as u32)) else {
        return Dictionary::new();
    };

    let mut dict = Dictionary::new();
    dict.set("id", building.id.0 as i64);
    dict.set("type", GString::from(&building.building_type));
    dict.set("x", building.x as i64);
    dict.set("y", building.y as i64);
    // ... more fields ...
    dict
}
```

**GDScript (sim_bridge.gd)**:
```gdscript
func get_building_detail(building_id: int) -> Dictionary:
    var runtime: Object = _get_native_runtime()
    if runtime == null or not runtime.has_method("get_building_detail"):
        return {}
    var raw: Variant = runtime.call("get_building_detail", building_id)
    if raw is Dictionary:
        return raw
    return {}
```

**GDScript (entity_renderer.gd)**:
```gdscript
# Click handling
if building == null:
    return
SimulationBus.building_selected.emit(building.id, building)
```

---

## 7. Implementation Road Map

### Part A: SimBridge `get_tile_info()` — Rust
**File**: `rust/crates/sim-bridge/src/lib.rs`

**Signature** (from prompt):
```rust
#[func]
fn get_tile_info(&self, tile_x: i64, tile_y: i64) -> VarDictionary {
    let Some(state) = self.state.as_ref() else {
        return VarDictionary::new();
    };
    let resources = state.engine.resources();
    let x = tile_x as u32;
    let y = tile_y as u32;
    if !resources.tile_grid.in_bounds(tile_x as i32, tile_y as i32) {
        return VarDictionary::new();
    }
    let tile = resources.tile_grid.get(x, y);

    // Build dictionary with wall/floor/furniture/room info
    // Return it
}
```

**Key methods to use**:
- `resources.tile_grid.in_bounds(x, y)` — bounds check (i32 input)
- `resources.tile_grid.get(x, y)` — get tile (u32 input)
- `resources.tile_grid.get_furniture(x, y)` — get furniture option
- `resources.rooms.iter().find(|r| r.id == room_id)` — look up room metadata

### Part B: GDScript Proxies
**Files**:
- `scripts/core/simulation/sim_bridge.gd` — wrap Rust method
- `scripts/core/simulation/simulation_engine.gd` — forward to sim_bridge

### Part C: Click Handler
**File**: `scripts/ui/renderers/entity_renderer.gd`

Add after building click check:
```gdscript
# Check tile_grid wall/furniture at clicked tile
if building == null and _sim_engine != null and _sim_engine.has_method("get_tile_info"):
    var tile_info: Dictionary = _sim_engine.get_tile_info(tile.x, tile.y)
    if tile_info.get("has_wall", false) or tile_info.get("has_furniture", false):
        selected_entity_id = -1
        SimulationBus.entity_deselected.emit()
        SimulationBus.tile_selected.emit(tile.x, tile.y, tile_info)
        return
```

### Part D: Signal Definition
**File**: `scripts/core/simulation/simulation_bus.gd`

```gdscript
signal tile_selected(tile_x: int, tile_y: int, tile_info: Dictionary)
signal tile_deselected
```

### Part E: HUD Display
**File**: `scripts/ui/hud.gd`

Connect and implement:
```gdscript
SimulationBus.tile_selected.connect(_on_tile_selected)

func _on_tile_selected(tile_x: int, tile_y: int, tile_info: Dictionary) -> void:
    _show_tile_info(tile_x, tile_y, tile_info)
```

### Part F: Localization Keys
**Files**: `localization/en/ui.json`, `localization/ko/ui.json`

22 keys needed (see prompt Part F)

---

## 8. Harness Test Structure (Pre-Implementation)

The test plan expects a harness test file to exist that validates tile_grid data integrity.

**Expected test function naming**: `harness_<category>_<assertion>`

Examples from plan:
- `harness_building_wall_hp_validity` (Type A)
- `harness_building_room_enclosed_floors_consistent` (Type A)
- `harness_building_structural_tiles_exist_after_one_year` (Type C)
- `harness_tile_grid_in_bounds_rejects_invalid` (Type E)

Each test:
1. Creates engine with `make_stage1_engine(42, 20)`
2. Runs for appropriate duration (1 year = 4380 ticks)
3. Queries `resources.tile_grid` for relevant data
4. Asserts invariant/existence/boundary condition
5. Prints `[harness] test_name: PASS` on success

---

## 9. Key Files Summary

| File | Role | Key Items |
|------|------|-----------|
| `rust/crates/sim-core/src/tile_grid.rs` | Data structure | TileGrid, StructuralTile, methods |
| `rust/crates/sim-core/src/room.rs` | Data structure | Room, RoomId, RoomRole |
| `rust/crates/sim-engine/src/engine.rs` | Resources container | SimResources with `tile_grid`, `rooms` |
| `rust/crates/sim-bridge/src/lib.rs` | FFI layer | `#[func]` methods, type conversion |
| `rust/crates/sim-test/src/main.rs` | Test harness | `make_stage1_engine()`, test patterns |
| `scripts/core/simulation/sim_bridge.gd` | GDScript proxy | Wraps Rust methods |
| `scripts/core/simulation/simulation_engine.gd` | GDScript proxy | Forwards to sim_bridge |
| `scripts/core/simulation/simulation_bus.gd` | Signal bus | Emits events for UI |
| `scripts/ui/renderers/entity_renderer.gd` | Click handler | Detects clicks, emits signals |
| `scripts/ui/hud.gd` | UI display | Shows selected entity/building/tile info |

---

## 10. Critical Implementation Notes

### 1. Coordinate Systems
- **Rust tile_grid**: `u32` for storage (`get(x: u32, y: u32)`)
- **in_bounds check**: Takes `i32` (`in_bounds(x: i32, y: i32)`)
- **From GDScript**: Comes as `int` (which is i32 in Godot)
- **Conversion**: Cast i64 (from Godot) → u32 for `get()`, i32 for `in_bounds()`

### 2. Option Handling
- `tile.wall_material: Option<String>` → check `.is_some()`, use `.as_deref()`
- `tile.room_id: Option<RoomId>` → pattern match or check `.is_some()`
- Return empty dictionary if out of bounds

### 3. Room Lookup
```rust
if let Some(room_id) = tile.room_id {
    if let Some(room) = resources.rooms.iter().find(|r| r.id == room_id) {
        dict.set("room_role", GString::from(format!("{:?}", room.role)));
    }
}
```

### 4. GDScript Signal Emission
```gdscript
SimulationBus.tile_selected.emit(tile.x, tile.y, tile_info)
# Must match signal definition:
signal tile_selected(tile_x: int, tile_y: int, tile_info: Dictionary)
```

### 5. Localization Pattern
All UI strings via `Locale.ltr("KEY")` — never hardcoded strings

---

## 11. Dispatch Strategy (from prompt)

| Part | File | Language | Mode | Notes |
|------|------|----------|------|-------|
| T1 | sim-bridge/src/lib.rs | Rust | DISPATCH | Single func, self-contained |
| T2 | sim_bridge.gd | GDScript | DIRECT | Simple proxy, depends on T1 |
| T3 | simulation_engine.gd | GDScript | DIRECT | Simple forward, depends on T2 |
| T4 | simulation_bus.gd + entity_renderer.gd | GDScript | DIRECT | Signal + click handler, depends on T3 |
| T5 | hud.gd | GDScript | DISPATCH | Display logic, depends on T4 |
| T6 | localization/*.json | — | DISPATCH | Key translations |

**Dispatch ratio**: 3/6 = 50% (T1, T5, T6)

---

## 12. Verification & Testing

### Unit Test (Rust)
```bash
cargo test -p sim-test harness_tile_grid_in_bounds_rejects_invalid -- --nocapture
```

### Integration Test (Headless Godot)
```bash
bash tools/harness/harness_pipeline.sh wall-click-info .harness/prompts/wall-click-info.md --quick
```

### In-Game Verification
1. Z1 zoom level
2. Click wall tile → sidebar shows "벽: 화강암, 내구도: 100"
3. Click furniture → shows "가구: 화덕"
4. Click empty tile → no change
5. Agent/building clicks unchanged

---

## 13. Conclusion

The codebase is well-structured for this feature:

✅ **TileGrid** — fully implemented with all needed methods
✅ **SimResources** — provides access to `tile_grid` and `rooms`
✅ **SimBridge** — clear #[func] pattern for FFI exposure
✅ **GDScript proxies** — established pattern in sim_bridge.gd
✅ **Signal bus** — SimulationBus ready for new signals
✅ **Click handler** — entity_renderer.gd has precedent
✅ **Harness tests** — clear assertion patterns

No architectural barriers. Implementation is straightforward translator work.

