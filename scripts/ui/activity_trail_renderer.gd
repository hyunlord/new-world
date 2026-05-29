extends Node2D

# V7 Phase 14-ε — activity trail renderer.
#
# Reads agent snapshots each frame and draws faint backward polyline
# trails through recent positions for non-Idle agents. Mounts as a
# sibling Node2D under Main (not under AgentRenderer), so
# agent_renderer.gd is not touched.
#
# Honest disclosure (verified Step 0 grep, 2026-05-29):
#   - Backend FFI does NOT expose agent target coordinates; only
#     current position + state_tag. Trails are therefore backward
#     (history of where the agent has been), NOT forward (where they
#     are heading). This matches the Stronghold / Factorio reference
#     idiom — those games show belt/villager flow as visible history,
#     not predictive intent.
#   - state_tag == 0 (Idle) agents do not get trails; Brownian motion
#     at rest would render as noise.
#   - Trails are decorative ONLY. Sim state is read-only and never
#     mutated by this renderer.

const TILE_SIZE := 16
const SPRITE_ORIGIN_X := 448
const SPRITE_ORIGIN_Y := 28
const TRAIL_LENGTH := 16   # ring-buffer history depth per agent
const TRAIL_WIDTH := 2.0
const TRAIL_ALPHA := 0.4
const Z_TRAIL := 2          # above terrain (0), below agents

# state_tag → trail colour. Idle (0) has no entry because Idle agents
# are skipped before lookup.
const TRAIL_COLOR_SEEKING: Color = Color(1.0, 0.65, 0.15, TRAIL_ALPHA)         # orange-yellow
const TRAIL_COLOR_CONSUMING_AGENT: Color = Color(1.0, 0.40, 0.75, TRAIL_ALPHA) # pink
const TRAIL_COLOR_CONSUMING_OTHER: Color = Color(0.30, 0.95, 0.35, TRAIL_ALPHA) # green

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
	if not (ids is PackedInt64Array and xs is PackedInt32Array \
			and ys is PackedInt32Array and states is PackedByteArray \
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
	# Defensive: unknown tag → transparent (no trail).
	return Color(0, 0, 0, 0)
