# H Phase A — Trackpad zoom + pause + camera tracking + palette-swap color fix

Feature: h-phase-a-input-camera-color
Lane: --quick (GDScript camera_controller + canvas shader + harness
updates; zero Rust crate change, zero FFI change)
Parent: G Phase A (`78681346`) successor. Addresses windowed-Godot
findings (claude.ai confirmed against commit 78681346) + the runtime
color root-cause confirmed by a standalone render-debug in this session.

## Section 1: Implementation Intent

Four confirmed defects from windowed Godot + a runtime shader debug:

1. **Trackpad zoom dead** — `camera_controller.gd` only handles
   `MOUSE_BUTTON_WHEEL_UP/DOWN`. A MacBook trackpad sends
   `InputEventPanGesture` (two-finger scroll) and
   `InputEventMagnifyGesture` (pinch), which are never handled. (The
   G Phase A `ZOOM_MAX 8.0` raise was real but orthogonal.)

2. **No pause** — there is no pause key. `KEY_SPACE` is taken
   (`world_renderer.gd` overlay-channel cycle). The sim ticks in
   `WorldSimNode.process()` (Rust, Gaffer accumulator, `self.engine.tick()`).

3. **Agents wander off-screen** — bootstrap spawns 8×8=64 agents on a
   grid; `AgentMovementSystem` Brownian-walks them every tick, so they
   scatter across tiles 0–63 while the camera is fixed at (960,540). By
   ~tick 1500 they've dispersed off-frame.

4. **Agents render GREEN** (ROOT CAUSE CONFIRMED at runtime this session):
   `palette_swap.gdshader` does `vec4 modulate = COLOR;` at the **fragment**
   stage assuming `COLOR` is the per-instance tint. In Godot 4.6
   canvas_item, `COLOR` at fragment entry is **already multiplied by the
   sampled TEXTURE** (the green-keyed agent sprite). So `modulate` captured
   texture-green, and the final `palette_color.rgb * modulate.rgb` re-applied
   green → every agent green regardless of the (correct, brown) palette.

   Render-debug evidence (standalone MultiMeshInstance2D, this session):
   - production shader → GREEN (2,37,3)
   - `texture(palette_lut, …)` direct → BROWN (LUT body is brown) ✓
   - INSTANCE_CUSTOM viz → correct (magenta) ✓ (transmits fine)
   - drop `* modulate.rgb` → BROWN ✓ (multiply is the culprit)
   - **FIX: capture `v_tint = COLOR` in `vertex()` (pre-texture), multiply by
     `v_tint.rgb` → BROWN ✓** with white tint; state-tinted otherwise.

### Scope (4 fixes; sprite-enlarge deliberately SKIPPED)

A. **Trackpad zoom** (`camera_controller.gd`): handle
   `InputEventPanGesture` + `InputEventMagnifyGesture` in
   `_unhandled_input`; keep the existing mouse-wheel path.

B. **Pause** (`camera_controller.gd`): `KEY_P` toggles
   `WorldSim.process_mode` between `PROCESS_MODE_INHERIT` and
   `PROCESS_MODE_DISABLED`. Disabling stops `WorldSimNode.process()` (the
   tick) while leaving its `#[func]` getters callable, so renderers keep
   drawing the frozen frame. Zero Rust change. `KEY_SPACE` untouched.

C. **Camera agent-tracking** (`camera_controller.gd` `_process`): read
   `get_agent_snapshot()` xs/ys, compute the mean tile, convert to world
   px (same `SPRITE_ORIGIN + tile*TILE_SIZE` basis as the renderers), and
   `position = position.lerp(target, …)` gently. Keeps the dispersing
   swarm on-screen. No backend movement change.

D. **Palette-swap color fix** (`shaders/palette_swap.gdshader`): capture
   the tint at the **vertex** stage (`varying vec4 v_tint; v_tint = COLOR;`)
   and multiply `palette_color.rgb * v_tint.rgb` in fragment. Removes the
   `vec4 modulate = COLOR;` fragment capture (the bug).

**SKIPPED — sprite enlarge**: `SPRITE_SCALE` (Phase 4-γ) is locked in 14
harness files and `ZOOM_DEFAULT` in 10 — enlarging via either is heavy
churn. With camera-tracking (agents stay centred) + trackpad/wheel zoom to
8.0×, the user can zoom in to view agents. Deferred; disclosed.

