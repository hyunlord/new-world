# Phase 13-β — Resource Node Placeholder Layer

Feature: p13-beta-resource-placeholders
Lane: --quick (GDScript-only addition + new harness)
Parent: Section 14+ (`f2efcd9b`) + `.harness/plans/phase13.md`
(local, P13Plan-4) + Phase 13-α (`0714c891`).

## Section 1: Implementation Intent

Phase 13-β is the **second substage** of the Phase 13 Game-like UI
Sprint. After α made sprites legible (zoom 3.0× → 48-54 px) and
distinguished the three building types, the next visible gap is
that the world looks empty between agents and buildings — there is
nothing for the player to interpret as "resources to gather".

β adds a **GDScript-side decorative layer** of resource
placeholder sprites: ~20 storage_pit sprites scattered procedurally
across the 64×64 world (seeded from the world seed for
reproducibility). They render at z=3, between the TileMapLayer
floor (z=0) and the ConstructionSite layer (z=5).

**Honest disclosure (substrate gap)**:
- No `ResourceNode` ECS component exists in sim-core.
- These placeholders are **purely visual**. Agents do NOT interact
  with them. Backend gathering substrate is Section 15+.
- The goal is "the world looks like a populated simulation map",
  not "agents harvest resources". Phase 13's contract is
  perceptual legibility, not richer simulation.

Substrate verified (Step 0 grep, live files):
- `assets/sprites/furniture/storage_pit/1.png` exists (32×32 RGBA),
  visually distinct from the building layer (no roof-like top edge)
- `scripts/ui/world_renderer.gd` z-order constants:
  `Z_TERRAIN = 0`, `Z_CONSTRUCTION = 5`, `Z_FURNITURE = 4`,
  `Z_OVERLAY = 10`. Free slot: z=3 (between terrain and furniture).
- World grid 64×64, `TILE_SIZE = 16`, `SPRITE_ORIGIN = (448, 28)`.
- Existing β.1 procedural seeding uses
  `RandomNumberGenerator.seed = TERRAIN_SEED` — same pattern reused.

Preserved invariants (all six test-verified):
- Phase 4-γ SPRITE_SCALE = 0.25
- Phase 11-α + D1 STATE_TINTS 4-color
- Phase 12-α Camera zoom (ZOOM_MIN/MAX/DEFAULT after α)
- Phase 12-β.1 TileMapLayer terrain + overlay alpha 0.65
- Phase 12-β.2 A3 ConstructionSite render z=5
- Phase 12-γ Settlement hearth z=4
- Phase 13-α campfire bootstrap + zoom 3.0×

## Section 2: What to Build

**Modified file**:
- `scripts/ui/world_renderer.gd` — add resource-placeholder
  initialisation to `_ready()` after the existing TileMapLayer +
  bootstrap building setup. Adds ~30 lines and three new constants
  (`RESOURCE_SPRITE_PATH`, `Z_RESOURCE`, `RESOURCE_COUNT`,
  `RESOURCE_SEED`).

**New file**:
- `rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs`
  — static file-inspection assertions following the Phase 13-α
  precedent. ≥12 assertions.

**Not changed**:
- Rust crate code (sim-core, sim-bridge, sim-engine, sim-systems)
- `agent_renderer.gd`, `camera_controller.gd`,
  `palette_swap.gdshader`, `scenes/main.tscn`,
  `assets/tilesets/world_terrain.tres`
- All existing harness files — remain green
- Phase 13-α `harness_p13_alpha_camera_and_buildings.rs` — its
  building-distinction A13 still passes (3 building sprite paths
  unchanged; resource path is a separate 4th)
- Any locale or asset

## Section 3: How to Implement

### `scripts/ui/world_renderer.gd`

Add to the file-level constants section (after the existing γ
furniture block):

```gdscript
# V7 Phase 13-β — resource-node placeholder layer.
#
# Decorative-only resource sprites placed deterministically from
# RESOURCE_SEED so the world reads as a populated simulation map.
# No sim-core ResourceNode component exists; agents do NOT interact
# with these sprites. Substrate-driven gathering is Section 15+.
# Sprite chosen: furniture/storage_pit/1.png (32×32) — visually
# distinct from buildings (no roof, looks like a hole/pile).
const RESOURCE_SPRITE_PATH := "res://assets/sprites/furniture/storage_pit/1.png"
const Z_RESOURCE := 3
const RESOURCE_COUNT := 20
const RESOURCE_SEED := 88675123
```

In `_ready()`, AFTER the existing TileMapLayer + bootstrap building
setup, ADD the resource placement block:

```gdscript
# V7 Phase 13-β — scatter resource placeholder sprites.
var resource_tex: Texture2D = load(RESOURCE_SPRITE_PATH) as Texture2D
if resource_tex != null:
    var rng_res := RandomNumberGenerator.new()
    rng_res.seed = RESOURCE_SEED
    for _i in RESOURCE_COUNT:
        var rtx: int = rng_res.randi_range(0, GRID_W - 1)
        var rty: int = rng_res.randi_range(0, GRID_H - 1)
        var res_sprite := Sprite2D.new()
        res_sprite.texture = resource_tex
        res_sprite.position = Vector2(
            float(SPRITE_ORIGIN_X + rtx * TILE_SIZE) + float(TILE_SIZE) / 2.0,
            float(SPRITE_ORIGIN_Y + rty * TILE_SIZE) + float(TILE_SIZE) / 2.0,
        )
        res_sprite.z_index = Z_RESOURCE
        add_child(res_sprite)
else:
    push_warning("WorldRenderer: failed to load resource sprite at %s" % RESOURCE_SPRITE_PATH)
```

