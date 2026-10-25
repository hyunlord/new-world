# camera_controller.gd (Camera2D in scenes/main.tscn): the colony camera and the simulation speed keys.
# M-ART2: zoom steps from assets/v8art/render_params.json `camera.zooms` (¼×, ½×, 1×, 2×; default `camera.default_zoom`
#   1×) with the wheel, `-` and `=`; below 1× the people and animals become LOD markers (agent_layer.gd); pan with
#   WASD / arrows (`camera.pan_px_per_second` screen pixels a second, so the same screen speed at every zoom) and a
#   middle-button drag; M-UX adds trackpad pan/pinch, cursor-anchored 120 ms zoom, left map drag and right drag.
#   Click actions are deferred until release so panning never selects, cancels or submits a draft order.
#   In debug builds F12 re-reads render_params.json (ArtData.reload_params) and shows the toast
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
const ZOOM_SECONDS: float = 0.12
const DRAG_THRESHOLD: float = 6.0
const PAN_GESTURE_PIXELS: float = 32.0
const PINCH_THRESHOLD: float = 1.41421356
const GESTURE_SOURCE_META: StringName = &"_worldsim_gesture_source_id"
const WINDOW_ASPECT_X: int = 16
const WINDOW_ASPECT_Y: int = 9
const EDGE_PAN_MARGIN: int = 16

signal camera_activity(action: String)

var _world_sim: Node = null
var _paused: bool = false
var _speed: float = 1.0
var _pointer_button: MouseButton = MOUSE_BUTTON_NONE
var _pointer_origin: Vector2 = Vector2.ZERO
var _pointer_last: Vector2 = Vector2.ZERO
var _pointer_dragged: bool = false
var _pointer_alt: bool = false
var _zoom_from: float = 1.0
var _zoom_target: float = 1.0
var _zoom_elapsed: float = ZOOM_SECONDS
var _zoom_anchor_screen: Vector2 = Vector2.ZERO
var _zoom_anchor_world: Vector2 = Vector2.ZERO
var _pinch_scale: float = 1.0
var _gesture_frame: int = -1
var _gesture_ids: Dictionary[int, bool] = {}
var _edge_pan: bool = false
var _invert_pan: bool = false
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
	_zoom_target = zoom.x
	position_smoothing_enabled = false
	fit_window()
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	get_window().focus_exited.connect(_release_pointer)
	get_window().window_input.connect(_stamp_gesture_source)
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


## Godot 4.6 dispatches a gesture twice; viewport transforms create new objects
## but preserve metadata. Record the original identity before that transform.
func _stamp_gesture_source(event: InputEvent) -> void:
	if event is InputEventGesture:
		event.set_meta(GESTURE_SOURCE_META, event.get_instance_id())


func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventGesture:
		var frame: int = Engine.get_process_frames()
		if frame != _gesture_frame:
			_gesture_frame = frame
			_gesture_ids.clear()
		var source_id: int = int(event.get_meta(GESTURE_SOURCE_META, event.get_instance_id()))
		if _gesture_ids.has(source_id):
			get_viewport().set_input_as_handled()
			return
		_gesture_ids[source_id] = true
	if event is InputEventPanGesture:
		pan_screen((event as InputEventPanGesture).delta * PAN_GESTURE_PIXELS)
		get_viewport().set_input_as_handled()
	elif event is InputEventMagnifyGesture:
		var gesture: InputEventMagnifyGesture = event
		if gesture.factor > 0.0 and is_finite(gesture.factor):
			_pinch_scale *= gesture.factor
			if _pinch_scale >= PINCH_THRESHOLD or _pinch_scale <= 1.0 / PINCH_THRESHOLD:
				step_zoom(1 if _pinch_scale > 1.0 else -1, gesture.position)
				_pinch_scale = 1.0
		get_viewport().set_input_as_handled()
	elif event is InputEventMouseButton:
		var button: InputEventMouseButton = event
		if button.button_index == MOUSE_BUTTON_WHEEL_UP and button.pressed:
			step_zoom(1, button.position)
			get_viewport().set_input_as_handled()
		elif button.button_index == MOUSE_BUTTON_WHEEL_DOWN and button.pressed:
			step_zoom(-1, button.position)
			get_viewport().set_input_as_handled()
		elif button.pressed and (_can_pan_with(button.button_index) or (button.button_index == MOUSE_BUTTON_LEFT and button.alt_pressed)):
			_pointer_button = button.button_index
			_pointer_alt = button.alt_pressed
			_pointer_origin = button.position
			_pointer_last = button.position
			_pointer_dragged = false
			get_viewport().set_input_as_handled()
	elif event is InputEventKey and event.pressed and not event.echo:
		var key: InputEventKey = event
		if key.keycode == KEY_F and key.ctrl_pressed and key.meta_pressed:
			toggle_fullscreen()
			get_viewport().set_input_as_handled()
		elif key.keycode in PAUSE_KEYS:
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


