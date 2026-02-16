class_name CameraController
extends Camera2D

var _target_zoom: float = 1.0
var _is_dragging: bool = false
var _drag_start: Vector2 = Vector2.ZERO

## Left-click drag pan state
var _left_dragging: bool = false
var _left_drag_start: Vector2 = Vector2.ZERO
var _left_was_dragged: bool = false
const DRAG_THRESHOLD: float = 5.0


func _ready() -> void:
	# Start at world center
	var center_px := Vector2(GameConfig.WORLD_SIZE) * GameConfig.TILE_SIZE * 0.5
	position = center_px
	zoom = Vector2(1.5, 1.5)
	_target_zoom = 1.5
	make_current()


func _unhandled_input(event: InputEvent) -> void:
	# Left-click drag pan
	if event is InputEventMouseButton and event.button_index == MOUSE_BUTTON_LEFT:
		if event.pressed:
			_left_dragging = true
			_left_drag_start = event.position
			_left_was_dragged = false
		else:
			_left_dragging = false
			if _left_was_dragged:
				# Was a drag — consume so EntityRenderer doesn't select
				get_viewport().set_input_as_handled()

	# Mouse wheel zoom
	if event is InputEventMouseButton:
		if event.pressed:
			if event.button_index == MOUSE_BUTTON_WHEEL_UP:
				_zoom_at_mouse(GameConfig.CAMERA_ZOOM_STEP)
				get_viewport().set_input_as_handled()
			elif event.button_index == MOUSE_BUTTON_WHEEL_DOWN:
				_zoom_at_mouse(-GameConfig.CAMERA_ZOOM_STEP)
				get_viewport().set_input_as_handled()
			elif event.button_index == MOUSE_BUTTON_MIDDLE:
				_is_dragging = true
				_drag_start = event.position
				get_viewport().set_input_as_handled()
		else:
			if event.button_index == MOUSE_BUTTON_MIDDLE:
				_is_dragging = false

	# Mouse motion: middle-click drag or left-click drag
	if event is InputEventMouseMotion:
		if _is_dragging:
			position += (_drag_start - event.position) / zoom.x
			_drag_start = event.position
			get_viewport().set_input_as_handled()
		elif _left_dragging:
			var moved: float = event.position.distance_to(_left_drag_start)
			if moved > DRAG_THRESHOLD:
				_left_was_dragged = true
			if _left_was_dragged:
				position += -event.relative / zoom.x
				get_viewport().set_input_as_handled()

	# macOS trackpad: pinch zoom
	if event is InputEventMagnifyGesture:
		var zoom_delta: float = (event.factor - 1.0) * 0.5
		_zoom_at_mouse(zoom_delta)
		get_viewport().set_input_as_handled()

	# macOS trackpad: two-finger scroll pan
	if event is InputEventPanGesture:
		position += event.delta * 2.0 / zoom.x
		get_viewport().set_input_as_handled()


func _process(delta: float) -> void:
	# Smooth zoom
	var new_zoom: float = lerpf(zoom.x, _target_zoom, GameConfig.CAMERA_ZOOM_SPEED)
	zoom = Vector2(new_zoom, new_zoom)

	# WASD / arrow key panning
	var pan_dir := Vector2.ZERO
	if Input.is_key_pressed(KEY_W) or Input.is_key_pressed(KEY_UP):
		pan_dir.y -= 1
	if Input.is_key_pressed(KEY_S) or Input.is_key_pressed(KEY_DOWN):
		pan_dir.y += 1
	if Input.is_key_pressed(KEY_A) or Input.is_key_pressed(KEY_LEFT):
		pan_dir.x -= 1
	if Input.is_key_pressed(KEY_D) or Input.is_key_pressed(KEY_RIGHT):
		pan_dir.x += 1
	if pan_dir != Vector2.ZERO:
		position += pan_dir.normalized() * GameConfig.CAMERA_PAN_SPEED * delta / zoom.x

	# Clamp to world bounds
	var world_px := Vector2(GameConfig.WORLD_SIZE) * GameConfig.TILE_SIZE
	position.x = clampf(position.x, 0, world_px.x)
	position.y = clampf(position.y, 0, world_px.y)


func _zoom_at_mouse(delta: float) -> void:
	_target_zoom = clampf(_target_zoom + delta, GameConfig.CAMERA_ZOOM_MIN, GameConfig.CAMERA_ZOOM_MAX)