The `for _i in RESOURCE_COUNT:` loop variable is intentionally
underscore-prefixed (Phase 13's strict-check infrastructure
catches UNUSED_PARAMETER per D Phase B `a437c547`).

The float-cast pattern for tile-centring mirrors β.2 / γ
exactly (the D Phase A fix that landed at `0238aef2`).

### Rust harness

Static file-inspection assertions (Phase 11-α / 12-α / 12-β / γ /
13-α precedent). Strip GDScript `#` comments before grep.

12+ assertions:

1. `a1_resource_sprite_path_constant_present` — stripped
   `world_renderer.gd` contains the exact path string
   `"res://assets/sprites/furniture/storage_pit/1.png"`.
2. `a2_z_resource_constant_eq_3` — stripped source contains
   `Z_RESOURCE := 3` and `Z_RESOURCE` is referenced in the
   resource block.
3. `a3_resource_count_eq_20` — stripped source contains
   `RESOURCE_COUNT := 20`.
4. `a4_resource_seed_constant_present` — stripped source contains
   `RESOURCE_SEED := 88675123`.
5. `a5_resource_placement_uses_rng` — stripped source contains
   `RandomNumberGenerator.new()` AND `rng_res.seed = RESOURCE_SEED`
   AND `rng_res.randi_range(0, GRID_W - 1)`.
6. `a6_resource_sprite_file_exists` — physical file
   `assets/sprites/furniture/storage_pit/1.png` exists, non-empty.
7. `a7_resource_z_order_below_furniture_above_terrain` — assert
   `Z_TERRAIN (0) < Z_RESOURCE (3) < Z_FURNITURE (4) <
   Z_CONSTRUCTION (5) < Z_OVERLAY (10)`. Detect by extracting
   each constant's numeric RHS from the source.
8. `a8_phase4_gamma_sprite_scale_invariant_preserved` — agent
   renderer SPRITE_SCALE = 0.25 unchanged.
9. `a9_d1_state_tints_palette_preserved` — agent renderer four
   D1 STATE_TINTS literals all present.
10. `a10_phase12_alpha_zoom_invariant_preserved` — camera_controller
    still declares ZOOM_MIN (0.5, 0.5) and ZOOM_MAX (4.0, 4.0).
11. `a11_phase13_alpha_zoom_default_3x_preserved` — camera_controller
    still declares `ZOOM_DEFAULT := Vector2(3.0, 3.0)`. Phase 13-α
    invariant.
12. `a12_phase13_alpha_bootstrap_campfire_preserved` —
    world_renderer still declares
    `BUILDING_SPRITE_PATH := "res://assets/sprites/buildings/campfire/1.png"`.
    Phase 13-α invariant.
13. `a13_phase12_beta1_terrain_tileset_preserved` —
    `TERRAIN_TILESET_PATH` still present + `OVERLAY_ALPHA = 0.65`.
14. `a14_phase12_beta2_construction_z_5_preserved` —
    `Z_CONSTRUCTION = 5` still present.
15. `a15_phase12_gamma_furniture_z_4_preserved` —
    `Z_FURNITURE = 4` still present.

## Section 4: Locale

No new keys.

## Section 5: Verification

```bash
cd rust && cargo test -p sim-test --test harness_p13_beta_resource_placeholders -- --nocapture
cd rust && cargo test --workspace
cd rust && cargo clippy --workspace --all-targets -- -D warnings
/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot \
    --path . --headless --check-only --script scripts/ui/world_renderer.gd
```

Expected:
- harness_p13_beta: ≥12 PASS
- All prior phase harnesses: green baselines preserved
- Workspace + clippy: clean
- GDScript parse: no errors (and no warnings via D Phase B Step 2.4)

Pipeline Step 2.4 (`a437c547`) will auto-run the GDScript strict
check — it must pass with zero parse errors / warnings / FFI
mismatches.

## Section 6: Lane

`--quick` — one GDScript file edit + one new Rust test file. Zero
Rust crate change, zero shader, zero asset, zero scene change.

## Section 7: 인게임 확인사항

**Expected visual change in pipeline VLM capture**:
- 20 storage_pit sprites scattered across the 64×64 world.
- Each rendered at 32×32 px native, scaled by camera 3.0× = 96×96
  px on viewport (well above perceptual threshold).
- Mixed amongst terrain + existing buildings + agents — should
  read as "this is a populated map with stuff in it" rather than
  "an empty tiled floor with one campfire".

**VLM signal for APPROVE / VISUAL_PASS**:
- Generic checklist tokens still pass.
- New non-building, non-agent sprites visible across the world.

**VLM signal for WARNING** (acceptable):
- 20 placeholders may overlap with bootstrap building at tile
  (32, 32); seeded RNG could land one on top. Acceptable —
  z-order handles it (resource z=3 < building z=5).

**VLM signal for FAIL**:
- Generic visual regression.
- Resource sprites in front of buildings (z-order broken).
- Scene crash on `_ready()`.

**Honest disclosure (carry-forward)**:
- These resources are decorative. They don't appear in any
  SimBridge snapshot, don't participate in agent decisions, don't
  decrease when "harvested".
- Phase 13's contract is "looks like a simulation". Substrate-driven
  gathering is Section 15+ scope.
- User confirmation still comes at full Phase 13 closure (post-ε),
  not mid-sprint.
