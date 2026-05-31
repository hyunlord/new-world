extends Node2D

# V7 Section 16-ε — state head-markers + goal lines.
#
# Reads get_agent_snapshot() each frame and draws two read-only overlays
# (mirrors activity_trail_renderer.gd's structure):
#   A — a small colored head-dot above each non-Idle agent, keyed to what
#       it is doing (Seeking-Food/Water/Sleep = red/blue/amber via
#       seek_kind; Consuming non-Agent = white via state_tag == 3).
#   B — a faint goal-line from each resource-Seeking agent to its
#       SeekTarget tile, colored by need kind.
#
# Honest disclosure:
#   - Read-only. Simulation state is never mutated by this renderer.
#   - Sub-resolution for the VLM whole-scene grader (per CLAUDE.md). Whether
#     the markers/lines read clearly at 64 agents is confirmed only by a
#     windowed Godot run.
#   - state_tag (0-3) and the seek_kind/target arrays come straight from the
#     FFI snapshot; this layer never re-derives them.

const TILE_SIZE := 16
const SPRITE_ORIGIN_X := 448
const SPRITE_ORIGIN_Y := 28
const Z_SEEK_VIZ := 6          # above AgentRenderer so markers are visible
const HEAD_OFFSET_Y := -12.0   # head-dot sits above the agent sprite
const HEAD_RADIUS := 3.5
const LINE_WIDTH := 1.5
const LINE_ALPHA := 0.35       # faint goal line — never obscures sprites

# Need-kind colours (locked, §2). seek_kind: 1=Food, 2=Water, 3=Sleep.
const COLOR_FOOD: Color = Color(0.9, 0.2, 0.2)
const COLOR_WATER: Color = Color(0.2, 0.5, 1.0)
const COLOR_SLEEP: Color = Color(0.95, 0.75, 0.2)
const COLOR_GRAY: Color = Color(0.6, 0.6, 0.6)

var _world_sim: Node = null
var _xs: PackedInt32Array = PackedInt32Array()
var _ys: PackedInt32Array = PackedInt32Array()
var _states: PackedByteArray = PackedByteArray()
var _seek_kinds: PackedByteArray = PackedByteArray()
var _target_xs: PackedInt32Array = PackedInt32Array()
var _target_ys: PackedInt32Array = PackedInt32Array()


func _ready() -> void:
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	z_index = Z_SEEK_VIZ


func _process(_delta: float) -> void:
	if _world_sim == null:
		return
	var snap: Variant = _world_sim.call("get_agent_snapshot")
	if not (snap is Dictionary):
		return
	var snap_dict: Dictionary = snap
	var xs: Variant = snap_dict.get("xs", null)
	var ys: Variant = snap_dict.get("ys", null)
	var states: Variant = snap_dict.get("states", null)
	var seek_kinds: Variant = snap_dict.get("seek_kinds", null)
	var target_xs: Variant = snap_dict.get("target_xs", null)
	var target_ys: Variant = snap_dict.get("target_ys", null)
	if not (xs is PackedInt32Array and ys is PackedInt32Array \
			and states is PackedByteArray and seek_kinds is PackedByteArray \
			and target_xs is PackedInt32Array and target_ys is PackedInt32Array):
		return
	_xs = xs
	_ys = ys
	_states = states
	_seek_kinds = seek_kinds
	_target_xs = target_xs
	_target_ys = target_ys
	queue_redraw()


func _draw() -> void:
	var n: int = _xs.size()
	if _ys.size() != n or _states.size() != n or _seek_kinds.size() != n \
			or _target_xs.size() != n or _target_ys.size() != n:
		return
	for i in n:
		var agent_px: Vector2 = _tile_to_px(_xs[i], _ys[i])
		var tag: int = int(_states[i])
		var kind: int = int(_seek_kinds[i])
		# B — goal line for resource-Seeking agents that carry a target.
		if tag == 1 and _target_xs[i] >= 0:
			var target_px: Vector2 = _tile_to_px(_target_xs[i], _target_ys[i])
			draw_line(agent_px, target_px, _kind_color(kind, LINE_ALPHA), LINE_WIDTH)
		# A — head dot above the agent.
		var head: Vector2 = agent_px + Vector2(0.0, HEAD_OFFSET_Y)
		if tag == 1:
			draw_circle(head, HEAD_RADIUS, _kind_color(kind, 1.0))
		elif tag == 3:
			draw_circle(head, HEAD_RADIUS, Color.WHITE)
		# Idle (0) / Consuming-Agent (2) → no head-dot.


func _tile_to_px(tx: int, ty: int) -> Vector2:
	var px: float = float(SPRITE_ORIGIN_X + tx * TILE_SIZE) + float(TILE_SIZE) / 2.0
	var py: float = float(SPRITE_ORIGIN_Y + ty * TILE_SIZE) + float(TILE_SIZE) / 2.0
	return Vector2(px, py)


func _kind_color(kind: int, a: float) -> Color:
	var c: Color = COLOR_GRAY
	if kind == 1:
		c = COLOR_FOOD
	elif kind == 2:
		c = COLOR_WATER
	elif kind == 3:
		c = COLOR_SLEEP
	return Color(c.r, c.g, c.b, a)
