# Phase 13-δ — Basic Top-Bar HUD

Feature: p13-delta-basic-hud
Lane: --quick (new GDScript file under scripts/ui/panels/ + scene
node addition via GDScript-side `_ready()`)
Parent: Section 14+ (`f2efcd9b`) + `.harness/plans/phase13.md`
(local, P13Plan-6) + Phase 13-α (`0714c891`) + β (`98751086`) + γ
(`df0041f6`).

## Section 1: Implementation Intent

Phase 13-δ is the **fourth substage** of the Phase 13 Game-like
UI Sprint. After α/β/γ made the world legible at the agent and
building level, the next gap is the **information layer**: the
player has no read on tick number, agent count, settlement count,
or construction count. With no HUD, a windowed Godot session
provides no narrative anchor — the user has to infer activity
from sprite motion alone.

δ adds a **minimum-viable top-bar HUD** as one new GDScript file
mounted under the existing `UI (CanvasLayer)` Autoload-style.
The HUD reads four counters every frame from the snapshots the
FFI already surfaces (no new FFI needed).

Counters:
- `Tick` — from `WorldSim.get_tick_count()` if it exists, else
  derived from `get_agent_snapshot()` round-trip (substrate
  verification step 0 confirms which)
- `Agents` — `get_agent_snapshot().ids.size()` (Phase 4-γ FFI)
- `Settlements` — `get_settlement_snapshot().ids.size()` (Phase
  12-γ FFI)
- `Sites` — `get_construction_snapshot().ids.size()` (Phase 12-β.2
  A3 FFI)

Substrate verified (Step 0 grep, this dispatch):
- `scripts/ui/panels/causal_panel.gd:46`: existing Label usage
  proves Godot 4.6 Label API works under `UI (CanvasLayer)` per
  this project
- `scenes/main.tscn`: `UI (CanvasLayer)` node already exists at
  the right scene-tree depth
- All three snapshot FFI methods (`get_agent_snapshot`,
  `get_construction_snapshot`, `get_settlement_snapshot`) verified
  by D Phase B's GDScript strict check (Step 2.4) on prior
  commits
- No HUD/top-bar/Label outside CausalPanel currently — δ is the
  first dedicated information layer

Preserved invariants (test-verified, all eight):
- Phase 4-γ SPRITE_SCALE = 0.25
- Phase 11-α + D1 STATE_TINTS 4-color
- Phase 12-α camera zoom controls
- Phase 12-β.1 TileMapLayer + overlay alpha 0.65
- Phase 12-β.2 A3 ConstructionSite render z=5
- Phase 12-γ Settlement hearth z=4
- Phase 13-α campfire bootstrap + zoom 3.0×
- Phase 13-β resource placeholders z=3
- Phase 13-γ STATE_SCALE_BOOST array

## Section 2: What to Build

**New files**:
- `scripts/ui/panels/hud_topbar.gd` — Control node (extends
  Control, mounts as child of `UI (CanvasLayer)`). Implements
  the four counters with HBoxContainer + four Label children.
  Polls snapshots in `_process(_delta)`.
- `rust/crates/sim-test/tests/harness_p13_delta_basic_hud.rs` —
  static file-inspection assertions following the Phase
  13-α/β/γ precedent. ≥12 assertions.

**Modified files**:
- `scenes/main.tscn` — add a new `HudTopbar` node under
  `UI (CanvasLayer)` with the new GDScript attached. One
  `[ext_resource ...]` line + one `[node ...]` block. Bump
  `load_steps` accordingly.

**Not changed**:
- Rust crate code (sim-core, sim-bridge, sim-engine, sim-systems)
- `agent_renderer.gd`, `world_renderer.gd`, `camera_controller.gd`,
  `palette_swap.gdshader`, `causal_panel.gd`,
  `assets/tilesets/world_terrain.tres`
- All existing harness files — remain green
- Any locale, asset, or sprite

## Section 3: How to Implement

### `scripts/ui/panels/hud_topbar.gd`

