# Phase 13-α — Camera Zoom 3.0× + Building Sprite Distinction

Feature: p13-alpha-camera-and-buildings
Lane: --quick (GDScript-only edits + new harness file; no Rust, no
shader, no asset, no scene structure change)
Parent: Section 14+ (`f2efcd9b`) + `.harness/plans/phase13.md`
(local) + user confirmation 2026-05-25 of P13Plan-1 Path B.

## Section 1: Implementation Intent

Phase 13-α is the **first substage** of the Phase 13 Game-like UI
Sprint. Targets the user-screen evidence (1779685795433_image.png,
"초딩도 이해 못 함"):

- At Phase 12-α's `Camera2D.zoom = (2,2)` agents render at
  `64×72 × 0.25 × 2 = 32×36 px` — barely above the perceptual
  threshold; building sprites all use the cairn texture and look
  identical regardless of role.
- α changes two GDScript constants: `ZOOM_DEFAULT` 2.0 → 3.0 (Path
  B per P13Plan-1 — preserves Phase 4-γ SPRITE_SCALE = 0.25
  invariant), and `BUILDING_SPRITE_PATH` from cairn to campfire so
  the three building layers (bootstrap / ConstructionSite /
  Settlement) become visually distinct.

Substrate verified (Step 0 grep, live files, 2026-05-25):
- `scripts/ui/camera_controller.gd:20-22`: `ZOOM_MIN := Vector2(0.5,
  0.5)`, `ZOOM_MAX := Vector2(4.0, 4.0)`,
  `ZOOM_DEFAULT := Vector2(2.0, 2.0)`. Range stays intact —
  user can still mouse-wheel-zoom-out via existing controls.
- `scripts/ui/world_renderer.gd:46`: `BUILDING_SPRITE_PATH :=
  "res://assets/sprites/buildings/cairn/1.png"` (bootstrap)
- `scripts/ui/world_renderer.gd:59`: `CONSTRUCTION_SPRITE_PATH :=
  "res://assets/sprites/buildings/cairn/1.png"` (Phase 12-β.2 A3)
- `scripts/ui/world_renderer.gd:69`: `FURNITURE_SPRITE_PATH :=
  "res://assets/sprites/furniture/hearth/1.png"` (Phase 12-γ)
- `assets/sprites/buildings/campfire/1.png` exists (32×32 RGBA,
  same dimensions as cairn).
- `cairn/1.png`, `campfire/1.png`, `hearth/1.png` all 32×32 — three
  distinct sprites for three distinct semantic layers.

Visual outcome after α:
- Agent sprites become 48-54 px (above sub-resolution threshold)
- Bootstrap building (the hardcoded centre stamp source): campfire
- Construction sites (in-progress): cairn (placeholder)
- Settlement centroids: hearth (existing)
- All three are recognisably different shapes

P13Plan-1 Path B preserves all six anti-regression invariants
(Phase 4-γ, Phase 11-α, D1, Phase 12-α, β.1, β.2, γ).

## Section 2: What to Build

**Modified files** (two single-line constant edits):
- `scripts/ui/camera_controller.gd` — line 22, change
  `ZOOM_DEFAULT := Vector2(2.0, 2.0)` →
  `ZOOM_DEFAULT := Vector2(3.0, 3.0)`. No other change. `ZOOM_MIN`
  and `ZOOM_MAX` stay (0.5, 4.0).
- `scripts/ui/world_renderer.gd` — line 46, change
  `BUILDING_SPRITE_PATH := "res://assets/sprites/buildings/cairn/1.png"` →
  `BUILDING_SPRITE_PATH := "res://assets/sprites/buildings/campfire/1.png"`.
  `CONSTRUCTION_SPRITE_PATH` (line 59) and `FURNITURE_SPRITE_PATH`
  (line 69) stay unchanged.

**New file**:
- `rust/crates/sim-test/tests/harness_p13_alpha_camera_and_buildings.rs`
  — static file-inspection harness following the precedent of
  `harness_p12_alpha_camera_zoom.rs` and
  `harness_p12_gamma_settlement_furniture.rs`. ≥10 assertions
  enumerated in §3.

