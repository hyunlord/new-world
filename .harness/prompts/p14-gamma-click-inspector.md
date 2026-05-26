# Phase 14-γ — Click Inspector (collect_agent_detail FFI + UI panel)

Feature: p14-gamma-click-inspector
Lane: --full (Rust sim-bridge FFI extension + GDScript UI panel)
Parent: `.harness/plans/phase14.md` (P14Plan-5 Conservative 8-field
confirmed 2026-05-25) + Phase 14-β (`0763dc95`).

## Section 1: Implementation Intent

Phase 14-γ is the **third substage** of the Phase 14 RimWorld-like
Visual Overhaul Sprint. Reference games: RimWorld + Oxygen Not
Included.

After Phase 14-α (per-role HUE) and Phase 14-β (resource +
village fixture variety), the user-facing map looks settled but
the simulation gives no feedback when an agent is clicked. There
is no way to see what an agent's current need levels are, what
they are doing, or what they are heading toward. γ closes that
gap with a single new FFI plus a click-driven inspector panel.

**Goal**: clicking on an agent's rendered sprite (in tile-grid
space) opens a right-side Control panel showing 8 fields:
agent_id, x, y, state_tag, hunger, thirst, sleep, target_kind.

Two design choices, locked in advance by the user:

1. **8-field Conservative scope** (P14Plan-5 = Conservative,
   confirmed 2026-05-25). No social/memory/relationships in γ —
   that expansion lives in Section 16+ when real systems land
   for it.

2. **Backend has all 8 fields** (Step 0 grep verified):
   - `Agent.id: AgentId` (u64) — existing
   - `Position.x: u32`, `Position.y: u32` — existing
   - `AgentState` enum + state_tag mapping — existing in
     `collect_agent_snapshot`
   - `Hunger.value: f32` — existing (SATURATION=100.0)
   - `Thirst.value: f64` — existing (SATURATION=100.0)
   - `Sleep.fatigue: f64` — existing (SATURATION=100.0)
   - `AgentState::target() -> Option<TargetKind>` — existing
     helper, returns Food/Water/Sleep/ConstructionSite/Agent
     (5 variants) or None for Idle

   **No new ECS components needed.** `target_kind` is the
   `Option<TargetKind>` already exposed by `AgentState::target()`.
   We encode it as a small int (0=None, 1=Food, 2=Water,
   3=Sleep, 4=ConstructionSite, 5=Agent) so the GDScript side
   can translate to a label without dealing with payload-bearing
   variants (the Agent(AgentId) payload is intentionally not
   surfaced this substage — it would require a relationship
   lookup which is out of γ scope).

**Honest disclosure**:
- The `Agent(AgentId)` variant of `TargetKind` is encoded as
  `5` without surfacing the inner AgentId in the Conservative
  8-field schema. The inner id would require a second FFI
  lookup to translate to that partner's identity — deferred to
  γ-extended or Section 16+.
- The inspector panel currently renders only the 8 fields.
  Relationship / memory / combat history surfacing is out of
  scope (P14Plan-5 Conservative).
- Click detection uses **option A** (world coord proximity to
  agents in the existing `collect_agent_snapshot`) because
  `MultiMeshInstance2D` does not expose per-instance picking in
  Godot 4.6. The mouse position is converted to tile-grid
  coords using the same `SPRITE_ORIGIN_X/Y` + `TILE_SIZE`
  basis as `world_renderer._handle_tile_click` (Phase 4-γ A5
  precedent). We pick the agent whose tile centre is closest
  to the click, within a radius of 1 tile (16 world px).
- Panel position: **right side**, anchored top-right, 280 px
  wide, full viewport height. Matches RimWorld idiom.

Substrate verified (Step 0 grep, 2026-05-26):
- `rust/crates/sim-core/src/components/agent_state.rs:29-51` —
  `TargetKind` enum 5 variants (Food, Water, Sleep,
  ConstructionSite, Agent(AgentId))
- `rust/crates/sim-core/src/components/agent_state.rs:82-87` —
  `AgentState::target()` helper already exists
- `rust/crates/sim-core/src/components/hunger.rs` — `Hunger {
  value: f32, growth_rate: f32 }` + `SATURATION: f32 = 100.0`
- `rust/crates/sim-core/src/components/thirst.rs` — `Thirst {
  value: f64, growth_rate: f64 }` + `SATURATION: f64 = 100.0`
