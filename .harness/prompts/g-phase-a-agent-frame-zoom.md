# G Phase A — Agent single-frame fix + zoom-in headroom (color/palette-swap excluded)

Feature: g-phase-a-agent-frame-zoom
Lane: --quick (GDScript form fix + camera constant + harness updates;
zero Rust crate change, zero shader change)
Parent: diagnosis report (2026-05-29) of the "agents look like green
grass clumps" + "mouse-wheel zoom won't zoom in" issues. Phase 14-ζ
(`135180a0`) chain successor.

## Section 1: Implementation Intent

A windowed-Godot review after Phase 14 surfaced two confirmed defects.
A read-only diagnosis pass (pixel analysis + shader/harness reading +
rendered-screenshot inspection) established the root causes. **G Phase A
fixes exactly two of them: agent FORM and ZOOM. Agent COLOR (the green
appearance) is explicitly deferred** because it needs runtime shader
debugging, not a static edit.

### Defect 1 — agents render as a "clump", not a single figure (FORM)

Diagnosis facts (verified 2026-05-29):
- `assets/sprites/agent_base.png` is **64×72** and is a **4-column ×
  3-row = 12-frame sprite sheet** (16×24 per frame). Proof: the
  per-column opaque profile repeats identically every 16 px with empty
  3 px margins between blocks (`[0,0,0,8,10,27,40,45,35,40,40,12,8,0,0,0]`
  ×4), and all 12 cells carry a figure.
- `scripts/ui/agent_renderer.gd` builds a `QuadMesh` of size
  `Vector2(SPRITE_W, SPRITE_H) = (64, 72)` whose UVs span the **whole
  texture (0→1)**, and assigns it to the `MultiMesh`. There is no frame
  / UV-region selection. So **every agent draws all 12 frames squished
  into one 16×18 px quad** → reads as a textured clump, not one person.
- This has been latent since Phase 4-γ. The pipeline never caught it:
  static harness checks inspect constants, not rendered geometry, and
  the VLM grader cannot resolve a 16–48 px sprite (documented "VLM
  Visual Verification — Known Limitation").

Fix: render **frame 0 only** (the top-left 16×24 cell). Do this by
reusing the QuadMesh's own generated vertex + UV arrays (so the UV
orientation that currently renders agents upright is preserved — **zero
flip risk**) and scaling every UV into frame 0. The result is one figure
per agent instead of a 4×3 grid of twelve.

### Defect 2 — mouse-wheel zoom barely zooms in (ZOOM)

Diagnosis facts:
- `scripts/ui/camera_controller.gd` `_unhandled_input` correctly handles
  `MOUSE_BUTTON_WHEEL_UP/DOWN` → tween `Camera2D.zoom`, clamped to
  `[ZOOM_MIN, ZOOM_MAX]`. No `_input`/`_gui_input` consumer exists
  anywhere; visible panels are `MOUSE_FILTER_IGNORE`; the inspector
  (default STOP) is hidden by default. The wheel path to the camera is
  clear — input is **not** intercepted.
- Root cause: `ZOOM_DEFAULT = 3.0` (Phase 13-α) sits only ~1.7
  wheel-notches (×1.25 each) below `ZOOM_MAX = 4.0`. So zoom-IN
  (3.0 → 3.75 → 4.0) is nearly exhausted at the default — "줌이 안
  당겨짐" (can't zoom in). Zoom-OUT (3.0 → 0.5) has full range.

Fix: raise `ZOOM_MAX` 4.0 → **8.0** (≈4.4 notches of zoom-in headroom
from the 3.0 default). Keep `ZOOM_MIN` (0.5) and `ZOOM_DEFAULT` (3.0)
unchanged — Phase 13-α deliberately set the default to 3.0 for agent
visibility, and the form fix makes agents clearer, so the default stays.

### Explicit non-scope (deferred — honest disclosure)
- **Agent COLOR (green)**: the agents render green because the
  palette-swap is not effective at runtime (the LUT body row is brown,
  so a working shader would produce brown bodies; the render is green =
  raw sprite showing through). The exact shader-failure mechanism needs
  runtime debugging (Godot window + shader inspection). **Touching
  `shaders/palette_swap.gdshader` is also blocked by the Phase 12-α A18
  "shader unchanged" git-diff lock.** Color is a separate follow-up.