**Not changed**:
- Any Rust crate code (sim-core, sim-bridge, sim-engine, sim-systems)
- `scripts/ui/agent_renderer.gd` (Phase 11-α + D1 invariant preserved)
- `shaders/palette_swap.gdshader`
- `scenes/main.tscn`
- `assets/tilesets/world_terrain.tres` (Phase 12-β.1 preserved)
- All existing harness files (`harness_p11_alpha_agent_renderer.rs`,
  `harness_p12_alpha_camera_zoom.rs`, `harness_p12_beta_terrain.rs`,
  `harness_p12_beta2_construction_sites.rs`,
  `harness_p12_gamma_settlement_furniture.rs`,
  `harness_d_phase_a_runtime_warnings.rs`) — all remain green
- Any locale, asset, or sprite file

## Section 3: How to Implement

### Edit 1: `scripts/ui/camera_controller.gd`

Find the line:
```gdscript
const ZOOM_DEFAULT: Vector2 = Vector2(2.0, 2.0)
```

Change to:
```gdscript
const ZOOM_DEFAULT: Vector2 = Vector2(3.0, 3.0)
```

Update the comment block above the constants to document the new
default. Add a short comment after the constant explaining the
Phase 13-α rationale:

```gdscript
# V7 Phase 13-α — default zoom raised from 2.0× to 3.0× so 16×18 px
# agent sprites (Phase 4-γ SPRITE_SCALE = 0.25 invariant preserved)
# render at 48-54 px, above the human-perceptual threshold for
# distinguishing agents and buildings. Mouse-wheel controls (Phase
# 12-α) still let the user zoom out to overview.
const ZOOM_DEFAULT: Vector2 = Vector2(3.0, 3.0)
```

### Edit 2: `scripts/ui/world_renderer.gd`

Find the line:
```gdscript
const BUILDING_SPRITE_PATH := "res://assets/sprites/buildings/cairn/1.png"
```

Change to:
```gdscript
const BUILDING_SPRITE_PATH := "res://assets/sprites/buildings/campfire/1.png"
```

Update the comment block immediately above to document the Phase
13-α distinction:

```gdscript
# V7 Phase 13-α — bootstrap building uses campfire sprite to
# visually distinguish it from ConstructionSite (cairn) and
# Settlement centroid (hearth). Three distinct 32×32 sprites for
# three distinct semantic layers.
const BUILDING_SPRITE_PATH := "res://assets/sprites/buildings/campfire/1.png"
```

`CONSTRUCTION_SPRITE_PATH` and `FURNITURE_SPRITE_PATH` are untouched.

### `rust/crates/sim-test/tests/harness_p13_alpha_camera_and_buildings.rs` (new)

Use the existing `project_root()` helper pattern. Strip GDScript
`#` comments before grep so comment text cannot satisfy assertions.

≥12 assertions:

1. `a1_camera_controller_zoom_default_3x` — `camera_controller.gd`
   stripped source contains `ZOOM_DEFAULT: Vector2 = Vector2(3.0, 3.0)`
   (whitespace-tolerant per the Phase 12-α A3-A5 precedent).
2. `a2_camera_controller_zoom_min_unchanged` — `ZOOM_MIN: Vector2 =
   Vector2(0.5, 0.5)` still present. Phase 12-α invariant.
3. `a3_camera_controller_zoom_max_unchanged` — `ZOOM_MAX: Vector2 =
   Vector2(4.0, 4.0)` still present. Phase 12-α invariant.
4. `a4_bootstrap_building_path_campfire` — `world_renderer.gd`
   stripped source contains
   `BUILDING_SPRITE_PATH := "res://assets/sprites/buildings/campfire/1.png"`.
5. `a5_construction_sprite_path_unchanged` — `CONSTRUCTION_SPRITE_PATH
   := "res://assets/sprites/buildings/cairn/1.png"` still present.
   Phase 12-β.2 A3 invariant.
6. `a6_furniture_sprite_path_unchanged` — `FURNITURE_SPRITE_PATH :=
   "res://assets/sprites/furniture/hearth/1.png"` still present.
   Phase 12-γ invariant.
7. `a7_campfire_sprite_file_exists` — physical file
   `assets/sprites/buildings/campfire/1.png` exists and is non-empty.
8. `a8_phase4_gamma_sprite_scale_invariant_preserved` —
   `agent_renderer.gd` stripped source contains `SPRITE_SCALE := 0.25`
   (or `SPRITE_SCALE: float = 0.25`). Phase 4-γ invariant.
9. `a9_d1_state_tints_palette_preserved` — `agent_renderer.gd`
   stripped source contains all four D1 STATE_TINTS literal Colors.
