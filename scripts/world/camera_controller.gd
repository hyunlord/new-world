# camera_controller.gd (Camera2D in scenes/main.tscn): the M1 colony camera and the simulation speed keys.
# Integer zoom only (1×/2×/3×/4×, default 2×) with the wheel, `-` and `=`; pan with WASD / arrows and a
#   middle-button drag; starts centred on the camp hearth, and Home or H centres on it again (C is the chronicle,
#   order_bar.gd shows the hint); bounds = the map plus a 4-tile margin. Keys 1–4 set
#   the speed 1×–4× (12/24/36/48 ticks per second, ADR 0004); Space (or Pause) pauses by disabling the WorldSim
#   node (M2: P opens the work-priority table, priority_panel.gd). M5: while the inspector shows a living person, D
#   drafts them (order_tool.gd), so D does not pan then (the arrow keys still do).
# Bridge methods: set_sim_speed (direct); via SimReads get_world_info, get_building_snapshot; signal world_reset.
extends Camera2D

const View = preload("res://scripts/world/colony/view.gd")
const ZOOM_STEPS: Array[float] = [1.0, 2.0, 3.0, 4.0]
const ZOOM_DEFAULT: float = 2.0
const PAN_PIXELS_PER_SECOND: float = 900.0
const MARGIN_TILES: int = 4
const SPEED_KEYS: Dictionary = {KEY_1: 1.0, KEY_2: 2.0, KEY_3: 3.0, KEY_4: 4.0}
# Centre on the camp: Home, or H for keyboards without a Home key (C opens the chronicle).
const HOME_KEYS: Array[Key] = [KEY_HOME, KEY_H]
const TICKS_PER_SECOND_AT_1X: int = 12
## M2: Space pauses (P became the priority table).
const PAUSE_KEYS: Array[Key] = [KEY_SPACE, KEY_PAUSE]

var _world_sim: Node = null
var _paused: bool = false
var _speed: float = 1.0
var _dragging: bool = false
var _center_pending: bool = true


func _ready() -> void:
	zoom = Vector2.ONE * ZOOM_DEFAULT
	position_smoothing_enabled = false
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	if _world_sim != null:
		_world_sim.connect("world_reset", _on_world_reset)
		_world_sim.call("set_sim_speed", _speed)
	_on_world_reset()


## Current speed multiplier (1–4).
func get_speed() -> float:
	return _speed


## Whether the simulation is paused.
func is_paused() -> bool:
	return _paused


## Ticks fed per real second at the current speed (0 when paused).
func ticks_per_second() -> int:
	return 0 if _paused else int(_speed) * TICKS_PER_SECOND_AT_1X


func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventMouseButton:
		var button: InputEventMouseButton = event
		if button.button_index == MOUSE_BUTTON_WHEEL_UP and button.pressed:
			step_zoom(1)
			get_viewport().set_input_as_handled()
		elif button.button_index == MOUSE_BUTTON_WHEEL_DOWN and button.pressed:
			step_zoom(-1)
			get_viewport().set_input_as_handled()
		elif button.button_index == MOUSE_BUTTON_MIDDLE:
			_dragging = button.pressed
			get_viewport().set_input_as_handled()
	elif event is InputEventMouseMotion and _dragging:
		var motion: InputEventMouseMotion = event
		position -= motion.relative / zoom.x
		_clamp_position()
		get_viewport().set_input_as_handled()
	elif event is InputEventKey and event.pressed and not event.echo:
		var key: InputEventKey = event
		if key.keycode in PAUSE_KEYS:
			_toggle_pause()
			get_viewport().set_input_as_handled()
		elif key.keycode in [KEY_MINUS, KEY_KP_SUBTRACT]:
			step_zoom(-1)
			get_viewport().set_input_as_handled()
		elif key.keycode in [KEY_EQUAL, KEY_KP_ADD]:
			step_zoom(1)
			get_viewport().set_input_as_handled()
		elif key.keycode in HOME_KEYS:
			center_on_camp()
			get_viewport().set_input_as_handled()
		elif SPEED_KEYS.has(key.keycode):
			set_speed(float(SPEED_KEYS[key.keycode]))
			get_viewport().set_input_as_handled()