- `rust/crates/sim-core/src/components/sleep.rs` — `Sleep {
  fatigue: f64, growth_rate: f64 }` + `SATURATION: f64 = 100.0`
- `rust/crates/sim-bridge/src/ffi/world_node.rs:1054-1130` —
  existing Bridge Identity Contract pattern for
  `collect_agent_snapshot` (pure-Rust collector + `#[func]`
  forwarder + dict marshaller)
- `scripts/ui/world_renderer.gd:300-330` — existing
  `_unhandled_input` + `_handle_tile_click` for left-click
  tile causal history
- `scenes/main.tscn` — existing `UI` CanvasLayer with
  `CausalPanel` + `HudTopbar` Control children; new
  `AgentInspectorPanel` lands here

Preserved invariants (≥14 cross-phase):
- Phase 4-γ SPRITE_SCALE = 0.25 (agent rendering geometry
  unchanged)
- Phase 4-γ A5 collect_agent_snapshot return shape (existing
  FFI unaffected — γ is a NEW additive FFI)
- Phase 8-δ RECALL_CUE_SCALE_BOOST = 1.25
- Phase 9-δ COMBAT_CUE_SCALE_BOOST = 1.3
- Phase 11-α + D1 STATE_TINTS palette
- Phase 12-α ZOOM_MIN/MAX
- Phase 12-β.2 A3 CONSTRUCTION_SPRITE_PATH = buildings/cairn/1.png
- Phase 12-γ FURNITURE_SPRITE_PATH = furniture/hearth/1.png
- Phase 13-α BUILDING_SPRITE_PATH = buildings/campfire/1.png +
  ZOOM_DEFAULT 3.0
- Phase 13-β RESOURCE_SPRITE_PATH + Z_RESOURCE + RESOURCE_COUNT
  + RESOURCE_SEED
- Phase 13-γ STATE_SCALE_BOOST 4-entry
- Phase 13-ε three-campfire bootstrap
- Phase 14-α ROLE_BUCKET_COUNT + ICON_OFFSET_PX
- Phase 14-β RESOURCE_TYPE_PATHS (5-entry) + VILLAGE_FIXTURE_*
- Existing `world_renderer._handle_tile_click` tile causal
  history path (γ adds agent-first dispatch but preserves the
  tile-history fallback when no agent is within radius)

## Section 2: What to Build

**Modified files**:
- `rust/crates/sim-bridge/src/ffi/world_node.rs` — add new
  `collect_agent_detail` pure-Rust collector + `#[func]
  fn get_agent_detail(entity_bits: i64) -> VarDictionary`
  forwarder, following the Bridge Identity Contract precedent
  set by `collect_agent_snapshot`.
- `scripts/ui/world_renderer.gd` — extend `_handle_tile_click`
  to first probe the agent snapshot for the nearest agent
  within 1-tile radius (16 world px). If one is found, query
  `get_agent_detail` and forward to the inspector panel; else
  fall back to the existing tile causal-history dispatch
  (preserves Phase 12-β.2 etc. behaviour).
- `scenes/main.tscn` — add `AgentInspectorPanel` Control child
  under `UI` CanvasLayer.

**New files**:
- `scripts/ui/panels/agent_inspector_panel.gd` —
  `extends Control`. Right-anchored 280 px wide panel showing
  the 8 fields with bar visualisations for hunger/thirst/
  sleep. Hidden by default; shown on agent click; hidden on
  ESC or click outside the panel.
- `rust/crates/sim-test/tests/harness_p14_gamma_click_inspector.rs`
  — combination of:
    - Static file-inspection assertions (FFI signature, panel
      script structure, click handler logic, invariant guards)
    - Integration test calling `collect_agent_detail` directly
      against a real `make_stage1_engine` world to verify the
      pure-Rust collector returns the expected 8-field row for
      a known agent entity.

**Not changed**:
- `rust/crates/sim-core` (no new components; γ is an FFI +
  GDScript-side feature)
- `rust/crates/sim-systems`, `sim-engine`, `sim-data` (untouched)
- `scripts/ui/agent_renderer.gd` (Phase 14-α just landed; no
  re-touch — click handling lives in `world_renderer.gd` as
  the existing dispatch point)
- `scripts/ui/camera_controller.gd`
- `scripts/ui/panels/hud_topbar.gd`
- `scripts/ui/panels/causal_panel.gd` (existing tile causal
  history panel — preserved; γ adds a NEW panel)