### Honest disclosure
- After the color fix, **Idle agents** (the majority at any tick) render in
  natural muted tones (palette × Idle blue-ish tint). **State-3 agents**
  (Consuming-other = eating/building/sleeping) render green-tinted — this is
  the **intended Phase 11-α STATE_TINTS state cue** (state-3 tint IS green),
  now applied correctly to the palette instead of to texture-green. So a
  minority of agents will still look green *by design*; the "ALL agents
  green" bug is fixed. If pure-palette (no state tint) is preferred, that's
  a one-line follow-up (drop the `* v_tint.rgb`).
- Trackpad gesture sign/sensitivity (`PAN_ZOOM_SENSITIVITY`) is a
  best-effort default; the user may want it inverted/retuned after feel.
- Camera tracking follows the swarm centroid; if the user later wants
  manual pan, that's a separate feature.

### Preserved invariants
- Phase 12-α `ZOOM_MIN`(0.5) / Phase 13-α `ZOOM_DEFAULT`(3.0) / G Phase A
  `ZOOM_MAX`(8.0) — UNCHANGED. `ZOOM_FACTOR`/`TWEEN_DURATION` unchanged.
- Phase 4-γ `SPRITE_SCALE`=0.25, G Phase A frame-0 mesh — UNCHANGED
  (agent_renderer.gd not touched).
- Phase 11-α STATE_TINTS (agent_renderer set_instance_color) — UNCHANGED
  (the fix makes them apply correctly).
- Phase 11-α shader row-selection (tex.g → hair/body/skin, palette_lut,
  palette_uv) — UNCHANGED; only the tint-capture stage moves.
- KEY_SPACE overlay cycle (world_renderer) — UNTOUCHED.
- Phase 14-ζ zoom_lod_controller thresholds — UNCHANGED.

## Section 2: What to Build

**Modified files (production)**:
- `scripts/ui/camera_controller.gd` — add trackpad gesture handling +
  KEY_P pause + `_process` camera tracking + WorldSim node resolve +
  TILE_SIZE/SPRITE_ORIGIN constants. Do NOT change ZOOM_MIN/MAX/DEFAULT/
  FACTOR/TWEEN_DURATION or the existing wheel/`_apply_zoom_delta` logic.
- `shaders/palette_swap.gdshader` — move the tint capture to vertex
  (`varying vec4 v_tint; v_tint = COLOR;`), multiply by `v_tint.rgb`,
  remove the fragment `vec4 modulate = COLOR;`. Keep the row-selection +
  palette_uv + LUT-sample logic byte-for-byte otherwise.

**Modified files (harness — required by the shader change)**:
- `rust/crates/sim-test/tests/harness_p11_alpha_agent_renderer.rs` — A16:
  assert the vertex tint capture (`v_tint = COLOR` in `vertex()` +
  `varying vec4 v_tint`) instead of `vec4 modulate = COLOR`. A17: assert
  `palette_color.rgb * v_tint.rgb` instead of `* modulate.rgb`.
- `rust/crates/sim-test/tests/harness_p12_alpha_camera_zoom.rs` — A18:
  remove `"shaders/palette_swap.gdshader"` from `forbidden_exact` (H Phase
  A intentionally fixes the palette bug; add a comment). Keep
  `causal_panel.gd` + `localization/` locks.
- `rust/crates/sim-test/tests/harness_g_phase_a_agent_frame_zoom.rs` —
  a11: change the asserted shader literals from `vec4 modulate = COLOR` +
  `palette_color.rgb * modulate.rgb` to `v_tint = COLOR` +
  `palette_color.rgb * v_tint.rgb` (keep `texture(TEXTURE, UV)`).

**New files**:
- `rust/crates/sim-test/tests/harness_h_phase_a_input_camera_color.rs` —
  static file-inspection assertions (≥14).

**Not changed (CRITICAL)**:
- `scripts/ui/agent_renderer.gd` — untouched (SPRITE_SCALE, frame-0 mesh,
  STATE_TINTS, cues all preserved).
- `scripts/ui/world_renderer.gd` — untouched (KEY_SPACE overlay stays).
- `scripts/ui/zoom_lod_controller.gd`, `activity_trail_renderer.gd`,
  `settlement_overview_renderer.gd`, panels — untouched.
- `scenes/main.tscn` — untouched (pause lives in camera_controller; no new
  node → no load_steps churn).
- All Rust crate code except the listed sim-test harness files.
- `palette_lut.png`, `agent_base.png`, all assets, locales.

**Cross-phase regression-guard reference files (READ-ONLY in plan)**:
- `scripts/ui/agent_renderer.gd`, `scripts/ui/world_renderer.gd`,
  `scripts/ui/zoom_lod_controller.gd` (all flat in `scripts/ui/`, NO
  `renderers/` subdir).