- **Aspect**: frame 0 (16×24, aspect 2:3) is sampled onto the existing
  64×72 quad (8:9), so the single figure renders ~1.33× wider than
  native (stocky but clearly one humanoid). Correcting the quad aspect
  is deferred — the primary win (one figure vs twelve) is achieved
  without changing `SPRITE_SCALE` or the quad size.
- **Movement pattern** (Brownian Idle wander): Section 16+.
- **New sprite art**: not created.

### Why the form fix avoids the shader (verified Step 0)
- `harness_p12_alpha_camera_zoom.rs` A18 fails if
  `shaders/palette_swap.gdshader` appears in the `git diff` vs
  origin/lead/main — a deliberate Phase 12-α scope lock. The form fix
  therefore lives entirely in `agent_renderer.gd` (UV/mesh), leaving the
  shader byte-identical. A16/A17 (shader color-logic strings) and A18
  (shader unchanged) all stay green.
- `agent_renderer.gd` and `camera_controller.gd` are NOT under any
  git-diff `forbidden_exact`/`forbidden_prefixes` lock (the prefixes
  only cover `rust/crates/sim-*` and `localization/`).

### Harness updates required (same commit — honest disclosure)
`ZOOM_MAX == Vector2(4.0, 4.0)` is asserted in THREE existing harnesses.
Raising it to 8.0 REQUIRES updating all three in this commit:
- `harness_p12_alpha_camera_zoom.rs` A4
- `harness_p13_alpha_camera_and_buildings.rs` A3
- `harness_p14_zeta_zoom_adaptive.rs` a17 (the Phase 14-ζ regression
  guard checks the literal `Vector2(4.0,4.0)`)

These are value updates to track the new contract, not lock removals.

Preserved invariants:
- **Phase 4-γ** `SPRITE_SCALE = 0.25` — UNCHANGED (form fixed via UV).
- **Phase 4-γ** agent MultiMesh / `SPRITE_W=64` / `SPRITE_H=72` consts —
  UNCHANGED (the quad size stays 64×72; only the UVs are scaled).
- **Phase 8-δ/9-δ/11-α/13-γ/14-α** agent_renderer cue logic + constants
  (`ROLE_BUCKET_COUNT`, `ICON_OFFSET_PX`, `STATE_TINTS`,
  `STATE_SCALE_BOOST`, recall/combat cues) — UNCHANGED.
- **Phase 11-α** `palette_swap.gdshader` (`vec4 modulate = COLOR`,
  `palette_color.rgb * modulate.rgb`) — UNTOUCHED.
- **Phase 12-α** `ZOOM_MIN = Vector2(0.5,0.5)` — UNCHANGED.
- **Phase 13-α** `ZOOM_DEFAULT = Vector2(3.0,3.0)` — UNCHANGED.
- **Phase 14-ζ** `zoom_lod_controller.gd` thresholds (1.0/2.0) — UNCHANGED
  (the LOD tiers are independent of ZOOM_MAX; zoom 8.0 is still CLOSE).

## Section 2: What to Build

**Modified files (production)**:
- `scripts/ui/agent_renderer.gd` — in `_ready`, replace the direct
  `multi_mesh.mesh = quad` assignment with a frame-0 `ArrayMesh` built
  from the QuadMesh's own arrays with UVs scaled into the top-left cell.
  Add `SHEET_COLS`, `SHEET_ROWS`, `AGENT_FRAME_INDEX` constants. Do NOT
  change `SPRITE_SCALE`, `SPRITE_W`, `SPRITE_H`, or any cue / palette /
  state constant or function.
- `scripts/ui/camera_controller.gd` — change `ZOOM_MAX` from
  `Vector2(4.0, 4.0)` to `Vector2(8.0, 8.0)`. Do NOT change `ZOOM_MIN`,
  `ZOOM_DEFAULT`, `ZOOM_FACTOR`, `TWEEN_DURATION`, or any function.