## Set the speed multiplier (1–4) on the bridge.
func set_speed(speed: float) -> void:
	_speed = clampf(speed, 1.0, 4.0)
	if _world_sim != null:
		_world_sim.call("set_sim_speed", _speed)


## Step the integer zoom ladder by `direction` (+1 in, -1 out).
func step_zoom(direction: int) -> void:
	var index: int = ZOOM_STEPS.find(zoom.x)
	if index < 0:
		index = 1
	index = clampi(index + direction, 0, ZOOM_STEPS.size() - 1)
	zoom = Vector2.ONE * ZOOM_STEPS[index]
	_clamp_position()


func _toggle_pause() -> void:
	_paused = not _paused
	if _world_sim != null:
		_world_sim.process_mode = Node.PROCESS_MODE_DISABLED if _paused else Node.PROCESS_MODE_INHERIT


func _process(delta: float) -> void:
	if _center_pending:
		_center_on_camp()
	var direction: Vector2 = Vector2.ZERO
	if Input.is_key_pressed(KEY_A) or Input.is_key_pressed(KEY_LEFT):
		direction.x -= 1.0
	if (Input.is_key_pressed(KEY_D) and not _person_selected()) or Input.is_key_pressed(KEY_RIGHT):
		direction.x += 1.0
	if Input.is_key_pressed(KEY_W) or Input.is_key_pressed(KEY_UP):
		direction.y -= 1.0
	if Input.is_key_pressed(KEY_S) or Input.is_key_pressed(KEY_DOWN):
		direction.y += 1.0
	if direction != Vector2.ZERO:
		position += direction * PAN_PIXELS_PER_SECOND * delta / zoom.x
		_clamp_position()


## Keep the view inside the map plus the margin, and on whole screen pixels.
func _clamp_position() -> void:
	var world_info: Dictionary = SimReads.read(_world_sim, "get_world_info") if _world_sim != null else {}
	var w: float = float(int(world_info.get("width", 192)) * View.TILE)
	var h: float = float(int(world_info.get("height", 192)) * View.TILE)
	var margin: float = float(MARGIN_TILES * View.TILE)
	limit_left = int(-margin)
	limit_top = int(-margin)
	limit_right = int(w + margin)
	limit_bottom = int(h + margin)
	var half: Vector2 = get_viewport_rect().size / (2.0 * zoom.x)
	position.x = clampf(position.x, -margin + half.x, maxf(-margin + half.x, w + margin - half.x))
	position.y = clampf(position.y, -margin + half.y, maxf(-margin + half.y, h + margin - half.y))
	position = (position * zoom.x).round() / zoom.x


## Centre on the camp hearth now (the Home/H key; also used by tools).
func center_on_camp() -> void:
	_center_on_camp()


## Centre on the camp hearth (building kind 0); stays pending until the hearth exists.
func _center_on_camp() -> void:
	if _world_sim == null:
		return
	var buildings: Dictionary = SimReads.read(_world_sim, "get_building_snapshot")
	var kinds: PackedByteArray = buildings.get("kinds", PackedByteArray())
	var xs: PackedInt32Array = buildings.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = buildings.get("ys", PackedInt32Array())
	for i: int in mini(kinds.size(), mini(xs.size(), ys.size())):
		if kinds[i] == 0:
			center_on_tile(Vector2i(xs[i], ys[i]))
			_center_pending = false
			return


## Centre the view on tile `t`.
func center_on_tile(t: Vector2i) -> void:
	position = View.tile_to_world(t) + Vector2(View.TILE, View.TILE) * 0.5
	_clamp_position()


func _on_world_reset() -> void:
	_center_pending = true
	if _world_sim != null:
		_world_sim.call("set_sim_speed", _speed)


## M5: whether the inspector shows a living person (D drafts them instead of panning).
func _person_selected() -> bool:
	var inspector: Node = get_node_or_null("/root/Main/UI/AgentInspectorPanel")
	return inspector != null and (inspector as Control).visible and int(inspector.call("get_selected_entity_bits")) >= 0
