# Phase 13-ε — Bootstrap Seed: 3 Buildings Around Centre

Feature: p13-epsilon-bootstrap-seed
Lane: --quick (GDScript-only — Rust sim-bridge bootstrap untouched
to preserve --quick lane and avoid changing sim-core invariants
mid-sprint)
Parent: Section 14+ (`f2efcd9b`) + `.harness/plans/phase13.md`
(local, P13Plan-7) + Phase 13-α (`0714c891`) + β (`98751086`) + γ
(`df0041f6`) + δ (`91041f3f`).

## Section 1: Implementation Intent

Phase 13-ε is the **final substage** of the Phase 13 Game-like UI
Sprint. After α/β/γ/δ shipped sprite legibility, building
distinction, resource placeholders, interaction cues, and a basic
HUD, the last visible deficit is **first-frame sparseness**: the
existing Rust bootstrap places ONE building at world centre
(via `on_building_placed`) and 64 agents on an 8×8 stride lattice
across the world. The result reads as "scattered population with
one campfire", not "populated settlement region".

ε's narrow honest scope (P13Plan-7 trimmed for substrate
discipline): keep the Rust bootstrap untouched (so this stays
`--quick` lane, sim-core invariants untouched, agents stay 64
which already exceeds the P13Plan-7 floor of 30) and add **two
extra `on_building_placed` calls** from GDScript so the first
frame shows three buildings clustered around the centre.

Substrate verified (Step 0 grep, this dispatch):
- Rust bootstrap: 64 agents on lattice from (4,4) to (60,60),
  stride 8 (`world_node.rs:57-64`).
- One building call from GDScript at `world_renderer.gd:111` —
  `world_sim.on_building_placed(BOOTSTRAP_X, BOOTSTRAP_Y,
  BOOTSTRAP_RADIUS)` with `BOOTSTRAP_X = BOOTSTRAP_Y = 32`,
  `BOOTSTRAP_RADIUS = 8`.
- Building Sprite2D added at (32, 32) with campfire texture (α).
- `on_building_placed` is an existing Rust `#[func]` (no new FFI).

Adding two more `on_building_placed` calls + their matching
Sprite2D children at (24, 32) and (40, 32) gives a 3-building
horizontal cluster spanning ~16 tiles, each visible and
distinguishable at zoom 3×.

