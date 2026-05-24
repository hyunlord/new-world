# Phase 12-α — Camera Zoom Controls + Default 2.0× Zoom

Feature: p12-alpha-camera-zoom
Lane: --quick (single new GDScript file + main.tscn Camera2D wiring,
no Rust, no shader, no Tier 1 sprite work)
Parent: Section 13+ (`6843176f`) + `.harness/plans/phase12.md` (local).

## Section 1: Implementation Intent

Phase 12-α is the **first sub-stage** of the Tier 1 Sprites Integration
+ Camera Zoom + Agent Sprite Scale Sprint (Section 13+ Phase 12 anchor).
It targets the **immediate visible-delta proof** for D1 (`450f39cd`) that
the pipeline's whole-scene VLM grading could not produce:

- D1 retuned STATE_TINTS to cool-blue / saturated yellow / pink / green,
  but at the current default `Camera2D.zoom = (1, 1)` agents render at
  16×18 px on a 1920×1080 viewport — sub-resolution to the human eye and
  to the VLM. Phase 12-α sets the default zoom to **2.0×**, bringing
  agents to 32×36 px where the STATE_TINTS palette is unambiguously
  visible.
- Adds **mouse-wheel zoom controls** (P12Plan-3) for runtime adjustment
  between 0.5× (overview) and 4.0× (sprite-detail), with smooth tween
  interpolation (~0.15 s ease-out per wheel notch).

Substrate verified by Step 0 grep (live files, not memory):

- `scenes/main.tscn` Camera2D currently has `position = Vector2(960, 540)`
  and `zoom = Vector2(1, 1)`, **no script attached**.
- `scripts/ui/world_renderer.gd:61` and `scripts/ui/panels/causal_panel.gd:211`
  both use `_unhandled_input` — the same pattern Phase 12-α will use,
  scoped to the Camera2D node so the existing handlers (SPACE for
  channel cycle, click for causal history) remain unaffected.
- No existing `create_tween` usage in `scripts/` — Phase 12-α
  introduces the first Tween, using the Godot 4 `Node.create_tween()`
  API.
- `scripts/test/p4_gamma_rendering/harness_agent_rendering.gd` exists
  as the Phase 4-γ rendering harness, runs windowed; it asserts
  AgentRenderer + WorldSim contract but does NOT exercise camera
  controls. Phase 12-α adds a Rust file-inspection harness in the same
  style as Phase 11-α + D1 (A12-A22) for static structural checks,
  since pipeline orchestration of feature-specific runtime visual
  harnesses is unresolved (C-1 lesson).

P12Plan-4 (no SPRITE_SCALE change) is preserved — Phase 4-γ
tile-fit invariant intact, all harness_p11_alpha + D1 A20/A21/A22
assertions remain green.

## Section 2: What to Build

**New files**:
- `scripts/ui/camera_controller.gd` — new GDScript extending Camera2D,
  ~50-70 lines. Handles default zoom, mouse-wheel input, clamping,
  smooth tween interpolation.

**Modified files**:
- `scenes/main.tscn` — attach `camera_controller.gd` to the existing
  Camera2D node; change `zoom = Vector2(1, 1)` to `zoom = Vector2(2, 2)`
  for the default 2.0× zoom requirement.

**New test file**:
- `rust/crates/sim-test/tests/harness_p12_alpha_camera_zoom.rs` — Rust
  file-content inspection harness (Phase 11-α / D1 precedent). ≥10
  assertions enumerated in §3.

**Not changed**:
- Any Rust crate code (sim-core, sim-bridge, sim-engine, sim-systems)
- `scripts/ui/agent_renderer.gd` (D1 STATE_TINTS palette preserved)
- `scripts/ui/world_renderer.gd` (influence overlay rendering preserved)
- `shaders/palette_swap.gdshader`
- `harness_p11_alpha_agent_renderer.rs` (A1-A22 remain green)
- `scripts/ui/panels/causal_panel.gd`
- Any locale file (no new keys)
- Any sprite asset

## Section 3: How to Implement

### `scripts/ui/camera_controller.gd` (new)

