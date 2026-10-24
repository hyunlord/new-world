# M-UX A1 behavioral regression: real main scene, dispatched pointer/gesture events.
# Bridge reads: main scene's normal reads only; camera pauses WorldSim during checks.
# Run after bridge build: godot --path . --script tools/v8/mux_camera_test.gd
# Windowed run verifies usable-screen fitting; synthetic input does not establish OS gesture acceptance.
extends SceneTree

const Tool = preload("res://scripts/world/colony/order_tool.gd")
const Store = preload("res://scripts/ui/mux/settings_store.gd")
var _camera: Camera2D
var _tool: Node
var _main: Node
var _failures: int = 0
var _orders: int = 0
var _trace_input: bool = false


class InputProbe extends Node:
	func _input(event: InputEvent) -> void:
		_trace("input", event)


	func _unhandled_input(event: InputEvent) -> void:
		_trace("unhandled", event)


	func _trace(phase: String, event: InputEvent) -> void:
		if event is InputEventMagnifyGesture:
			print("MUX_CAMERA_INPUT %s pinch factor=%s position=%s id=%s device=%s window=%s frame=%s usec=%s" % [phase, (event as InputEventMagnifyGesture).factor, (event as InputEventMagnifyGesture).position, event.get_instance_id(), event.device, (event as InputEventMagnifyGesture).window_id, Engine.get_process_frames(), Time.get_ticks_usec()])
		elif event is InputEventKey and (event as InputEventKey).keycode == KEY_3:
			print("MUX_CAMERA_INPUT %s key3 pressed=%s echo=%s" % [phase, event.is_pressed(), event.is_echo()])


func _diagnose(stage: String) -> void:
	var hovered: Control = root.gui_get_hovered_control()
	var focused: Control = root.gui_get_focus_owner()
	print("MUX_CAMERA_STATE %s zoom=%s target=%s elapsed=%s pinch=%s speed=%s pointer=%s hovered=%s focus=%s" % [stage, _camera.zoom.x, _camera.get("_zoom_target"), _camera.get("_zoom_elapsed"), _camera.get("_pinch_scale"), _camera.call("get_speed"), _camera.get("_pointer_button"), hovered.get_path() if hovered != null else "none", focused.get_path() if focused != null else "none"])


func _activity(action: String) -> void:
	if _trace_input:
		_diagnose("activity:" + action)


func _window_event(event: InputEvent) -> void:
	if event is InputEventMagnifyGesture:
		print("MUX_CAMERA_WINDOW pinch id=%s device=%s window=%s frame=%s usec=%s" % [event.get_instance_id(), event.device, (event as InputEventMagnifyGesture).window_id, Engine.get_process_frames(), Time.get_ticks_usec()])


func _init() -> void:
	call_deferred("_run")


func _check(label: String, ok: bool) -> void:
	print("MUX_CAMERA_%s %s" % ["PASS" if ok else "FAIL", label])
	if not ok:
		_failures += 1


func _frames(count: int = 3) -> void:
	for _i: int in count:
		await process_frame


func _button(button: MouseButton, pressed: bool, point: Vector2) -> void:
	var event: InputEventMouseButton = InputEventMouseButton.new()
	event.button_index = button
	event.pressed = pressed
	event.position = point
	Input.parse_input_event(event)


func _motion(point: Vector2, relative: Vector2 = Vector2.ZERO) -> void:
	var event: InputEventMouseMotion = InputEventMouseMotion.new()
	event.position = point
	event.relative = relative
	Input.parse_input_event(event)


func _key(code: Key, pressed: bool, fullscreen: bool = false) -> void:
	var event: InputEventKey = InputEventKey.new()
	event.keycode = code
	event.physical_keycode = code
	event.pressed = pressed
	event.ctrl_pressed = fullscreen
	event.meta_pressed = fullscreen
	Input.parse_input_event(event)


func _drag(button: MouseButton, point: Vector2) -> void:
	_motion(point)
	await _frames()
	_button(button, true, point)
	_motion(point + Vector2(80, 40), Vector2(80, 40))
	_button(button, false, point + Vector2(80, 40))
	await _frames()