**Modified files (harness — required by the ZOOM_MAX change)**:
Step 0 (G Phase A) confirmed **NINE** existing harness files assert
`ZOOM_MAX == Vector2(4.0,4.0)` as a camera-controller regression guard.
Raising ZOOM_MAX to 8.0 REQUIRES updating ALL NINE in this commit (each
is the identical literal swap `Vector2(4.0,4.0)` → `Vector2(8.0,8.0)` plus
its assertion message + `println!`). The Generator MUST grep
`Vector2(4.0,4.0)` across `rust/crates/sim-test/tests/` and update every
camera_controller ZOOM_MAX assertion — leaving any one unchanged FAILS
`cargo test --workspace`:
- `rust/crates/sim-test/tests/harness_p12_alpha_camera_zoom.rs` — A4
  (accepted-literal list + message).
- `rust/crates/sim-test/tests/harness_p13_alpha_camera_and_buildings.rs` — A3.
- `rust/crates/sim-test/tests/harness_p13_beta_resource_placeholders.rs` — A10.4.
- `rust/crates/sim-test/tests/harness_p13_gamma_interaction_cues.rs` — A12.4.
- `rust/crates/sim-test/tests/harness_p13_delta_basic_hud.rs` — A13.4.
- `rust/crates/sim-test/tests/harness_p13_epsilon_bootstrap_seed.rs` — A10.5.
- `rust/crates/sim-test/tests/harness_p14_alpha_agent_sprite_overhaul.rs` — A15.5.
- `rust/crates/sim-test/tests/harness_p14_gamma_click_inspector.rs` — A28.
- `rust/crates/sim-test/tests/harness_p14_zeta_zoom_adaptive.rs` — a19
  (camera-controller regression literal list; keep `Vector2(0.5,0.5)` +
  `Vector2(3.0,3.0)`, swap only the `4.0` literal).

**New files**:
- `rust/crates/sim-test/tests/harness_g_phase_a_agent_frame_zoom.rs` —
  static file-inspection assertions (≥10) following Phase 14-ε/ζ
  precedent (project_root + strip_gd_comments + find_decl_rhss +
  unique_decl_rhs + parse_int_rhs + find_func_body + no_ws helpers).

**Not changed (CRITICAL — hard rules)**:
- `shaders/palette_swap.gdshader` — **absolutely untouched** (Phase 12-α
  A18 git-diff lock + color is out of scope). The form fix is UV-only in
  agent_renderer.gd.
- `scripts/ui/panels/*.gd` — untouched.
- `scripts/ui/world_renderer.gd`, `scripts/ui/activity_trail_renderer.gd`,
  `scripts/ui/settlement_overview_renderer.gd`,
  `scripts/ui/zoom_lod_controller.gd` — untouched.
- `assets/sprites/agent_base.png` and all assets — untouched (no new art).
- All Rust crate code except the listed sim-test harness files.
- All locales.

**Cross-phase regression-guard reference files (READ-ONLY in plan — the
harness MAY reference these to verify prior invariants; the plan must NOT
propose modifications to them)**:
- `shaders/palette_swap.gdshader` (Phase 11-α A16/A17 + Phase 12-α A18).
- `scripts/ui/zoom_lod_controller.gd` (Phase 14-ζ).
ALL `scripts/ui/*.gd` files are FLAT in `scripts/ui/` — there is NO
`scripts/ui/renderers/` subdirectory. Use these EXACT paths.

**Forbidden plan assertions**:
- Do NOT propose modifying `shaders/palette_swap.gdshader` (locked +
  out of scope).
- Do NOT propose changing `SPRITE_SCALE`, `SPRITE_W`, `SPRITE_H`,
  `ZOOM_MIN`, or `ZOOM_DEFAULT`.
- Do NOT add locale requirements (no text).
- Do NOT propose a new sprite asset.
- Do NOT propose any `.rs` change outside `rust/crates/sim-test/tests/`.

## Section 3: How to Implement

### `scripts/ui/agent_renderer.gd` — add frame-sheet constants

Add near `SPRITE_SCALE` (after line `const SPRITE_SCALE := 0.25`):

```gdscript
# V7 G Phase A — agent_base.png is a 4-col × 3-row = 12-frame sprite
# sheet (16×24 per frame). Prior to G Phase A the renderer assigned a
# QuadMesh whose UVs span the WHOLE sheet (0→1), so every agent drew all
# 12 frames squished into one quad and read as a "clump" rather than a
# single figure. _ready now renders frame AGENT_FRAME_INDEX only.
const SHEET_COLS := 4
const SHEET_ROWS := 3
const AGENT_FRAME_INDEX := 0   # 0 = top-left cell (front-facing idle)
```

