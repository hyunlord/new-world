# Phase 14-ζ — Zoom-Adaptive Rendering (LOD controller + overview renderer, invariant renderers unchanged)

Feature: p14-zeta-zoom-adaptive
Lane: --quick (GDScript-only — two new sibling nodes + scene
registration; zero Rust crate changes, zero modification to any
existing renderer/panel/controller)
Parent: `.harness/plans/phase14.md` (P14Plan-8 — zoom-level
adaptive rendering, Songs of Syx reference) + Phase 14-ε
(`bf3a7493`) chain successor. **Final substage of Phase 14.**

## Section 1: Implementation Intent

Phase 14-ζ is the **sixth and final substage** of the Phase 14
RimWorld-like Visual Overhaul Sprint. Reference game: Songs of Syx.

User's original mandate (carried forward from project-open):
> "agent(인물)가 뭐하는건지 그냥 아무이유없이 돌아다니고"
> (agents look like they're wandering for no reason)

User's Phase 14-ζ mandate:
> "Songs of Syx 처럼 줌별 적응" — close zoom shows individual
> detail (sprite + head-icon + trail), far zoom collapses to a
> clustered/overview representation.

After Phase 14-α (4-bucket role colour), β (resource/village
variety), γ (click inspector), δ (HUD status panel), ε (activity
trails), the remaining gap is **information density at every
zoom level is identical**. At the Phase 13-α default zoom (3.0×)
the scene reads well, but when the user zooms out to the overview
(Phase 12-α allows down to 0.5×) the same per-agent trails and
detail render as map-wide noise, and there is no overview
affordance telling the player *where the populated regions are*.

ζ adds a **zoom level-of-detail (LOD) controller** that reads the
live `Camera2D.zoom` (tweened by Phase 12-α camera_controller) and
toggles the *whole-node* `visible` flag of decorative sibling
renderers across three tiers, plus a new **settlement overview
renderer** that draws translucent region discs at settlement
centroids when zoomed out.

**Why the Option-B sibling pattern (NOT renderer refactor)**:

The Phase 14-δ disaster (Generator rewrote `hud_topbar.gd`,
breaking Phase 13-δ A5 Variant-safe guards) and the Phase 14-ε
success (separate `activity_trail_renderer.gd` sibling,
`agent_renderer.gd` untouched) established the project rule:
**when an existing renderer is invariant-heavy, orchestrate it
from a sibling node — never modify its internals.** ζ follows
this exactly. The LOD controller toggles sibling `visible` flags;
it does NOT reach inside any renderer.

**Locked scope (Conservative — verified Step 0 grep 2026-05-29)**:

A. **Zoom-LOD controller** = `scripts/ui/zoom_lod_controller.gd`
   (`extends Node`) reads `Camera2D.zoom.x` each frame, maps it
   to one of three tiers, and on tier change toggles the
   `visible` flag of two decorative sibling renderers.
   - Tier thresholds over `Camera2D.zoom.x`:
     - **CLOSE**  : zoom ≥ 2.0 (full per-agent detail)
     - **MEDIUM** : 1.0 ≤ zoom < 2.0 (transitional)
     - **FAR**    : zoom < 1.0 (overview)
   - Phase 13-α default zoom (3.0×) lands in CLOSE, so the
     out-of-the-box view is full detail; the user must zoom out
     past 2.0 / 1.0 to cross into MEDIUM / FAR.
   - Behaviour table (the only two clean whole-node toggles
     available — see "Explicit non-scope" for why head-icons /
     resources are deferred):

     | Tier   | zoom      | ActivityTrailRenderer.visible | SettlementOverviewRenderer.visible |
     |--------|-----------|-------------------------------|-------------------------------------|
     | CLOSE  | ≥ 2.0     | true                          | false                               |
     | MEDIUM | 1.0–2.0   | true                          | true                                |
     | FAR    | < 1.0     | false                         | true                                |

   - Trails hidden only at FAR (where the polyline mesh reads as
     map-wide noise). Overview discs shown at MEDIUM + FAR (as
     the player pulls back), hidden at CLOSE (individual sprites
     are legible). All three tiers are visually distinct.

B. **Settlement overview renderer** =
   `scripts/ui/settlement_overview_renderer.gd` (`extends
   Node2D`) draws one translucent filled circle per Settlement at
   its substrate-derived centroid, with radius scaling by
   `member_count`. Uses the EXISTING `get_settlement_snapshot()`
   FFI (Phase 12-γ) — `centroid_xs` / `centroid_ys` /
   `member_counts` are already in the dictionary; **no new
   backend surface**. Starts hidden; the LOD controller flips its
   `visible`. Read-only snapshot consumer; never mutates sim
   state. Uses the same `SPRITE_ORIGIN + TILE_SIZE` coordinate
   basis as `world_renderer._update_settlement_furniture` so the
   discs centre exactly on the Phase 12-γ hearth placeholders.

**Explicit non-scope** (deferred / not added — honest disclosure):
- **Head-icon hide at FAR** — head-icons are rendered by
  `agent_renderer.gd` via the Phase 4-γ MultiMesh driver (a
  second MultiMeshInstance2D / per-agent Sprite2D at
  `ICON_OFFSET_PX`). Toggling them requires modifying
  invariant-heavy `agent_renderer.gd` internals (Phase 4-γ + 8-δ
  + 9-δ + 11-α + 13-γ + 14-α). That violates the Option-B
  precedent and is deferred to a future phase that explicitly
  authorises that surface. The trail-hide + overview-show pair
  already delivers a clearly distinct overview tier.
- **Resource-sprite simplification at FAR** — resource sprites
  are `add_child`'d directly onto `WorldRenderer` (mixed with the
  terrain TileMapLayer + building/fixture sprites), so they
  cannot be toggled by whole-node `visible` without hiding
  terrain too. Per-sprite toggling requires storing sprite refs +
  modifying `world_renderer.gd` (Phase 14-β invariants). Deferred
  for the same reason; Section 16+ candidate.