func _run() -> void:
	_main = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	var settings_path: String = "/Users/rexxa/work/mux-data/camera-test-settings-%d.cfg" % OS.get_process_id()
	_main.get_node("UI/MuxHud").set("store", Store.new(settings_path))
	print("MUX_CAMERA_SETTINGS " + settings_path)
	root.add_child(_main)
	await _frames(15)
	_camera = _main.get_node("Camera2D") as Camera2D
	root.add_child(InputProbe.new())
	root.window_input.connect(_window_event)
	_camera.connect("camera_activity", _activity)
	_tool = _main.get_node("ColonyView/OrderTool")
	_tool.connect("order_sent", func(_command: String, _result: Dictionary) -> void: _orders += 1)
	_camera.call("set_paused", true)
	_camera.call("center_on_tile", Vector2i(96, 96))
	await _frames()
	var point: Vector2 = root.get_visible_rect().size * Vector2(0.55, 0.52)
	_motion(point)
	await _frames()
	_check("test pointer clear of GUI", root.gui_get_hovered_control() == null)
	var causal: Control = _main.get_node("UI/CausalPanel") as Control
	causal.hide()
	var before: Vector2 = _camera.position
	await _drag(MOUSE_BUTTON_LEFT, point)
	_check("no-tool left drag pans without selection or orders", _camera.position != before and not causal.visible and _orders == 0)
	_motion(point)
	await _frames()
	_button(MOUSE_BUTTON_LEFT, true, point)
	_check("left press defers selection", not causal.visible)
	_button(MOUSE_BUTTON_LEFT, false, point)
	await _frames()
	var inspector: Control = _main.get_node("UI/AgentInspectorPanel") as Control
	_check("left click opens an inspector", causal.visible or inspector.visible)
	causal.hide()
	inspector.hide()
	_tool.call("set_mode", Tool.Mode.CHOP)
	before = _camera.position
	await _drag(MOUSE_BUTTON_RIGHT, point)
	_check("right drag preserves active tool and sends no order", _camera.position != before and int(_tool.get("mode")) == Tool.Mode.CHOP and _orders == 0)
	_motion(point)
	await _frames()
	_button(MOUSE_BUTTON_RIGHT, true, point)
	_check("right press does not cancel tool", int(_tool.get("mode")) == Tool.Mode.CHOP)
	_button(MOUSE_BUTTON_RIGHT, false, point)
	await _frames()
	_check("right click cancels tool on release", int(_tool.get("mode")) == Tool.Mode.NONE)
	before = _camera.position
	var pan: InputEventPanGesture = InputEventPanGesture.new()
	pan.position = point
	pan.delta = Vector2(2, 1)
	Input.parse_input_event(pan)
	await _frames()
	_check("trackpad pan moves camera", _camera.position != before)
	_check("trackpad pan applies exact distance once", ((before - _camera.position) * _camera.zoom).distance_to(Vector2(64, 32)) <= 1.0)
	before = _camera.position
	for _i: int in 2:
		var distinct_pan: InputEventPanGesture = InputEventPanGesture.new()
		distinct_pan.position = point
		distinct_pan.delta = Vector2(2, 1)
		Input.parse_input_event(distinct_pan)
	await _frames()
	_check("distinct same-frame pans both apply", ((before - _camera.position) * _camera.zoom).distance_to(Vector2(128, 64)) <= 1.0)
	before = _camera.position
	Input.parse_input_event(pan)
	await _frames()
	_check("same pan object on later frame applies again", ((before - _camera.position) * _camera.zoom).distance_to(Vector2(64, 32)) <= 1.0)
	_camera.call("center_on_tile", Vector2i(96, 96))
	await _frames()
	var anchor: Vector2 = root.get_canvas_transform().affine_inverse() * point
	var zoom_before: float = _camera.zoom.x
	_button(MOUSE_BUTTON_WHEEL_UP, true, point)
	_check("wheel starts animation instead of jumping", is_equal_approx(_camera.zoom.x, zoom_before))
	await create_timer(0.2).timeout
	var after: Vector2 = root.get_canvas_transform().affine_inverse() * point
	_check("wheel advances ladder", _camera.zoom.x > zoom_before)
	_check("wheel preserves cursor world anchor", anchor.distance_to(after) <= 2.0)
	var pinch: InputEventMagnifyGesture = InputEventMagnifyGesture.new()
	pinch.position = point
	pinch.factor = 0.5
	_trace_input = true
	_diagnose("pinch-before")
	print("MUX_CAMERA_SUBMIT pinch id=%s device=%s window=%s frame=%s usec=%s" % [pinch.get_instance_id(), pinch.device, pinch.window_id, Engine.get_process_frames(), Time.get_ticks_usec()])
	Input.parse_input_event(pinch)
	_diagnose("pinch-dispatched")
	await create_timer(0.2).timeout
	_diagnose("pinch-after-200ms")
	_check("pinch reverses one zoom step", is_equal_approx(_camera.zoom.x, zoom_before))
	for _i: int in 2:
		var distinct_pinch: InputEventMagnifyGesture = InputEventMagnifyGesture.new()
		distinct_pinch.position = point
		distinct_pinch.factor = 0.5
		Input.parse_input_event(distinct_pinch)
	await create_timer(0.2).timeout
	_check("distinct same-frame pinches both advance ladder", is_equal_approx(_camera.zoom.x, zoom_before * 0.25))
	_trace_input = false
	for _i: int in 8:
		_camera.call("step_zoom", -1, point)
	await create_timer(0.2).timeout
	var ladder: Array = _camera.call("zoom_steps")
	_check("zoom lower bound", is_equal_approx(_camera.zoom.x, float(ladder.front())))
	for _i: int in 8:
		_camera.call("step_zoom", 1, point)
	await create_timer(0.2).timeout
	_check("zoom upper bound", is_equal_approx(_camera.zoom.x, float(ladder.back())))
	_camera.call("step_zoom", -1, point)
	_camera.call("center_on_tile", Vector2i(96, 96))
	_check("center during zoom retains ladder level", ladder.has(_camera.zoom.x))
	before = _camera.position
	_key(KEY_RIGHT, true)
	await create_timer(0.08).timeout
	_key(KEY_RIGHT, false)
	_check("keyboard pan preserved", _camera.position.x > before.x)
	_trace_input = true
	_diagnose("speed-before")
	_key(KEY_3, true)
	_key(KEY_3, false)
	_diagnose("speed-dispatched")
	await _frames(3)
	_diagnose("speed-after-3frames")
	_check("speed key preserved", is_equal_approx(float(_camera.call("get_speed")), 3.0))
	_trace_input = false
	_check("speed does not unpause", bool(_camera.call("is_paused")))
	_camera.call("center_on_tile", Vector2i(-1000, -1000))
	_check("camera bounds preserved", _camera.position.x >= float(_camera.limit_left) and _camera.position.y >= float(_camera.limit_top))
	if DisplayServer.get_name() != "headless":
		var size: Vector2i = root.size
		var usable: Rect2i = DisplayServer.screen_get_usable_rect(root.current_screen)
		_check("launch client is 16:9 capped at 1920x1080", size.x * 9 == size.y * 16 and size.x <= 1920 and size.y <= 1080)
		_check("outer launch window fits usable display", DisplayServer.window_get_size_with_decorations().x <= usable.size.x and DisplayServer.window_get_size_with_decorations().y <= usable.size.y)
		_key(KEY_F, true, true)
		_key(KEY_F, false, true)
		await _frames(10)
		_check("Ctrl+Meta+F enters fullscreen", root.mode == Window.MODE_FULLSCREEN)
		_key(KEY_F, true, true)
		_key(KEY_F, false, true)
		await _frames(10)
		_check("Ctrl+Meta+F restores fitted window", root.mode == Window.MODE_WINDOWED and root.size == size)
	else:
		print("MUX_CAMERA_NOT_TESTED native window fitting/fullscreen: headless display")
	print("MUX_CAMERA_DONE failures=%d" % _failures)
	quit(0 if _failures == 0 else 1)
