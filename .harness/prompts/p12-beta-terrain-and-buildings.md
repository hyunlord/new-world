# Phase 12-β — TileMapLayer Floor Terrain + Bootstrap Building Sprite

Feature: p12-beta-terrain-and-buildings
Lane: --quick (TileSet `.tres` resource + GDScript WorldRenderer
extension + sprite assignment; no Rust crate change)
Parent: Section 13+ (`6843176f`) + `.harness/plans/phase12.md` (local)
+ Phase 12-α (`7f6a6d76`).

## Section 1: Implementation Intent

Phase 12-β is the **second sub-stage** of the Phase 12 sprint. Where α
delivered visible-delta proof for D1 STATE_TINTS via 2× default zoom, β
replaces the dominant visual element of the current screen — the black
1024×1024 rectangle that is the influence overlay's transparent
backdrop — with **actual terrain rendering**. After β land, the user
sees a tiled world surface with one bootstrap building, layered under
the influence overlay (preserved at z=10 with alpha).

### Scope decisions (Step 0 grep + planning §β intersection)

The full P12Plan-1 / P12Plan-7 design calls for walls + multi-building
rendering, but Step 0 grep revealed:

- **No `collect_building_snapshot` FFI exists** in `sim-bridge`. The
  only write-side affordance is `building_event_queue` which is a
  drained queue, not a persistent list. Rendering the actual building
  set would require new Rust FFI (`pub placed_building_log:
  Vec<…>` in SimResources + a `#[func]` collector).
- Adding Rust FFI escalates this from `--quick` to `--full` and
  expands scope materially.

Phase 12-β therefore narrows to the **highest-visible-delta minimal
slice that preserves `--quick`**:

1. **TileMapLayer floor terrain** (TileSet `.tres` resource +
   GDScript procedural placement). The 64×64 world surface is tiled
   with the 9 floor sprites (3 materials × 3 variants), seeded
   deterministically so the visual is stable across runs.
2. **One bootstrap building Sprite2D** at the hardcoded
   `BOOTSTRAP_X = 32, BOOTSTRAP_Y = 32` position that `world_renderer.gd`
   already exposes — matches the existing influence-stamp source.
   Uses one of the 4 building categories (cairn / campfire /
   gathering_marker / stockpile) → `shelter`-equivalent placeholder
   (cairn chosen for being a single-variant unambiguous symbol).
3. **Influence overlay preserved** at z=10 with reduced alpha so the
   terrain layer is visible underneath.

Walls (P12Plan-1 walls portion), multi-building rendering, and
ConstructionState alpha (P12Plan-7) are deferred to **Phase 12-β.2**
(separate dispatch with `--full` lane once the building snapshot FFI is
designed).

### Preserved invariants

- Phase 4-γ `SPRITE_SCALE = 0.25` (agent tile-fit invariant)
- Phase 11-α + D1 STATE_TINTS 4-color palette (preserved verbatim)
- Phase 12-α camera default zoom 2.0× + mouse-wheel controls
- World grid 64×64, tile 16 px, sprite origin (448, 28) — all
  matching `world_renderer.gd` constants

## Section 2: What to Build

**New files**:
- `assets/tilesets/world_terrain.tres` — Godot TileSet resource with
  one TileSetAtlasSource per floor material (3 atlas sources × 3
  variants = 9 tiles total). Tile size 16×16.
- `scripts/test/p12_beta_terrain_and_buildings/harness_terrain.gd` —
  windowed Godot SceneTree script that boots `main.tscn`, lets ~30
  frames elapse, captures a screenshot showing the rendered terrain
  and bootstrap building, and writes the standard pipeline artefacts
  (`interactive_results.txt` etc.). C-1 lesson applied: harness exit
  code is non-zero when hard assertions fail.
- `rust/crates/sim-test/tests/harness_p12_beta_terrain.rs` — static
  file-inspection assertions (≥12 per phase12.md spec).