- **True polygon settlement boundaries** — the substrate has no
  `Settlement.position` or boundary polygon; the collector
  derives only a centroid (mean of member positions) +
  member_count. "Boundary" is therefore approximated as a
  member-count-scaled disc. Honest approximation, not a hull.
- **camera_controller.gd zoom_changed signal** — the controller
  polls `Camera2D.zoom` in `_process` instead. Adding a signal
  would modify Phase 12-α / 13-α camera_controller.gd (invariant
  file). Polling is the zero-modification path and tracks the
  smooth tween, not just the wheel-notch target.
- **Locale keys** — pure visual layer, no text.

Substrate verified (Step 0 grep, 2026-05-29):
- `scripts/ui/camera_controller.gd` (64 lines): `ZOOM_MIN =
  Vector2(0.5, 0.5)`, `ZOOM_MAX = Vector2(4.0, 4.0)`,
  `ZOOM_DEFAULT = Vector2(3.0, 3.0)`, `ZOOM_FACTOR = 1.25`. The
  live `zoom` property is tweened toward `_target_zoom`. NO
  `zoom_changed` signal.
- `get_settlement_snapshot()` (Phase 12-γ
  `collect_settlement_snapshot` + `settlement_rows_to_dict`)
  returns a Dictionary with keys `ids` (PackedInt64Array),
  `settlement_ids` (PackedInt32Array), `centroid_xs`
  (PackedInt32Array), `centroid_ys` (PackedInt32Array),
  `member_counts` (PackedInt32Array). All parallel arrays of
  equal length.
- `world_renderer.gd`: `TILE_SIZE = 16`, `SPRITE_ORIGIN_X = 448`,
  `SPRITE_ORIGIN_Y = 28`, `GRID_W = GRID_H = 64`. The
  `_update_settlement_furniture` hearth sprite is positioned at
  `SPRITE_ORIGIN_X + centroid_x * TILE_SIZE + TILE_SIZE/2.0`.
- `activity_trail_renderer.gd` (Phase 14-ε, 124 lines): node
  name `ActivityTrailRenderer`, sibling of `Main`. `visible`
  defaults true; `_process` runs regardless of visibility (so
  hiding it via `visible=false` cleanly stops `_draw` while
  keeping the history ring buffer fresh — no modification needed).
- `scenes/main.tscn`: `load_steps=9`, 8 ext_resources. `Main`
  Node2D has children WorldSim / WorldRenderer / AgentRenderer /
  ActivityTrailRenderer / Camera2D / UI. UI has CausalPanel /
  HudTopbar / AgentInspectorPanel / HudStatusPanel.

