# Phase 14-δ — HUD Status Panel (separate panel, hud_topbar.gd unchanged)

Feature: p14-delta-status-panel
Lane: --quick (GDScript-only — one new panel + scene registration;
zero Rust changes, zero hud_topbar.gd changes)
Parent: `.harness/plans/phase14.md` (P14Plan-6 + P14Plan-9 — option
B: separate panel, NOT hud_topbar.gd refactor) + Phase 14-γ
(`ad49cdb9`) chain successor.

## Section 1: Implementation Intent

Phase 14-δ is the **fourth substage** of the Phase 14 RimWorld-like
Visual Overhaul Sprint. Reference games: RimWorld + Tropico.

**Failed prior attempt** (2026-05-28~29, killed by user):
The original Phase 14-δ prompt instructed Generator to refactor
`scripts/ui/panels/hud_topbar.gd` from a single HBoxContainer to
a 2-row VBoxContainer. Generator faithfully rewrote `_ready()` and
`_refresh()`, in the process **removing** the Phase 13-δ A5
Variant-safe type guards (`is Dictionary`, `is PackedInt64Array`)
and breaking Phase 13-δ A4 (3-snapshot polling literal presence).
The Evaluator's RE-PLAN then introduced a Drafter-invented A19
"forbidden snapshot" assertion plus an out-of-scope locale
requirement — both contradicting Phase 13-δ contract.

**Root cause**: the prior prompt's "refactor `_ready()`" wording
granted Generator full-rewrite authority over hud_topbar.gd. Any
rewrite path could violate Phase 13-δ A4/A5/A6 invariants.

**This prompt — Option B**: **separate panel, hud_topbar.gd
absolutely untouched**. The new panel mounts as a sibling
Control under `UI` CanvasLayer and polls existing SimBridge
snapshots independently. Phase 13-δ harness assertions are
preserved by construction because hud_topbar.gd is not in the
authorized-files manifest.

The new panel's three concerns and their honest data sources:

A. **Day / Year display** = UI convention, NOT backend time
   - Step 0 grep verified: no `get_current_tick` / FFI for
     sim-tick exists. No `TICKS_PER_DAY` / `FIXED_TIMESTEP`
     constant in sim-core. The existing hud_topbar.gd uses an
     internal frame counter (`_tick_count += 1` in `_process`)
     since "Phase 13-δ does not have a sim-tick FFI — this is
     a frame counter. A real sim-tick FFI is Section 15+".
   - The new panel uses the **same internal frame-counter
     pattern** (its own private `_frame_tick: int = 0`,
     incremented in `_process`). Day display is
     `Day = floor(frame_tick / TICKS_PER_DAY)`, Year is
     `Year = floor(day / DAYS_PER_YEAR)`.
   - **Honest disclosure**: this is a UI convention. Day/Year
     reflect elapsed real-time frames at the GDScript layer,
     NOT a backend simulation calendar. Real calendar/season
     deferred to Section 16+.

B. **Resource cells** = visible-placeholder count, NOT inventory
   - Step 0 grep verified: no `Inventory` / `Stockpile`
     component exists in `rust/crates/sim-core/src/`.
   - Phase 14-β established `RESOURCE_COUNT = 20` placeholder
     sprites across 5 types (Berry / Wood / Stone / Water /
     Food) via `i % 5` cycle = 4 per type.
   - Panel displays the 5 type labels with constant `4` per
     type as a visible-only accounting reference.
   - **Honest disclosure**: cells display placeholder-sprite
     distribution constants. Real gathered-resource inventory
     deferred to Section 16+ when `Stockpile`-type component
     lands.

