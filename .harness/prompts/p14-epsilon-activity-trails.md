# Phase 14-ε — Activity Trails (separate trail renderer, agent_renderer.gd unchanged)

Feature: p14-epsilon-activity-trails
Lane: --quick (GDScript-only — one new Node2D + scene
registration; zero Rust changes, zero agent_renderer.gd changes)
Parent: `.harness/plans/phase14.md` (P14Plan-7 — option B:
separate trail renderer, NOT agent_renderer.gd refactor) +
Phase 14-δ (`3c3cc333`) chain successor.

## Section 1: Implementation Intent

Phase 14-ε is the **fifth substage** of the Phase 14 RimWorld-like
Visual Overhaul Sprint. Reference games: Stronghold + Factorio.

User's original mandate (carried forward from project-open):
> "agent(인물)가 뭐하는건지 그냥 아무이유없이 돌아다니고"
> (agents look like they're wandering for no reason)

After Phase 14-α (4-bucket role colour), β (resource/village
variety), γ (click inspector), δ (HUD status panel), the
sense-of-purpose gap remains: an agent's movement shows the
sprite changing position frame-to-frame, but the **history** of
where they have just been is invisible. In Factorio, belt items
trace a visible path; in Stronghold, peasants' walking history
is implied by stepping animation. Static screenshot of WorldSim
shows none of that — every frame is a snapshot, every agent is
a dot.

ε surfaces **backward movement history** as a faint polyline
trailing each non-Idle agent. This is what the reference games
actually use for "activity routes": history of recent positions,
NOT forward intent. Forward intent (target tile coordinates)
would require backend FFI extension because:

- Step 0 grep verified: `AgentSnapshotRow` exposes
  `(entity_bits, x, y, state_tag, agent_id)`. NO target_x /
  target_y / target_position fields.
- `AgentDetailRow` (Phase 14-γ) exposes `target_kind` (0=None /
  1=Food / 2=Water / 3=Sleep / 4=ConstructionSite / 5=Agent)
  but NO target coordinates.
- Backend `SimResources` stores `food_tiles`, `water_tiles`,
  `sleep_tiles` as `HashMap<(u32, u32), u8>`. The
  AgentDecisionSystem picks the nearest matching tile
  internally, but the chosen tile is not exposed per-agent on
  the FFI surface.
- Surfacing target coordinates would require a new
  collector + `#[func]` method = **--full lane scope**.
- ε stays --quick by deriving trails from snapshot-history
  observations on the GDScript side.

**Locked scope (Conservative)**:

A. **Backward movement trail per agent** = polyline from
   recent positions
   - Each frame the trail renderer reads the current agent
     snapshot, stores `(agent_id → recent_positions)` history
     ring buffer (length TRAIL_LENGTH = 16 positions = 16/60s
     ≈ 0.27s of motion at ~60fps).
   - For each agent whose `state_tag != 0` (non-Idle), the
     renderer draws a polyline through its recent positions
     using `_draw()` via `draw_polyline()`.
   - Idle agents (`state_tag == 0`) do NOT get a trail — they
     are physically stationary or jittering on Brownian motion,
     a trail there would be noise.
   - Trail colour keyed on `state_tag`:
     - state_tag = 1 (Seeking) → orange-yellow
     - state_tag = 2 (Consuming Agent / Socialising) → pink
     - state_tag = 3 (Consuming other) → green
   - Alpha 0.4 (faint, doesn't dominate the scene).
   - Width 2 px (legible at zoom 3.0× Phase 13-α default).

B. **Activity cues already exist** — preserved, not extended
   - Phase 13-γ `STATE_SCALE_BOOST` (4-entry [1.0, 1.15, 1.15,
     1.15]) already enlarges non-Idle agents by 15%.
   - Phase 11-α + D1 `STATE_TINTS` already tints non-Idle
     agents by state.
   - ε ADDS trails as a third visual cue — does NOT modify the
     existing two.

**Explicit non-scope** (deferred / not added):
- Forward intent visualisation (target coordinates) — would
  need FFI extension, --full lane scope. Phase 14-ζ candidate.
- Gather/build flash effect — would need ConstructionSite or
  food-tile snapshot delta on a per-tile basis, complex
  attribution. Section 16+ candidate.
- Per-agent trail length scaling by speed — agents in WorldSim
  move 1 tile per tick at most, speed scaling has no signal.
- Locale keys — pure visual layer, no text.

Substrate verified (Step 0 grep, 2026-05-29):
- `AgentSnapshotRow` has `entity_bits`, `x`, `y`, `state_tag`,
  `agent_id` — no target fields.
- `AgentDetailRow` (Phase 14-γ) has 9 keys including
  `target_kind` but no target coords.
- `scripts/ui/agent_renderer.gd` HEAD = 473 lines (Phase 14-α
  ROLE_BUCKET_COUNT + ICON_OFFSET_PX, Phase 13-γ
  STATE_SCALE_BOOST, Phase 11-α + D1 STATE_TINTS, Phase 8-δ
  recall cue, Phase 9-δ combat cue, Phase 4-γ MultiMesh).
- `scenes/main.tscn` UI has 4 panel scripts (Phase 14-γ + δ).
  load_steps=8.
- No existing `_draw()` / `draw_polyline()` / `Line2D` usage in
  `scripts/ui/`.
- No `target_position` / `goal_position` / `move_target` /
  `MoveTarget` / `pathfind` symbols in
  `rust/crates/sim-core/src/components/` or `sim-systems/src/`.

Preserved invariants (≥16, agent_renderer.gd is read-only):
- **Phase 4-γ A5** agent_renderer.gd MultiMesh + SPRITE_SCALE =
  0.25 (preserved by not touching agent_renderer.gd)
- **Phase 8-δ** RECALL_CUE_SCALE_BOOST = 1.25 (preserved by
  not touching)
- **Phase 9-δ** COMBAT_CUE_SCALE_BOOST = 1.3 (preserved)
- **Phase 11-α + D1** STATE_TINTS 4-color (preserved)
- **Phase 13-γ** STATE_SCALE_BOOST 4-entry (preserved)
- **Phase 14-α** ROLE_BUCKET_COUNT = 4, ICON_OFFSET_PX
  (preserved)
- **Phase 13-δ** A4/A5/A6 hud_topbar.gd (untouched)
- Phase 12-α/β.1/β.2/γ
- Phase 13-α/β/ε bootstrap + resources
- Phase 14-β RESOURCE_TYPE_PATHS + VILLAGE_FIXTURE_*
- Phase 14-γ AgentInspectorPanel + get_agent_detail
- Phase 14-δ HudStatusPanel (separate panel option B
  precedent)

## Section 2: What to Build

**New files**:
- `scripts/ui/activity_trail_renderer.gd` — `extends Node2D`.
  Reads `get_agent_snapshot()` each frame, maintains
  `agent_id → Array[Vector2]` history ring buffer of recent
  agent world positions. Draws polylines via `_draw()` for
  non-Idle agents. Mounts as sibling under `Main` (NOT under
  `AgentRenderer`) so `agent_renderer.gd` is not in the FFI
  chain or render hierarchy of trails. `z_index` chosen so
  trails sit ABOVE terrain (Phase 12-β z=0) and BELOW agents
  (Phase 4-γ MultiMesh implicit z) — `z_index = 2`.
- `rust/crates/sim-test/tests/harness_p14_epsilon_activity_trails.rs`
  — static file-inspection assertions following Phase 14-γ
  and 14-δ precedent. ≥10 assertions.

**Modified files**:
- `scenes/main.tscn` — add `ActivityTrailRenderer` Node2D
  child of `Main` (sibling of `WorldRenderer` + `AgentRenderer`).
  `load_steps` 8 → 9; one new `ExtResource` entry for
  `activity_trail_renderer.gd`. Do NOT modify any existing
  node line. Place the new node block right after
  `AgentRenderer` and right before `Camera2D`.

**Not changed (CRITICAL — hard rules)**:
- `scripts/ui/agent_renderer.gd` — **absolutely untouched**.
  Phase 4-γ + 8-δ + 9-δ + 11-α + 13-γ + 14-α invariants depend
  on it. Any change here is CRITICAL.
- `scripts/ui/world_renderer.gd` — untouched (Phase 12-β +
  13-α/β/ε + 14-β invariants).
- `scripts/ui/camera_controller.gd` — untouched (Phase 12-α +
  13-α invariants).
- `scripts/ui/panels/hud_topbar.gd` — untouched (Phase 13-δ
  A4/A5/A6).
- `scripts/ui/panels/agent_inspector_panel.gd` — untouched
  (Phase 14-γ).
- `scripts/ui/panels/hud_status_panel.gd` — untouched (Phase
  14-δ).
- All Rust crate code (zero `.rs` change).
- All shaders, assets, locales.
- Any other harness file — must remain green.

**Cross-phase regression-guard reference files (READ-ONLY in
plan — the harness MAY reference these as subjects to verify
prior invariants; the plan must NOT propose modifications to
them)**:
- `scripts/ui/agent_renderer.gd` (Phase 4-γ + 8-δ + 9-δ +
  11-α + 13-γ + 14-α). Not under any `renderers/` subdir.
- `scripts/ui/world_renderer.gd` (Phase 12-β + 13-α/β/ε +
  14-β). Not under any `renderers/` subdir.
- `scripts/ui/camera_controller.gd` (Phase 12-α + 13-α).
- `scripts/ui/panels/hud_topbar.gd` (Phase 13-δ A4/A5/A6).
- `scripts/ui/panels/agent_inspector_panel.gd` (Phase 14-γ).
- `scripts/ui/panels/hud_status_panel.gd` (Phase 14-δ).

Use these EXACT path strings when authoring the plan; do NOT
invent intermediate directories such as `scripts/ui/renderers/`.

**Forbidden plan assertions**:
- Do NOT invent assertions that forbid snapshot identifiers in
  `agent_renderer.gd`. Like Phase 13-δ A4, the file legitimately
  references snapshot FFI literals.
- Do NOT add locale requirements — pure visual layer, no text.
- Do NOT propose target-coordinate FFI extension — that is
  Section 16+ scope, NOT this substage.

## Section 3: How to Implement

### `scripts/ui/activity_trail_renderer.gd` — new file

```gdscript
extends Node2D

# V7 Phase 14-ε — activity trail renderer.
#
# Reads agent snapshots each frame and draws faint backward
# polyline trails through recent positions for non-Idle agents.
# Mounts as a sibling Node2D under Main (not under
# AgentRenderer), so agent_renderer.gd is not touched.
#
# Honest disclosure (verified Step 0 grep, 2026-05-29):
#   - Backend FFI does NOT expose agent target coordinates;
#     only current position + state_tag. Trails are therefore
#     backward (history of where the agent has been), NOT
#     forward (where they are heading). This matches the
#     Stronghold / Factorio reference idiom — those games show
#     belt/villager flow as visible history, not predictive
#     intent.
#   - state_tag == 0 (Idle) agents do not get trails; Brownian
#     motion at rest would be rendered as noise.
#   - Trails are decorative ONLY. Sim state is read-only and
#     never mutated by this renderer.

const TILE_SIZE := 16
const SPRITE_ORIGIN_X := 448
const SPRITE_ORIGIN_Y := 28
const TRAIL_LENGTH := 16   # ring-buffer history depth per agent
const TRAIL_WIDTH := 2.0
const TRAIL_ALPHA := 0.4
const Z_TRAIL := 2          # above terrain (0), below construction (5)

# state_tag → trail colour. Idle (0) has no entry because Idle
# agents are skipped before lookup.
const TRAIL_COLOR_SEEKING: Color = Color(1.0, 0.65, 0.15, TRAIL_ALPHA)  # orange-yellow
const TRAIL_COLOR_CONSUMING_AGENT: Color = Color(1.0, 0.40, 0.75, TRAIL_ALPHA)  # pink
const TRAIL_COLOR_CONSUMING_OTHER: Color = Color(0.30, 0.95, 0.35, TRAIL_ALPHA)  # green

var _world_sim: Node = null
# agent_id (int) → Array[Vector2] history (most-recent at end).
var _history: Dictionary = {}
# agent_id (int) → int state_tag (last observed; used in _draw).
var _state_tag: Dictionary = {}

func _ready() -> void:
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	z_index = Z_TRAIL

func _process(_delta: float) -> void:
	if _world_sim == null:
		return
	var snap: Variant = _world_sim.call("get_agent_snapshot")
	if not (snap is Dictionary):
		return
	var snap_dict: Dictionary = snap
	var ids: Variant = snap_dict.get("ids", null)
	var xs: Variant = snap_dict.get("xs", null)
	var ys: Variant = snap_dict.get("ys", null)
	var states: Variant = snap_dict.get("states", null)
	var agent_ids: Variant = snap_dict.get("agent_ids", null)
	if not (ids is PackedInt64Array and xs is PackedInt32Array
			and ys is PackedInt32Array and states is PackedByteArray
			and agent_ids is PackedInt64Array):
		return
	var ids_arr: PackedInt64Array = ids
	var xs_arr: PackedInt32Array = xs
	var ys_arr: PackedInt32Array = ys
	var states_arr: PackedByteArray = states
	var agent_ids_arr: PackedInt64Array = agent_ids
	var n: int = ids_arr.size()
	if xs_arr.size() != n or ys_arr.size() != n \
			or states_arr.size() != n or agent_ids_arr.size() != n:
		return
	# Update history ring buffer per agent_id.
	var seen: Dictionary = {}
	for i in n:
		var aid: int = int(agent_ids_arr[i])
		seen[aid] = true
		var px: float = float(SPRITE_ORIGIN_X + xs_arr[i] * TILE_SIZE) + float(TILE_SIZE) / 2.0
		var py: float = float(SPRITE_ORIGIN_Y + ys_arr[i] * TILE_SIZE) + float(TILE_SIZE) / 2.0
		var hist: Array = _history.get(aid, []) as Array
		hist.append(Vector2(px, py))
		while hist.size() > TRAIL_LENGTH:
			hist.pop_front()
		_history[aid] = hist
		_state_tag[aid] = int(states_arr[i])
	# Drop history for despawned agents.
	var stale: Array = []
	for aid in _history.keys():
		if not seen.has(aid):
			stale.append(aid)
	for aid in stale:
		_history.erase(aid)
		_state_tag.erase(aid)
	queue_redraw()

func _draw() -> void:
	for aid in _history.keys():
		var tag: int = int(_state_tag.get(aid, 0))
		if tag == 0:
			continue  # Idle — no trail
		var hist: Array = _history[aid]
		if hist.size() < 2:
			continue
		var color: Color = _color_for_state_tag(tag)
		var pts: PackedVector2Array = PackedVector2Array()
		for p in hist:
			pts.append(p as Vector2)
		draw_polyline(pts, color, TRAIL_WIDTH, true)

func _color_for_state_tag(tag: int) -> Color:
	# state_tag domain (Phase 11-α + D1):
	#   1 = Seeking
	#   2 = Consuming Agent (socialising)
	#   3 = Consuming other (eating / building / sleeping)
	if tag == 1:
		return TRAIL_COLOR_SEEKING
	if tag == 2:
		return TRAIL_COLOR_CONSUMING_AGENT
	if tag == 3:
		return TRAIL_COLOR_CONSUMING_OTHER
	# Defensive: unknown tag → Idle colour (transparent).
	return Color(0, 0, 0, 0)
```

### `scenes/main.tscn` — append ActivityTrailRenderer

After the existing `[ext_resource ... id="7_status"]` line
(Phase 14-δ), add:

```
[ext_resource type="Script" path="res://scripts/ui/activity_trail_renderer.gd" id="8_trails"]
```

After the existing `[node name="AgentRenderer" ...]` block and
BEFORE the `[node name="Camera2D" ...]` block, add:

```
[node name="ActivityTrailRenderer" type="Node2D" parent="."]
script = ExtResource("8_trails")
```

Update header: `[gd_scene load_steps=8 ...]` → `load_steps=9`.

Do not modify any other line in main.tscn. Existing WorldSim /
WorldRenderer / AgentRenderer / Camera2D / UI/* nodes stay
verbatim.

### Rust harness

`rust/crates/sim-test/tests/harness_p14_epsilon_activity_trails.rs`

Follow Phase 14-γ / 14-δ structure (project_root +
strip_gd_comments + find_decl_rhss helpers). Strict
file-inspection assertions only.

1. `a1_activity_trail_renderer_file_exists` —
   `scripts/ui/activity_trail_renderer.gd` exists, starts with
   `extends Node2D`.
2. `a2_trail_length_constant_declared` — file contains
   `const TRAIL_LENGTH := 16`.
3. `a3_trail_width_constant_declared` — file contains
   `const TRAIL_WIDTH := 2.0`.
4. `a4_trail_alpha_constant_declared` — file contains
   `const TRAIL_ALPHA := 0.4`.
5. `a5_z_trail_equals_two` — file contains
   `const Z_TRAIL := 2` AND assigns `z_index = Z_TRAIL` in
   `_ready()`.
6. `a6_three_state_tag_trail_colours_declared` — file
   contains `TRAIL_COLOR_SEEKING`, `TRAIL_COLOR_CONSUMING_AGENT`,
   `TRAIL_COLOR_CONSUMING_OTHER`.
7. `a7_polls_agent_snapshot_only` — file contains
   `get_agent_snapshot` AND does NOT contain
   `get_settlement_snapshot` / `get_construction_snapshot`
   (single-snapshot scope).
8. `a8_variant_safe_pattern_present` — file contains
   `is Dictionary` AND `is PackedInt64Array` AND
   `is PackedByteArray` AND `is PackedInt32Array`.
9. `a9_draw_polyline_present` — file contains `draw_polyline`
   inside `_draw` function.
10. `a10_idle_agents_skipped` — file contains
    `if tag == 0:` AND `continue` (the Idle-skip guard).
11. `a11_main_tscn_registers_activity_trail_renderer` —
    `scenes/main.tscn` contains the literal
    `activity_trail_renderer.gd` AND a node
    `[node name="ActivityTrailRenderer" type="Node2D"`.
12. `a12_main_tscn_load_steps_updated` — `scenes/main.tscn`
    contains `load_steps=9`.
13. `a13_phase14_alpha_agent_renderer_unchanged` —
    `scripts/ui/agent_renderer.gd` still contains
    `ROLE_BUCKET_COUNT` AND `ICON_OFFSET_PX` AND
    `STATE_SCALE_BOOST` AND `STATE_TINTS` AND `SPRITE_SCALE`
    (Phase 14-α + 13-γ + 11-α + 4-γ invariants intact).
14. `a14_phase14_beta_world_renderer_unchanged` —
    `scripts/ui/world_renderer.gd` still contains
    `RESOURCE_TYPE_PATHS` AND `RESOURCE_COUNT := 20` AND
    `RESOURCE_SEED := 88675123` AND `VILLAGE_FIXTURE_PATHS`.
15. `a15_phase13_delta_hud_topbar_unchanged` —
    `scripts/ui/panels/hud_topbar.gd` still contains
    `get_agent_snapshot` AND `get_settlement_snapshot` AND
    `get_construction_snapshot` AND `is Dictionary` AND
    `is PackedInt64Array` AND `MOUSE_FILTER_IGNORE`.
16. `a16_phase14_delta_hud_status_panel_unchanged` —
    `scripts/ui/panels/hud_status_panel.gd` still contains
    `TICKS_PER_DAY` AND `RESOURCE_TYPES_COUNT` AND
    `RESOURCE_LABELS`.

## Section 4: Locale

No new locale keys. Trails are purely visual (lines + colours);
no text. Matches Phase 14-γ / 14-δ precedent.

## Section 5: Verification

```bash
cd rust && cargo test -p sim-test --test harness_p14_epsilon_activity_trails -- --nocapture
cd rust && cargo test -p sim-test --test harness_p13_delta_basic_hud -- --nocapture
cd rust && cargo test --workspace
cd rust && cargo clippy --workspace --all-targets -- -D warnings
/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot \
    --path . --headless --check-only --script scripts/ui/activity_trail_renderer.gd
```

Expected: harness_p14_epsilon ≥10 PASS, all prior phases green,
workspace + clippy clean, GDScript parse clean.

## Section 6: Lane

`--quick` — one new GDScript file + scene registration. Zero
Rust crate change. Zero FFI extension. Zero
agent_renderer.gd / world_renderer.gd / camera_controller.gd /
hud_topbar.gd / agent_inspector_panel.gd / hud_status_panel.gd
modification. Zero asset / shader / locale change.

## Section 7: 인게임 확인사항

**Expected visual change in pipeline VLM capture**:
- Most agents (Idle, the dominant state at simulation startup)
  have NO trails — scene looks similar to Phase 14-δ.
- Any non-Idle agents (Seeking / Consuming) leave a faint
  colored polyline behind them (orange-yellow / pink /
  green per state_tag).
- HUD top-left (hud_topbar) + top-right (hud_status_panel)
  unchanged from Phase 14-δ.

**VLM signal for APPROVE / VISUAL_PASS**: scene similar to
Phase 14-δ baseline; trails are below VLM sub-resolution
threshold per CLAUDE.md "VLM Visual Verification — Known
Limitation" (trails at 2 px width × zoom 3.0× = 6 px on screen,
below VLM grader resolution for line work). The Rust harness's
strict file checks are the authoritative validation.

**VLM signal for WARNING** (acceptable): VLM may not articulate
trails in textual output due to the sub-resolution issue. This
is intentional — windowed Godot run is the perceptual gate.

**VLM signal for FAIL**: scene crash, agents disappear, layout
broken.

**Honest disclosure**:
- Trails are **backward** (history of where the agent has been),
  NOT forward (where they are heading). Backend does not expose
  target coordinates per agent.
- Idle agents are excluded — their Brownian jitter would
  render as random noise. Once an agent enters Seeking /
  Consuming, the trail begins building.
- Trail visibility at zoom 3.0× = 2 px × 3 = 6 px on screen,
  which is at the edge of human perceptual threshold and below
  VLM grader threshold. Reference games (Factorio, Stronghold)
  rely on the human reader recognising trails at high
  magnification — δ's HUD plus γ's click inspector remain the
  primary explanation surfaces for "what is this agent doing
  and why".
- Forward intent (target coordinates) is **Section 16+
  candidate** because it would require either an FFI extension
  surfacing the AgentDecisionSystem's chosen target tile per
  agent, or duplicating the nearest-tile selection on the
  GDScript side (both are out of `--quick` scope).
- `agent_renderer.gd` is the explicit Phase 4-γ + 8-δ + 9-δ +
  11-α + 13-γ + 14-α contract surface and is NOT touched by
  this substage — all 6 phase invariants preserved by
  construction (option B pattern repeated from Phase 14-δ
  success).