Preserved invariants (≥20; ALL renderers/panels/controllers
read-only — ζ adds only new sibling nodes):
- **Phase 12-α** camera_controller.gd ZOOM_MIN = (0.5,0.5),
  ZOOM_MAX = (4.0,4.0) (untouched).
- **Phase 13-α** camera_controller.gd ZOOM_DEFAULT = (3.0,3.0)
  (untouched).
- **Phase 4-γ** agent_renderer.gd MultiMesh + SPRITE_SCALE = 0.25.
- **Phase 8-δ / 9-δ / 11-α / 13-γ / 14-α** agent_renderer.gd
  recall/combat/state cues + STATE_TINTS + STATE_SCALE_BOOST +
  ROLE_BUCKET_COUNT + ICON_OFFSET_PX.
- **Phase 14-β** world_renderer.gd RESOURCE_TYPE_PATHS +
  VILLAGE_FIXTURE_* + RESOURCE_COUNT=20 + RESOURCE_SEED=88675123.
- **Phase 14-ε** activity_trail_renderer.gd TRAIL_LENGTH=16 +
  TRAIL_WIDTH + TRAIL_ALPHA + Z_TRAIL + 3 TRAIL_COLOR_* (ζ must
  NOT modify this file — only toggle its `visible` externally).
- **Phase 13-δ** hud_topbar.gd A4/A5/A6.
- **Phase 14-γ** AgentInspectorPanel + get_agent_detail.
- **Phase 14-δ** hud_status_panel.gd TICKS_PER_DAY +
  RESOURCE_TYPES_COUNT + RESOURCE_LABELS.

## Section 2: What to Build

**New files**:
- `scripts/ui/zoom_lod_controller.gd` — `extends Node`. Reads
  `Camera2D.zoom.x` each `_process` frame, maps to a 3-tier
  enum, toggles two decorative sibling renderers' `visible`
  flags on tier change. Mounts as a sibling under `Main`.
- `scripts/ui/settlement_overview_renderer.gd` — `extends
  Node2D`. Reads `get_settlement_snapshot()` each frame (only
  when visible), draws member-count-scaled translucent discs at
  settlement centroids via `_draw()`/`draw_circle()`. Mounts as a
  sibling under `Main`. `z_index = 1` (above terrain z=0, below
  trails z=2 / agents). Starts `visible = false`.
- `rust/crates/sim-test/tests/harness_p14_zeta_zoom_adaptive.rs`
  — static file-inspection assertions following Phase 14-ε
  precedent (project_root + strip_gd_comments + find_decl_rhss +
  find_func_body helpers). ≥18 assertions.

**Modified files**:
- `scenes/main.tscn` — add `SettlementOverviewRenderer` (Node2D)
  and `ZoomLodController` (Node) as children of `Main`, placed
  AFTER the existing `ActivityTrailRenderer` block and BEFORE the
  `Camera2D` block. Add two new `ExtResource` entries
  (`id="9_overview"` for settlement_overview_renderer.gd,
  `id="10_zoomlod"` for zoom_lod_controller.gd). `load_steps`
  9 → 11. Do NOT modify any existing node or ext_resource line.

**Not changed (CRITICAL — hard rules)**:
- `scripts/ui/camera_controller.gd` — **absolutely untouched**.
  Phase 12-α ZOOM_MIN/ZOOM_MAX + Phase 13-α ZOOM_DEFAULT depend
  on it. The controller READS `Camera2D.zoom` via
  `get_node_or_null` — it does NOT modify camera_controller.gd.
- `scripts/ui/agent_renderer.gd` — untouched (Phase 4-γ + 8-δ +
  9-δ + 11-α + 13-γ + 14-α). Head-icon toggle is explicit
  non-scope.
- `scripts/ui/world_renderer.gd` — untouched (Phase 12-β +
  13-α/β/ε + 14-β). Resource simplification is explicit
  non-scope.
- `scripts/ui/activity_trail_renderer.gd` — untouched (Phase
  14-ε). The controller toggles its `visible` EXTERNALLY; the
  file's own source is not edited.
- `scripts/ui/panels/hud_topbar.gd` — untouched (Phase 13-δ).
- `scripts/ui/panels/agent_inspector_panel.gd` — untouched
  (Phase 14-γ).
- `scripts/ui/panels/hud_status_panel.gd` — untouched (Phase
  14-δ).