C. **Notification feed** = snapshot-id-set delta observer
   - Step 0 grep verified: backend has `CausalEvent` enum
     (BuildingPlaced, ConstructionStarted/Completed,
     SocialInteractionStarted/Completed, MemoryRecalled,
     CombatStarted/Completed, AgentBorn, SettlementFormed/
     Dissolved) but the FFI exposes only `get_tile_causal_history(x, y)`
     (single-tile) — no global recent-events FFI.
   - Panel derives notifications from snapshot delta:
     - `get_settlement_snapshot()` ids new ⇒ "Settlement formed"
     - `get_settlement_snapshot()` ids removed ⇒ "Settlement dissolved"
     - `get_construction_snapshot()` ids new ⇒ "Site started"
     - `get_construction_snapshot()` ids removed ⇒ "Site completed"
     - `get_agent_snapshot()` row count up ⇒ "Agent born (pop N)"
   - Queue: max 5 visible entries, each removed after
     NOTIF_FADE_FRAMES = 480 (~8 sec at 60 fps).
   - **Honest disclosure**: deltas-only feed; combat / social /
     memory events not surfaced (would require new global-events
     FFI = --full lane scope). Real causal feed deferred to
     Section 16+.

**Explicit non-scope** (deferred / not added):
- Speed control — no backend speed FFI; --full lane scope.
- Population breakdown — no `Age` / `profession` component.
- Locale keys — embedded English labels. No new locale keys.
  This matches Phase 14-γ's `agent_inspector_panel.gd`
  precedent (English constants for fields).
- Any modification of `hud_topbar.gd`, `world_renderer.gd`,
  `agent_renderer.gd`, `agent_inspector_panel.gd`, or any
  Rust crate — explicitly out of scope.

Substrate verified (Step 0 grep, 2026-05-29):
- `rust/crates/sim-bridge/src/ffi/world_node.rs` exposes 10
  `#[func]` methods. No `get_current_tick` or equivalent.
  The 3 snapshots the new panel polls
  (`get_agent_snapshot`, `get_settlement_snapshot`,
  `get_construction_snapshot`) all return a `VarDictionary`
  with an `"ids": PackedInt64Array` key.
- No `Inventory` / `Stockpile` / `Age` / `profession` / `TICKS_PER_DAY`
  in `rust/crates/sim-core/`.
- `scripts/ui/panels/hud_topbar.gd` HEAD = Phase 13-δ (4-cell
  HBoxContainer, polling 3 snapshots, Variant-safe type guards).
- `scenes/main.tscn` UI CanvasLayer has CausalPanel, HudTopbar,
  AgentInspectorPanel (Phase 14-γ); load_steps = 7.

Preserved invariants (≥18, hud_topbar.gd is read-only):
- **Phase 13-δ A4**: hud_topbar.gd references all 3 snapshot
  FFI literals (preserved by not touching hud_topbar.gd)
- **Phase 13-δ A5**: hud_topbar.gd contains `is Dictionary` +
  `is PackedInt64Array` (preserved by not touching)
- **Phase 13-δ A6**: hud_topbar.gd contains `MOUSE_FILTER_IGNORE`
  (preserved by not touching)
- Phase 4-γ SPRITE_SCALE = 0.25
- Phase 8-δ / 9-δ cue scale boosts
- Phase 11-α + D1 STATE_TINTS
- Phase 12-α ZOOM_MIN/MAX
- Phase 12-β.2 A3 CONSTRUCTION_SPRITE_PATH = buildings/cairn/1.png
- Phase 12-γ FURNITURE_SPRITE_PATH = furniture/hearth/1.png
- Phase 13-α BUILDING_SPRITE_PATH = buildings/campfire/1.png
- Phase 13-β RESOURCE_SPRITE_PATH + RESOURCE_COUNT = 20 +
  RESOURCE_SEED = 88675123
- Phase 13-γ STATE_SCALE_BOOST 4-entry
- Phase 13-ε three-campfire bootstrap row
- Phase 14-α ROLE_BUCKET_COUNT + ICON_OFFSET_PX
- Phase 14-β RESOURCE_TYPE_PATHS 5-entry + VILLAGE_FIXTURE_*
- Phase 14-γ AgentInspectorPanel + get_agent_detail FFI +
  CLICK_RADIUS_WORLD_PX

## Section 2: What to Build

**New files**:
- `scripts/ui/panels/hud_status_panel.gd` — `extends Control`.
  Top-right anchored, 320 px wide × 180 px tall. Three sections
  stacked vertically (top-down): Day/Year header, 5-cell
  resource row, notification VBoxContainer feed. Polls existing
  3 snapshots through the same Variant-safe pattern used by
  hud_topbar.gd (so harness assertions can verify both files
  follow the same safety idiom).