### `scripts/ui/agent_renderer.gd` — frame-0 mesh in `_ready`

Replace this existing block:

```gdscript
	var quad := QuadMesh.new()
	quad.size = Vector2(SPRITE_W, SPRITE_H)

	multi_mesh = MultiMesh.new()
	multi_mesh.transform_format = MultiMesh.TRANSFORM_2D
	multi_mesh.use_colors = true
	multi_mesh.use_custom_data = true
	multi_mesh.mesh = quad
	multi_mesh.instance_count = 0
```

with:

```gdscript
	# V7 G Phase A — render a SINGLE frame (AGENT_FRAME_INDEX) of the 4×3
	# agent_base sheet instead of the whole sheet. Reuse the QuadMesh's own
	# generated vertices + UVs (so the UV orientation that already renders
	# agents upright is preserved — zero flip risk) and remap every UV into
	# the chosen frame's sub-rectangle. The palette_swap shader is untouched:
	# it still samples `texture(TEXTURE, UV)` and reads tex.g for the
	# hair/body/skin row from the single sampled frame. SPRITE_SCALE
	# (Phase 4-γ 0.25) and the 64×72 quad size are unchanged.
	var quad := QuadMesh.new()
	quad.size = Vector2(SPRITE_W, SPRITE_H)
	var quad_arrays: Array = quad.get_mesh_arrays()
	var src_uvs: PackedVector2Array = quad_arrays[Mesh.ARRAY_TEX_UV]
	# floori(float/float) computes the integer row WITHOUT triggering the
	# GDScript INTEGER_DIVISION warning. The harness GDScript strict check
	# (D Phase A) treats warnings as errors, so a bare int `/` int here
	# (e.g. `AGENT_FRAME_INDEX / SHEET_COLS`) FAILS the gdcheck. `%` (modulo)
	# does not warn, so frame_col is fine as-is.
	var frame_col: int = AGENT_FRAME_INDEX % SHEET_COLS
	var frame_row: int = floori(float(AGENT_FRAME_INDEX) / float(SHEET_COLS))
	var uv_offset := Vector2(
		float(frame_col) / float(SHEET_COLS),
		float(frame_row) / float(SHEET_ROWS),
	)
	var uv_scale := Vector2(1.0 / float(SHEET_COLS), 1.0 / float(SHEET_ROWS))
	var frame_uvs := PackedVector2Array()
	for uv in src_uvs:
		frame_uvs.append(uv_offset + uv * uv_scale)
	quad_arrays[Mesh.ARRAY_TEX_UV] = frame_uvs
	var frame_mesh := ArrayMesh.new()
	frame_mesh.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, quad_arrays)

	multi_mesh = MultiMesh.new()
	multi_mesh.transform_format = MultiMesh.TRANSFORM_2D
	multi_mesh.use_colors = true
	multi_mesh.use_custom_data = true
	multi_mesh.mesh = frame_mesh
	multi_mesh.instance_count = 0
```

Leave the rest of `_ready` (texture / palette / shader / material /
`add_child`) and all other functions byte-for-byte unchanged.

### `scripts/ui/camera_controller.gd` — raise ZOOM_MAX

Replace:

```gdscript
const ZOOM_MAX: Vector2 = Vector2(4.0, 4.0)
```

with:

```gdscript
# V7 G Phase A — zoom-in ceiling raised 4.0 → 8.0. Phase 13-α set
# ZOOM_DEFAULT to 3.0, which sat only ~1.7 wheel-notches (×1.25) below the
# old 4.0 ceiling, so zoom-IN felt broken ("줌이 안 당겨짐"). 8.0 gives
# ~4.4 notches of zoom-in headroom from the 3.0 default. ZOOM_MIN and
# ZOOM_DEFAULT are unchanged.
const ZOOM_MAX: Vector2 = Vector2(8.0, 8.0)
```

Leave every other line of camera_controller.gd unchanged.

### Existing-harness updates (ZOOM_MAX 4.0 → 8.0)

`harness_p12_alpha_camera_zoom.rs` A4 — change the accepted-literal list
to the 8.0 variants and the message/print:

