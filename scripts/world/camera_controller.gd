# camera_controller.gd (Camera2D in scenes/main.tscn): the colony camera and the simulation speed keys.
# M-ART2: zoom steps from assets/v8art/render_params.json `camera.zooms` (¼×, ½×, 1×, 2×; default `camera.default_zoom`
#   1×) with the wheel, `-` and `=`; below 1× the people and animals become LOD markers (agent_layer.gd); pan with
#   WASD / arrows (`camera.pan_px_per_second` screen pixels a second, so the same screen speed at every zoom) and a
#   middle-button drag; in debug builds F12 re-reads render_params.json (ArtData.reload_params) and shows the toast
#   "렌더 설정 다시 읽음" (UI_RENDER_PARAMS_RELOADED) for two seconds; starts centred on the camp hearth, and Home or H centres on it again (C is the chronicle,
#   order_bar.gd shows the hint); bounds = the map plus a 4-tile margin. Keys 1–4 set
#   the speed 1×–4× (12/24/36/48 ticks per second, ADR 0004); Space (or Pause) pauses by disabling the WorldSim
#   node (M2: P opens the work-priority table, priority_panel.gd). M5: while the inspector shows a living person, D
#   drafts them (order_tool.gd), so D does not pan then (the arrow keys still do). M5.5: set_paused() lets the
#   migrant dialog pause the clock when a wave arrives (only at 1×, migrant_panel.gd).
# Bridge methods: set_sim_speed (direct); via SimReads get_world_info, get_building_snapshot; signal world_reset.
extends Camera2D

const View = preload("res://scripts/world/colony/view.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
const TOAST_SECONDS: float = 2.0
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
var _toast: Label = null
var _toast_left: float = 0.0


## The zoom ladder (camera zoom factors, ascending) from render_params.json.
func zoom_steps() -> Array[float]:
	var out: Array[float] = []
	var raw: Variant = ArtData.param("camera.zooms", [0.25, 0.5, 1.0, 2.0])
	if raw is Array:
		for z: Variant in raw:
			out.append(clampf(float(z), 0.05, 8.0))
	if out.is_empty():
		out.append(1.0)
	out.sort()
	return out


func _ready() -> void:
	zoom = Vector2.ONE * float(ArtData.param("camera.default_zoom", 1.0))
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
		elif key.keycode == KEY_F12 and OS.is_debug_build():
			reload_render_params()
			get_viewport().set_input_as_handled()


## Re-read render_params.json and show the toast (F12 in debug builds). Returns whether the file parsed.
func reload_render_params() -> bool:
	var ok: bool = ArtData.reload_params()
	var locale: Node = get_node_or_null("/root/Locale")
	var text: String = str(locale.call("ltr", "UI_RENDER_PARAMS_RELOADED" if ok else "UI_RENDER_PARAMS_BROKEN")) if locale != null else ""
	var ui: Node = get_node_or_null("/root/Main/UI")
	if ui != null:
		if _toast == null:
			_toast = Label.new()
			_toast.name = "RenderToast"
			_toast.add_theme_font_size_override("font_size", 24)
			_toast.position = Vector2(24, 140)
			_toast.mouse_filter = Control.MOUSE_FILTER_IGNORE
			var back: StyleBoxFlat = StyleBoxFlat.new()
			back.bg_color = Color(0.094, 0.082, 0.075, 0.92)
			back.set_content_margin_all(8)
			_toast.add_theme_stylebox_override("normal", back)
			ui.add_child(_toast)
		_toast.text = text
		_toast.visible = true
		_toast_left = TOAST_SECONDS
	step_zoom(0)
	return ok


## The toast's text while it shows ("" when hidden; for tests).
func toast_text() -> String:
	return _toast.text if _toast != null and _toast.visible else ""


## Set the speed multiplier (1–4) on the bridge.
func set_speed(speed: float) -> void:
	_speed = clampf(speed, 1.0, 4.0)
	if _world_sim != null:
		_world_sim.call("set_sim_speed", _speed)


## Step the zoom ladder by `direction` (+1 in, -1 out; 0 snaps to the nearest step).
func step_zoom(direction: int) -> void:
	var steps: Array[float] = zoom_steps()
	var index: int = 0
	for k: int in steps.size():
		if absf(steps[k] - zoom.x) < absf(steps[index] - zoom.x):
			index = k
	index = clampi(index + direction, 0, steps.size() - 1)
	zoom = Vector2.ONE * steps[index]
	_clamp_position()


## Pause (`on`) or resume the simulation (as Space does).
func set_paused(on: bool) -> void:
	if on != _paused:
		_toggle_pause()


func _toggle_pause() -> void:
	_paused = not _paused
	if _world_sim != null:
		_world_sim.process_mode = Node.PROCESS_MODE_DISABLED if _paused else Node.PROCESS_MODE_INHERIT


func _process(delta: float) -> void:
	if _toast != null and _toast.visible:
		_toast_left -= delta
		_toast.visible = _toast_left > 0.0
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
		position += direction * float(ArtData.param("camera.pan_px_per_second", 900.0)) * delta / zoom.x
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


## Centre on the camp hearth (the building whose binding gives light); stays pending until it exists.
func _center_on_camp() -> void:
	if _world_sim == null:
		return
	var buildings: Dictionary = SimReads.read(_world_sim, "get_building_snapshot")
	var kinds: PackedByteArray = buildings.get("kinds", PackedByteArray())
	var xs: PackedInt32Array = buildings.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = buildings.get("ys", PackedInt32Array())
	for i: int in mini(kinds.size(), mini(xs.size(), ys.size())):
		if bool(ArtData.binding("buildings", kinds[i]).get("light", false)):
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
