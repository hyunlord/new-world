# P2-B3: Component Building Phase 1 — Independent Wall/Furniture Entities

## Section 1: Implementation Intent

### The shift

Current system: `economy.rs` creates a `Building` → agent builds it → `stamp_shelter_structure()` auto-places walls. The agent never decides WHERE to put a wall.

Target: Agent decides "I need shelter" → selects a blueprint → walks to site → places walls one at a time → places furniture → room emerges from BFS.

### This phase (1 of 3)

Phase 1 focuses on **making walls and furniture independent placeable actions** while keeping the existing Building system running. The two systems coexist — old shelters still work via stamp, but agents can ALSO place individual walls and furniture.

### What gets built

1. **PlaceWall action** — agent places one wall tile (sets `tile_grid.set_wall()`)
2. **PlaceFurniture action** — agent places one furniture item on a tile
3. **Simple blueprint: `wall_ring` plan** — agent follows a sequence: gather materials → place walls in a ring → place floor → place furniture inside
4. **WallPlan / FurniturePlan queues** — per-settlement queue of planned wall/furniture placements that agents can pick up as tasks
5. **Cognition integration** — agents with builder job pick up wall/furniture placement tasks

### What does NOT change

- Existing `Building` struct, `EarlyStructurePlan`, `ConstructionRuntimeSystem` — all stay
- `stamp_shelter_structure()` still runs for the existing shelter flow
- No GOAP yet — simple task queue approach first

---

## Section 2: What to Build

### Part A: New ActionTypes

**File: `rust/crates/sim-core/src/enums.rs`**

Add two new action types at the END of ActionType enum (after VisitPartner):

```rust
pub enum ActionType {
    // ... existing 28 variants (0-27) ...
    VisitPartner,
    PlaceWall,       // NEW — index 28
    PlaceFurniture,  // NEW — index 29
}
```

**Important**: Adding at the end preserves existing discriminant values (0-27). The `#[repr(u8)]` attribute ensures this.

### Part B: WallPlan and FurniturePlan

**File: `rust/crates/sim-core/src/building.rs`** (or new file `building_plan.rs`)

```rust
/// A planned wall placement that an agent can pick up as a task.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WallPlan {
    pub id: u64,
    pub settlement_id: SettlementId,
    pub x: i32,
    pub y: i32,
    pub material_id: String,
    pub claimed_by: Option<EntityId>,
    pub created_tick: u64,
}

/// A planned furniture placement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FurniturePlan {
    pub id: u64,
    pub settlement_id: SettlementId,
    pub x: i32,
    pub y: i32,
    pub furniture_id: String,
    pub claimed_by: Option<EntityId>,
    pub created_tick: u64,
}
```

### Part C: Plan queues in SimResources

**File: `rust/crates/sim-engine/src/engine.rs`**

Add to SimResources:

```rust
pub wall_plans: Vec<WallPlan>,
pub furniture_plans: Vec<FurniturePlan>,
pub next_plan_id: u64,
```

Initialize with empty vecs and 1 in `SimResources::new()`.

### Part D: Blueprint-based plan generation

**File: `rust/crates/sim-systems/src/runtime/economy.rs`**

Add `generate_wall_ring_plans()` that creates a ring of WallPlan entries + a FurniturePlan for the fire pit when a settlement needs a shelter.

```rust
fn generate_wall_ring_plans(
    resources: &mut SimResources,
    settlement_id: SettlementId,
    center_x: i32,
    center_y: i32,
    tick: u64,
) {
    let wall_material = resolve_shelter_wall_material_for_plans(resources, settlement_id);
    let wall_radius = config::BUILDING_SHELTER_WALL_RING_RADIUS;
    
    for offset_y in -wall_radius..=wall_radius {
        for offset_x in -wall_radius..=wall_radius {
            let is_perimeter = offset_x.abs() == wall_radius || offset_y.abs() == wall_radius;
            if !is_perimeter {
                continue;
            }
            if offset_x == config::BUILDING_SHELTER_DOOR_OFFSET_X
                && offset_y == config::BUILDING_SHELTER_DOOR_OFFSET_Y
            {
                continue;
            }
            
            let plan_id = resources.next_plan_id;
            resources.next_plan_id += 1;
            resources.wall_plans.push(WallPlan {
                id: plan_id,
                settlement_id,
                x: center_x + offset_x,
                y: center_y + offset_y,
                material_id: wall_material.clone(),
                claimed_by: None,
                created_tick: tick,
            });
        }
    }
    
    let plan_id = resources.next_plan_id;
    resources.next_plan_id += 1;
    resources.furniture_plans.push(FurniturePlan {
        id: plan_id,
        settlement_id,
        x: center_x,
        y: center_y,
        furniture_id: "fire_pit".to_string(),
        claimed_by: None,
        created_tick: tick,
    });
}
```

### Part E: Cognition — agents pick up wall/furniture plans