**Honest disclosure**:
- Agent clustering for Settlement formation (P13Plan-7's "agents
  cluster around (32,32) so a Settlement forms in the first few
  ticks") requires Rust sim-bridge change → would reclassify to
  --full lane and break the --quick streak. ε defers that
  refinement; the spread bootstrap still produces Settlement
  formation eventually via the 5-tile proximity threshold + organic
  movement, just not in the first frame.
- Resource placeholders (P13Plan-7's "20 resource placeholders")
  already landed in Phase 13-β. Nothing new on that front.

Preserved invariants (all nine, test-verified):
- Phase 4-γ SPRITE_SCALE = 0.25
- Phase 11-α + D1 STATE_TINTS 4-color
- Phase 12-α camera zoom controls
- Phase 12-β.1 TileMapLayer + overlay
- Phase 12-β.2 A3 ConstructionSite render z=5
- Phase 12-γ Settlement hearth z=4
- Phase 13-α campfire bootstrap + zoom 3.0×
- Phase 13-β resource placeholders z=3
- Phase 13-γ STATE_SCALE_BOOST
- Phase 13-δ HudTopbar mounted under UI

## Section 2: What to Build

**Modified file**:
- `scripts/ui/world_renderer.gd` — add two extra
  `on_building_placed` calls in `_ready()` after the existing
  call, plus two extra Sprite2D children at (24, 32) and
  (40, 32) using the same campfire texture (no new asset).
  ~20 lines added.

**New file**:
- `rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs`
  — static file-inspection assertions following the Phase
  13-α/β/γ/δ precedent. ≥10 assertions.

**Not changed**:
- Any Rust crate code (sim-core, sim-bridge, sim-engine,
  sim-systems) — explicit invariant; Rust bootstrap stays at
  the existing 8×8 lattice
- `agent_renderer.gd`, `camera_controller.gd`, `hud_topbar.gd`,
  `causal_panel.gd`, `palette_swap.gdshader`
- `scenes/main.tscn`
- `assets/tilesets/world_terrain.tres`
- All existing harness files
- Any locale or sprite asset

## Section 3: How to Implement

### `scripts/ui/world_renderer.gd`

Add to the file-level constants (after the existing
`BOOTSTRAP_X/Y/RADIUS`):

```gdscript
# V7 Phase 13-ε — extra bootstrap building positions so the first
# capture frame reads as a clustered settlement region instead of
# a single isolated building. Two additional buildings flanking
# the existing centre stamp at (32, 32). Same radius as the
# centre stamp so influence overlay coverage is symmetric.
const BOOTSTRAP_X_LEFT := 24
const BOOTSTRAP_X_RIGHT := 40
```

In `_ready()`, AFTER the existing
`world_sim.on_building_placed(BOOTSTRAP_X, BOOTSTRAP_Y, BOOTSTRAP_RADIUS)`
call (around line 111), add the two extra placement calls:

```gdscript
# V7 Phase 13-ε — additional bootstrap buildings flanking the centre.
world_sim.on_building_placed(BOOTSTRAP_X_LEFT, BOOTSTRAP_Y, BOOTSTRAP_RADIUS)
world_sim.on_building_placed(BOOTSTRAP_X_RIGHT, BOOTSTRAP_Y, BOOTSTRAP_RADIUS)
```

AFTER the existing bootstrap Sprite2D block (around line 158),
add two more Sprite2D children sharing the loaded campfire texture:

```gdscript
# V7 Phase 13-ε — flanking bootstrap building sprites at the
# left and right positions. Reuse `building_tex` already loaded
# above; if it failed to load, the parent guard handles it.
if building_tex != null:
    for extra_x in [BOOTSTRAP_X_LEFT, BOOTSTRAP_X_RIGHT]:
        var extra_sprite := Sprite2D.new()
        extra_sprite.texture = building_tex
        extra_sprite.position = Vector2(
            float(SPRITE_ORIGIN_X + extra_x * TILE_SIZE) + float(TILE_SIZE) / 2.0,
            float(SPRITE_ORIGIN_Y + BOOTSTRAP_Y * TILE_SIZE) + float(TILE_SIZE) / 2.0,
        )
        extra_sprite.z_index = Z_BUILDING
        add_child(extra_sprite)
```

(`Z_BUILDING` is the existing z-index constant from β.1.)

### Rust harness

10+ assertions:

1. `a1_extra_bootstrap_x_constants_declared` — stripped
   `world_renderer.gd` declares `BOOTSTRAP_X_LEFT := 24` and
   `BOOTSTRAP_X_RIGHT := 40`.
2. `a2_extra_on_building_placed_calls` — stripped source contains
   exactly three `on_building_placed(` calls (the original at
   BOOTSTRAP_X plus two new ones at BOOTSTRAP_X_LEFT and
   BOOTSTRAP_X_RIGHT).
3. `a3_extra_sprite_block_uses_building_tex` — stripped source
   contains a `for extra_x in [BOOTSTRAP_X_LEFT, BOOTSTRAP_X_RIGHT]`
   loop AND `extra_sprite.texture = building_tex` (reuse, no new
   load).
4. `a4_extra_sprite_z_index_is_building` — stripped source
   contains `extra_sprite.z_index = Z_BUILDING` inside the loop.
5. `a5_extra_sprite_y_uses_bootstrap_y` — the loop's position
   y-coordinate reads `BOOTSTRAP_Y` (all three buildings on the
   same horizontal line).
6. `a6_phase4_gamma_sprite_scale_invariant_preserved`
7. `a7_d1_state_tints_palette_preserved`
8. `a8_phase12_alpha_zoom_invariants_preserved`
9. `a9_phase13_alpha_zoom_default_3x_preserved`
10. `a10_phase13_alpha_bootstrap_campfire_preserved`
11. `a11_phase13_beta_resource_constants_preserved`
12. `a12_phase13_gamma_state_scale_boost_preserved`
13. `a13_phase13_delta_hud_topbar_file_present` — file
    `scripts/ui/panels/hud_topbar.gd` exists and is non-empty.
14. `a14_no_rust_sim_core_modification` — `git status --porcelain`
    contains NO `rust/crates/sim-core/` or `rust/crates/sim-bridge/`
    entries in the working tree. Scope-discipline guard.

## Section 4: Locale

No new keys.

## Section 5: Verification

```bash
cd rust && cargo test -p sim-test --test harness_p13_epsilon_bootstrap_seed -- --nocapture
cd rust && cargo test --workspace
cd rust && cargo clippy --workspace --all-targets -- -D warnings
/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot \
    --path . --headless --check-only --script scripts/ui/world_renderer.gd
```

Expected: harness_p13_epsilon ≥10 PASS, all prior phases green,
workspace + clippy clean, GDScript parse clean. Pipeline Step 2.4
(`a437c547`) runs automatically.

## Section 6: Lane

`--quick` — one GDScript file edit + one new Rust test file. Zero
Rust crate change. Zero shader, zero asset, zero new node/scene
change.

## Section 7: 인게임 확인사항

**Expected visual change in pipeline VLM capture**:
- Three campfire sprites visible in a horizontal row at world
  centre — at tiles (24, 32), (32, 32), (40, 32).
- Each at zoom 3.0× × 32×32 = 96×96 px, easily readable.
- The 64 spread agents continue to render normally.
- The 20 resource placeholders from β continue to render.
- The HUD top-bar continues to show counters; "Tick" increments
  every frame.

**VLM signal for APPROVE / VISUAL_PASS**: generic checklist passes;
three buildings visible.

**VLM signal for WARNING** (acceptable): if VLM doesn't enumerate
the three buildings by position, the Rust harness numeric assertions
(A2-A5) remain authoritative.

**VLM signal for FAIL**: generic visual regression; scene crash;
fewer than three buildings visible.

**Honest disclosure**:
- ε does NOT cluster agents — agent positions remain controlled
  by the existing Rust 8×8 lattice. Adding agent clustering would
  reclassify to --full lane.
- ε does NOT add resources — those landed in Phase 13-β.
- The three buildings share the same campfire texture; visual
  variety across building types would require new asset selection
  or Section 15+ substrate work.
- **Phase 13 closure** lands with this commit. User confirmation
  is the next step per the mandate.