- `rust/crates/sim-test/tests/harness_p14_delta_status_panel.rs`
  — static file-inspection assertions following Phase 14-γ
  precedent. ≥10 assertions.

**Modified files**:
- `scenes/main.tscn` — add `HudStatusPanel` Control child
  under `UI` CanvasLayer; `load_steps` 7 → 8; one new
  `ExtResource` entry for `hud_status_panel.gd`.

**Not changed (CRITICAL — hard rules)**:
- `scripts/ui/panels/hud_topbar.gd` — **absolutely untouched**.
  Phase 13-δ A4/A5/A6 invariants depend on its exact current
  content. Any change here is a CRITICAL violation.
- All Rust crate code (zero `.rs` change)
- All other existing GDScript files
- All shaders, assets, locales
- Any other harness file — must remain green

**Cross-phase regression-guard reference files (READ-ONLY in
plan — the harness MAY reference these as subjects to verify
prior invariants are preserved; the plan must NOT propose
modifications to them)**:
- `scripts/ui/agent_renderer.gd` (Phase 14-α ROLE_BUCKET_COUNT,
  ICON_OFFSET_PX). Not under any `renderers/` subdir.
- `scripts/ui/world_renderer.gd` (Phase 14-β RESOURCE_TYPE_PATHS,
  RESOURCE_COUNT, RESOURCE_SEED, VILLAGE_FIXTURE_*; Phase 13-α
  bootstrap; Phase 13-β resource constants). Not under any
  `renderers/` subdir.
- `scripts/ui/panels/agent_inspector_panel.gd` (Phase 14-γ
  click inspector panel).
- `scripts/ui/panels/hud_topbar.gd` (Phase 13-δ A4/A5/A6).

These four paths are the canonical regression-reference set for
this substage. Use these EXACT path strings when authoring the
plan; do NOT invent intermediate directories such as
`scripts/ui/renderers/`.

**Forbidden plan assertions**:
- Do NOT invent assertions that forbid snapshot identifiers in
  hud_topbar.gd. The prior plan_attempt 2 introduced an A19
  "forbidden snapshot" rule that directly contradicted Phase
  13-δ A4 (which REQUIRES those identifiers). The new panel
  may freely reference snapshots; it must not assert
  hud_topbar.gd lacks them.
- Do NOT add locale requirements. English labels embedded as
  GDScript constants are the explicit pattern, matching Phase
  14-γ's `agent_inspector_panel.gd` precedent.

## Section 3: How to Implement

### `scripts/ui/panels/hud_status_panel.gd` — new file

