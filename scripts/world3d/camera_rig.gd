# camera_rig.gd (World3D/CameraRig, made by world3d.gd): the 3D view's perspective camera (M3D-1 scope 2).
# Purpose: the 2D Camera2D (camera_controller.gd) stays the one view state — pan keys, edge and drag pan, the zoom
#   ladder, Home/H, map clamping and every panel that centres the camera keep working. This rig follows it: the look
#   target is Camera2D.position / 64 (1 tile = 1 unit, x → X, y → Z) on the terrain, the distance makes one tile about
#   64·zoom screen pixels at the target (so 2D pan speeds and drags feel the same), narrow FOV (params3d
#   `camera.fov_deg`, 30°) and pitch `camera.pitch_deg` (45°) for a near-isometric look. Q/E turn the view in 90° steps,
#   eased over `camera.rotate_seconds`; the 2D camera's `screen_rotation` follows, so W stays "up on screen". The 3D
#   zoom ladder is params3d `camera.zooms` (Z1–Z5, Z5 = close-up cutaway). Each frame it also sets the global
#   shader parameters of the flora x-ray (trees between the camera and the target dissolve; `camera.xray_radius`).
# Bridge methods called: none (the terrain height comes from Terrain3D).
extends Node3D

const Data3D = preload("res://scripts/world3d/data3d.gd")

var camera: Camera3D
var _camera2d: Camera2D = null
var _terrain: Node = null
var _yaw: float = 0.0
var _yaw_from: float = 0.0
var _yaw_to: float = 0.0
var _yaw_t: float = 1.0
var _target_y: float = 0.0
var _px_per_tile: float = 64.0


func setup(camera2d: Camera2D, terrain: Node) -> void:
	_camera2d = camera2d
	_terrain = terrain
	_px_per_tile = float(Data3D.param("camera.px_per_tile_at_1x", 64.0))
	var steps: Array[float] = []
	for z: Variant in Data3D.param("camera.zooms", [0.25, 0.5, 1.0, 2.0, 4.0]):
		steps.append(float(z))
	steps.sort()
	if _camera2d != null:
		_camera2d.set("zoom_steps_override", steps)


func _ready() -> void:
	camera = Camera3D.new()
	camera.name = "Camera3D"
	camera.fov = float(Data3D.param("camera.fov_deg", 30.0))
	camera.near = float(Data3D.param("camera.near", 0.3))
	camera.far = float(Data3D.param("camera.far", 900.0))
	add_child(camera)
	camera.make_current()


## The yaw (radians) the view is turned to (0 = looking north).
func yaw() -> float:
	return _yaw


## Distance from the look target for 2D zoom `z`.
func distance_for_zoom(z: float) -> float:
	var h: float = get_viewport().get_visible_rect().size.y
	return h / (2.0 * _px_per_tile * maxf(0.05, z) * tan(deg_to_rad(camera.fov) * 0.5))


func zoom() -> float:
	return _camera2d.zoom.x if _camera2d != null else 1.0


## The ground point the camera looks at (world units).
func target() -> Vector3:
	var p: Vector2 = _camera2d.position / _px_per_tile if _camera2d != null else Vector2.ZERO
	return Vector3(p.x, _target_y, p.y)


func rotate_view(steps: int) -> void:
	_yaw_from = _yaw
	_yaw_to = _yaw_to + steps * PI * 0.5
	_yaw_t = 0.0


func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo:
		var key: InputEventKey = event
		if key.ctrl_pressed or key.meta_pressed or key.alt_pressed:
			return
		if key.keycode == KEY_Q:
			rotate_view(-1)
			get_viewport().set_input_as_handled()
		elif key.keycode == KEY_E:
			rotate_view(1)
			get_viewport().set_input_as_handled()


func _process(delta: float) -> void:
	if _yaw_t < 1.0:
		_yaw_t = minf(1.0, _yaw_t + delta / maxf(0.01, float(Data3D.param("camera.rotate_seconds", 0.35))))
		var e: float = _yaw_t * _yaw_t * (3.0 - 2.0 * _yaw_t)
		_yaw = lerpf(_yaw_from, _yaw_to, e)
	if _camera2d != null:
		_camera2d.set("screen_rotation", -_yaw)
	update_camera(delta)


## Place the camera for the current 2D view state (also called by tools after moving the 2D camera).
func update_camera(delta: float = 0.0) -> void:
	var t: Vector3 = target()
	if _terrain != null:
		var ground: float = float(_terrain.call("height_at", t.x, t.z))
		_target_y = ground if delta <= 0.0 else lerpf(_target_y, ground, minf(1.0, delta * 6.0))
		t.y = _target_y
	var pitch: float = deg_to_rad(float(Data3D.param("camera.pitch_deg", 50.0)))
	var d: float = distance_for_zoom(zoom())
	var back: Vector3 = Vector3(sin(_yaw), 0.0, cos(_yaw))
	camera.global_position = t + back * cos(pitch) * d + Vector3.UP * sin(pitch) * d
	camera.look_at(t, Vector3.UP)
	# Flora x-ray (shaders/world3d/flora.gdshader): wider when the camera is close, off when far.
	RenderingServer.global_shader_parameter_set("w3d_camera", camera.global_position)
	RenderingServer.global_shader_parameter_set("w3d_focus", t)
	RenderingServer.global_shader_parameter_set("w3d_xray", clampf(float(Data3D.param("camera.xray_radius", 4.5)) * (1.6 - d / 40.0), 0.0, 6.0) if d < 40.0 else 0.0)