- All Rust crate code (zero `.rs` change except the new harness
  test file under `rust/crates/sim-test/tests/`).
- All shaders, assets, locales.

**Cross-phase regression-guard reference files (READ-ONLY in
plan — the harness MAY reference these as subjects to verify
prior invariants; the plan must NOT propose modifications to
them)**:
- `scripts/ui/camera_controller.gd` (Phase 12-α + 13-α).
- `scripts/ui/agent_renderer.gd` (Phase 4-γ + 8-δ + 9-δ + 11-α +
  13-γ + 14-α). Not under any `renderers/` subdir.
- `scripts/ui/world_renderer.gd` (Phase 12-β + 13-α/β/ε + 14-β).
  Not under any `renderers/` subdir.
- `scripts/ui/activity_trail_renderer.gd` (Phase 14-ε). Not
  under any `renderers/` subdir.
- `scripts/ui/panels/hud_topbar.gd` (Phase 13-δ A4/A5/A6).
- `scripts/ui/panels/hud_status_panel.gd` (Phase 14-δ).

Use these EXACT path strings when authoring the plan. ALL
`scripts/ui/*.gd` renderers are FLAT in `scripts/ui/` — there is
NO `scripts/ui/renderers/` subdirectory. Do NOT invent one.

**Forbidden plan assertions**:
- Do NOT propose modifying any existing `.gd` file. ζ adds only
  two new sibling `.gd` files + a scene edit + a harness test.
- Do NOT invent a `camera_controller.gd` `zoom_changed` signal
  — the controller polls `Camera2D.zoom`. Adding a signal is
  explicit non-scope.
- Do NOT add head-icon toggling or resource-sprite toggling —
  both are explicit non-scope (would require invariant-file
  modification).
- Do NOT add locale requirements — pure visual layer, no text.
- Do NOT propose a new FFI collector or `#[func]` method — ζ
  uses the EXISTING `get_settlement_snapshot()`. Any `.rs`
  change outside `rust/crates/sim-test/tests/` is out of scope.

## Section 3: How to Implement

### `scripts/ui/zoom_lod_controller.gd` — new file

```gdscript
extends Node

# V7 Phase 14-ζ — zoom level-of-detail (LOD) controller.
#
# Songs-of-Syx-style zoom adaptation: as the camera zooms out,
# decorative per-agent detail fades and an overview affordance
# fades in. This node owns the zoom→tier mapping and toggles the
# *whole-node* `visible` flag of two decorative sibling
# renderers. It modifies NO invariant file — the Phase 14-δ/ε
# Option-B precedent: orchestrate siblings, never refactor
# invariant-heavy renderers.
#
# Honest disclosure (verified Step 0 grep, 2026-05-29):
#   - Two of the four zoom behaviours sketched in the Phase 14-ζ
#     brief are implemented as clean whole-node toggles (trail
#     hide at FAR, settlement overview show at MEDIUM/FAR). The
#     other two — head-icon hide (agent_renderer.gd Phase 4-γ
#     MultiMesh + ICON_OFFSET_PX) and resource-sprite
#     simplification (world_renderer.gd Phase 14-β) — would
#     require modifying invariant-heavy renderers and are
#     deferred to a future phase that explicitly authorises that
#     surface.
#   - Tier thresholds key off the LIVE Camera2D.zoom (tweened by
#     camera_controller Phase 12-α), so the toggle tracks the
#     smooth zoom, not just the wheel-notch target.

enum Tier { FAR, MEDIUM, CLOSE }

# Tier thresholds over Camera2D.zoom.x (Phase 12-α range 0.5–4.0,
# Phase 13-α default 3.0). zoom ≥ CLOSE_MIN → CLOSE; zoom <
# FAR_MAX → FAR; in between → MEDIUM. Chosen so the Phase 13-α
# default (3.0) lands in CLOSE (full detail) and the user must
# zoom out past 1.0 to reach the overview affordance.
const ZOOM_FAR_MAX: float = 1.0
const ZOOM_CLOSE_MIN: float = 2.0

var _camera: Camera2D = null
var _trail_renderer: Node2D = null
var _overview_renderer: Node2D = null
var _current_tier: int = Tier.CLOSE


func _ready() -> void:
	_camera = get_node_or_null("/root/Main/Camera2D") as Camera2D
	_trail_renderer = get_node_or_null("/root/Main/ActivityTrailRenderer") as Node2D
	_overview_renderer = get_node_or_null("/root/Main/SettlementOverviewRenderer") as Node2D
	# Apply the initial tier once so visibility matches the Phase
	# 13-α default zoom (3.0 → CLOSE) before the first change.
	_apply_tier(Tier.CLOSE)


func _process(_delta: float) -> void:
	if _camera == null:
		return
	var z: float = _camera.zoom.x
	var tier: int = _tier_for_zoom(z)
	if tier != _current_tier:
		_apply_tier(tier)


func _tier_for_zoom(z: float) -> int:
	if z < ZOOM_FAR_MAX:
		return Tier.FAR
	if z >= ZOOM_CLOSE_MIN:
		return Tier.CLOSE
	return Tier.MEDIUM


func _apply_tier(tier: int) -> void:
	_current_tier = tier
	# Trails: shown at CLOSE + MEDIUM, hidden at FAR (overview)
	# where the polyline mesh would read as map-wide noise.
	if _trail_renderer != null:
		_trail_renderer.visible = (tier != Tier.FAR)
	# Settlement overview region discs: shown at MEDIUM + FAR (as
	# the player pulls back), hidden at CLOSE where individual
	# sprites are legible.
	if _overview_renderer != null:
		_overview_renderer.visible = (tier != Tier.CLOSE)
```