10. `a10_phase12_beta1_terrain_tileset_preserved` —
    `world_renderer.gd` stripped source still declares
    `TERRAIN_TILESET_PATH` and `OVERLAY_ALPHA = 0.65`. Phase 12-β.1
    invariant.
11. `a11_phase12_beta2_construction_z_preserved` —
    `world_renderer.gd` stripped source declares
    `Z_CONSTRUCTION = 5`. Phase 12-β.2 A3 invariant.
12. `a12_phase12_gamma_furniture_z_preserved` —
    `world_renderer.gd` stripped source declares `Z_FURNITURE = 4`.
    Phase 12-γ invariant.
13. `a13_three_distinct_building_sprite_paths` — read
    `world_renderer.gd`, parse the three sprite path constants,
    confirm they reference three different asset paths (no two
    equal). Anti-regression guard preventing future merges from
    collapsing them back to identical placeholders.
14. `a14_main_tscn_camera2d_script_attached` — `scenes/main.tscn`
    contains both `[node name="Camera2D"` and
    `script = ExtResource("4_camera")`. Phase 12-α scene structure
    preserved.

## Section 4: Locale

No new keys.

## Section 5: Verification

```bash
# New Phase 13-α harness
cd rust && cargo test -p sim-test --test harness_p13_alpha_camera_and_buildings -- --nocapture

# All prior invariants
cd rust && cargo test -p sim-test --test harness_p11_alpha_agent_renderer
cd rust && cargo test -p sim-test --test harness_p12_alpha_camera_zoom
cd rust && cargo test -p sim-test --test harness_p12_beta_terrain
cd rust && cargo test -p sim-test --test harness_p12_beta2_construction_sites
cd rust && cargo test -p sim-test --test harness_p12_gamma_settlement_furniture
cd rust && cargo test -p sim-test --test harness_d_phase_a_runtime_warnings

# Workspace + clippy
cd rust && cargo test --workspace
cd rust && cargo clippy --workspace --all-targets -- -D warnings

# GDScript parse (the new pipeline Step 2.4 also runs this)
/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot \
    --path . --headless --check-only --script scripts/ui/camera_controller.gd
/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot \
    --path . --headless --check-only --script scripts/ui/world_renderer.gd
```

Expected:
- harness_p13_alpha: ≥12 PASS
- All prior harnesses: green baselines
- Workspace: PASS, clippy clean, GDScript parse clean

The new pipeline Step 2.4 (`a437c547`) runs automatically:
- `gdscript_strict_check.sh` flags parse errors, INTEGER_DIVISION /
  UNUSED_PARAMETER warnings, FFI binding mismatches at build time

## Section 6: Lane

`--quick` — two GDScript constant edits + one new Rust test file.
Zero Rust crate change, zero shader, zero asset, zero scene
structure change.

## Section 7: 인게임 확인사항

**Expected visual change in pipeline VLM capture**:
- Default zoom 3.0× (was 2.0×): agents render visibly larger
  (48-54 px vs 32-36 px). VLM whole-scene capture should finally
  cross perceptual threshold for sprite-level changes.
- Bootstrap building at world centre is now a campfire (was cairn).
- ConstructionSites (when agents start building) still cairn.
- Settlement centroid (when ≥3 proximate agents form a settlement)
  still hearth.

**VLM signal for APPROVE / VISUAL_PASS**:
- Generic visual checklist tokens (Warmth disc, influence stamps,
  tile floor, agent rendering) all pass.
- Bootstrap building visibly distinct from current cairn.

**VLM signal for WARNING** (acceptable):
- VLM may not articulate the sprite-distinction change in its
  text output; whole-scene grading is coarse. The grader can
  confirm "the scene rendered correctly without crashes" but is
  not the source of truth for legibility — that comes at Phase
  13-ε with the full bootstrap seed and a windowed Godot run by
  the user.

**VLM signal for FAIL**:
- Generic visual regression.
- Camera zoom controls broken (mouse-wheel no longer works).
- Building sprites missing (file path typo).

**Honest disclosure**:
- α is the first of 5 substages. It establishes the visible-size
  baseline that β/γ/δ/ε build on. User confirmation comes only
  at full Phase 13 closure per user mandate.
- Camera zoom 3.0× makes the visible window
  `1920/3 × 1080/3 = 640×360 px = 40×22 tiles`. The 64×64 world
  partially extends beyond view. Acceptable: mouse-wheel zoom-out
  available; default is "you can see sprites".
- Phase 4-γ SPRITE_SCALE = 0.25 invariant preserved end-to-end —
  no harness rewrite required.
