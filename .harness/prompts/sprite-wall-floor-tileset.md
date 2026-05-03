# Feature: sprite-wall-floor-tileset

## Summary

Replace solid-color wall/floor tile rendering in `building_renderer.gd` with 16×16 material
sprite textures. Add 30 PNG sprites (21 wall + 9 floor) and delete the legacy
`assets/sprites/buildings/shelter.png` placeholder.

## What Was Built

### Assets
- `assets/sprites/walls/{granite,basalt,limestone,sandstone,oak,birch,pine}/{1,2,3}.png` — 21 PNGs
- `assets/sprites/floors/{packed_earth,stone_slab,wood_plank}/{1,2,3}.png` — 9 PNGs
- `assets/sprites/buildings/shelter.png` — **deleted** (tile-grid rendering takes over)

All sprites are 16×16, fully opaque RGB PNG. Stone materials use speckle noise; wood uses
horizontal grain; packed_earth uses irregular soil texture; wood_plank uses vertical divisions.

### GDScript — `scripts/ui/renderers/building_renderer.gd`

**New functions:**
- `_load_wall_material_texture(material_id, tile_x, tile_y) -> Texture2D`
  Loads `res://assets/sprites/walls/{material_id}/{variant}.png`, variant chosen by
  `_pick_variant_for_tile(tx, ty, count)` (prime-multiplier deterministic hash).
  Cache key: `"wall_mat/{material_id}/{variant_idx}"` into `_building_textures`.
  Returns null if no sprite folder exists (caller falls back to solid color).
- `_load_floor_material_texture(material_id, tile_x, tile_y) -> Texture2D`
  Mirror for `res://assets/sprites/floors/`. Cache key: `"floor_mat/..."`.

**Modified: `_draw_wall_tile()`**
- Added `material_id: String` parameter (before `wall_set`).
- Fill: `draw_texture_rect(tex, rect, false)` when sprite loads, else `draw_rect(color)`.
- Outline logic unchanged — code draws perimeter outlines, sprites ship WITHOUT outlines.

**Modified: `_draw_tile_grid_walls()` — wall call site**
- Passes `mat` (from `wall_mats[i]`) to `_draw_wall_tile(...)`.

**Modified: `_draw_tile_grid_walls()` — floor rendering**
- Floor material defaults to `"packed_earth"` (floor_material not yet exported as
  PackedStringArray by sim-bridge; tracked for Feature 3.5).
- `draw_texture_rect(floor_tex, rect, false)` when sprite loads, else existing solid fill.

### Rust — `rust/crates/sim-test/src/main.rs`

**Updated:** `harness_sprite_assets_round1_a5_shelter_preserved`
- Assertion flipped: now checks `!shelter_path.exists()` (Feature 3 deleted the placeholder).

**New module:** `harness_sprite_wall_floor_tileset`
- `harness_sprite_wall_floor_tileset_a13_wall_material_dirs` — 7 wall dirs × ≥3 PNGs each
- `harness_sprite_wall_floor_tileset_a14_floor_material_dirs` — 3 floor dirs × ≥3 PNGs each

## Visual Checks

### Scenario 1: Shelter wall rendering
- Wall tiles show **material-specific texture** (granite=gray speckle, oak=dark wood grain).
- NOT plain solid gray/brown fallback rectangles.
- Outline (perimeter edge lines) still drawn by code — NOT baked into sprites (no double lines).

### Scenario 2: Shelter floor rendering
- Floor tiles show **packed_earth texture** (dark soil, irregular noise) instead of flat brown.
- Floor is fully opaque (no transparency artifacts).
- Furniture sprites render correctly on top of floor.

### Scenario 3: Non-shelter buildings unchanged
- Campfire, stockpile, cairn, etc. render unchanged.
- Agent rendering unchanged.
- Map terrain unchanged.

### DENY Criteria
- Wall tiles show only flat solid color (sprite load failed for all materials) → FAIL
- Floor shows transparency/alpha-blended look → FAIL
- Double outline lines on wall tiles (sprite contains outline) → FAIL
- New Godot console "Failed to load" errors for wall/floor sprites → FAIL
- FPS drops >10% from Feature 2 baseline (32-33 → below ~29) → FAIL

## Harness Gate Commands

```bash
# Rust workspace
cd rust && cargo test --workspace && cargo clippy --workspace -- -D warnings

# Specific assertions
cargo test -p sim-test -- harness_sprite_wall_floor_tileset --nocapture
cargo test -p sim-test -- harness_sprite_assets_round1_a5 --nocapture
```

Expected: A13/A14/A5 all PASS.