## Continue a map gesture even when its pointer crosses a GUI control.
func _input(event: InputEvent) -> void:
	if _pointer_button == MOUSE_BUTTON_NONE:
		return
	if event is InputEventMouseMotion:
		var point: Vector2 = (event as InputEventMouseMotion).position
		if not _pointer_dragged and point.distance_to(_pointer_origin) >= DRAG_THRESHOLD:
			_pointer_dragged = true
			pan_screen(point - _pointer_origin)
		elif _pointer_dragged:
			pan_screen(point - _pointer_last)
		_pointer_last = point
		get_viewport().set_input_as_handled()
	elif event is InputEventMouseButton:
		var button: InputEventMouseButton = event
		if button.button_index == _pointer_button and not button.pressed:
			var clicked: bool = not _pointer_dragged and button.position.distance_to(_pointer_origin) < DRAG_THRESHOLD
			_pointer_button = MOUSE_BUTTON_NONE
			if clicked and get_viewport().gui_get_hovered_control() == null:
				var tool: Node = get_node_or_null("/root/Main/ColonyView/OrderTool")
				if tool != null:
					tool.call("map_click", button.button_index, button.position, _pointer_alt)
			get_viewport().set_input_as_handled()


func _can_pan_with(button: MouseButton) -> bool:
	if button in [MOUSE_BUTTON_RIGHT, MOUSE_BUTTON_MIDDLE]:
		return true
	var tool: Node = get_node_or_null("/root/Main/ColonyView/OrderTool")
	return button == MOUSE_BUTTON_LEFT and (tool == null or int(tool.get("mode")) == 0)


func _release_pointer() -> void:
	_pointer_button = MOUSE_BUTTON_NONE
	_pointer_dragged = false
	_pinch_scale = 1.0


## A screen-space drag; settings affect gestures, not keyboard direction.
func pan_screen(distance: Vector2) -> void:
	position -= distance * (-1.0 if _invert_pan else 1.0) / zoom.x
	_zoom_anchor_world -= distance * (-1.0 if _invert_pan else 1.0) / zoom.x
	_clamp_position()
	camera_activity.emit("pan")


func set_edge_pan(on: bool) -> void:
	_edge_pan = on


func set_invert_pan(on: bool) -> void:
	_invert_pan = on


## Fit the outer window inside the current screen while preserving the design viewport.
func fit_window() -> void:
	if DisplayServer.get_name() == "headless" or get_window().mode != Window.MODE_WINDOWED:
		return
	var usable: Rect2i = DisplayServer.screen_get_usable_rect(get_window().current_screen)
	var decorations: Vector2i = DisplayServer.window_get_size_with_decorations() - DisplayServer.window_get_size()
	var available: Vector2i = usable.size - Vector2i(maxi(0, decorations.x), maxi(0, decorations.y))
	var unit: int = maxi(1, mini(120, int(minf(float(available.x) / WINDOW_ASPECT_X, float(available.y) / WINDOW_ASPECT_Y))))
	get_window().size = Vector2i(WINDOW_ASPECT_X, WINDOW_ASPECT_Y) * unit
	get_window().position = usable.position + Vector2i(Vector2(usable.size - get_window().size) * 0.5)


