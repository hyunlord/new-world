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

# V7 slice 2-5b — carry indicator (Food being hauled). Deliberately DISTINCT
# from the seek head-dot (a circle ABOVE the agent, red/blue/amber by need) and
# the need-bars: a small filled SQUARE BESIDE the agent in a grain/wheat tint, so
# the two overlays never collide or read as the same thing. Drawn only for agents
# whose Inventory Food > 0 (carried_foods[i] > 0).
const CARRY_OFFSET: Vector2 = Vector2(9.0, -1.0)   # beside the sprite, not above
const CARRY_HALF: float = 2.5                       # half-side of the 5px glyph
const COLOR_CARRY: Color = Color(0.85, 0.72, 0.30)  # grain/wheat — not the red seek-Food dot
const CARRY_FILL_ALPHA: float = 0.85                # >= 0.15 UI minimum
const CARRY_STROKE_ALPHA: float = 1.0               # >= 0.40 UI minimum
const CARRY_STROKE_W: float = 1.0

var _world_sim: Node = null
var _xs: PackedInt32Array = PackedInt32Array()
var _ys: PackedInt32Array = PackedInt32Array()
var _states: PackedByteArray = PackedByteArray()
var _seek_kinds: PackedByteArray = PackedByteArray()
var _target_xs: PackedInt32Array = PackedInt32Array()
var _target_ys: PackedInt32Array = PackedInt32Array()
var _carried_foods: PackedInt32Array = PackedInt32Array()  # slice 2-5b (additive)


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
	# slice 2-5b — additive carry array; defensive so an older/short snapshot
	# omitting it disables only the carry glyph, never the head-dots/goal-lines.
	var carried_foods: Variant = snap_dict.get("carried_foods", null)
	if carried_foods is PackedInt32Array:
		_carried_foods = carried_foods
	else:
		_carried_foods = PackedInt32Array()
	queue_redraw()


func _draw() -> void:
	var n: int = _xs.size()
	if _ys.size() != n or _states.size() != n or _seek_kinds.size() != n \
			or _target_xs.size() != n or _target_ys.size() != n:
		return
	# slice 2-5b — carry glyph only when its parallel array matches (additive,
	# never blocks the head-dot/goal-line overlays).
	var has_carry: bool = _carried_foods.size() == n
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
		# slice 2-5b — carry indicator: a grain-tinted square BESIDE the agent
		# (distinct anchor + shape from the head-dot circle ABOVE) whenever the
		# agent is hauling Food. Reconciled every frame from the snapshot — it
		# disappears the frame carried_food returns to 0 (deposit complete).
		if has_carry and _carried_foods[i] > 0:
			var carry_c: Vector2 = agent_px + CARRY_OFFSET
			var carry_r: Rect2 = Rect2(
				carry_c - Vector2(CARRY_HALF, CARRY_HALF),
				Vector2(CARRY_HALF * 2.0, CARRY_HALF * 2.0))
			draw_rect(carry_r, Color(COLOR_CARRY.r, COLOR_CARRY.g, COLOR_CARRY.b, CARRY_FILL_ALPHA), true)
			draw_rect(carry_r, Color(COLOR_CARRY.r, COLOR_CARRY.g, COLOR_CARRY.b, CARRY_STROKE_ALPHA), false, CARRY_STROKE_W)


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