```rust
    let accepted = [
        "Vector2(8.0, 8.0)",
        "Vector2(8.0,8.0)",
        "Vector2(8.0 , 8.0)",
    ];
    let matched = accepted.iter().any(|c| line.contains(c));
    assert!(
        matched,
        "A4: ZOOM_MAX must be declared with `Vector2(8.0, 8.0)` (whitespace \
         tolerant, raised from 4.0 in G Phase A). Line: `{line}`"
    );
    println!("[P12-α A4] ZOOM_MAX = Vector2(8.0, 8.0) (G Phase A) ✓");
```

`harness_p13_alpha_camera_and_buildings.rs` A3 — expect the new literal:

```rust
    assert_eq!(
        compact, "Vector2(8.0,8.0)",
        "A3: ZOOM_MAX RHS must equal Vector2(8.0,8.0) (whitespace-stripped; \
         raised from 4.0 in G Phase A); got `{rhs}`"
    );
    println!("[P13-α A3] ZOOM_MAX = Vector2(8.0, 8.0) (G Phase A) ✓");
```

`harness_p14_zeta_zoom_adaptive.rs` (camera-controller regression literal
list, ~a19) — change `"Vector2(4.0,4.0)"` to `"Vector2(8.0,8.0)"` (keep
`Vector2(0.5,0.5)` and `Vector2(3.0,3.0)`).

**The remaining SIX** (`harness_p13_beta_resource_placeholders.rs` A10.4,
`harness_p13_gamma_interaction_cues.rs` A12.4,
`harness_p13_delta_basic_hud.rs` A13.4,
`harness_p13_epsilon_bootstrap_seed.rs` A10.5,
`harness_p14_alpha_agent_sprite_overhaul.rs` A15.5,
`harness_p14_gamma_click_inspector.rs` A28) each contain an identical
`ZOOM_MAX == Vector2(4.0,4.0)` regression assertion (plus a `ZOOM_MIN
(0.5,0.5) + ZOOM_MAX(4.0,4.0) preserved` println in most). Apply the SAME
swap: `Vector2(4.0,4.0)` → `Vector2(8.0,8.0)` in both the `assert_eq!`
expected literal/message and the `println!`. Do NOT touch their
`ZOOM_MIN(0.5,0.5)` checks. Verify with
`grep -rn "Vector2(4.0,4.0)" rust/crates/sim-test/tests/` returning ZERO
camera_controller ZOOM_MAX matches after the edit.

### New harness — `harness_g_phase_a_agent_frame_zoom.rs`

Follow Phase 14-ε/ζ structure. Helpers: project_root, read_file,
strip_gd_comments, find_decl_rhss, unique_decl_rhs, parse_int_rhs,
find_func_body, no_ws. Read sources for agent_renderer.gd,
camera_controller.gd, palette_swap.gdshader.

1. `a1_sheet_cols_equals_4` — agent_renderer.gd `SHEET_COLS` RHS == 4.
2. `a2_sheet_rows_equals_3` — `SHEET_ROWS` RHS == 3.
3. `a3_agent_frame_index_declared` — `AGENT_FRAME_INDEX` declared, RHS
   parses as int (0).
4. `a4_ready_reuses_quad_mesh_arrays` — `_ready` body contains
   `get_mesh_arrays` AND `Mesh.ARRAY_TEX_UV` (UV orientation reused from
   QuadMesh = flip-safe).
5. `a5_ready_scales_uv_by_sheet_grid` — `_ready` body contains both
   `SHEET_COLS` and `SHEET_ROWS` AND `ArrayMesh` AND
   `add_surface_from_arrays` (frame sub-rect built).
6. `a6_multimesh_uses_frame_mesh_not_whole_quad` — `_ready` body
   contains `multi_mesh.mesh = frame_mesh` (whitespace-collapsed) AND
   does NOT contain `multi_mesh.mesh = quad` (whitespace-collapsed; the
   whole-sheet assignment is gone).
7. `a7_sprite_scale_preserved` — agent_renderer.gd `SPRITE_SCALE` RHS ==
   0.25 (Phase 4-γ invariant intact; form fixed via UV not scale).
8. `a8_agent_renderer_cue_invariants_intact` — agent_renderer.gd
   contains `ROLE_BUCKET_COUNT` AND `ICON_OFFSET_PX` AND `STATE_TINTS`
   AND `STATE_SCALE_BOOST` AND `SPRITE_W` AND `SPRITE_H`.
