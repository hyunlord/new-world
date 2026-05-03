---
feature: sprite-wall-floor-tileset
code_attempt: 3
---

## Files Changed
- `rust/crates/sim-test/src/main.rs`: Replaced A11 test body with stronger version — now extracts `_load_wall_material_texture` and `_load_floor_material_texture` function bodies individually and asserts each contains the exact format string `"%s/%d.png" % [variant_dir_res, variant_idx + 1]`; also asserts `_draw_wall_tile` calls `_load_wall_material_texture(material_id, wx, wy)` and `_draw_tile_grid_walls` calls `_load_floor_material_texture(`. No production code changed.

## Observed Values (seed 42, 20 agents — ticks: 0, pure filesystem assertions)
- shelter.png exists: false (absent ✓)
- missing wall PNGs (21 expected): 0
- missing floor PNGs (9 expected): 0
- invalid PNG signatures (of 30): 0
- wrong-dimension PNGs (not 16×16, of 30): 0
- wrong-colortype PNGs (not RGB type-2, of 30): 0
- granite/1.png != oak/1.png: true (stone ≠ wood)
- all 3 floor materials distinct: true
- wall materials with all-identical variants: 0
- `_load_wall_material_texture` body contains `"%s/%d.png" % [variant_dir_res, variant_idx + 1]`: true
- `_load_floor_material_texture` body contains `"%s/%d.png" % [variant_dir_res, variant_idx + 1]`: true
- `_draw_wall_tile` calls `_load_wall_material_texture(material_id, wx, wy)`: true
- `_draw_tile_grid_walls` calls `_load_floor_material_texture(`: true

## Threshold Compliance
- A1  (shelter_placeholder_deleted): plan=NOT exists, observed=absent, PASS
- A2  (all_21_wall_pngs_present_by_exact_name): plan=missing_count==0, observed=0, PASS
- A3  (all_9_floor_pngs_present_by_exact_name): plan=missing_count==0, observed=0, PASS
- A4  (all_30_pngs_have_valid_png_signature): plan=invalid_signature_count==0, observed=0, PASS
- A5  (all_30_pngs_are_16x16_pixels): plan=wrong_dimension_count==0, observed=0, PASS
- A6  (all_30_pngs_are_rgb_no_alpha): plan=wrong_colortype_count==0, observed=0, PASS
- A7  (stone_and_wood_wall_textures_are_not_identical): plan=content_differs==true, observed=true, PASS
- A8  (floor_textures_are_not_identical): plan=content_differs==true, observed=true, PASS
- A9  (wall_variant_files_are_not_all_identical_within_material): plan=same_variants_count==0, observed=0, PASS
- A11 (gdscript_uses_one_based_variant_index — upgraded): exact format string in wall loader=true, exact format string in floor loader=true, _draw_wall_tile wiring=true, _draw_tile_grid_walls wiring=true, PASS

## Gate Result
- cargo test: PASS (workspace exit code 0; sim-test 13 harness tests passed)
- clippy: PASS (exit code 0, no warnings)
- harness: PASS (13/13 sprite-wall-floor-tileset + 1/1 harness_sprite_assets_round1_a5)

## Notes
- **Why no RED phase**: The assets, GDScript functions, and previous A11 test skeleton were all already implemented by attempt 2. Attempt 3's change is purely a test-quality upgrade — the A11 test was too weak (global `variant_idx + 1` count ≥ 2 would pass even if `_load_wall_material_texture`/`_load_floor_material_texture` were deleted, because `building_variant_path` and `furniture_variant_path` each also use `variant_idx + 1`). The new A11 assertions are scoped to the specific function bodies and cannot be gamed by pre-existing code.
- **Regression safety**: If `_load_wall_material_texture` is deleted → `extract_func_body` returns None → `.expect()` panics → test fails. If the format string is changed to 0-based → exact format match fails. If `_draw_wall_tile` stops calling `_load_wall_material_texture(material_id, wx, wy)` → wiring assertion fails. Same logic for floor loader.
- `harness_sprite_assets_round1_a5_shelter_preserved` PASS — shelter.png correctly absent.
- All 30 PNG assets confirmed: valid PNG magic bytes, 16×16 IHDR, color type 0x02 (RGB no alpha).