```gdscript
extends Camera2D

# V7 Phase 12-α — Camera2D zoom controller.
#
# Provides mouse-wheel zoom-in / zoom-out with smooth tween
# interpolation between 0.5× (overview, whole 64×64 world fits) and
# 4.0× (sprite-detail, 64×72 agent sprite renders at 64×72 px). Default
# zoom 2.0× brings the D1 STATE_TINTS palette (Phase 11-α + 450f39cd)
# into clearly observable resolution (16×18 px → 32×36 px on screen).
#
# Geometric zoom step (1.25× per wheel notch) gives ~4 notches to
# traverse the full 0.5×–4.0× range, matching common 2D-camera UX.
# Tween duration is short enough (0.15 s) that successive wheel
# notches feel responsive but smooth enough that the change is not
# a hard snap.
#
# Phase 4-γ SPRITE_SCALE invariant is preserved — this controller
# only modifies Camera2D.zoom, never agent sprite scale.

const ZOOM_MIN: Vector2 = Vector2(0.5, 0.5)
const ZOOM_MAX: Vector2 = Vector2(4.0, 4.0)
const ZOOM_DEFAULT: Vector2 = Vector2(2.0, 2.0)
const ZOOM_FACTOR: float = 1.25
const TWEEN_DURATION: float = 0.15

var _zoom_tween: Tween = null
var _target_zoom: Vector2 = ZOOM_DEFAULT


func _ready() -> void:
	print("CameraController ready (V7 Phase 12-α — default zoom 2.0×)")
	zoom = ZOOM_DEFAULT
	_target_zoom = ZOOM_DEFAULT


func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventMouseButton and event.pressed:
		if event.button_index == MOUSE_BUTTON_WHEEL_UP:
			_apply_zoom_delta(ZOOM_FACTOR)
			get_viewport().set_input_as_handled()
		elif event.button_index == MOUSE_BUTTON_WHEEL_DOWN:
			_apply_zoom_delta(1.0 / ZOOM_FACTOR)
			get_viewport().set_input_as_handled()


func _apply_zoom_delta(factor: float) -> void:
	var new_zoom: Vector2 = _target_zoom * factor
	new_zoom.x = clamp(new_zoom.x, ZOOM_MIN.x, ZOOM_MAX.x)
	new_zoom.y = clamp(new_zoom.y, ZOOM_MIN.y, ZOOM_MAX.y)
	_target_zoom = new_zoom

	if _zoom_tween != null and _zoom_tween.is_valid():
		_zoom_tween.kill()
	_zoom_tween = create_tween()
	_zoom_tween.tween_property(self, "zoom", _target_zoom, TWEEN_DURATION) \
		.set_trans(Tween.TRANS_QUAD) \
		.set_ease(Tween.EASE_OUT)
```

### `scenes/main.tscn` (modify Camera2D node)

Replace the existing Camera2D block:

```
[node name="Camera2D" type="Camera2D" parent="."]
position = Vector2(960, 540)
zoom = Vector2(1, 1)
```

With:

```
[ext_resource type="Script" path="res://scripts/ui/camera_controller.gd" id="4_camera"]

[node name="Camera2D" type="Camera2D" parent="."]
position = Vector2(960, 540)
zoom = Vector2(2, 2)
script = ExtResource("4_camera")
```

(Add the `[ext_resource ...]` line near the other ext_resource declarations
in the existing `load_steps`; bump `load_steps` accordingly.)

### `rust/crates/sim-test/tests/harness_p12_alpha_camera_zoom.rs` (new)

Use the same project-root resolution helper as
`harness_p11_alpha_agent_renderer.rs`. Read
`scripts/ui/camera_controller.gd` + `scenes/main.tscn` + the existing
`scripts/ui/agent_renderer.gd` (for invariant preservation checks).

Strip GDScript `#` comments before grep so a `# … 0.25` comment in
agent_renderer.gd cannot accidentally satisfy a SPRITE_SCALE assertion.

≥10 assertions:

1. `harness_p12_alpha_a1_camera_controller_file_exists` — confirms
   `scripts/ui/camera_controller.gd` is a non-empty file.
2. `harness_p12_alpha_a2_extends_camera2d` — stripped source contains
   `extends Camera2D`.
3. `harness_p12_alpha_a3_zoom_min_constant` — declares
   `ZOOM_MIN: Vector2 = Vector2(0.5, 0.5)`.
4. `harness_p12_alpha_a4_zoom_max_constant` — declares
   `ZOOM_MAX: Vector2 = Vector2(4.0, 4.0)`.
5. `harness_p12_alpha_a5_zoom_default_constant` — declares
   `ZOOM_DEFAULT: Vector2 = Vector2(2.0, 2.0)`.
6. `harness_p12_alpha_a6_zoom_factor_geometric` — declares
   `ZOOM_FACTOR: float = 1.25`.
7. `harness_p12_alpha_a7_tween_duration_constant` — declares
   `TWEEN_DURATION: float = 0.15`.
8. `harness_p12_alpha_a8_wheel_up_handler` — stripped source contains
   `MOUSE_BUTTON_WHEEL_UP`.
9. `harness_p12_alpha_a9_wheel_down_handler` — stripped source contains
   `MOUSE_BUTTON_WHEEL_DOWN`.
10. `harness_p12_alpha_a10_zoom_clamp` — stripped source contains
    `clamp(` referencing `ZOOM_MIN` and `ZOOM_MAX`.
11. `harness_p12_alpha_a11_create_tween_used` — stripped source contains
    both `create_tween(` and `tween_property(self, "zoom"`.
12. `harness_p12_alpha_a12_input_handled_marked` — stripped source
    contains `get_viewport().set_input_as_handled()` so the wheel
    events do not propagate to other listeners (e.g. world_renderer
    SPACE handler scope).