### `scripts/ui/settlement_overview_renderer.gd` — new file

```gdscript
extends Node2D

# V7 Phase 14-ζ — settlement overview region renderer.
#
# Overview-zoom affordance: when the player zooms out (Tier.MEDIUM
# / FAR, toggled by zoom_lod_controller.gd), this node draws one
# translucent filled circle per Settlement at its
# substrate-derived centroid, radius scaling with member_count.
# At overview zoom — where individual 16×18 px agent sprites
# collapse below the perceptual threshold — these discs make the
# populated regions legible at a glance.
#
# Honest disclosure (verified Step 0 grep, 2026-05-29):
#   - Uses the EXISTING get_settlement_snapshot() FFI (Phase
#     12-γ); no new backend surface. The snapshot exposes
#     centroid_xs / centroid_ys / member_counts. There is no true
#     polygon boundary in the substrate, so "boundary" is
#     approximated as a member-count-scaled disc centred on the
#     centroid.
#   - Starts hidden; zoom_lod_controller flips `visible`. It never
#     mutates sim state (read-only snapshot consumer). Same
#     SPRITE_ORIGIN + TILE_SIZE basis as
#     world_renderer._update_settlement_furniture so the discs
#     centre on the Phase 12-γ hearth placeholders.

const TILE_SIZE := 16
const SPRITE_ORIGIN_X := 448
const SPRITE_ORIGIN_Y := 28
const Z_OVERVIEW := 1                       # above terrain (0), below trails (2)/agents
const OVERVIEW_ALPHA := 0.28
const OVERVIEW_COLOR: Color = Color(0.35, 0.75, 1.0, OVERVIEW_ALPHA)  # soft blue region
const BASE_RADIUS_PX := 24.0                # radius for a 1-member settlement
const PER_MEMBER_RADIUS_PX := 6.0           # additional radius per resolvable member
const MAX_RADIUS_PX := 160.0                # clamp so one settlement can't fill the map

var _world_sim: Node = null
# Cached per-frame parallel arrays consumed by _draw.
var _centroids: PackedVector2Array = PackedVector2Array()
var _radii: PackedFloat32Array = PackedFloat32Array()


func _ready() -> void:
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	z_index = Z_OVERVIEW
	visible = false   # zoom_lod_controller shows this only at MEDIUM/FAR


func _process(_delta: float) -> void:
	# Skip all snapshot work while hidden (CLOSE zoom) — overview off.
	if not visible:
		return
	if _world_sim == null:
		return
	var snap: Variant = _world_sim.call("get_settlement_snapshot")
	if not (snap is Dictionary):
		return
	var snap_dict: Dictionary = snap
	var xs: Variant = snap_dict.get("centroid_xs", null)
	var ys: Variant = snap_dict.get("centroid_ys", null)
	var members: Variant = snap_dict.get("member_counts", null)
	if not (xs is PackedInt32Array and ys is PackedInt32Array \
			and members is PackedInt32Array):
		return
	var xs_arr: PackedInt32Array = xs
	var ys_arr: PackedInt32Array = ys
	var members_arr: PackedInt32Array = members
	var n: int = xs_arr.size()
	if ys_arr.size() != n or members_arr.size() != n:
		return
	var new_centroids: PackedVector2Array = PackedVector2Array()
	var new_radii: PackedFloat32Array = PackedFloat32Array()
	for i in n:
		var px: float = float(SPRITE_ORIGIN_X + xs_arr[i] * TILE_SIZE) + float(TILE_SIZE) / 2.0
		var py: float = float(SPRITE_ORIGIN_Y + ys_arr[i] * TILE_SIZE) + float(TILE_SIZE) / 2.0
		new_centroids.append(Vector2(px, py))
		var mc: int = int(members_arr[i])
		var r: float = clampf(BASE_RADIUS_PX + PER_MEMBER_RADIUS_PX * float(mc), \
				BASE_RADIUS_PX, MAX_RADIUS_PX)
		new_radii.append(r)
	_centroids = new_centroids
	_radii = new_radii
	queue_redraw()


func _draw() -> void:
	for i in _centroids.size():
		draw_circle(_centroids[i], _radii[i], OVERVIEW_COLOR)
```