func toggle_fullscreen() -> void:
	get_window().mode = Window.MODE_WINDOWED if get_window().mode == Window.MODE_FULLSCREEN else Window.MODE_FULLSCREEN
	if get_window().mode == Window.MODE_WINDOWED:
		fit_window()
	camera_activity.emit("fullscreen")


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
	camera_activity.emit("speed")


## Step the zoom ladder by `direction` (+1 in, -1 out; 0 snaps to the nearest step).
func step_zoom(direction: int, screen_anchor: Vector2 = Vector2(-1, -1)) -> void:
	var steps: Array[float] = zoom_steps()
	var reference: float = _zoom_target if _zoom_elapsed < ZOOM_SECONDS else zoom.x
	var index: int = 0
	for k: int in steps.size():
		if absf(steps[k] - reference) < absf(steps[index] - reference):
			index = k
	index = clampi(index + direction, 0, steps.size() - 1)
	_zoom_from = zoom.x
	_zoom_target = steps[index]
	_zoom_elapsed = 0.0
	_zoom_anchor_screen = get_viewport().get_mouse_position() if screen_anchor.x < 0 else screen_anchor
	_zoom_anchor_world = get_canvas_transform().affine_inverse() * _zoom_anchor_screen
	camera_activity.emit("zoom")


## The selected ladder level, including while the camera animates toward it.
func get_zoom_level() -> float:
	return _zoom_target if _zoom_elapsed < ZOOM_SECONDS else zoom.x


func _animate_zoom(delta: float) -> void:
	if _zoom_elapsed >= ZOOM_SECONDS:
		return
	_zoom_elapsed = minf(ZOOM_SECONDS, _zoom_elapsed + delta)
	zoom = Vector2.ONE * lerpf(_zoom_from, _zoom_target, _zoom_elapsed / ZOOM_SECONDS)
	position = _zoom_anchor_world - (_zoom_anchor_screen - get_viewport_rect().size * 0.5) / zoom.x
	_clamp_position()


## Pause (`on`) or resume the simulation (as Space does).
func set_paused(on: bool) -> void:
	if on != _paused:
		_toggle_pause()


func _toggle_pause() -> void:
	_paused = not _paused
	if _world_sim != null:
		_world_sim.process_mode = Node.PROCESS_MODE_DISABLED if _paused else Node.PROCESS_MODE_INHERIT
	camera_activity.emit("speed")


func _process(delta: float) -> void:
	_animate_zoom(delta)
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
	if _edge_pan and get_viewport().gui_get_hovered_control() == null:
		var cursor: Vector2 = get_viewport().get_mouse_position()
		var viewport_size: Vector2 = get_viewport_rect().size
		if Rect2(Vector2.ZERO, viewport_size).has_point(cursor):
			direction += Vector2(float(cursor.x > viewport_size.x - EDGE_PAN_MARGIN) - float(cursor.x < EDGE_PAN_MARGIN), float(cursor.y > viewport_size.y - EDGE_PAN_MARGIN) - float(cursor.y < EDGE_PAN_MARGIN))
	if direction != Vector2.ZERO:
		var movement: Vector2 = direction * float(ArtData.param("camera.pan_px_per_second", 900.0)) * delta / zoom.x
		position += movement
		_zoom_anchor_world += movement
		_clamp_position()
		camera_activity.emit("pan")


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
	if _zoom_elapsed < ZOOM_SECONDS:
		zoom = Vector2.ONE * _zoom_target
	_zoom_elapsed = ZOOM_SECONDS
	_zoom_target = zoom.x
	position = View.tile_to_world(t) + Vector2(View.TILE, View.TILE) * 0.5
	_clamp_position()
	camera_activity.emit("center")


func _on_world_reset() -> void:
	_center_pending = true
	if _world_sim != null:
		_world_sim.call("set_sim_speed", _speed)


## M5: whether the inspector shows a living person (D drafts them instead of panning).
func _person_selected() -> bool:
	var inspector: Node = get_node_or_null("/root/Main/UI/AgentInspectorPanel")
	return inspector != null and (inspector as Control).visible and int(inspector.call("get_selected_entity_bits")) >= 0