**Forbidden plan assertions**:
- Do NOT change `SPRITE_SCALE`, `ZOOM_MIN`, `ZOOM_MAX`, `ZOOM_DEFAULT`.
- Do NOT add a new scene node / modify `scenes/main.tscn`.
- Do NOT add locale keys.
- Do NOT modify `agent_renderer.gd` or `world_renderer.gd`.
- Do NOT propose backend (movement) changes.
- No `.rs` change outside `rust/crates/sim-test/tests/`.

## Section 3: How to Implement

### `scripts/ui/camera_controller.gd`

Add constants near the existing ZOOM consts:

```gdscript
# V7 H Phase A — camera agent-tracking coordinate basis. Mirrors
# world_renderer / agent_renderer (SPRITE_ORIGIN + tile*TILE_SIZE) so the
# tracked centroid lands on the same world-pixel grid the sprites use.
const TILE_SIZE: int = 16
const SPRITE_ORIGIN_X: int = 448
const SPRITE_ORIGIN_Y: int = 28
const CAMERA_TRACK_SPEED: float = 2.0          # lerp rate toward swarm centroid
const PAN_ZOOM_SENSITIVITY: float = 0.1        # trackpad two-finger-scroll → zoom
```

Add state:

```gdscript
var _world_sim: Node = null
var _paused: bool = false
```

In `_ready()` (after the existing zoom init), resolve WorldSim:

```gdscript
	_world_sim = get_node_or_null("/root/Main/WorldSim")
```

Extend `_unhandled_input` — keep the existing wheel branch; add:

```gdscript
	elif event is InputEventPanGesture:
		# Trackpad two-finger scroll → zoom (scroll up = delta.y < 0 = zoom in).
		_apply_zoom_delta(1.0 - event.delta.y * PAN_ZOOM_SENSITIVITY)
		get_viewport().set_input_as_handled()
	elif event is InputEventMagnifyGesture:
		# Trackpad pinch → zoom (factor > 1 = zoom in).
		_apply_zoom_delta(event.factor)
		get_viewport().set_input_as_handled()
	elif event is InputEventKey and event.pressed and not event.echo \
			and event.keycode == KEY_P:
		_toggle_pause()
		get_viewport().set_input_as_handled()
```

Add pause + tracking functions:

```gdscript
func _toggle_pause() -> void:
	# Freeze/resume the simulation by gating WorldSimNode.process() (the Rust
	# tick). Getters still work, so renderers keep drawing the frozen frame.
	_paused = not _paused
	if _world_sim != null:
		_world_sim.process_mode = (
			Node.PROCESS_MODE_DISABLED if _paused else Node.PROCESS_MODE_INHERIT
		)


func _process(delta: float) -> void:
	# V7 H Phase A — gently track the agent swarm centroid so Brownian-
	# dispersing agents (AgentMovementSystem) stay on-screen.
	if _world_sim == null:
		return
	var snap: Variant = _world_sim.call("get_agent_snapshot")
	if not (snap is Dictionary):
		return
	var snap_dict: Dictionary = snap
	var xs: Variant = snap_dict.get("xs", null)
	var ys: Variant = snap_dict.get("ys", null)
	if not (xs is PackedInt32Array and ys is PackedInt32Array):
		return
	var xs_arr: PackedInt32Array = xs
	var ys_arr: PackedInt32Array = ys
	var n: int = xs_arr.size()
	if n == 0 or ys_arr.size() != n:
		return
	var sum_x: int = 0
	var sum_y: int = 0
	for i in n:
		sum_x += xs_arr[i]
		sum_y += ys_arr[i]
	var mean_tx: float = float(sum_x) / float(n)
	var mean_ty: float = float(sum_y) / float(n)
	var target := Vector2(
		float(SPRITE_ORIGIN_X) + mean_tx * float(TILE_SIZE) + float(TILE_SIZE) / 2.0,
		float(SPRITE_ORIGIN_Y) + mean_ty * float(TILE_SIZE) + float(TILE_SIZE) / 2.0,
	)
	position = position.lerp(target, clampf(delta * CAMERA_TRACK_SPEED, 0.0, 1.0))
```

Leave all existing lines (ZOOM consts, `_zoom_tween`, `_target_zoom`,
wheel branch, `_apply_zoom_delta`) unchanged.

### `shaders/palette_swap.gdshader` (the runtime-verified fix)

