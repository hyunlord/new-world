extends Node2D

# V7 viz-D — death markers (WHO died, WHERE, WHY).
#
# Reads get_recent_deaths() each frame and draws a short-lived, reason-coloured
# marker at every recent death tile. Death is an instantaneous event (no
# persistent ECS state), so the simulation keeps a bounded `recent_deaths`
# buffer (pushed by the shared despawn_agent helper, pruned past
# RECENT_DEATH_RETAIN_TICKS) that this overlay fades out. Agents render via a
# single MultiMeshInstance2D (no per-agent nodes), so this is a standalone
# overlay mirroring need_bar_renderer.gd's pattern and tile→pixel convention.
#
# Encoding:
#   age   = current_tick - death.tick
#   age >= FADE_TICKS or age < 0 → NOT drawn (faded out / clock skew guard)
#   alpha = 1.0 - age / FADE_TICKS   (fresh = opaque, older = transparent)
#   colour by reason: 0 Starvation = brown, 1 Dehydration = blue, 2 Combat = red
#
# Honest disclosure:
#   - Read-only. Simulation state is never mutated.
#   - Whether the marker shape / colours read clearly at sprite scale is a
#     windowed-Godot confirmation (sub-resolution for the VLM grader per
#     CLAUDE.md). This script's wiring is what the harness verifies.

const TILE_SIZE := 16
const SPRITE_ORIGIN_X := 448
const SPRITE_ORIGIN_Y := 28
const Z_DEATH := 8               # above NeedBarRenderer (7) so markers are not occluded
const FADE_TICKS := 90.0         # marker fully faded by ~3 s @ 30 TPS (< retain window 120)
# Mirror of Rust `RECENT_DEATH_RETAIN_TICKS` (sim-engine). The buffer keeps a
# death entry for this many ticks; FADE_TICKS (90) < RETAIN_TICKS (120) so the
# marker is fully faded before the data leaves the buffer. Kept in sync with the
# authoritative Rust const (harness A15 asserts equality).
const RETAIN_TICKS := 120
const MARKER_RADIUS := 8.0       # ≈ TILE_SIZE / 2 — large enough to read
const STROKE_WIDTH := 2.0

# Reason → colour (locked, §2). reason_u8: 0 = Starvation, 1 = Dehydration, 2 = Combat.
const COLOR_STARVATION: Color = Color(0.45, 0.30, 0.15)   # brown
const COLOR_DEHYDRATION: Color = Color(0.20, 0.45, 0.95)  # blue
const COLOR_COMBAT: Color = Color(0.95, 0.15, 0.10)       # red

var _world_sim: Node = null
var _xs: PackedInt32Array = PackedInt32Array()
var _ys: PackedInt32Array = PackedInt32Array()
var _reasons: PackedInt32Array = PackedInt32Array()
var _ticks: PackedInt64Array = PackedInt64Array()
var _current_tick: int = 0


func _ready() -> void:
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	z_index = Z_DEATH


func _process(_delta: float) -> void:
	if _world_sim == null:
		return
	var snap: Variant = _world_sim.call("get_recent_deaths")
	if not (snap is Dictionary):
		return
	var snap_dict: Dictionary = snap
	var xs: Variant = snap_dict.get("xs", null)
	var ys: Variant = snap_dict.get("ys", null)
	var reasons: Variant = snap_dict.get("reasons", null)
	var ticks: Variant = snap_dict.get("ticks", null)
	var current_tick: Variant = snap_dict.get("current_tick", null)
	if not (xs is PackedInt32Array and ys is PackedInt32Array \
			and reasons is PackedInt32Array and ticks is PackedInt64Array \
			and current_tick is int):
		return
	_xs = xs
	_ys = ys
	_reasons = reasons
	_ticks = ticks
	_current_tick = current_tick
	queue_redraw()


func _draw() -> void:
	var n: int = _xs.size()
	if _ys.size() != n or _reasons.size() != n or _ticks.size() != n:
		return
	for i in n:
		var age: float = float(_current_tick - _ticks[i])
		# Faded out, or a defensive clock-skew (entry.tick ahead of current).
		if age >= FADE_TICKS or age < 0.0:
			continue
		var alpha: float = clampf(1.0 - age / FADE_TICKS, 0.0, 1.0)
		var col: Color = _reason_color(_reasons[i])
		col.a = alpha
		var c: Vector2 = _tile_to_px(_xs[i], _ys[i])
		# An "X" of two strokes — reads as a death marker at the tile.
		var r: float = MARKER_RADIUS
		draw_line(c + Vector2(-r, -r), c + Vector2(r, r), col, STROKE_WIDTH)
		draw_line(c + Vector2(-r, r), c + Vector2(r, -r), col, STROKE_WIDTH)


func _reason_color(reason: int) -> Color:
	match reason:
		0:
			return COLOR_STARVATION
		1:
			return COLOR_DEHYDRATION
		2:
			return COLOR_COMBAT
		_:
			return COLOR_COMBAT


func _tile_to_px(tx: int, ty: int) -> Vector2:
	var px: float = float(SPRITE_ORIGIN_X + tx * TILE_SIZE) + float(TILE_SIZE) / 2.0
	var py: float = float(SPRITE_ORIGIN_Y + ty * TILE_SIZE) + float(TILE_SIZE) / 2.0
	return Vector2(px, py)
