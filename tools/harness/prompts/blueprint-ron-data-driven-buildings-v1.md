# P2-B4: Blueprint RON — Data-Driven Buildings

## Feature slug
`blueprint-ron-data-driven-buildings-v1`

## What was built

Extended `rust/crates/sim-data/data/structures/shelters.ron` with 3 new
StructureDef entries that use the existing inline `Blueprint` mechanism
(already present on `StructureDef` and working for "shelter").

The `Blueprint` struct, `BlueprintTile`, `BlueprintFurniture`, and
`generate_plans_from_blueprint()` are all pre-existing and fully working.
The "shelter" StructureDef already uses the blueprint path.

### New StructureDefs added to shelters.ron

1. **small_shelter** (3×3 outer ring)
   - 7 walls (3×3 ring − 1 door gap at top)
   - 1 floor tile (packed_earth)
   - 1 furniture: fire_pit at center
   - 1 door at (0, −1)

2. **hearth_house** (5×5 outer ring)
   - 15 walls (5×5 ring − 1 door gap at top)
   - 9 floor tiles (3×3 interior, packed_earth)
   - 2 furniture: fire_pit at center, lean_to at (−1, −1)
   - 1 door at (0, −2)

3. **storage_hut** (3×3 outer ring)
   - 7 walls (3×3 ring − 1 door gap at top)
   - 1 floor tile (packed_earth)
   - 1 furniture: storage_pit at center
   - 1 door at (0, −1)
   - role_recognition: Manual("storage")

### Localization keys added
- `BUILDING_TYPE_SMALL_SHELTER` (en: "Small Shelter", ko: "작은 거처")
- `BUILDING_TYPE_HEARTH_HOUSE` (en: "Hearth House", ko: "화로 가옥")
- `BUILDING_TYPE_STORAGE_HUT` (en: "Storage Hut", ko: "창고")

### Harness tests added (8, in mod harness_blueprint_v1)
1. `harness_blueprint_shelter_has_blueprint` — existing "shelter" has blueprint
2. `harness_blueprint_shelter_wall_count` — 15 walls (5×5 ring − 1 door)
3. `harness_blueprint_shelter_door_not_in_walls` — door gap not in wall list
4. `harness_blueprint_small_shelter_in_registry` — small_shelter: 7 walls, 1 door, ≥1 furniture
5. `harness_blueprint_hearth_house_in_registry` — hearth_house: ≥7 walls, 1 door, ≥1 furniture
6. `harness_blueprint_storage_hut_in_registry` — storage_hut: 7 walls, 1 door
7. `harness_blueprint_all_new_doors_not_in_walls` — door offsets absent from wall lists
8. `harness_blueprint_legacy_no_blueprint` — stockpile & campfire have blueprint=None

## Pre-existing state
- `Blueprint` struct + all fields: already on StructureDef (`#[serde(default)]`)
- `generate_plans_from_blueprint()`: fully implemented in economy.rs
- `shelter` StructureDef: already uses inline blueprint (15 walls + floors + furniture)
- DataRegistry loads all structures from `data/structures/*.ron` automatically

## Scope boundary
- No economy.rs changes (new structures not yet in early planning AI)
- No Rust type changes (StructureDef already has blueprint field)
- No new loader (DataRegistry.load_from_directory handles it)
- Fallback preserved: stockpile/campfire have no blueprint → legacy path

## Verification
```bash
cargo test -p sim-test harness_blueprint_v1 -- --nocapture
# Expected: 8 passed, 0 failed

cargo test -p sim-data
# Expected: 5 passed, 0 failed

cargo clippy --workspace -- -D warnings
# Expected: clean
```

Note: `tests::harness_blueprint_regression_shelter_complete` was ALREADY
failing before this commit (confirmed via git stash check). Not introduced
by these changes.
