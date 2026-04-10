---
feature: building-visuals
code_attempt: 1
---

## Files Changed
- `rust/crates/sim-systems/src/runtime/influence.rs`: Added `stamp_enclosed_floors()` for PlaceWall-based shelters; updated `building_structure_signature()` to include wall_plans/furniture_plans counts; added VecDeque import
- `rust/crates/sim-test/src/main.rs`: Added 6 harness tests (`harness_building_visuals_*`) covering floor tiles, wall materials, adjacent walls, storage_pit furniture, localization keys, and floor count bounds
- `scripts/ui/renderers/building_renderer.gd`: No changes — visual features (floor alpha 0.55, 0.5px border, icon size 0.7, stockpile label, wall autotile) were already implemented

## Observed Values (seed 42, 20 agents)
- floor_count: 10 (1 stockpile + 9 shelter interior)
- wall_count: 15
- empty_material_count: 0
- adjacent_wall_pairs: 14
- storage_pit_count: 1
- BUILDING_TYPE_STOCKPILE en: "Stockpile"
- BUILDING_TYPE_STOCKPILE ko: "보관소"

## Threshold Compliance
- Assertion 1 (floor_tiles_stamped): plan=≥6, observed=10, PASS
- Assertion 2 (wall_material_valid): plan=≥8 walls AND 0 empty, observed=15 walls / 0 empty, PASS
- Assertion 3 (adjacent_wall_pairs): plan=≥4, observed=14, PASS
- Assertion 4 (storage_pit_present): plan=≥1, observed=1, PASS
- Assertion 5 (localization_stockpile): plan=present+non-empty in both, observed=present in both, PASS
- Assertion 6 (floor_count_bounded): plan=≤500, observed=10, PASS

## Gate Result
- cargo test: PASS (100 passed, 1 failed pre-existing, 1 ignored)
- clippy: PASS (0 warnings)
- harness: PASS (6/6 passed)

## Notes
- **Data layer fix required**: The plan assumed floor tiles were pre-existing from P2-B3, but PlaceWall-based shelters create wall rings WITHOUT stamping interior floors. The shelter Building entity no longer exists in P2-B3, so `stamp_shelter_structure` (which stamps floors + doors) was never called. Fix: added `stamp_enclosed_floors()` which (1) seals single-tile gaps in wall perimeters as doors, (2) flood-fills from grid edges to identify enclosed areas, (3) stamps `packed_earth` floor on enclosed non-wall tiles.
- **Signature cache miss**: `building_structure_signature()` only hashed Building entities, not tile_grid wall changes from PlaceWall. Added wall_plans/furniture_plans counts to the signature so `refresh_structural_context` re-runs when walls are placed.
- **False door cleanup**: Added Step 0 in `stamp_enclosed_floors` to clear `is_door` flags on tiles that also have `wall_material` — a race condition artifact where a door was detected before a wall was placed at the same position on a subsequent tick.
- **Pre-existing test failure**: `harness_shift_rules_execute_at_least_once_over_two_years` fails (temperament shift system not implemented). Unrelated to building-visuals.
- **GDScript already implemented**: All three visual enhancements (floor alpha 0.55 + border, icon size 0.7 + stockpile label, wall autotile bridges) were already in `building_renderer.gd`. No GDScript changes needed.