### `scenes/main.tscn` — append two sibling nodes

After the existing `[ext_resource ... id="8_trails"]` line
(Phase 14-ε), add:

```
[ext_resource type="Script" path="res://scripts/ui/settlement_overview_renderer.gd" id="9_overview"]
[ext_resource type="Script" path="res://scripts/ui/zoom_lod_controller.gd" id="10_zoomlod"]
```

After the existing `[node name="ActivityTrailRenderer" ...]`
block and BEFORE the `[node name="Camera2D" ...]` block, add:

```
[node name="SettlementOverviewRenderer" type="Node2D" parent="."]
script = ExtResource("9_overview")

[node name="ZoomLodController" type="Node" parent="."]
script = ExtResource("10_zoomlod")
```

Update header: `[gd_scene load_steps=9 ...]` → `load_steps=11`.

Do NOT modify any other line in main.tscn. Existing WorldSim /
WorldRenderer / AgentRenderer / ActivityTrailRenderer / Camera2D
/ UI/* nodes stay verbatim.

### Rust harness

`rust/crates/sim-test/tests/harness_p14_zeta_zoom_adaptive.rs`

Follow Phase 14-ε structure (project_root + strip_gd_comments +
find_decl_rhss + unique_decl_rhs + parse_int_rhs + parse_float_rhs
+ find_func_body + no_ws helpers). Strict file-inspection
assertions only.

zoom_lod_controller.gd:
1. `a1_zoom_lod_controller_exists_extends_node` — file exists,
   first non-comment non-blank line == `extends Node`.
2. `a2_zoom_far_max_equals_one` — `ZOOM_FAR_MAX` RHS parses 1.0.
3. `a3_zoom_close_min_equals_two` — `ZOOM_CLOSE_MIN` RHS parses
   2.0.
4. `a4_tier_enum_has_three_tiers` — file contains `enum Tier`
   AND `FAR` AND `MEDIUM` AND `CLOSE`.
5. `a5_tier_for_zoom_uses_both_thresholds` — `_tier_for_zoom`
   func body contains `ZOOM_FAR_MAX` AND `ZOOM_CLOSE_MIN` AND
   `Tier.FAR` AND `Tier.CLOSE`.
6. `a6_apply_tier_toggles_both_renderers` — `_apply_tier` func
   body contains `_trail_renderer` AND `_overview_renderer` AND
   `.visible` (whitespace-collapsed `_trail_renderer.visible` AND
   `_overview_renderer.visible`).
7. `a7_resolves_three_sibling_node_paths` — file contains the
   three literals `/root/Main/Camera2D`,
   `/root/Main/ActivityTrailRenderer`,
   `/root/Main/SettlementOverviewRenderer`.
8. `a8_process_reads_live_camera_zoom` — `_process` func body
   contains `_camera.zoom` (whitespace-collapsed; proves it
   reads the live tweened zoom, not a target).

settlement_overview_renderer.gd:
9. `a9_settlement_overview_exists_extends_node2d` — file exists,
   first non-comment non-blank line == `extends Node2D`.
10. `a10_z_overview_equals_one` — `Z_OVERVIEW` RHS parses 1.
11. `a11_overview_alpha_equals_0_28` — `OVERVIEW_ALPHA` RHS
    parses 0.28.
12. `a12_reads_settlement_snapshot_only` — file contains
    `get_settlement_snapshot` AND does NOT contain any of
    `get_agent_snapshot`, `get_construction_snapshot`,
    `get_agent_detail`, `get_tile_detail`,
    `get_relationship_snapshot`, `get_influence_overlay`,
    `get_event_chain`, `get_tile_causal_history` (single-snapshot
    scope).
13. `a13_starts_hidden_and_skips_when_hidden` — `_ready` body
    contains `visible = false` (ws-collapsed `visible=false`) AND
    `_process` body contains `not visible` (the early-return
    perf guard).
14. `a14_draw_circle_with_member_count_radius` — `_draw` body
    contains `draw_circle`; file contains `member_counts` AND
    `BASE_RADIUS_PX` (radius scales by member count).

main.tscn:
15. `a15_main_tscn_load_steps_equals_11` — `load_steps=11`
    (ws around `=` tolerated).
16. `a16_main_tscn_registers_both_new_nodes` — main.tscn
    references `zoom_lod_controller.gd` AND
    `settlement_overview_renderer.gd` AND has a
    `[node name="ZoomLodController" type="Node"` line AND a
    `[node name="SettlementOverviewRenderer" type="Node2D"` line.

Regression guards (Type D — prior-phase invariants intact):
17. `a17_phase12a_13a_camera_controller_zoom_invariants_intact`
    — `scripts/ui/camera_controller.gd` contains `ZOOM_MIN` AND
    `ZOOM_MAX` AND `ZOOM_DEFAULT` AND the whitespace-collapsed
    literals `Vector2(0.5,0.5)` AND `Vector2(4.0,4.0)` AND
    `Vector2(3.0,3.0)`.
18. `a18_phase14a_agent_renderer_invariants_intact` —
    `scripts/ui/agent_renderer.gd` contains `ROLE_BUCKET_COUNT`
    AND `ICON_OFFSET_PX` AND `STATE_SCALE_BOOST` AND `STATE_TINTS`
    AND `SPRITE_SCALE`.
19. `a19_phase14b_world_renderer_invariants_intact` —
    `scripts/ui/world_renderer.gd` contains `RESOURCE_TYPE_PATHS`
    AND `VILLAGE_FIXTURE_` AND `RESOURCE_COUNT := 20` (RHS 20)
    AND `RESOURCE_SEED := 88675123` (RHS 88675123).
20. `a20_phase14e_activity_trail_renderer_intact` —
    `scripts/ui/activity_trail_renderer.gd` contains
    `TRAIL_LENGTH` AND `TRAIL_WIDTH` AND `TRAIL_ALPHA` AND
    `Z_TRAIL` AND `TRAIL_COLOR_SEEKING` AND
    `TRAIL_COLOR_CONSUMING_AGENT` AND `TRAIL_COLOR_CONSUMING_OTHER`
    (ζ must NOT have edited the ε file).
21. `a21_phase14d_hud_status_panel_intact` —
    `scripts/ui/panels/hud_status_panel.gd` contains
    `TICKS_PER_DAY` AND `RESOURCE_TYPES_COUNT` AND
    `RESOURCE_LABELS`.
22. `a22_phase13d_hud_topbar_intact` —
    `scripts/ui/panels/hud_topbar.gd` contains
    `get_agent_snapshot` AND `get_settlement_snapshot` AND
    `get_construction_snapshot` AND `is Dictionary` AND
    `is PackedInt64Array` AND `MOUSE_FILTER_IGNORE`.

## Section 4: Locale

No new locale keys. The LOD controller toggles visibility; the
overview renderer draws coloured discs. No text. Matches Phase
14-γ / 14-δ / 14-ε precedent.

## Section 5: Verification

```bash
cd rust && cargo test -p sim-test --test harness_p14_zeta_zoom_adaptive -- --nocapture
cd rust && cargo test -p sim-test --test harness_p14_epsilon_activity_trails -- --nocapture
cd rust && cargo test --workspace
cd rust && cargo clippy --workspace --all-targets -- -D warnings
/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot \
    --path . --headless --check-only --script scripts/ui/zoom_lod_controller.gd
/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot \
    --path . --headless --check-only --script scripts/ui/settlement_overview_renderer.gd
```

Expected: harness_p14_zeta ≥18 PASS, Phase 14-ε + all prior
phases green, workspace + clippy clean, GDScript parse clean.

## Section 6: Lane

`--quick` — two new GDScript sibling files + scene registration
+ one new sim-test harness file. Zero Rust crate change (the new
`.rs` is a test under `rust/crates/sim-test/tests/`, not a crate
modification). Zero FFI extension. Zero
camera_controller.gd / agent_renderer.gd / world_renderer.gd /
activity_trail_renderer.gd / hud_topbar.gd /
agent_inspector_panel.gd / hud_status_panel.gd modification.
Zero asset / shader / locale change.

## Section 7: 인게임 확인사항

**Expected visual change in pipeline VLM capture**:
- At the Phase 13-α default zoom (3.0× → CLOSE tier), the scene
  is IDENTICAL to Phase 14-ε: trails visible, no overview discs.
  The VLM capture (single screenshot at default zoom) should
  therefore look the same as the Phase 14-ε baseline.
- The zoom adaptation only manifests when the user zooms out
  past 2.0× (MEDIUM: overview discs fade in) and 1.0× (FAR:
  trails fade out, overview discs remain). The headless VLM
  capture does NOT exercise mouse-wheel zoom, so the tier
  transition is NOT visible in the automated screenshot.

**VLM signal for APPROVE / VISUAL_PASS**: scene identical to
Phase 14-ε baseline at default zoom (full detail, no overview
discs). The zoom-adaptive behaviour is verified by the Rust
harness's strict file checks (threshold constants, tier logic,
toggle wiring), NOT by the static screenshot — the VLM cannot
drive the camera. This is the authoritative validation per
CLAUDE.md "VLM Visual Verification — Known Limitation".

**VLM signal for WARNING** (acceptable): VLM may not articulate
any change from Phase 14-ε (because at default zoom there is
none). This is intentional — the windowed Godot run with
mouse-wheel zoom is the perceptual gate.

**VLM signal for FAIL**: scene crash, agents disappear, layout
broken, overview discs visible at default zoom (would indicate
the controller failed to apply the initial CLOSE tier).

**Honest disclosure**:
- ζ implements **two of the four** zoom behaviours from the
  brief as clean Option-B whole-node toggles: trail-hide at FAR
  and settlement-overview-show at MEDIUM/FAR. The other two —
  head-icon hide and resource-sprite simplification — require
  modifying invariant-heavy `agent_renderer.gd` (Phase 4-γ
  MultiMesh) and `world_renderer.gd` (Phase 14-β), which
  violates the Option-B precedent established by the Phase 14-δ
  rewrite disaster. They are deferred to a future phase that
  explicitly authorises invariant-renderer modification (or a
  refactor that extracts those elements into toggleable
  sub-nodes — Section 16+).
- The MEDIUM tier is genuinely distinct from CLOSE (overview
  discs fade in) and from FAR (trails still visible). All three
  tiers produce distinct visible output despite only two
  toggleable elements.
- "Settlement boundary" is a **member-count-scaled disc** at the
  substrate-derived centroid, NOT a true polygon hull. The
  substrate has no boundary geometry; the centroid + member_count
  are the only spatial signals the existing FFI exposes. This is
  an honest approximation appropriate to overview zoom.
- The controller **polls** `Camera2D.zoom` each frame rather than
  subscribing to a signal, because camera_controller.gd has no
  `zoom_changed` signal and adding one would modify a Phase
  12-α / 13-α invariant file. Polling also correctly tracks the
  smooth tween, not just the wheel-notch target.
- All six prior Phase 14 substages + Phase 12-α/13-α camera
  invariants are preserved by construction (Option-B pattern:
  new sibling nodes only, zero modification to existing
  renderers/panels/controllers).

### ★ Phase 14 sprint completion

ζ is the final substage. On APPROVE, Phase 14 (α→β→γ→δ→ε→ζ) is
complete and the user performs the windowed-Godot perceptual
review of the cumulative six-stage effect (role colour →
resource/village variety → click inspector → HUD status panel →
activity trails → zoom adaptation).
