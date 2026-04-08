# P2-B1: Room Role Assignment from Furniture + Room Influence Effects

## Section 1: Implementation Intent

### Current state

The room detection pipeline is already running:
- `InfluenceSystem.run()` calls `refresh_structural_context()` when building layout changes
- `refresh_structural_context()` calls `detect_rooms()` → `assign_room_ids()` → stores in `resources.rooms`
- `detect_rooms()` BFS finds enclosed floor regions and assigns `RoomRole::Shelter` (enclosed) or `Unknown` (open)

But rooms have no gameplay effect beyond this:
- **No furniture→role mapping** — a room with a fire_pit (hearth) and a room with a workbench (crafting) both get `Shelter`
- **No room-based stat bonuses** — enclosed rooms don't provide warmth/comfort/safety bonuses to agents inside
- **No furniture tracking per room** — buildings have furniture lists but rooms don't know which furniture they contain

### What this solves

After this feature:
1. Each room knows which `FurnitureDef.role_contribution` tags are present within it
2. Room role is determined by furniture majority: hearth(fire_pit) → Hearth, shelter(lean_to) → Shelter, storage(storage_pit) → Storage, crafting(workbench) → Crafting
3. Enclosed rooms with role provide stat bonuses to agents inside via EffectPrimitive:
   - Shelter → Safety +0.1, Warmth +0.05
   - Hearth → Warmth +0.15, Comfort +0.1
   - Storage → (no direct agent bonus, future resource preservation)

### Design