```gdscript
extends Control

# V7 Phase 14-δ — HUD status panel.
#
# Top-right anchored Control with three sections stacked
# vertically: Day/Year header, 5-cell resource row, notification
# feed. Polls the three existing SimBridge snapshots
# (get_agent_snapshot, get_settlement_snapshot,
# get_construction_snapshot) through the same Variant-safe
# pattern used by hud_topbar.gd. The HUD never modifies sim
# state; it is read-only.
#
# Honest disclosure (verified Step 0 grep, 2026-05-29):
#   - No backend sim-tick FFI exists; Day/Year are derived from
#     a private frame counter (UI convention only).
#   - No backend Inventory component; resource cells display the
#     Phase 14-β placeholder-sprite distribution (4 per type).
#   - No global causal-events FFI; notifications derive from
#     snapshot id-set deltas only. Combat / social / memory
#     events are out of scope this substage.

const HUD_MARGIN: int = 12
const PANEL_WIDTH: float = 320.0
const PANEL_HEIGHT: float = 180.0
const TICKS_PER_DAY: int = 100
const DAYS_PER_YEAR: int = 30

const RESOURCE_TYPES_COUNT: int = 5
const RESOURCE_PER_TYPE: int = 4
const RESOURCE_LABELS: Array = ["Berry", "Wood", "Stone", "Water", "Food"]

const MAX_VISIBLE_NOTIFICATIONS: int = 5
const NOTIF_FADE_FRAMES: int = 480

var _world_sim: Node = null
var _frame_tick: int = 0
var _day_label: Label
var _year_label: Label
var _resource_labels: Array = []
var _notification_vbox: VBoxContainer
var _prev_settlement_ids: Dictionary = {}
var _prev_construction_ids: Dictionary = {}
var _prev_agent_count: int = -1
var _entries: Array = []  # each: { text: String, born: int }

func _ready() -> void:
	_world_sim = get_node_or_null("/root/Main/WorldSim")

	# Anchor top-right of viewport with HUD_MARGIN inset.
	anchor_left = 1.0
	anchor_right = 1.0
	anchor_top = 0.0
	anchor_bottom = 0.0
	offset_left = -(PANEL_WIDTH + float(HUD_MARGIN))
	offset_right = -float(HUD_MARGIN)
	offset_top = float(HUD_MARGIN)
	offset_bottom = float(HUD_MARGIN) + PANEL_HEIGHT
	mouse_filter = Control.MOUSE_FILTER_IGNORE

	var vbox := VBoxContainer.new()
	vbox.mouse_filter = Control.MOUSE_FILTER_IGNORE
	vbox.anchor_right = 1.0
	vbox.anchor_bottom = 1.0
	add_child(vbox)

	# Section 1: Day/Year header
	var time_hbox := HBoxContainer.new()
	time_hbox.mouse_filter = Control.MOUSE_FILTER_IGNORE
	time_hbox.add_theme_constant_override("separation", 12)
	vbox.add_child(time_hbox)
	_day_label = Label.new()
	_day_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_day_label.text = "Day 0"
	time_hbox.add_child(_day_label)
	_year_label = Label.new()
	_year_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_year_label.text = "Year 0"
	time_hbox.add_child(_year_label)

	# Section 2: Resource cells (5 fixed cells)
	var res_hbox := HBoxContainer.new()
	res_hbox.mouse_filter = Control.MOUSE_FILTER_IGNORE
	res_hbox.add_theme_constant_override("separation", 8)
	vbox.add_child(res_hbox)
	for i in RESOURCE_TYPES_COUNT:
		var lbl := Label.new()
		lbl.mouse_filter = Control.MOUSE_FILTER_IGNORE
		lbl.text = "%s %d" % [RESOURCE_LABELS[i], RESOURCE_PER_TYPE]
		_resource_labels.append(lbl)
		res_hbox.add_child(lbl)

	# Section 3: Notification feed
	_notification_vbox = VBoxContainer.new()
	_notification_vbox.mouse_filter = Control.MOUSE_FILTER_IGNORE
	vbox.add_child(_notification_vbox)

	_refresh()

func _process(_delta: float) -> void:
	if _world_sim == null:
		return
	_frame_tick += 1
	_ingest_snapshots()
	_prune_old()
	_refresh()

func _ingest_snapshots() -> void:
	var settle_snap: Variant = _world_sim.call("get_settlement_snapshot")
	var cur_s: Dictionary = {}
	if settle_snap is Dictionary:
		var sd: Dictionary = settle_snap
		var sids: Variant = sd.get("ids", null)
		if sids is PackedInt64Array:
			var sids_arr: PackedInt64Array = sids
			for i in sids_arr.size():
				cur_s[int(sids_arr[i])] = true
	for id in cur_s.keys():
		if not _prev_settlement_ids.has(id):
			_push("Settlement %d formed" % int(id))
	for id in _prev_settlement_ids.keys():
		if not cur_s.has(id):
			_push("Settlement %d dissolved" % int(id))
	_prev_settlement_ids = cur_s

	var construct_snap: Variant = _world_sim.call("get_construction_snapshot")
	var cur_c: Dictionary = {}
	if construct_snap is Dictionary:
		var cd: Dictionary = construct_snap
		var cids: Variant = cd.get("ids", null)
		if cids is PackedInt64Array:
			var cids_arr: PackedInt64Array = cids
			for i in cids_arr.size():
				cur_c[int(cids_arr[i])] = true
	for id in cur_c.keys():
		if not _prev_construction_ids.has(id):
			_push("Site %d started" % int(id))
	for id in _prev_construction_ids.keys():
		if not cur_c.has(id):
			_push("Site %d completed" % int(id))
	_prev_construction_ids = cur_c

	var agent_snap: Variant = _world_sim.call("get_agent_snapshot")
	var cur_count: int = 0
	if agent_snap is Dictionary:
		var ad: Dictionary = agent_snap
		var aids: Variant = ad.get("ids", null)
		if aids is PackedInt64Array:
			var aids_arr: PackedInt64Array = aids
			cur_count = aids_arr.size()
	if _prev_agent_count >= 0 and cur_count > _prev_agent_count:
		_push("Agent born (pop %d)" % cur_count)
	_prev_agent_count = cur_count

func _push(text: String) -> void:
	_entries.append({ "text": text, "born": _frame_tick })
	if _entries.size() > MAX_VISIBLE_NOTIFICATIONS:
		_entries.pop_front()

func _prune_old() -> void:
	var fresh: Array = []
	for e in _entries:
		if _frame_tick - int(e.get("born", 0)) < NOTIF_FADE_FRAMES:
			fresh.append(e)
	_entries = fresh

func _refresh() -> void:
	var day: int = _frame_tick / TICKS_PER_DAY
	var year: int = day / DAYS_PER_YEAR
	if _day_label != null:
		_day_label.text = "Day %d" % day
	if _year_label != null:
		_year_label.text = "Year %d" % year
	if _notification_vbox != null:
		for child in _notification_vbox.get_children():
			child.queue_free()
		for e in _entries:
			var lbl := Label.new()
			lbl.mouse_filter = Control.MOUSE_FILTER_IGNORE
			lbl.text = String(e.get("text", ""))
			_notification_vbox.add_child(lbl)
```

