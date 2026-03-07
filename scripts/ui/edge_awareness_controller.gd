extends ColorRect
class_name EdgeAwarenessController

const EDGE_SHADER_PATH: String = "res://shaders/edge_awareness.gdshader"
const TILE_SIZE_F: float = 16.0
const CRISIS_INTENSITY: float = 0.8
const DRAMA_INTENSITY: float = 0.4
const INTENSITY_LERP_SPEED: float = 6.0

var _camera: Camera2D = null
var _notification_manager: Control = null
var _shader_material: ShaderMaterial = null
var _current_top: float = 0.0
var _current_bottom: float = 0.0
var _current_left: float = 0.0
var _current_right: float = 0.0


func init(camera: Camera2D, notification_manager: Control) -> void:
	_camera = camera
	_notification_manager = notification_manager


func _ready() -> void:
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	color = Color(1.0, 1.0, 1.0, 1.0)
	set_anchors_preset(Control.PRESET_FULL_RECT)
	_ensure_material()


func _process(delta: float) -> void:
	if _shader_material == null:
		return
	var target_top: float = 0.0
	var target_bottom: float = 0.0
	var target_left: float = 0.0
	var target_right: float = 0.0
	if _camera != null and _notification_manager != null and _notification_manager.has_method("get_active_notifications"):
		var active_notifications: Array = _notification_manager.call("get_active_notifications")
		var camera_position: Vector2 = _camera.global_position
		var viewport_size: Vector2 = get_viewport_rect().size
		var half_size: Vector2 = Vector2(
			viewport_size.x / maxf(_camera.zoom.x * 2.0, 0.001),
			viewport_size.y / maxf(_camera.zoom.y * 2.0, 0.001)
		)
		var view_rect: Rect2 = Rect2(camera_position - half_size, half_size * 2.0)
		for notif_raw: Variant in active_notifications:
			if not (notif_raw is Dictionary):
				continue
			var notif_data: Dictionary = notif_raw
			var tier: int = int(notif_data.get("tier", 3))
			if tier > 1:
				continue
			var event_world: Vector2 = _tile_to_world_position(Vector2(
				float(notif_data.get("position_x", 0.0)),
				float(notif_data.get("position_y", 0.0))
			))
			if view_rect.has_point(event_world):
				continue
			var direction: Vector2 = (event_world - camera_position).normalized()
			var intensity: float = CRISIS_INTENSITY if tier == 0 else DRAMA_INTENSITY
			if direction.y < -0.3:
				target_top = maxf(target_top, intensity)
			if direction.y > 0.3:
				target_bottom = maxf(target_bottom, intensity)
			if direction.x < -0.3:
				target_left = maxf(target_left, intensity)
			if direction.x > 0.3:
				target_right = maxf(target_right, intensity)
	_current_top = _smooth_intensity(_current_top, target_top, delta)
	_current_bottom = _smooth_intensity(_current_bottom, target_bottom, delta)
	_current_left = _smooth_intensity(_current_left, target_left, delta)
	_current_right = _smooth_intensity(_current_right, target_right, delta)
	_shader_material.set_shader_parameter("intensity_top", _current_top)
	_shader_material.set_shader_parameter("intensity_bottom", _current_bottom)
	_shader_material.set_shader_parameter("intensity_left", _current_left)
	_shader_material.set_shader_parameter("intensity_right", _current_right)


func _ensure_material() -> void:
	if _shader_material != null:
		return
	var shader_resource: Variant = load(EDGE_SHADER_PATH)
	if not (shader_resource is Shader):
		return
	_shader_material = ShaderMaterial.new()
	_shader_material.shader = shader_resource
	material = _shader_material


func _smooth_intensity(current: float, target: float, delta: float) -> float:
	return lerpf(current, target, clampf(delta * INTENSITY_LERP_SPEED, 0.0, 1.0))


func _tile_to_world_position(tile_position: Vector2) -> Vector2:
	return tile_position * TILE_SIZE_F + Vector2(TILE_SIZE_F * 0.5, TILE_SIZE_F * 0.5)