9. `a9_zoom_max_equals_eight` — camera_controller.gd `ZOOM_MAX` RHS
   whitespace-collapsed == `Vector2(8.0,8.0)`.
10. `a10_zoom_min_and_default_preserved` — camera_controller.gd
    `ZOOM_MIN` ws-collapsed == `Vector2(0.5,0.5)` AND `ZOOM_DEFAULT`
    ws-collapsed == `Vector2(3.0,3.0)`.
11. `a11_palette_shader_untouched` — `shaders/palette_swap.gdshader`
    still contains `texture(TEXTURE, UV)` AND `vec4 modulate = COLOR`
    AND `palette_color.rgb * modulate.rgb` (shader byte-path intact;
    color logic not modified by the form fix).
12. `a12_zoom_lod_controller_thresholds_intact` —
    `scripts/ui/zoom_lod_controller.gd` still contains `ZOOM_FAR_MAX`
    AND `ZOOM_CLOSE_MIN` (Phase 14-ζ unaffected by the new ceiling).

## Section 4: Locale

No new locale keys. Pure visual / camera-constant change.

## Section 5: Verification

```bash
cd rust && cargo test -p sim-test --test harness_g_phase_a_agent_frame_zoom -- --nocapture
cd rust && cargo test -p sim-test --test harness_p12_alpha_camera_zoom -- --nocapture
cd rust && cargo test -p sim-test --test harness_p13_alpha_camera_and_buildings -- --nocapture
cd rust && cargo test -p sim-test --test harness_p14_zeta_zoom_adaptive -- --nocapture
cd rust && cargo test --workspace
cd rust && cargo clippy --workspace --all-targets -- -D warnings
/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot \
    --path . --headless --check-only --script scripts/ui/agent_renderer.gd
/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot \
    --path . --headless --check-only --script scripts/ui/camera_controller.gd
```

Expected: harness_g_phase_a ≥10 PASS; p12-α / p13-α / p14-ζ updated and
green; workspace + clippy clean; GDScript parse clean.

## Section 6: Lane

`--quick` — GDScript form fix (agent_renderer.gd) + one camera constant
(camera_controller.gd) + one new sim-test harness + three sim-test
harness value updates. Zero Rust crate change. Zero shader change. Zero
FFI change. Zero asset / locale change.

## Section 7: 인게임 확인사항

**Expected visual change (windowed Godot)**:
- Each agent renders as ONE figure (frame 0 of the sheet) instead of a
  4×3 grid of twelve squished mini-figures — no more "grass clump"
  texture.
- Mouse-wheel zoom-in now travels well past the old 4.0 ceiling (up to
  8.0×), so the player can zoom in close on agents/buildings.

**Still NOT fixed this round (honest disclosure)**:
- **Agents are still GREEN.** The form fix does not touch color. The
  green is the palette-swap not taking effect at runtime; that is a
  separate follow-up requiring a windowed shader debug. After G Phase A
  you will see a single green figure per agent (recognisable shape,
  wrong colour) — that is expected.
- The single figure is ~1.33× wider than native (frame 2:3 sampled onto
  the 8:9 quad). Recognisable as one humanoid; aspect polish deferred.

**Pipeline VLM**: at default zoom the scene composition is similar to
Phase 14-ζ; the per-agent frame change is at/below VLM sub-resolution
(documented limitation). The Rust harness file checks + the windowed
review are the authoritative gates. VLM `VISUAL_OK`/`WARNING` both
acceptable; FAIL only on crash / layout break.

**Honest disclosure**:
- Form fix is UV-only in `agent_renderer.gd` (reuses QuadMesh arrays,
  scales UVs to frame 0). The `palette_swap.gdshader` is byte-identical
  (Phase 12-α A18 lock respected; color out of scope).
- `ZOOM_MAX` raised 4.0 → 8.0; three prior-phase harness assertions
  (p12-α A4, p13-α A3, p14-ζ a17) updated in the SAME commit to track
  the new contract. `ZOOM_MIN` / `ZOOM_DEFAULT` unchanged.
- `SPRITE_SCALE` 0.25 and all agent cue/palette constants preserved.

### 다음 단계 (자동 진행 X — 사용자 결정 대기)
- 색 수정 (palette-swap 런타임 디버그) — 사용자 windowed 확인 + 디버그.
- Stage 56 `135180a0` → Stage 57 (이 commit).