**File: `rust/crates/sim-systems/src/runtime/cognition.rs`**

Builder agents with unclaimed wall/furniture plans get boosted PlaceWall/PlaceFurniture scores. Find nearest unclaimed plan as action target. Claim on selection.

### Part F: World system — execute PlaceWall/PlaceFurniture

**File: `rust/crates/sim-systems/src/runtime/world.rs`**

On action completion for PlaceWall/PlaceFurniture, set the tile and emit CausalEvent.

### Part G: Claim system

Agents claim plans on action selection. Unclaim on death or action change. Stale plans cleaned after 1000 ticks.

### Part H: Action timer + icon mapping

Config: `ACTION_TIMER_PLACE_WALL: i32 = 8`, `ACTION_TIMER_PLACE_FURNITURE: i32 = 12`.
GDScript icons: 28 → 🧱, 29 → 🪑.

---

## Section 3: How to Implement

### Coexistence with existing system

Both systems run simultaneously:
- `ensure_early_construction_sites()` still creates `Building` entries for stockpile/campfire
- For shelter: call `generate_wall_ring_plans()` which creates individual WallPlan/FurniturePlan entries
- Agents with builder job pick up plans and execute them one at a time
- `detect_rooms()` detects the enclosed room from component walls
- `assign_room_roles_from_buildings()` assigns the role

### The transition flag

In `choose_early_structure_plan()`, when the plan would be `Shelter`:
- Call `generate_wall_ring_plans()` directly
- Return `None` so no `Building` is created for shelter
- Stockpile and Campfire still use the old `Building` path

---

## Section 4: Dispatch Plan

| # | Ticket | File | Language | Mode | Depends On |
|---|--------|------|----------|:----:|:----------:|
| T1 | ActionType PlaceWall/PlaceFurniture | sim-core/src/enums.rs | Rust | 🟢 DISPATCH | — |
| T2 | WallPlan/FurniturePlan structs | sim-core/src/building.rs | Rust | 🟢 DISPATCH | — |
| T3 | SimResources plan queues | sim-engine/src/engine.rs | Rust | 🔴 DIRECT | T2 |
| T4 | Blueprint plan generation | sim-systems/src/runtime/economy.rs | Rust | 🟢 DISPATCH | T2, T3 |
| T5 | Cognition: plan pickup + claim | sim-systems/src/runtime/cognition.rs | Rust | 🟢 DISPATCH | T1, T2 |
| T6 | World: PlaceWall/PlaceFurniture execution | sim-systems/src/runtime/world.rs | Rust | 🟢 DISPATCH | T1, T2 |
| T7 | Config + GDScript icon update | config.rs + entity_renderer.gd | Rust+GD | 🟢 DISPATCH | T1 |
| T8 | Harness test | sim-test/src/main.rs | Rust | 🟢 DISPATCH | T4, T5, T6 |

---

## Section 5: Localization Checklist

| Key | en | ko |
|-----|----|----|
| ACTION_PLACE_WALL | Place Wall | 벽 건설 |
| ACTION_PLACE_FURNITURE | Place Furniture | 가구 배치 |

---

## Section 6: Verification & Harness

### Gate

```bash
cd rust && cargo test --workspace && cargo clippy --workspace -- -D warnings
```

### Harness test

```rust
#[test]
fn harness_component_building_wall_placement() {
    let mut engine = make_stage1_engine(42, 20);
    engine.run_ticks(8760);
    
    let resources = engine.resources();
    
    let (grid_w, grid_h) = resources.tile_grid.dimensions();
    let mut wall_tiles = 0u32;
    for y in 0..grid_h {
        for x in 0..grid_w {
            if resources.tile_grid.get(x, y).wall_material.is_some() {
                wall_tiles += 1;
            }
        }
    }
    
    assert!(wall_tiles > 0, "expected wall tiles after 2 years, found 0");
    
    let stale_walls = resources.wall_plans.iter()
        .filter(|p| p.claimed_by.is_none() && 8760 - p.created_tick > 2000)
        .count();
    eprintln!("wall_tiles: {}, pending wall_plans: {}, stale: {}", 
        wall_tiles, resources.wall_plans.len(), stale_walls);
}
```

---

## Section 7: 인게임 확인사항

1. **벽 배치**: 에이전트가 PlaceWall 행동으로 타일에 벽을 놓는지.
2. **가구 배치**: 벽 완성 후 에이전트가 PlaceFurniture로 가구를 놓는지.
3. **방 감지**: 벽이 완성되면 BFS가 enclosed room을 감지하는지.
4. **행동 아이콘**: 🧱 (PlaceWall) / 🪑 (PlaceFurniture) 아이콘이 보이는지.
5. **CausalLog**: "place_wall" / "place_furniture" 이벤트가 기록되는지.
6. **기존 건물**: stockpile/campfire가 기존 방식으로 계속 작동하는지.
7. **FPS 영향 없음**.