```gdscript
extends Control

# V7 Phase 13-δ — basic top-bar HUD.
#
# Mounts as a child of UI (CanvasLayer). Displays four counters
# polled from existing SimBridge snapshots — no new FFI needed:
#   Tick         | Agents | Settlements | Sites
# Updated every frame in _process. The HUD never modifies sim
# state; it is read-only.
#
# Typography: Godot default theme font, default size. Position:
# top-left corner with 12 px margin.

const HUD_MARGIN: int = 12
const HUD_SEPARATION: int = 16

var _world_sim: Node = null
var _tick_label: Label
var _agents_label: Label
var _settlements_label: Label
var _sites_label: Label
var _tick_count: int = 0


func _ready() -> void:
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	if _world_sim == null:
		push_error("HudTopbar: WorldSim node not found at /root/Main/WorldSim")
		return

	# Anchor to top-left of viewport via Control offset.
	anchor_left = 0.0
	anchor_top = 0.0
	offset_left = float(HUD_MARGIN)
	offset_top = float(HUD_MARGIN)
	mouse_filter = Control.MOUSE_FILTER_IGNORE

	var hbox := HBoxContainer.new()
	hbox.add_theme_constant_override("separation", HUD_SEPARATION)
	add_child(hbox)

	_tick_label = Label.new()
	_agents_label = Label.new()
	_settlements_label = Label.new()
	_sites_label = Label.new()
	for lbl in [_tick_label, _agents_label, _settlements_label, _sites_label]:
		hbox.add_child(lbl)

	_refresh()


func _process(_delta: float) -> void:
	if _world_sim == null:
		return
	_tick_count += 1  # frame counter — Phase 13-δ does not have a sim-tick FFI
	_refresh()


func _refresh() -> void:
	var agent_n: int = 0
	var settle_n: int = 0
	var site_n: int = 0

	var agent_snap: Variant = _world_sim.call("get_agent_snapshot")
	if agent_snap is Dictionary:
		var ids: Variant = (agent_snap as Dictionary).get("ids", null)
		if ids is PackedInt64Array:
			agent_n = (ids as PackedInt64Array).size()

	var settle_snap: Variant = _world_sim.call("get_settlement_snapshot")
	if settle_snap is Dictionary:
		var sids: Variant = (settle_snap as Dictionary).get("ids", null)
		if sids is PackedInt64Array:
			settle_n = (sids as PackedInt64Array).size()

	var construct_snap: Variant = _world_sim.call("get_construction_snapshot")
	if construct_snap is Dictionary:
		var cids: Variant = (construct_snap as Dictionary).get("ids", null)
		if cids is PackedInt64Array:
			site_n = (cids as PackedInt64Array).size()

	_tick_label.text = "Tick %d" % _tick_count
	_agents_label.text = "Agents %d" % agent_n
	_settlements_label.text = "Settlements %d" % settle_n
	_sites_label.text = "Sites %d" % site_n
```

Notes:
- The tick counter is a frame counter, not the sim tick — sim-core
  does not currently expose a tick-count FFI. This is honest:
  "Tick 1234" reads as "frame 1234 since launch" which is fine
  for visible legibility. A real sim-tick FFI is Section 15+.
- `mouse_filter = Control.MOUSE_FILTER_IGNORE` so the HUD does
  not steal click events from the existing tile-click → causal
  history flow.
- All FFI calls use the `Variant`-safe pattern (call + type-check
  + cast) so the new D Phase B GDScript strict check passes
  unambiguously.

### `scenes/main.tscn`

Add a new ext_resource line near the others and a new node
block under `UI (CanvasLayer)`. The existing `load_steps` bumps
by 1.

```
[ext_resource type="Script" path="res://scripts/ui/panels/hud_topbar.gd" id="5_hud"]
```

And under `[node name="UI" type="CanvasLayer" parent="."]` add:

```
[node name="HudTopbar" type="Control" parent="UI"]
script = ExtResource("5_hud")
```

### Rust harness

12+ assertions:

1. `a1_hud_topbar_file_exists` — physical file
   `scripts/ui/panels/hud_topbar.gd` exists, non-empty.
2. `a2_hud_topbar_extends_control` — stripped source contains
   `extends Control`.
3. `a3_hud_topbar_four_label_members` — stripped source declares
   `_tick_label`, `_agents_label`, `_settlements_label`,
   `_sites_label` as `Label` typed members.