```gdscript
shader_type canvas_item;

// V7 Phase 4-γ — per-instance palette indices via INSTANCE_CUSTOM.
// V7 H Phase A — FIX: the per-instance tint is captured at the VERTEX stage
// (v_tint). In Godot 4.6 canvas_item, COLOR at FRAGMENT entry is already
// multiplied by the sampled TEXTURE (the green-keyed agent sprite), so the
// previous `vec4 modulate = COLOR;` in fragment() captured texture-green and
// `palette_color * modulate` re-applied it → every agent rendered green.
// Capturing COLOR in vertex() (before texture sampling) yields the true
// per-instance state tint. (Confirmed by render-debug: vertex-tint → brown.)
//
// Channel layout of INSTANCE_CUSTOM (normalised 0..=1):
//   .r hair col [0,8)  .g body col [0,4)  .b skin col [0,8)

uniform sampler2D palette_lut : filter_nearest, repeat_disable;

varying flat vec3 v_palette;
varying vec4 v_tint;

vec2 palette_uv(float row_index, float column_index) {
	return vec2((column_index + 0.5) / 8.0, (row_index + 0.5) / 3.0);
}

void vertex() {
	v_palette = INSTANCE_CUSTOM.rgb;
	v_tint = COLOR;
}

void fragment() {
	vec4 tex = texture(TEXTURE, UV);
	if (tex.a <= 0.01) {
		COLOR = tex;
	} else {
		float hair_col = floor(v_palette.r * 7.0 + 0.5);
		float body_col = floor(v_palette.g * 3.0 + 0.5);
		float skin_col = floor(v_palette.b * 7.0 + 0.5);

		float row_selector = tex.g;
		float row_index = 1.0;
		float column_index = body_col;

		if (row_selector < 0.25) {
			row_index = 0.0;
			column_index = hair_col;
		} else if (row_selector < 0.75) {
			row_index = 1.0;
			column_index = body_col;
		} else {
			row_index = 2.0;
			column_index = skin_col;
		}

		vec4 palette_color = texture(palette_lut, palette_uv(row_index, column_index));
		COLOR = vec4(palette_color.rgb * v_tint.rgb, tex.a);
	}
}
```

### Harness updates

`harness_p11_alpha_agent_renderer.rs` A16 — assert the vertex tint capture:
the shader contains `varying vec4 v_tint` AND the `vertex()` body contains
`v_tint = COLOR` (whitespace-collapsed). Update the message + println.
**MUST also update the assertion's `// Type:` annotation comment to
`// Type: D` (regression guard for the H Phase A shader change) — do NOT
leave the stale `Type C` comment; the plan classifies A16/A17 as Type D and
the code comment must match (a Type C/D mismatch triggers an Evaluator
RE-CODE).**

`harness_p11_alpha_agent_renderer.rs` A17 — assert
`palette_color.rgb * v_tint.rgb` (whitespace-tolerant) appears in the
visible-α path. Update the message + println. **Same Type-comment rule:
set the `// Type:` annotation to `// Type: D` to match the plan.**

`harness_p12_alpha_camera_zoom.rs` A18 — remove
`"shaders/palette_swap.gdshader"` from `forbidden_exact` (keep
`scripts/ui/panels/causal_panel.gd` + the `localization/` prefix). Add a
comment: "palette_swap.gdshader intentionally modified in H Phase A
(palette tint bug fix); lock released."

`harness_g_phase_a_agent_frame_zoom.rs` a11 — change the asserted shader
literals to `["texture(TEXTURE, UV)", "v_tint = COLOR", "palette_color.rgb * v_tint.rgb"]`
(remove the two `modulate` literals). Update the message.

### New harness — `harness_h_phase_a_input_camera_color.rs`

Helpers per Phase 14-ε/G precedent (project_root, read_file,
strip_gd_comments, find_func_body, no_ws, find_decl_rhss, unique_decl_rhs).
Read camera_controller.gd + palette_swap.gdshader + agent_renderer.gd +
world_renderer.gd + zoom_lod_controller.gd.

1. `a1_trackpad_pan_gesture_handled` — camera_controller contains
   `InputEventPanGesture`.
2. `a2_trackpad_magnify_gesture_handled` — contains
   `InputEventMagnifyGesture`.
3. `a3_mouse_wheel_preserved` — still contains `MOUSE_BUTTON_WHEEL_UP`
   AND `MOUSE_BUTTON_WHEEL_DOWN`.
4. `a4_pause_key_p` — `_unhandled_input` path references `KEY_P`.
5. `a5_pause_toggles_process_mode` — contains `PROCESS_MODE_DISABLED` AND
   `PROCESS_MODE_INHERIT` AND `process_mode` (the pause mechanism).
6. `a6_camera_track_process` — has `func _process(` AND its body contains
   `get_agent_snapshot` AND `lerp` (centroid tracking).
