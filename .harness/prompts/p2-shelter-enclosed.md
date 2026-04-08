# P2-B2: Shelter Enclosed Room Fix + Wall Construction Foundations

## Section 1: Implementation Intent

### Root cause of enclosed room = 0

`stamp_shelter_structure()` in influence.rs had two problems:

1. **Floor only on center tile** — `set_floor(center_x, center_y, ...)` set 1 floor tile. But walls were placed in a ring at radius=1 (8 tiles, minus 1 door = 7 walls). The single floor tile was "enclosed" but `detect_rooms()` requires `is_room_floor()` → `floor_material.is_some()`. Only 1 tile had floor, so the "room" was just 1 tile.

2. **Door gap breaks enclosure** — The door at `(offset_x=0, offset_y=1)` had no wall, so BFS could escape through it. `blocks_room_flow()` returned false for tiles without `wall_material`, including the door tile.

### Solution

1. **Set floor on ALL interior tiles** — For wall_ring_radius=2 shelter, interior is the 3×3 center = 9 floor tiles.

2. **Door handling** — Add `is_door: bool` on StructuralTile. Door tiles block room flow (BFS boundaries) but can be distinguished from walls by downstream systems.

---

## Section 2: What to Build

### Part A: Add is_door flag to StructuralTile
`rust/crates/sim-core/src/tile_grid.rs` — new `is_door: bool` field with `#[serde(default)]`, updated `blocks_room_flow()`, new `set_door()` method.

### Part B: Expand shelter floor stamping + door marking
`rust/crates/sim-systems/src/runtime/influence.rs` — `stamp_shelter_structure()` now:
- Stamps floor + roof on ALL interior tiles (3×3 for radius=2)
- Perimeter walls with door tile marked via `set_door()` instead of skipping
- Caller passes center = `building.x + width/2, building.y + height/2`

### Part C: Increase wall ring radius
`rust/crates/sim-core/src/config.rs`:
- `BUILDING_SHELTER_WALL_RING_RADIUS: i32 = 2` (was 1)
- `BUILDING_SHELTER_DOOR_OFFSET_Y: i32 = 2` (was 1)

`rust/crates/sim-data/data/structures/shelters.ron`:
- `min_size: (5, 5)` (was (2, 2))

### Part E: Harness test

```rust
#[test]
fn harness_shelter_creates_enclosed_room() {
    let mut engine = make_stage1_engine(42, 20);
    engine.run_ticks(4380);

    let resources = engine.resources();
    let complete_shelters = resources.buildings.values()
        .filter(|b| b.is_complete && b.building_type == "shelter")
        .count();
    let enclosed_rooms = resources.rooms.iter().filter(|r| r.enclosed).count();

    if complete_shelters > 0 {
        assert!(enclosed_rooms > 0, "expected enclosed rooms from {} shelters", complete_shelters);
    }
    // door tile invariants + floor count lower bound
}
```

---

## Section 3: Dispatch Plan

| # | Ticket | File | Mode |
|---|--------|------|:----:|
| T1 | StructuralTile is_door + set_door() | sim-core/src/tile_grid.rs | 🟢 |
| T2 | Shelter stamp fix (interior floor + door) | sim-systems/src/runtime/influence.rs | 🟢 |
| T3 | Wall ring radius 1→2 + shelters.ron | sim-core/config.rs + sim-data | 🟢 |
| T4 | Harness test | sim-test/src/main.rs | 🟢 |

## Section 4: Verification
```bash
cd rust && cargo test --workspace && cargo clippy --workspace -- -D warnings
```

## Section 5: 인게임 확인
1. shelter 완성 후 내부 3×3 floor에 room_id 할당
2. shelter 방 역할이 Shelter
3. 문 위치에 is_door=true, 벽 아님
4. Safety 보너스 방 안 에이전트에 적용
5. 건물 겹침 없음