### `scenes/main.tscn` — append HudStatusPanel

After the existing `[ext_resource ... id="6_inspector"]` line,
add:

```
[ext_resource type="Script" path="res://scripts/ui/panels/hud_status_panel.gd" id="7_status"]
```

After the `[node name="AgentInspectorPanel" ...]` block (the
last UI child), add:

```
[node name="HudStatusPanel" type="Control" parent="UI"]
script = ExtResource("7_status")
```

Update header: `[gd_scene load_steps=7 ...]` → `load_steps=8`.

**Do not modify any other line in main.tscn.** Existing
HudTopbar / CausalPanel / AgentInspectorPanel nodes stay
verbatim.

### Rust harness

`rust/crates/sim-test/tests/harness_p14_delta_status_panel.rs`

Follow Phase 14-γ structure (project_root + strip_gd_comments +
find_decl_rhss helpers). Strict file-inspection assertions only.

1. `a1_hud_status_panel_file_exists` —
   `scripts/ui/panels/hud_status_panel.gd` exists, starts with
   `extends Control`.
2. `a2_ticks_per_day_constant_declared` — file contains
   `const TICKS_PER_DAY: int = 100` (or `TICKS_PER_DAY := 100`).
3. `a3_days_per_year_constant_declared` — file contains
   `const DAYS_PER_YEAR: int = 30`.
4. `a4_resource_constants_consistent_with_phase_14_beta` — file
   contains `const RESOURCE_TYPES_COUNT: int = 5` AND
   `const RESOURCE_PER_TYPE: int = 4`. The product
   5 × 4 = 20 must equal Phase 13-β `RESOURCE_COUNT`.
5. `a5_resource_labels_array_correct` — file contains
   `const RESOURCE_LABELS: Array = ["Berry", "Wood", "Stone",
   "Water", "Food"]` (whitespace-tolerant 5-string check, in
   order).
6. `a6_anchored_top_right_with_mouse_filter_ignore` — file
   contains `anchor_left = 1.0` AND `anchor_right = 1.0` AND
   `mouse_filter = Control.MOUSE_FILTER_IGNORE`.
7. `a7_polls_three_snapshots` — file contains all three FFI
   literals: `get_agent_snapshot`, `get_settlement_snapshot`,
   `get_construction_snapshot`.
8. `a8_variant_safe_pattern_present` — file contains
   `is Dictionary` AND `is PackedInt64Array` (matching
   hud_topbar.gd safety idiom).
9. `a9_notification_constants_present` — file contains
   `const MAX_VISIBLE_NOTIFICATIONS: int = 5` AND
   `const NOTIF_FADE_FRAMES: int = 480`.