7. `a7_resolves_world_sim` — contains `/root/Main/WorldSim`.
8. `a8_track_consts` — `CAMERA_TRACK_SPEED` declared (float) AND
   `TILE_SIZE` == 16 AND `SPRITE_ORIGIN_X` == 448.
9. `a9_zoom_invariants_intact` — camera_controller `ZOOM_MIN` ws==
   `Vector2(0.5,0.5)`, `ZOOM_MAX` ws== `Vector2(8.0,8.0)`, `ZOOM_DEFAULT`
   ws== `Vector2(3.0,3.0)`.
10. `a10_space_not_repurposed` — camera_controller does NOT contain
    `KEY_SPACE` (pause is KEY_P; SPACE stays in world_renderer).
11. `a11_shader_vertex_tint_capture` — palette_swap.gdshader contains
    `varying vec4 v_tint` AND `vertex()` body contains `v_tint = COLOR`.
12. `a12_shader_multiplies_v_tint` — palette_swap.gdshader contains
    `palette_color.rgb * v_tint.rgb` AND does NOT contain
    `vec4 modulate = COLOR` (old bug removed).
13. `a13_shader_rowselect_preserved` — palette_swap.gdshader still contains
    `row_selector = tex.g` AND `palette_uv` AND `palette_lut` (row-select
    logic intact).
14. `a14_agent_renderer_sprite_scale_intact` — agent_renderer.gd
    `SPRITE_SCALE` RHS == 0.25 AND contains `STATE_TINTS` AND `SHEET_COLS`
    (Phase 4-γ + 11-α + G Phase A preserved; agent_renderer untouched).
15. `a15_world_renderer_space_overlay_intact` — world_renderer.gd still
    contains `KEY_SPACE` (overlay cycle untouched).
16. `a16_zoom_lod_thresholds_intact` — zoom_lod_controller.gd contains
    `ZOOM_FAR_MAX` AND `ZOOM_CLOSE_MIN` (Phase 14-ζ intact).

## Section 4: Locale
No new locale keys. Input/camera/shader only.

## Section 5: Verification
```bash
cd rust && cargo test -p sim-test --test harness_h_phase_a_input_camera_color -- --nocapture
cd rust && cargo test -p sim-test --test harness_p11_alpha_agent_renderer -- --nocapture
cd rust && cargo test -p sim-test --test harness_p12_alpha_camera_zoom -- --nocapture
cd rust && cargo test -p sim-test --test harness_g_phase_a_agent_frame_zoom -- --nocapture
cd rust && cargo test --workspace
cd rust && cargo clippy --workspace --all-targets -- -D warnings
/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot \
    --path . --headless --check-only --script scripts/ui/camera_controller.gd
```
Expected: harness_h_phase_a ≥14 PASS; p11-α / p12-α / g-phase-a green;
workspace + clippy clean; GDScript parse clean.

## Section 6: Lane
`--quick` — GDScript (camera_controller) + canvas shader + sim-test harness
(1 new + 3 updates). Zero Rust crate change. Zero FFI. Zero asset/locale.

## Section 7: 인게임 확인사항

**Expected (windowed Godot)**:
- Trackpad two-finger scroll + pinch now zoom (mouse wheel still works).
- `P` pauses/resumes the sim (agents freeze; camera/zoom still respond).
- Camera gently follows the agent swarm — agents no longer disappear.
- **Agents render in natural tones** (Idle majority = muted palette);
  state-3 (eating/building) agents are green-tinted by design (Phase 11-α
  state cue). The "all agents green" bug is fixed.

**Pipeline VLM**: at default zoom the agent color change may be at/below
VLM sub-resolution; the camera now centres on the swarm so agents are more
likely visible. Rust harness file checks are authoritative. The color fix
is **runtime-verified** in this session (render-debug: vertex-tint → brown),
not speculative.

**Honest disclosure**:
- Sprite-enlarge SKIPPED (SPRITE_SCALE/ZOOM_DEFAULT locked in 14/10 harness
  files; zoom + tracking cover the need).
- State-3 agents remain green-tinted (intended cue); pure-palette is a
  one-line follow-up if the user prefers no tint.
- Trackpad gesture sign/sensitivity is a best-effort default; retune after
  feel if inverted.
- Shader change releases the Phase 12-α A18 `palette_swap.gdshader` lock +
  updates p11-α A16/A17 + g-phase-a a11 (4 harness updates, same commit,
  disclosed).

### Governance chain
Stage 57 `78681346` → Stage 58 (this commit).

### G Phase A Rule 7.1 reminder
G Phase A (`78681346`) ENV-BYPASS still owes a formal re-run by 2026-06-05.
Independent of this phase.
