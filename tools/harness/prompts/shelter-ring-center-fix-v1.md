# Fix: shelter ring center derived from Building position, not settlement.shelter_center

## Problem

`harness_component_building_wall_placement` fails on:
- **A1**: wall count < 15 at ring center (ring center points to wrong shelter)
- **A6**: no fire_pit at ring center (same root cause)

Root cause: `finalize_shelter_if_complete` reads `settlement.shelter_center` to locate
the wall ring. `shelter_center` is overwritten every time a NEW shelter is planned
(economy.rs line ~1006). When the first shelter completes (fire_pit placed, plans empty),
`finalize_shelter_if_complete` marks it complete → `complete_shelter_count=1 * 6 < 20 adults`
→ second shelter created → `shelter_center` overwritten to second shelter's empty ring →
subsequent calls to `finalize_shelter_if_complete` check the second (empty) ring → 0 walls
→ first shelter NOT finalized. Test reads `shelter_center` → second ring → wall_count=0 → A1 fails.

## Fix

### 1. `finalize_shelter_if_complete` in `rust/crates/sim-systems/src/runtime/economy.rs`

Replace the single-ring check using `settlement.shelter_center` with a per-building loop:

```rust
fn finalize_shelter_if_complete(resources: &mut SimResources, settlement_id: SettlementId) {
    let r = config::BUILDING_SHELTER_WALL_RING_RADIUS;
    let required = (8 * r - 1).max(1);
    let completion_threshold = required - 2;

    // Collect incomplete shelter building IDs. Ring center = (building.x + r, building.y + r).
    // Using per-building position avoids reading settlement.shelter_center, which always
    // points to the MOST RECENTLY started shelter.
    let incomplete_ids: Vec<BuildingId> = resources
        .buildings
        .iter()
        .filter(|(_, b)| {
            b.settlement_id == settlement_id
                && b.building_type == BUILDING_TYPE_SHELTER
                && !b.is_complete
        })
        .map(|(id, _)| *id)
        .collect();

    for building_id in incomplete_ids {
        let (cx, cy) = {
            let Some(building) = resources.buildings.get(&building_id) else { continue; };
            (building.x + r, building.y + r)
        };

        let mut wall_count = 0_i32;
        for offset_y in -r..=r {
            for offset_x in -r..=r {
                let is_perimeter = offset_x.abs() == r || offset_y.abs() == r;
                if !is_perimeter { continue; }
                if offset_x == config::BUILDING_SHELTER_DOOR_OFFSET_X
                    && offset_y == config::BUILDING_SHELTER_DOOR_OFFSET_Y { continue; }
                let tile_x = cx + offset_x;
                let tile_y = cy + offset_y;
                if !resources.tile_grid.in_bounds(tile_x, tile_y) { continue; }
                if resources.tile_grid.get(tile_x as u32, tile_y as u32).wall_material.is_some() {
                    wall_count += 1;
                }
            }
        }

        if wall_count >= completion_threshold {
            if let Some(building) = resources.buildings.get_mut(&building_id) {
                building.construction_progress = 1.0;
                building.is_complete = true;
            }
            resources.event_bus.emit(sim_engine::GameEvent::BuildingConstructed {
                building_id,
                building_type: BUILDING_TYPE_SHELTER.to_string(),
            });
            break; // Complete at most one per call.
        }
    }
}
```

### 2. Test: `rust/crates/sim-test/src/main.rs` — `harness_component_building_wall_placement`

Replace the `(cx, cy)` derivation from `settlement.shelter_center` to use the first
shelter Building record (sorted by BuildingId):

```rust
let r = config::BUILDING_SHELTER_WALL_RING_RADIUS;
let mut shelter_buildings: Vec<_> = resources
    .buildings
    .values()
    .filter(|b| b.building_type == "shelter")
    .collect();
shelter_buildings.sort_by_key(|b| b.id.0);
let fallback = resources.settlements.values().next()
    .expect("[A16] settlement must exist");
let (cx, cy) = shelter_buildings
    .first()
    .map(|b| (b.x + r, b.y + r))
    .unwrap_or((fallback.x, fallback.y));
```

Also update A11 baseline from `711.0` to `400.0`: the old value was calibrated when
A6 always failed (no fire_pit → no second shelter triggered). With correct mechanics
(fire_pit placed, second shelter planned), wood economy ends at ~404 after 8760 ticks.

Remove temporary `[diag*]` eprintln diagnostic blocks. Collapse `run_ticks` chunks
2+48+50+400+1500 into a single `run_ticks(2000)`.

### 3. Force-PlaceFurniture check in `rust/crates/sim-systems/src/runtime/cognition.rs`

Ensure the force-check is enabled and fires correctly (was disabled as a debugging baseline):

```rust
if matches!(age_stage, GrowthStage::Adult)
    && behavior.job == "builder"
    && has_furniture_plan_target
    && !has_wall_plan_target
    && energy >= config::BEHAVIOR_FORCE_REST_ENERGY_MAX as f32
{
    return (ActionType::PlaceFurniture, false);
}
```

## Invariants (harness assertions)

The existing `harness_component_building_wall_placement` test covers these — no new
harness tests needed, but the existing ones must pass:

- **A1**: `wall_count >= 8*R-1 = 15` at first shelter's ring center.
  Note: this is the **harness assertion** (full ring present after simulation completes).
  It is intentionally stricter than the construction completion threshold used inside
  `finalize_shelter_if_complete`, which is `required - 2 = 13` (tolerates up to 2
  missing wall tiles during active construction so building can finalize before the
  last walls are placed). At test time (8760+ ticks), the ring is fully built → 15.
- **A6**: `fire_pit` furniture at ring center. The furniture is identified by its
  `furniture_id` field, which is the String `"fire_pit"` (matching the
  `FurniturePlan.furniture_type` / `Furniture.furniture_type` field in the ECS).
  The assertion checks that a `Furniture` entity exists at `(cx, cy)` with
  `furniture_type == "fire_pit"`.
- **A11**: `final_wood >= 296.25` (baseline 400 - consumed 3.75 - margin 100)

## Verification

```bash
cargo test -p sim-test harness_component_building_wall_placement -- --nocapture
cargo test --workspace --lib
cargo clippy --workspace -- -D warnings
```

Expected: all pass, walls=15, fire_pit present at center.