10. `a10_main_tscn_registers_hud_status_panel` —
    `scenes/main.tscn` contains the literal
    `hud_status_panel.gd` AND a node
    `[node name="HudStatusPanel" type="Control"`.
11. `a11_main_tscn_load_steps_updated` — `scenes/main.tscn`
    contains `load_steps=8`.
12. `a12_phase13_delta_hud_topbar_unchanged_a4_a5_a6` —
    `scripts/ui/panels/hud_topbar.gd` still contains
    `get_agent_snapshot`, `get_settlement_snapshot`,
    `get_construction_snapshot` AND `is Dictionary` AND
    `is PackedInt64Array` AND `MOUSE_FILTER_IGNORE` (Phase
    13-δ A4 + A5 + A6 invariants).
13. `a13_phase14_alpha_role_bucket_preserved` —
    `agent_renderer.gd` `ROLE_BUCKET_COUNT := 4`.
14. `a14_phase14_beta_resource_types_preserved` —
    `world_renderer.gd` `RESOURCE_TYPE_PATHS` 5-entry array +
    `RESOURCE_COUNT := 20` + `RESOURCE_SEED := 88675123`.
15. `a15_phase14_gamma_agent_inspector_panel_preserved` —
    `scripts/ui/panels/agent_inspector_panel.gd` exists.

## Section 4: Locale

No new locale keys. English labels embedded as GDScript
constants (Day / Year / Berry / Wood / Stone / Water / Food /
Settlement / Site / Agent born). Matches Phase 14-γ
`agent_inspector_panel.gd` precedent.

## Section 5: Verification

```bash
cd rust && cargo test -p sim-test --test harness_p14_delta_status_panel -- --nocapture
cd rust && cargo test -p sim-test --test harness_p13_delta_basic_hud -- --nocapture
cd rust && cargo test --workspace
cd rust && cargo clippy --workspace --all-targets -- -D warnings
/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot \
    --path . --headless --check-only --script scripts/ui/panels/hud_status_panel.gd
```

Expected: harness_p14_delta_status_panel ≥10 PASS,
harness_p13_delta_basic_hud all PASS (hud_topbar.gd unchanged),
all prior phases green, workspace + clippy clean, GDScript parse
clean.

## Section 6: Lane

`--quick` — one new GDScript file + scene registration. Zero
Rust crate change. Zero FFI extension. Zero hud_topbar.gd
change. Zero asset / shader change.

## Section 7: 인게임 확인사항

**Expected visual change in pipeline VLM capture**:
- Top-right corner: new HudStatusPanel showing
  `Day 0 | Year 0` header, `Berry 4 | Wood 4 | Stone 4 |
  Water 4 | Food 4` resource row, empty notification feed
  (no events in static screenshot).
- Top-left: existing hud_topbar.gd unchanged
  (`Tick X | Agents Y | Settlements Z | Sites W`).

**VLM signal for APPROVE / VISUAL_PASS**: generic checklist
tokens pass; both HUDs visible.

**VLM signal for WARNING** (acceptable): empty notification
feed in static capture, constant resource cells. These are
intentional (deltas-only feed, placeholder accounting).

**VLM signal for FAIL**: HudStatusPanel missing, layout broken,
scene crash, hud_topbar visually changed.

**Honest disclosure**:
- Day/Year derive from a private frame counter
  (`_frame_tick / TICKS_PER_DAY` and `day / DAYS_PER_YEAR`);
  UI convention only, NOT a backend calendar. Real time system
  deferred to Section 16+.
- Resource cells display constant 4-per-type placeholder counts,
  NOT a gathered inventory. Real `Stockpile` deferred to
  Section 16+.
- Notifications observe id-set deltas across the 3 existing
  snapshots; combat/social/memory events out of scope (would
  require new global-events FFI = --full lane).
- Speed control, population breakdown, locale extraction
  deliberately deferred (--quick scope discipline).
- `hud_topbar.gd` is the explicit Phase 13-δ contract surface
  and is NOT touched by this substage — Phase 13-δ A4/A5/A6
  invariants preserved by construction.