This is NOT about agents building walls autonomously (that's P2-B2). This is about making the existing room detection pipeline **produce gameplay effects** — connecting the dots between "room exists" and "room does something."

---

## Section 2: What to Build

### Part A: Add Crafting to RoomRole enum

**File: `rust/crates/sim-core/src/room.rs`**

Add `Crafting` variant:

```rust
pub enum RoomRole {
    Unknown,
    Shelter,
    Hearth,
    Storage,
    Crafting,  // NEW
}
```

### Part B: Furniture-based role assignment

**File: `rust/crates/sim-systems/src/runtime/influence.rs`**

After `detect_rooms()` and `assign_room_ids()`, add a role assignment pass. This reads building positions and matches them against room tiles to determine which furniture (via building type → StructureDef → required_components → FurnitureDef.role_contribution) is in each room.

Since current buildings don't track individual furniture placement on tiles, use a simpler approach: **check which buildings overlap with which rooms**, and use the building's type to infer furniture role contributions.

Add after the existing `resources.rooms = rooms;` line in `refresh_structural_context()`:

```rust
assign_room_roles_from_buildings(resources);
```

New function:

```rust
fn assign_room_roles_from_buildings(resources: &mut SimResources) {
    let mut room_role_votes: HashMap<RoomId, Vec<String>> = HashMap::new();

    for building in resources.buildings.values() {
        if !building.is_complete {
            continue;
        }
        let bx = building.x.max(0) as u32;
        let by = building.y.max(0) as u32;
        let (grid_w, grid_h) = resources.tile_grid.dimensions();
        if bx >= grid_w || by >= grid_h {
            continue;
        }
        let tile = resources.tile_grid.get(bx, by);
        let Some(room_id) = tile.room_id else {
            continue;
        };

        let role = match building.building_type.as_str() {
            "campfire" => Some("hearth"),
            "shelter" => Some("shelter"),
            "stockpile" => Some("storage"),
            _ => None,
        };

        if let Some(role_str) = role {
            room_role_votes
                .entry(room_id)
                .or_default()
                .push(role_str.to_string());
        }
    }

    for room in &mut resources.rooms {
        if !room.enclosed {
            room.role = RoomRole::Unknown;
            continue;
        }
        let Some(votes) = room_role_votes.get(&room.id) else {
            room.role = RoomRole::Shelter;
            continue;
        };

        room.role = majority_role(votes);
    }
}

fn majority_role(votes: &[String]) -> RoomRole {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for vote in votes {
        *counts.entry(vote.as_str()).or_default() += 1;
    }

    let winner = counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(role, _)| role);

    match winner {
        Some("hearth") => RoomRole::Hearth,
        Some("shelter") => RoomRole::Shelter,
        Some("storage") => RoomRole::Storage,
        Some("crafting") => RoomRole::Crafting,
        _ => RoomRole::Shelter,
    }
}
```

### Part C: Room stat bonuses via EffectQueue

**File: `rust/crates/sim-systems/src/runtime/influence.rs`**

```rust
fn apply_room_effects(world: &World, resources: &mut SimResources, tick: u64) {
    if resources.rooms.is_empty() {
        return;
    }

    let room_lookup: HashMap<(u32, u32), &Room> = resources.rooms.iter()
        .flat_map(|room| room.tiles.iter().map(move |&tile| (tile, room)))
        .collect();

    for (entity, (position, _needs)) in world.query::<(&Position, &Needs)>().iter() {
        let entity_id = EntityId(entity.id() as u64);
        let tx = position.tile_x().max(0) as u32;
        let ty = position.tile_y().max(0) as u32;

        let Some(room) = room_lookup.get(&(tx, ty)) else {
            continue;
        };
        if !room.enclosed {
            continue;
        }

        match room.role {
            RoomRole::Shelter => {
                resources.effect_queue.push(EffectEntry {
                    entity: entity_id,
                    effect: EffectPrimitive::AddStat {
                        stat: EffectStat::Safety,
                        amount: 0.02,
                    },
                    source: EffectSource {
                        system: "room_effect".to_string(),
                        kind: "shelter_safety".to_string(),
                    },
                });
            }
            RoomRole::Hearth => {
                resources.effect_queue.push(EffectEntry {
                    entity: entity_id,
                    effect: EffectPrimitive::AddStat {
                        stat: EffectStat::Warmth,
                        amount: 0.03,
                    },
                    source: EffectSource {
                        system: "room_effect".to_string(),
                        kind: "hearth_warmth".to_string(),
                    },
                });
            }
            RoomRole::Crafting => {}
            RoomRole::Storage | RoomRole::Unknown => {}
        }
    }
}
```

### Part D: SimBridge room data

**File: `rust/crates/sim-bridge/src/lib.rs`**

```rust
let tx = position.tile_x().max(0) as u32;
let ty = position.tile_y().max(0) as u32;
if let Some(tile) = resources.tile_grid.get(tx, ty) {
    if let Some(room_id) = tile.room_id {
        dict.set("room_id", room_id.0 as i64);
        if let Some(room) = resources.rooms.iter().find(|r| r.id == room_id) {
            dict.set("room_role", format!("{:?}", room.role));
            dict.set("room_enclosed", room.enclosed);
        }
    }
}
```

---

## Section 3: How to Implement

### Key integration point

`refresh_structural_context()` in influence.rs already:
1. Stamps shelter walls/floors on tile_grid
2. Calls detect_rooms() + assign_room_ids()
3. Stores rooms in resources

Add step 4: `assign_room_roles_from_buildings(resources)`

For room effects, the cleanest approach is adding the `apply_room_effects()` call inside `InfluenceSystem.run()` after the structural refresh.

### Performance

- Role assignment: O(buildings × 1) — just a hashmap lookup per building
- Room effects: O(agents × 1) — tile lookup per agent, runs every influence tick (~10 ticks)
- Effect queue: already batched by EffectApplySystem

---

## Section 4: Dispatch Plan

| # | Ticket | File | Language | Mode | Depends On |
|---|--------|------|----------|:----:|:----------:|
| T1 | RoomRole::Crafting + role assignment | sim-core/room.rs + sim-systems/influence.rs | Rust | 🟢 DISPATCH | — |
| T2 | Room stat effects via EffectQueue | sim-systems/influence.rs | Rust | 🟢 DISPATCH | T1 |
| T3 | SimBridge room data | sim-bridge | Rust | 🔴 DIRECT | T1 |
| T4 | Harness test | sim-test/src/main.rs | Rust | 🟢 DISPATCH | T1, T2 |

---

## Section 5: Localization Checklist

No new localization keys (room roles are internal data, not player-facing text yet).

---

## Section 6: Verification & Harness

### Gate command

```bash
cd rust && cargo test --workspace && cargo clippy --workspace -- -D warnings
```

### Harness test

```rust
#[test]
fn harness_rooms_have_role_from_buildings() {
    let mut engine = make_stage1_engine(42, 20);
    engine.run_ticks(4380);

    let resources = engine.resources();

    let building_count = resources.buildings.values()
        .filter(|b| b.is_complete)
        .count();

    if building_count >= 3 {
        assert!(
            !resources.rooms.is_empty(),
            "expected rooms to be detected with {} complete buildings",
            building_count
        );

        let roles: Vec<_> = resources.rooms.iter()
            .filter(|r| r.enclosed)
            .map(|r| r.role)
            .collect();

        eprintln!("rooms: {}, enclosed roles: {:?}", resources.rooms.len(), roles);
    }

    let (grid_w, grid_h) = resources.tile_grid.dimensions();
    for room in &resources.rooms {
        for &(x, y) in &room.tiles {
            assert!(x < grid_w && y < grid_h, "room tile ({},{}) out of bounds", x, y);
        }
    }
}
```

---

## Section 7: 인게임 확인사항

1. **방 감지**: shelter 건설 후 그 주변에 방이 감지되는지 (room_id가 tile_grid에 할당).
2. **역할 할당**: campfire가 있는 방 = Hearth, shelter만 있는 방 = Shelter.
3. **안전/온기 보너스**: 방 안에 있는 에이전트의 Safety/Warmth가 방 밖보다 높은지.
4. **CausalLog**: "room_effect: shelter_safety" / "room_effect: hearth_warmth" 이벤트 기록.
5. **기존 harness 전부 통과**.