**Modified files**:
- `scripts/ui/world_renderer.gd` — extend `_ready()` to:
  1. Add a `TileMapLayer` child at z_index = 0, populate it from the
     new TileSet with procedural per-tile material assignment.
  2. Add a `Sprite2D` child at z_index = 5 textured with
     `assets/sprites/buildings/cairn/1.png`, positioned at the
     existing `BOOTSTRAP_X/Y` world tile.
  3. Move the existing influence overlay `Sprite2D` to `z_index = 10`
     and set `modulate = Color(1, 1, 1, 0.65)` so terrain shows
     through. (Existing logic — channel cycle, click handler — stays
     intact; only the parent's z-index + alpha changes.)
- `scenes/main.tscn` — no changes needed; the new nodes are created
  by `world_renderer.gd:_ready()` as children of the existing
  WorldRenderer Node2D.

**Not changed**:
- Any Rust crate code (sim-core, sim-bridge, sim-engine, sim-systems)
- `scripts/ui/agent_renderer.gd` (D1 STATE_TINTS palette preserved)
- `scripts/ui/camera_controller.gd` (Phase 12-α preserved)
- `shaders/palette_swap.gdshader`
- `harness_p11_alpha_agent_renderer.rs` (A1-A22 remain green)
- `harness_p12_alpha_camera_zoom.rs` (A1-A16 remain green)
- `scripts/ui/panels/causal_panel.gd`
- Any locale file (no new keys)
- Any agent-related sprite asset

## Section 3: How to Implement

### `assets/tilesets/world_terrain.tres`

Godot 4.6 `.tres` text format. One `TileSet` resource with 3
`TileSetAtlasSource` entries (one per floor material). Each atlas
source references the `[1-3].png` variants of its material as
separate `TileSetAtlasSource.create_tile` calls.

Atlas source IDs:
- 0 → `assets/sprites/floors/packed_earth/` (1.png, 2.png, 3.png)
- 1 → `assets/sprites/floors/stone_slab/` (1.png, 2.png, 3.png)
- 2 → `assets/sprites/floors/wood_plank/` (1.png, 2.png, 3.png)

Tile size 16×16. Each variant becomes a single 1×1 tile entry at
atlas coords (0, 0) of its file. The TileSet resource can be authored
either as Godot 4 `.tres` text or instantiated programmatically; the
`.tres` text path is cleaner for VCS diff review.

Minimal `.tres` skeleton (Godot 4.6 syntax):

```
[gd_resource type="TileSet" load_steps=10 format=3]

[ext_resource type="Texture2D" path="res://assets/sprites/floors/packed_earth/1.png" id="1_pe1"]
[ext_resource type="Texture2D" path="res://assets/sprites/floors/packed_earth/2.png" id="2_pe2"]
[ext_resource type="Texture2D" path="res://assets/sprites/floors/packed_earth/3.png" id="3_pe3"]
[ext_resource type="Texture2D" path="res://assets/sprites/floors/stone_slab/1.png" id="4_ss1"]
[ext_resource type="Texture2D" path="res://assets/sprites/floors/stone_slab/2.png" id="5_ss2"]
[ext_resource type="Texture2D" path="res://assets/sprites/floors/stone_slab/3.png" id="6_ss3"]
[ext_resource type="Texture2D" path="res://assets/sprites/floors/wood_plank/1.png" id="7_wp1"]
[ext_resource type="Texture2D" path="res://assets/sprites/floors/wood_plank/2.png" id="8_wp2"]
[ext_resource type="Texture2D" path="res://assets/sprites/floors/wood_plank/3.png" id="9_wp3"]

[sub_resource type="TileSetAtlasSource" id="atlas_pe1"]
texture = ExtResource("1_pe1")
0:0/0 = 0

[sub_resource type="TileSetAtlasSource" id="atlas_pe2"]
texture = ExtResource("2_pe2")
0:0/0 = 0

… (repeat for all 9 variants — each its own atlas source with 1 tile)

[resource]
tile_size = Vector2i(16, 16)
sources/0 = SubResource("atlas_pe1")
sources/1 = SubResource("atlas_pe2")
…
sources/8 = SubResource("atlas_wp3")
```

Each variant being its own atlas source (rather than 9 cells of one
atlas) is the simplest correct shape given the source PNGs are 1×1
tiles each, not packed atlases. Source IDs 0–8 map to materials and
variants below.

### `scripts/ui/world_renderer.gd` extension

Top-of-file constants (existing) plus:

```gdscript
const TERRAIN_TILESET_PATH := "res://assets/tilesets/world_terrain.tres"
const BUILDING_SPRITE_PATH := "res://assets/sprites/buildings/cairn/1.png"
const TERRAIN_SEED := 19349663  # deterministic seed for reproducible terrain
const OVERLAY_ALPHA := 0.65
const Z_TERRAIN := 0
const Z_BUILDING := 5
const Z_OVERLAY := 10
```

In `_ready()`, AFTER the existing influence-overlay setup, ADD:

```gdscript
# V7 Phase 12-β — TileMapLayer terrain.
var terrain_set: TileSet = load(TERRAIN_TILESET_PATH) as TileSet
if terrain_set != null:
    var terrain_layer := TileMapLayer.new()
    terrain_layer.tile_set = terrain_set
    terrain_layer.z_index = Z_TERRAIN
    terrain_layer.position = Vector2(SPRITE_ORIGIN_X, SPRITE_ORIGIN_Y)
    add_child(terrain_layer)
    var rng := RandomNumberGenerator.new()
    rng.seed = TERRAIN_SEED
    for tx in GRID_W:
        for ty in GRID_H:
            var source_id: int = rng.randi_range(0, 8)
            terrain_layer.set_cell(Vector2i(tx, ty), source_id, Vector2i(0, 0), 0)

# V7 Phase 12-β — bootstrap building sprite at the hardcoded
# BOOTSTRAP_X/Y position. One building only; multi-building rendering
# requires a new SimBridge FFI and is deferred to Phase 12-β.2.
var building_tex: Texture2D = load(BUILDING_SPRITE_PATH) as Texture2D
if building_tex != null:
    var building_sprite := Sprite2D.new()
    building_sprite.texture = building_tex
    # Place at the centre of the bootstrap building tile, matching the
    # influence stamp origin so terrain + building + overlay align.
    building_sprite.position = Vector2(
        SPRITE_ORIGIN_X + BOOTSTRAP_X * TILE_SIZE + TILE_SIZE / 2.0,
        SPRITE_ORIGIN_Y + BOOTSTRAP_Y * TILE_SIZE + TILE_SIZE / 2.0,
    )
    building_sprite.z_index = Z_BUILDING
    add_child(building_sprite)

# V7 Phase 12-β — overlay layer above terrain + building, semi-transparent
# so terrain shows through. Existing `sprite` is the influence overlay.
sprite.z_index = Z_OVERLAY
sprite.modulate = Color(1.0, 1.0, 1.0, OVERLAY_ALPHA)
```

No change to `_process(_delta: float)` or `_unhandled_input()` —
existing influence overlay update and click handling continue
unmodified.

### `scripts/test/p12_beta_terrain_and_buildings/harness_terrain.gd`

Modelled on Phase 4-γ rendering harness. Boots `main.tscn`, waits 30
frames, captures a viewport screenshot, writes
`interactive_results.txt` + `assertion_log.txt` + `console_log.txt`.
Hard assertions (failing any causes non-zero exit per C-1 lesson):

- WorldRenderer node present
- WorldRenderer has a `TileMapLayer` child
- The TileMapLayer's `tile_set` is non-null
- WorldRenderer has at least one `Sprite2D` child whose texture is
  loaded from the building sprite path (i.e. the bootstrap building
  sprite, not the influence overlay)
- The influence overlay `Sprite2D`'s `z_index == 10` and
  `modulate.a < 1.0`
- Screenshot saved successfully (non-fallback, dimensions > 1×1)

Soft assertion: at least one terrain tile occupies the bootstrap
building footprint (sanity check for the procedural placement loop).

### `rust/crates/sim-test/tests/harness_p12_beta_terrain.rs`

Static file-inspection assertions (Phase 11-α + 12-α precedent — strip
GDScript `#` comments before grep):

1. `harness_p12_beta_a1_world_terrain_tileset_exists` — confirms
   `assets/tilesets/world_terrain.tres` is a non-empty file.
2. `harness_p12_beta_a2_tileset_tile_size_16` — the `.tres` text
   contains `tile_size = Vector2i(16, 16)`.
3. `harness_p12_beta_a3_tileset_has_9_sources` — at least 9
   `[sub_resource type="TileSetAtlasSource"…]` declarations in
   the `.tres` text (one per floor variant).
4. `harness_p12_beta_a4_tileset_references_all_3_materials` — the
   `.tres` text contains paths for `packed_earth/`, `stone_slab/`,
   and `wood_plank/`.
5. `harness_p12_beta_a5_world_renderer_loads_tileset` — stripped
   `world_renderer.gd` source contains
   `load(TERRAIN_TILESET_PATH)`.
6. `harness_p12_beta_a6_world_renderer_creates_tilemaplayer` —
   stripped source contains `TileMapLayer.new()`.
7. `harness_p12_beta_a7_world_renderer_loop_sets_cells` — stripped
   source contains `terrain_layer.set_cell(`.
8. `harness_p12_beta_a8_world_renderer_loads_building_sprite` —
   stripped source contains `load(BUILDING_SPRITE_PATH)`.
9. `harness_p12_beta_a9_world_renderer_overlay_alpha` — stripped
   source contains `OVERLAY_ALPHA` constant declaration and a
   `sprite.modulate = Color(…)` assignment referring to it.
10. `harness_p12_beta_a10_z_order_constants` — stripped source
    declares `Z_TERRAIN = 0`, `Z_BUILDING = 5`, `Z_OVERLAY = 10`.
11. `harness_p12_beta_a11_runtime_harness_file_present` — confirms
    `scripts/test/p12_beta_terrain_and_buildings/harness_terrain.gd`
    is a non-empty file with `extends SceneTree`.
12. `harness_p12_beta_a12_phase4_gamma_sprite_scale_invariant` —
    reads `scripts/ui/agent_renderer.gd`, strips comments, confirms
    `SPRITE_SCALE := 0.25` (or `SPRITE_SCALE: float = 0.25`) remains.
13. `harness_p12_beta_a13_d1_state_tints_palette_preserved` — reads
    agent_renderer.gd, strips comments, confirms all four D1
    STATE_TINTS literal Colors remain.
14. `harness_p12_beta_a14_phase12_alpha_zoom_invariant` — reads
    `scenes/main.tscn`, confirms Camera2D `zoom = Vector2(2, 2)`
    and the `camera_controller.gd` `script = ExtResource(...)`
    attachment is still present.

## Section 4: Locale

No new localization keys. Phase 12-β is renderer-only.

## Section 5: Verification

```bash
# New Phase 12-β harness
cd rust && cargo test -p sim-test --test harness_p12_beta_terrain -- --nocapture

# Existing Phase 11-α + D1 + 12-α invariants
cd rust && cargo test -p sim-test --test harness_p11_alpha_agent_renderer -- --nocapture
cd rust && cargo test -p sim-test --test harness_p12_alpha_camera_zoom -- --nocapture

# Workspace regression
cd rust && cargo test --workspace

# Clippy
cd rust && cargo clippy --workspace --all-targets -- -D warnings

# Godot GDScript parse check
/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot \
    --headless --check-only --script scripts/ui/world_renderer.gd
```

Expected:
- harness_p12_beta_terrain: ≥14 PASS.
- harness_p11_alpha + 12_alpha: green baselines preserved.
- workspace: PASS.
- clippy: clean.
- GDScript parse: no errors.

## Section 6: Lane

`--quick` — Godot resource + GDScript renderer extension + new test
files only. No Rust crate change, no shader change, no Camera2D
change, no agent rendering change.

Pipeline stages: Visual Verify (generic `harness_visual_verify.gd`
windowed run captures a screenshot of the now-terrained world) +
Evaluator.

## Section 7: 인게임 확인사항 (VLM + Human Visual Verification)

**Expected pipeline visual evidence**:
- VLM whole-scene capture shows a **tiled floor surface** filling the
  64×64 world area (vs the pre-β solid black backdrop).
- A single building sprite (32×32 cairn) is visible at the centre
  region (tile 32, 32), overlapping the influence stamp.
- Influence overlay (Warmth disc default) renders **above** the
  terrain at reduced alpha — both terrain texture and overlay are
  visible simultaneously.
- The 2× default zoom (Phase 12-α) places each 16-px tile at 32 px
  on screen, well above sub-resolution.

**VLM signal for APPROVE / VISUAL_PASS**:
- Generic checklist tokens for active channel (Warmth) remain
  identifiable (disc rendered, central building stamp visible).
- Non-black backdrop confirms terrain layer rendering.

**VLM signal for WARNING** (acceptable, do not block):
- Tile material variety subtle (3 materials × 3 variants, all
  earth-tone palette by design); the human eye may see "uniform
  brown floor" rather than three distinct materials. Acceptable —
  the visible-delta-vs-black is what matters for β.

**VLM signal for FAIL** (block and fix):
- Black backdrop still dominates (TileMapLayer not added or invisible
  due to z-order misconfiguration).
- Influence overlay missing or fully opaque (alpha not applied).
- Scene fails to instantiate (TileSet load error, script parse error).

**Honest disclosure for the human reviewer**:
- β delivers the floor + bootstrap building visible delta. It does
  NOT yet render multiple buildings, walls, or ConstructionState
  alpha — those require a `collect_building_snapshot` Rust FFI
  extension deferred to Phase 12-β.2.
- The β.2 scope split is documented here so the pipeline's pass on
  this dispatch does not imply "Phase 12-β complete in
  phase12.md terms" — it implies "Phase 12-β.1 complete; β.2 still
  open".
- User-driven windowed Godot run remains the source of truth for
  perceptual quality.
