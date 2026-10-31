# world_marker.gd (ColonyView/WorldMarker in scenes/main.tscn): at most 1 marker (2 for a two-place
#   task) on the tile from get_current_task. Never more than 2 markers on screen. Pulsing ring drawn
#   with _draw(). Updates per tick, clears on world_reset.
# Bridge methods: via SimReads get_current_task; signal world_reset.
extends Node2D

const TILE: float = 64.0
const RING_RADIUS: float = 24.0
const RING_WIDTH: float = 2.5
const RING_COLOR := Color("e8c46e")
const PULSE_SPEED: float = 2.5
const PULSE_RANGE: float = 0.35

var _world_sim: Node = null
var _marker_tile: Vector2i = Vector2i(-1, -1)
var _marker_count: int = 0
var _task_tick: int = -1
var _phase: float = 0.0


func _ready() -> void:
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	if _world_sim != null:
		_world_sim.connect("world_reset", _world_reset)
	z_index = 100


func _process(delta: float) -> void:
	if _world_sim == null:
		return
	var tick: int = int(SimReads.read(_world_sim, "get_time_snapshot").get("tick", -1))
	if tick != _task_tick:
		_task_tick = tick
		_refresh_marker()
	if _marker_count > 0:
		_phase += delta * PULSE_SPEED
		queue_redraw()


func _draw() -> void:
	if _marker_count <= 0 or _marker_tile.x < 0:
		return
	var alpha: float = 0.65 + PULSE_RANGE * sin(_phase)
	var color: Color = RING_COLOR
	color.a = alpha
	var centre: Vector2 = Vector2(_marker_tile) * TILE + Vector2(32, 32)
	draw_arc(centre, RING_RADIUS, 0.0, TAU, 32, color, RING_WIDTH, true)
	# A smaller inner dot.
	var dot_color: Color = color
	dot_color.a = alpha * 0.5
	draw_circle(centre, 4.0, dot_color)


func _refresh_marker() -> void:
	var task: Dictionary = SimReads.read(_world_sim, "get_current_task")
	var has_marker: bool = bool(task.get("has_marker", false))
	if has_marker:
		_marker_tile = Vector2i(int(task.get("marker_x", -1)), int(task.get("marker_y", -1)))
		_marker_count = 1
	else:
		_marker_tile = Vector2i(-1, -1)
		_marker_count = 0
	queue_redraw()


func _world_reset() -> void:
	_task_tick = -1
	_marker_tile = Vector2i(-1, -1)
	_marker_count = 0
	queue_redraw()


## The current marker count (for tests: must be ≤ 2).
func marker_count() -> int:
	return _marker_count


## The marker tile (for tests).
func marker_tile() -> Vector2i:
	return _marker_tile
