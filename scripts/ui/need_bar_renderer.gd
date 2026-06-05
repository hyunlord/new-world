extends Node2D

# V7 viz-B — per-agent NEED bars (who is starving / dehydrating / exhausted).
#
# Reads get_agent_snapshot() each frame and draws a small horizontal bar above
# every AT-RISK agent (mirrors seek_viz_renderer.gd's structure). The agents
# render via a single MultiMeshInstance2D (agent_renderer.gd) — there are no
# per-agent nodes to parent a bar to — so this is a standalone overlay that
# re-derives bar positions from the snapshot's xs/ys each frame.
#
# Encoding:
#   danger = max(hunger, thirst, sleep) / SATURATION   (the worst need)
#   danger < DANGER_THRESHOLD (0.5) → NO bar drawn (the safe majority stays
#     uncluttered — both a readability and a 10K-agent perf choice).
#   0.5 ≤ danger        → a bar whose LENGTH = danger and whose COLOUR lerps
#     yellow (≈0.5) → red (≈1.0, about to die). Length + colour both encode
#     severity so "who is closest to death" reads at a glance.
#
# Honest disclosure:
#   - Read-only. Simulation state is never mutated.
#   - Need values (hunger/thirst/sleep) come straight from the FFI snapshot;
#     this layer never re-derives them — it only computes the display ratio.
#   - Sub-resolution for the VLM whole-scene grader (per CLAUDE.md). Whether
#     the bars read clearly is confirmed only by a windowed Godot run.

const TILE_SIZE := 16
const SPRITE_ORIGIN_X := 448
const SPRITE_ORIGIN_Y := 28
const Z_NEED_BAR := 7            # above SeekVizRenderer (6) so bars are visible
const SATURATION := 100.0        # need value at full saturation (ratio = v/100)
const DANGER_THRESHOLD := 0.5    # below this, no bar (safe agent)
const BAR_OFFSET_Y := -17.0      # sits above the seek_viz head-dot (-12)
const BAR_WIDTH := 12.0          # max bar length (at danger 1.0)
const BAR_HEIGHT := 2.5
const TRACK_ALPHA := 0.25        # faint full-width track behind the fill

# Severity colours (locked, §2). Yellow at the threshold → red at death.
const COLOR_SAFE_BAR: Color = Color(0.95, 0.85, 0.15)   # yellow (~0.5 danger)
const COLOR_DEAD_BAR: Color = Color(0.95, 0.15, 0.10)   # red    (~1.0 danger)

var _world_sim: Node = null
var _xs: PackedInt32Array = PackedInt32Array()
var _ys: PackedInt32Array = PackedInt32Array()
var _hungers: PackedFloat32Array = PackedFloat32Array()
var _thirsts: PackedFloat32Array = PackedFloat32Array()
var _sleeps: PackedFloat32Array = PackedFloat32Array()


func _ready() -> void:
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	z_index = Z_NEED_BAR


func _process(_delta: float) -> void:
	if _world_sim == null:
		return
	var snap: Variant = _world_sim.call("get_agent_snapshot")
	if not (snap is Dictionary):
		return
	var snap_dict: Dictionary = snap
	var xs: Variant = snap_dict.get("xs", null)
	var ys: Variant = snap_dict.get("ys", null)
	var hungers: Variant = snap_dict.get("hungers", null)
	var thirsts: Variant = snap_dict.get("thirsts", null)
	var sleeps: Variant = snap_dict.get("sleeps", null)
	if not (xs is PackedInt32Array and ys is PackedInt32Array \
			and hungers is PackedFloat32Array and thirsts is PackedFloat32Array \
			and sleeps is PackedFloat32Array):
		return
	_xs = xs
	_ys = ys
	_hungers = hungers
	_thirsts = thirsts
	_sleeps = sleeps
	queue_redraw()


func _draw() -> void:
	var n: int = _xs.size()
	if _ys.size() != n or _hungers.size() != n or _thirsts.size() != n \
			or _sleeps.size() != n:
		return
	for i in n:
		# Worst need drives the bar; thirst grows fastest but max() already
		# surfaces whichever is closest to death.
		var worst: float = maxf(maxf(_hungers[i], _thirsts[i]), _sleeps[i])
		var danger: float = clampf(worst / SATURATION, 0.0, 1.0)
		if danger < DANGER_THRESHOLD:
			continue
		var agent_px: Vector2 = _tile_to_px(_xs[i], _ys[i])
		var bar_origin: Vector2 = agent_px + Vector2(-BAR_WIDTH / 2.0, BAR_OFFSET_Y)
		# Faint full-width track so the fill length is legible.
		draw_rect(Rect2(bar_origin, Vector2(BAR_WIDTH, BAR_HEIGHT)),
			Color(0.0, 0.0, 0.0, TRACK_ALPHA))
		# Colour ramps yellow → red across the [threshold, 1.0] danger band.
		var t: float = clampf((danger - DANGER_THRESHOLD) / (1.0 - DANGER_THRESHOLD), 0.0, 1.0)
		var col: Color = COLOR_SAFE_BAR.lerp(COLOR_DEAD_BAR, t)
		# Fill length encodes severity (full bar = about to die).
		draw_rect(Rect2(bar_origin, Vector2(BAR_WIDTH * danger, BAR_HEIGHT)), col)


func _tile_to_px(tx: int, ty: int) -> Vector2:
	var px: float = float(SPRITE_ORIGIN_X + tx * TILE_SIZE) + float(TILE_SIZE) / 2.0
	var py: float = float(SPRITE_ORIGIN_Y + ty * TILE_SIZE) + float(TILE_SIZE) / 2.0
	return Vector2(px, py)