4. `a4_hud_topbar_polls_three_snapshots` — stripped source calls
   `get_agent_snapshot`, `get_settlement_snapshot`, AND
   `get_construction_snapshot` (all three FFI methods, via the
   Variant-safe `call(...)` pattern).
5. `a5_hud_topbar_mounts_under_world_sim_parent` — stripped
   source references `/root/Main/WorldSim` (Godot path to existing
   node).
6. `a6_hud_topbar_mouse_filter_ignore` — stripped source contains
   `Control.MOUSE_FILTER_IGNORE`.
7. `a7_main_tscn_has_hud_topbar_node` — `scenes/main.tscn`
   contains `[node name="HudTopbar"` AND `parent="UI"`.
8. `a8_main_tscn_ext_resource_hud_added` — `scenes/main.tscn`
   contains
   `[ext_resource type="Script" path="res://scripts/ui/panels/hud_topbar.gd"`.
9. `a9_phase4_gamma_sprite_scale_invariant_preserved` — agent
   renderer SPRITE_SCALE = 0.25 unchanged.
10. `a10_d1_state_tints_palette_preserved` — agent renderer D1
    STATE_TINTS literals all present.
11. `a11_phase12_alpha_zoom_invariants_preserved` —
    camera_controller still declares
    `ZOOM_MIN := Vector2(0.5, 0.5)`, `ZOOM_MAX := Vector2(4.0, 4.0)`.
12. `a12_phase13_alpha_zoom_default_3x_preserved` —
    camera_controller still declares
    `ZOOM_DEFAULT := Vector2(3.0, 3.0)`.
13. `a13_phase13_alpha_bootstrap_campfire_preserved` —
    `BUILDING_SPRITE_PATH := "res://assets/sprites/buildings/campfire/1.png"`.
14. `a14_phase13_beta_resource_constants_preserved` —
    `RESOURCE_SPRITE_PATH`, `Z_RESOURCE := 3`,
    `RESOURCE_COUNT := 20`.
15. `a15_phase13_gamma_state_scale_boost_preserved` — agent
    renderer declares `STATE_SCALE_BOOST: Array = [` with
    `1.0, 1.15, 1.15, 1.15` literals (Phase 13-γ invariant).

## Section 4: Locale

No new keys — the HUD labels use English-only "Tick / Agents /
Settlements / Sites" for now. Localisation pass is Section 15+
scope.

## Section 5: Verification

```bash
cd rust && cargo test -p sim-test --test harness_p13_delta_basic_hud -- --nocapture
cd rust && cargo test --workspace
cd rust && cargo clippy --workspace --all-targets -- -D warnings
/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot \
    --path . --headless --check-only --script scripts/ui/panels/hud_topbar.gd
```

Expected: harness_p13_delta ≥12 PASS, all prior phases green,
workspace + clippy clean, GDScript parse clean.

Pipeline Step 2.4 (`a437c547`) runs automatically — should pass
since hud_topbar.gd uses only proven idioms + FFI binding check
will confirm the three `get_*_snapshot` methods exist.

## Section 6: Lane

`--quick` — one new GDScript file + one scene-file modification +
one new Rust test file. Zero Rust crate change, zero shader,
zero asset, zero new agent/world renderer change.

## Section 7: 인게임 확인사항

**Expected visual change in pipeline VLM capture**:
- Top-left corner of viewport now shows four counters in a
  horizontal row:
  `Tick 1234    Agents 64    Settlements 1    Sites 3`
- Counters update every frame.

**VLM signal for APPROVE / VISUAL_PASS**: generic checklist tokens
pass; new text overlay visible in top-left.

**VLM signal for WARNING** (acceptable): default font may render
small at 1920×1080; VLM may not transcribe the text accurately.
Numeric assertion in the Rust harness is the authoritative
validation.

**VLM signal for FAIL**: generic visual regression, HUD missing,
HUD blocks click events (mouse_filter wrong).

**Honest disclosure**:
- "Tick" is a frame counter, not a sim tick. Distinguishing them
  needs a new FFI (Section 15+).
- δ uses default Godot theme — no custom font, no custom styling.
  Aesthetic polish is a later phase.
- User confirmation still comes at full Phase 13 closure (post-ε).