13. `harness_p12_alpha_a13_main_tscn_camera_zoom_default_2x` — reads
    `scenes/main.tscn` and confirms the Camera2D node block contains
    `zoom = Vector2(2, 2)` (or equivalent `Vector2(2.0, 2.0)`).
14. `harness_p12_alpha_a14_main_tscn_camera_script_attached` — reads
    `scenes/main.tscn` and confirms `script = ExtResource(...)` is
    present on the Camera2D node.
15. `harness_p12_alpha_a15_phase4_gamma_sprite_scale_invariant` —
    reads `scripts/ui/agent_renderer.gd`, strips comments, confirms
    `SPRITE_SCALE := 0.25` (or `SPRITE_SCALE: float = 0.25`) remains
    declared. Anti-regression guard for Phase 4-γ tile-fit invariant.
16. `harness_p12_alpha_a16_d1_state_tints_palette_preserved` — reads
    `scripts/ui/agent_renderer.gd`, strips comments, confirms all four
    D1 STATE_TINTS literal Colors remain present
    (`Color(0.55, 0.70, 0.95, 1.0)`, `Color(1.0, 0.85, 0.15, 1.0)`,
    `Color(1.0, 0.40, 0.75, 1.0)`, `Color(0.30, 0.95, 0.35, 1.0)`).
    Anti-regression guard for D1 (`450f39cd`).

## Section 4: Locale

No new localization keys. Phase 12-α is camera/input only.

## Section 5: Verification

```bash
# New Phase 12-α harness
cd rust && cargo test -p sim-test --test harness_p12_alpha_camera_zoom -- --nocapture

# Existing Phase 11-α + D1 substrate harness — must remain green
cd rust && cargo test -p sim-test --test harness_p11_alpha_agent_renderer -- --nocapture

# Workspace regression
cd rust && cargo test --workspace

# Clippy
cd rust && cargo clippy --workspace --all-targets -- -D warnings

# Godot GDScript parse check (manual, post-pipeline)
/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot \
    --headless --check-only --script scripts/ui/camera_controller.gd
```

Expected:
- harness_p12_alpha_camera_zoom: ≥16 PASS (all assertions listed in §3).
- harness_p11_alpha_agent_renderer: 22 PASS (D1 baseline preserved).
- cargo test --workspace: PASS (no test depends on Camera2D zoom).
- clippy: clean.
- GDScript parse: no errors.

## Section 6: Lane

`--quick` — single new GDScript file, single scene-file modification
(Camera2D node + ext_resource), single new Rust test file. No Rust
crate change, no shader change, no asset change.

Pipeline stages: Visual Verify (generic harness_visual_verify.gd, with
the default 2.0× zoom now in effect for the captured screenshot) +
Evaluator. No planning debate.

## Section 7: 인게임 확인사항 (VLM + Human Visual Verification)

**Expected pipeline visual evidence**:
- Generic `harness_visual_verify.gd` captures a screenshot from a
  windowed Godot run; with the Camera2D default zoom now at 2.0×,
  the captured frame should show the world at twice the previous
  pixel density. Influence overlay, agents (now 32×36 px), and the
  empty grey/black background tiles are all 2× their D1-era size.
- VLM standard checklist (Warmth / Light / Noise / Danger / Spiritual /
  Beauty disc render) remains intact — the channels render
  identically, just zoomed.

**VLM signal for APPROVE / VISUAL_PASS**:
- Generic checklist tokens pass.
- Agent sprites visibly larger than D1-era capture (would require
  pixel-diff to confirm automatically; the static `visual_checklist_rendered.md`
  PASS suffices for the gate).

**VLM signal for WARNING** (acceptable, do not block):
- No mouse-wheel interaction exercised — zoom-in/out smoothness is a
  runtime UX feature not exercisable in a static screenshot. Rule 7
  +8 env-cost applies (D1 / Phase 11-α precedent).

**VLM signal for FAIL** (block and fix):
- Generic checklist regression (Warmth disc absent, overlay corrupted,
  agents missing).
- Scene fails to instantiate (CameraController script parse error,
  ext_resource mis-reference).

**Honest disclosure for the human reviewer**:
- Pipeline pass proves: camera default 2.0×, controller script
  attached, harness assertions on script structure pass, no
  regression to Phase 4-γ / 11-α / D1 invariants.
- Pipeline pass does NOT prove: mouse-wheel zoom feels right,
  tween duration is perceptually correct, zoom range is the right
  product call. Those require the user to run Godot windowed and
  scroll the wheel.
- Pipeline pass DOES bring user-visible delta: when the user launches
  the game post-α, the default view is 2× zoomed; D1 STATE_TINTS are
  finally observable at 32×36 px.
- Phase 12 chain continues with β (Tier 1 terrain + buildings) and γ
  (furniture, conditional). User confirmation expected at full Phase
  12 closure, not per-substage.