- `shaders/palette_swap.gdshader`
- All assets
- All existing harness files — must remain green

## Section 3: How to Implement

### `rust/crates/sim-bridge/src/ffi/world_node.rs` — pure-Rust collector

Add a new public struct + collector mirroring the
`AgentSnapshotRow` + `collect_agent_snapshot` pattern (~line
1057):

```rust
/// V7 Phase 14-γ — single-agent detail row for the click
/// inspector panel.
///
/// 8 Conservative fields (P14Plan-5, locked 2026-05-25):
///   - `agent_id`     — `Agent.id` (AgentId domain, matches
///                       snapshot agent_ids)
///   - `x`, `y`        — tile coords (Position.x/y as i32)
///   - `state_tag`    — same locked mapping as
///                       AgentSnapshotRow (0-3)
///   - `hunger`       — `Hunger.value` (f32; [0, SATURATION=100])
///   - `thirst`       — `Thirst.value` (f64; [0, SATURATION=100])
///   - `sleep`        — `Sleep.fatigue` (f64; [0, SATURATION=100])
///   - `target_kind`  — i32 encoding of `Option<TargetKind>`:
///                       0 = None (Idle)
///                       1 = Food
///                       2 = Water
///                       3 = Sleep
///                       4 = ConstructionSite
///                       5 = Agent (inner AgentId NOT surfaced
///                                   in Conservative scope)
///
/// The `found` field distinguishes "row populated" from "entity
/// not found / not an Agent / missing required components" so the
/// GDScript caller can handle stale clicks gracefully.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AgentDetailRow {
    pub found: bool,
    pub agent_id: u64,
    pub x: i32,
    pub y: i32,
    pub state_tag: u8,
    pub hunger: f32,
    pub thirst: f64,
    pub sleep: f64,
    pub target_kind: i32,
}

/// V7 Phase 14-γ pure-Rust collector — look up a single agent by
/// `Entity::to_bits()` and return its 8-field detail row.
///
/// Returns a row with `found = false` (other fields zeroed) when:
///   - `entity_bits` does not correspond to a live entity, OR
///   - the entity lacks the required components `(Agent,
///     Position, AgentState, Hunger, Thirst, Sleep)`.
///
/// Mirrors `WorldSimNode::get_agent_detail` minus the Godot
/// marshalling — sim-test exercises this directly.
pub fn collect_agent_detail(world: &hecs::World, entity_bits: u64) -> AgentDetailRow {
    use std::num::NonZeroU64;
    let nz = match NonZeroU64::new(entity_bits) {
        Some(v) => v,
        None => return AgentDetailRow::default(),
    };
    let entity = hecs::Entity::from_bits(nz);
    let Some(entity) = entity else {
        return AgentDetailRow::default();
    };
    let q = world.query_one::<(&Agent, &Position, &AgentState, &Hunger, &Thirst, &Sleep)>(entity);
    let mut q = match q {
        Ok(q) => q,
        Err(_) => return AgentDetailRow::default(),
    };
    let Some((agent, pos, state, hunger, thirst, sleep)) = q.get() else {
        return AgentDetailRow::default();
    };
    let state_tag: u8 = match state {
        AgentState::Idle => 0,
        AgentState::Seeking { .. } => 1,
        AgentState::Consuming { target: TargetKind::Agent(_) } => 2,
        AgentState::Consuming { .. } => 3,
    };
    let target_kind: i32 = match state.target() {
        None => 0,
        Some(TargetKind::Food) => 1,
        Some(TargetKind::Water) => 2,
        Some(TargetKind::Sleep) => 3,
        Some(TargetKind::ConstructionSite) => 4,
        Some(TargetKind::Agent(_)) => 5,
    };
    AgentDetailRow {
        found: true,
        agent_id: agent.id,
        x: pos.x as i32,
        y: pos.y as i32,
        state_tag,
        hunger: hunger.value,
        thirst: thirst.value,
        sleep: sleep.fatigue,
        target_kind,
    }
}

impl Default for AgentDetailRow {
    fn default() -> Self {
        Self {
            found: false,
            agent_id: 0,
            x: 0,
            y: 0,
            state_tag: 0,
            hunger: 0.0,
            thirst: 0.0,
            sleep: 0.0,
            target_kind: 0,
        }
    }
}

fn agent_detail_to_dict(row: AgentDetailRow) -> VarDictionary {
    let mut d = VarDictionary::new();
    d.set("found", row.found);
    d.set("agent_id", row.agent_id as i64);
    d.set("x", row.x);
    d.set("y", row.y);
    d.set("state_tag", row.state_tag as i64);
    d.set("hunger", row.hunger as f64);
    d.set("thirst", row.thirst);
    d.set("sleep", row.sleep);
    d.set("target_kind", row.target_kind as i64);
    d
}
```

### `rust/crates/sim-bridge/src/ffi/world_node.rs` — `#[func]` forwarder

Inside the `impl WorldSimNode` block, immediately after the
existing `get_agent_snapshot` `#[func]` (~line 238-242), add:

```rust
/// V7 Phase 14-γ FFI — single-agent detail dictionary for the
/// click inspector panel. Returns 9 keys (`found`, `agent_id`,
/// `x`, `y`, `state_tag`, `hunger`, `thirst`, `sleep`,
/// `target_kind`). When the entity is not found, `found` is
/// `false` and the numeric fields are zeroed.
///
/// The `#[func]` body consists solely of forwarding to
/// [`collect_agent_detail`] (Bridge Identity Contract — Phase
/// 14-γ extension). Sim-test exercises the pure-Rust collector
/// directly.
#[func]
fn get_agent_detail(&self, entity_bits: i64) -> VarDictionary {
    let row = collect_agent_detail(&self.engine.world, entity_bits as u64);
    agent_detail_to_dict(row)
}
```

Add the necessary `use` lines at the top of `world_node.rs` if
not already present: `Hunger`, `Thirst`, `Sleep`, `TargetKind`.

### `scripts/ui/world_renderer.gd` — click dispatch update

Locate the existing `_unhandled_input` handler (~line 300). The
left-click branch currently dispatches directly to
`_handle_tile_click`. Replace it with an agent-first probe that
falls back to the tile dispatch when no agent is within radius.

Add constants (~line 93, near the resource constants):

```gdscript
# V7 Phase 14-γ — click inspector probe radius.
# Mouse position is converted to world coords; an agent is
# considered "clicked" if its tile centre lies within
# CLICK_RADIUS_WORLD_PX of the mouse. 16 = one tile width
# (Phase 4-γ TILE_SIZE precedent).
const CLICK_RADIUS_WORLD_PX := 16.0
```

Replace the `_handle_tile_click` body to:

```gdscript
func _handle_tile_click(pos: Vector2) -> void:
	# V7 Phase 14-γ — try agent probe first.
	if _try_agent_click(pos):
		return
	# Fallback: tile causal history (Phase 12-β.2 precedent).
	var tile_x := int(floor((pos.x - SPRITE_ORIGIN_X) / float(TILE_SIZE)))
	var tile_y := int(floor((pos.y - SPRITE_ORIGIN_Y) / float(TILE_SIZE)))
	if tile_x < 0 or tile_x >= GRID_W or tile_y < 0 or tile_y >= GRID_H:
		return
	_fetch_causal_history(tile_x, tile_y)

# V7 Phase 14-γ — agent click probe.
# Iterates the most recent agent snapshot (parallel arrays) and
# finds the agent whose tile centre is closest to the mouse
# world coords, within CLICK_RADIUS_WORLD_PX. Returns true when
# an agent was found AND the inspector was shown.
func _try_agent_click(world_pos: Vector2) -> bool:
	if world_sim == null:
		return false
	var snap: Dictionary = world_sim.get_agent_snapshot()
	var ids: PackedInt64Array = snap.get("ids", PackedInt64Array())
	var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
	var n: int = ids.size()
	if n == 0:
		return false
	var best_idx: int = -1
	var best_dist2: float = CLICK_RADIUS_WORLD_PX * CLICK_RADIUS_WORLD_PX
	for i in n:
		var cpx: float = float(SPRITE_ORIGIN_X + xs[i] * TILE_SIZE) + float(TILE_SIZE) / 2.0
		var cpy: float = float(SPRITE_ORIGIN_Y + ys[i] * TILE_SIZE) + float(TILE_SIZE) / 2.0
		var dx: float = cpx - world_pos.x
		var dy: float = cpy - world_pos.y
		var d2: float = dx * dx + dy * dy
		if d2 < best_dist2:
			best_dist2 = d2
			best_idx = i
	if best_idx < 0:
		return false
	var detail: Dictionary = world_sim.get_agent_detail(int(ids[best_idx]))
	if not bool(detail.get("found", false)):
		return false
	var panel := get_node_or_null("/root/Main/UI/AgentInspectorPanel")
	if panel != null and panel.has_method("display_agent"):
		panel.call("display_agent", detail)
		panel.visible = true
		return true
	return false
```

Note: the mouse position passed into `_handle_tile_click` is
already in world coordinates (the camera is `Camera2D`, so
`InputEventMouseButton.position` lies in viewport space which
the camera translates — the existing tile-x/y computation
already works in this frame, so we reuse the same input).

### `scripts/ui/panels/agent_inspector_panel.gd` — inspector panel

```gdscript
extends Control

# V7 Phase 14-γ — agent click inspector panel.
#
# Right-anchored 280 px wide Control. Hidden by default;
# `display_agent(detail: Dictionary)` populates the 8 fields and
# `visible = true` reveals the panel. ESC or `close()` hides it.
#
# Conservative 8-field scope (P14Plan-5, locked 2026-05-25).
# Relationship / memory / combat history surfacing deferred to
# Section 16+ when supporting systems land.

const PANEL_WIDTH := 280.0
const TARGET_LABELS: Array = ["None", "Food", "Water", "Sleep", "ConstructionSite", "Agent"]
const STATE_LABELS: Array = ["Idle", "Seeking", "Consuming(Agent)", "Consuming(other)"]
const NEED_SATURATION := 100.0  # matches Rust Hunger/Thirst/Sleep SATURATION

var _vbox: VBoxContainer
var _label_id: Label
var _label_pos: Label
var _label_state: Label
var _label_target: Label
var _hunger_bar: ProgressBar
var _thirst_bar: ProgressBar
var _sleep_bar: ProgressBar

func _ready() -> void:
	# Anchor right edge of viewport, full height, fixed width.
	anchor_left = 1.0
	anchor_right = 1.0
	anchor_top = 0.0
	anchor_bottom = 1.0
	offset_left = -PANEL_WIDTH
	offset_right = 0.0
	offset_top = 0.0
	offset_bottom = 0.0
	visible = false
	var bg := ColorRect.new()
	bg.color = Color(0.08, 0.08, 0.10, 0.92)
	bg.anchor_right = 1.0
	bg.anchor_bottom = 1.0
	add_child(bg)
	_vbox = VBoxContainer.new()
	_vbox.anchor_right = 1.0
	_vbox.anchor_bottom = 1.0
	_vbox.offset_left = 8.0
	_vbox.offset_top = 8.0
	_vbox.offset_right = -8.0
	_vbox.offset_bottom = -8.0
	add_child(_vbox)
	_label_id = _make_label("Agent: -")
	_label_pos = _make_label("Pos: -")
	_label_state = _make_label("State: -")
	_label_target = _make_label("Target: -")
	_hunger_bar = _make_bar("Hunger")
	_thirst_bar = _make_bar("Thirst")
	_sleep_bar = _make_bar("Sleep")

func _make_label(text: String) -> Label:
	var lbl := Label.new()
	lbl.text = text
	_vbox.add_child(lbl)
	return lbl

func _make_bar(name: String) -> ProgressBar:
	var hdr := Label.new()
	hdr.text = name
	_vbox.add_child(hdr)
	var bar := ProgressBar.new()
	bar.min_value = 0.0
	bar.max_value = NEED_SATURATION
	bar.value = 0.0
	_vbox.add_child(bar)
	return bar

# V7 Phase 14-γ — populate the 8 fields from the FFI detail dict.
func display_agent(detail: Dictionary) -> void:
	var agent_id: int = int(detail.get("agent_id", 0))
	var x: int = int(detail.get("x", 0))
	var y: int = int(detail.get("y", 0))
	var state_tag: int = clampi(int(detail.get("state_tag", 0)), 0, 3)
	var target_kind: int = clampi(int(detail.get("target_kind", 0)), 0, 5)
	var hunger: float = float(detail.get("hunger", 0.0))
	var thirst: float = float(detail.get("thirst", 0.0))
	var sleep: float = float(detail.get("sleep", 0.0))
	_label_id.text = "Agent: %d" % agent_id
	_label_pos.text = "Pos: (%d, %d)" % [x, y]
	_label_state.text = "State: %s" % STATE_LABELS[state_tag]
	_label_target.text = "Target: %s" % TARGET_LABELS[target_kind]
	_hunger_bar.value = hunger
	_thirst_bar.value = thirst
	_sleep_bar.value = sleep

func _unhandled_input(event: InputEvent) -> void:
	if not visible:
		return
	if event is InputEventKey and event.pressed and event.keycode == KEY_ESCAPE:
		close()
		get_viewport().set_input_as_handled()

func close() -> void:
	visible = false
```

### `scenes/main.tscn` — panel node addition

After the existing `HudTopbar` node under `UI`, add:

```
[node name="AgentInspectorPanel" type="Control" parent="UI"]
script = ExtResource("6_inspector")
```

And add the `ExtResource` reference near the existing ones at
the top:

```
[ext_resource type="Script" path="res://scripts/ui/panels/agent_inspector_panel.gd" id="6_inspector"]
```

Increment the `load_steps` count by 1 (was 6, becomes 7).

### Rust harness

`rust/crates/sim-test/tests/harness_p14_gamma_click_inspector.rs`

≥14 assertions. Mix of static file-inspection (FFI + panel
script structure) and integration tests against the real engine.

**Integration tests** (using `make_stage1_engine` from
`sim-test`):

1. `a1_collect_agent_detail_returns_found_for_known_entity` —
   build a stage-1 engine with seed=42, agent_count=20, run 1
   tick. Pull the agent snapshot, take the first row's
   `entity_bits` (= ids[0] reinterpreted), call
   `collect_agent_detail`, assert `found == true`.
2. `a2_collect_agent_detail_returns_not_found_for_zero` —
   `collect_agent_detail(world, 0)` must return `found ==
   false`.
3. `a3_collect_agent_detail_agent_id_matches_snapshot` — for
   the first snapshot row, the detail row's `agent_id` must
   equal the snapshot row's `agent_ids[0]`.
4. `a4_collect_agent_detail_x_y_match_snapshot` — detail row
   `(x, y)` equals snapshot row `(xs[0], ys[0])`.
5. `a5_collect_agent_detail_state_tag_in_domain` — `state_tag`
   ∈ {0, 1, 2, 3}.
6. `a6_collect_agent_detail_needs_in_range` — hunger ∈
   [0, 100.0], thirst ∈ [0, 100.0], sleep ∈ [0, 100.0].
7. `a7_collect_agent_detail_target_kind_in_domain` —
   `target_kind` ∈ {0, 1, 2, 3, 4, 5}.

**Static file inspection** (mirrors prior phases):

8. `a8_get_agent_detail_func_declared_in_bridge` — search
   `world_node.rs` for the `#[func]` `fn get_agent_detail`
   signature with `entity_bits: i64 -> VarDictionary`.
9. `a9_agent_detail_to_dict_emits_nine_keys` — search
   `world_node.rs` for the 9 keys set on the dictionary:
   `found`, `agent_id`, `x`, `y`, `state_tag`, `hunger`,
   `thirst`, `sleep`, `target_kind`.
10. `a10_inspector_panel_script_exists` — file
    `scripts/ui/panels/agent_inspector_panel.gd` exists and
    declares `extends Control`.
11. `a11_inspector_panel_display_agent_method_present` —
    panel script contains `func display_agent(detail:
    Dictionary)`.
12. `a12_inspector_panel_uses_progress_bars` — panel script
    references `ProgressBar.new()` (for hunger/thirst/sleep
    bars).
13. `a13_world_renderer_try_agent_click_present` —
    `world_renderer.gd` contains `func _try_agent_click(`
    and `CLICK_RADIUS_WORLD_PX := 16.0`.
14. `a14_world_renderer_tile_click_fallback_preserved` —
    `_handle_tile_click` in world_renderer.gd still calls
    `_fetch_causal_history` as the fallback (i.e. tile
    causal-history path is preserved when no agent is found).
15. `a15_main_tscn_has_agent_inspector_panel_node` —
    `scenes/main.tscn` references
    `agent_inspector_panel.gd` AND has a node named
    `AgentInspectorPanel`.

**Invariant guards**:

16. `a16_phase4_gamma_sprite_scale_preserved` — agent_renderer
    SPRITE_SCALE = 0.25.
17. `a17_phase14_alpha_role_bucket_preserved` — agent_renderer
    ROLE_BUCKET_COUNT = 4 + ICON_OFFSET_PX = Vector2(0, -12).
18. `a18_phase14_beta_resource_types_preserved` — world_renderer
    RESOURCE_TYPE_PATHS has 5 entries; RESOURCE_COUNT = 20;
    RESOURCE_SEED = 88675123; Z_RESOURCE = 3.
19. `a19_phase14_beta_village_fixtures_preserved` —
    world_renderer VILLAGE_FIXTURE_PATHS has 4 entries; Z_VILLAGE_FIXTURE
    = 5.
20. `a20_phase13_alpha_bootstrap_preserved` — BUILDING_SPRITE_PATH
    = buildings/campfire/1.png; BOOTSTRAP_X = 32; BOOTSTRAP_Y =
    32; BOOTSTRAP_X_LEFT = 24; BOOTSTRAP_X_RIGHT = 40.
21. `a21_phase12_construction_furniture_preserved` —
    CONSTRUCTION_SPRITE_PATH = buildings/cairn/1.png; Z_CONSTRUCTION
    = 5; FURNITURE_SPRITE_PATH = furniture/hearth/1.png;
    Z_FURNITURE = 4.
22. `a22_phase12_alpha_zoom_preserved` — camera_controller
    ZOOM_MIN = Vector2(0.5, 0.5); ZOOM_MAX = Vector2(4.0, 4.0);
    ZOOM_DEFAULT = Vector2(3.0, 3.0).
23. `a23_existing_collect_agent_snapshot_unchanged` —
    `collect_agent_snapshot` still returns 4-field
    AgentSnapshotRow (entity_bits, x, y, state_tag + agent_id).
    Phase 4-γ A5 contract preserved.

## Section 4: Locale

No new locale keys this substage (English labels embedded in
the panel script as constants — Phase 14-γ scope omits
localisation; pass to a separate substage when other panels
adopt the same pattern).

## Section 5: Verification

```bash
cd rust && cargo test -p sim-test --test harness_p14_gamma_click_inspector -- --nocapture
cd rust && cargo test --workspace
cd rust && cargo clippy --workspace --all-targets -- -D warnings
/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot \
    --path . --headless --check-only --script scripts/ui/panels/agent_inspector_panel.gd
/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot \
    --path . --headless --check-only --script scripts/ui/world_renderer.gd
```

Expected: harness_p14_gamma ≥14 PASS, all prior phases green,
workspace + clippy clean, GDScript parse clean.

Pipeline Step 2.4 (`a437c547`) GDScript strict check runs
automatically — verifies the new `get_agent_detail` FFI binding
matches between GDScript caller and Rust `#[func]`.

## Section 6: Lane

`--full` — Rust sim-bridge FFI extension. Planning debate +
Visual Verify + Codex Evaluator required.

## Section 7: 인게임 확인사항

**Expected visual change in pipeline VLM capture**:
- No automatic agent inspector (the panel is hidden by default
  and only opens on left-click). VLM capture is a single
  screenshot without a click event — the panel stays hidden.
- All other rendering (terrain, agents, fixtures, resources,
  HUD) unchanged from Phase 14-β.

**VLM signal for APPROVE / VISUAL_PASS**: generic checklist
tokens still pass; scene unchanged from Phase 14-β.

**VLM signal for WARNING** (acceptable, expected): the inspector
panel does not appear in the static screenshot because no click
is fired in the harness. The Rust harness's integration tests
(A1-A7) calling `collect_agent_detail` directly are the
authoritative validation that the FFI works end-to-end. User
windowed verify is the perceptual gate for the panel itself.

**VLM signal for FAIL**: generic visual regression — agents
disappear, fixtures off-position, scene crash, terrain broken.

**Honest disclosure**:
- The `Agent(AgentId)` `TargetKind` variant is encoded as `5`
  WITHOUT surfacing the partner's AgentId. This keeps the
  schema strict 8-field Conservative.
- Click detection uses world-coord proximity to the agent
  snapshot, not Godot picking — MultiMeshInstance2D does not
  expose per-instance hit testing in Godot 4.6. The 1-tile
  radius is the visible legibility floor at zoom 3.0×.
- The inspector panel is the first new UI Control under `UI/`
  since Phase 13-δ HudTopbar. It coexists with HudTopbar and
  CausalPanel — no overlap because panel right-anchors and
  HudTopbar top-anchors.
- Tile causal history (left-click on empty tile) is preserved
  as a fallback when no agent is within 1-tile click radius.
- The `found = false` return path is required because in a
  paused world or across rapid clicks, the GDScript caller
  might pass a stale `entity_bits` whose entity has despawned.
  Returning `found = false` instead of panicking is the
  correct contract.
