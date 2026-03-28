extends Node2D

const GameConfig = preload("res://scripts/core/simulation/game_config.gd")

const SHADER_PATH: String = "res://shaders/heatmap_overlay.gdshader"
const DESAT_SHADER_PATH: String = "res://shaders/terrain_desaturate.gdshader"
const UPDATE_INTERVAL: float = 0.5
const MAX_SIMULTANEOUS: int = 4

const CHANNEL_PRESETS: Dictionary = {
	"food": {
		"color_low": Color(0.10, 0.15, 0.05, 1.0),
		"color_mid_low": Color(0.16, 0.35, 0.08, 1.0),
		"color_mid": Color(0.30, 0.70, 0.10, 1.0),
		"color_mid_high": Color(0.60, 0.92, 0.24, 1.0),
		"color_high": Color(0.10, 1.00, 0.10, 1.0),
	},
	"danger": {
		"color_low": Color(0.30, 0.10, 0.05, 1.0),
		"color_mid_low": Color(0.55, 0.18, 0.07, 1.0),
		"color_mid": Color(0.80, 0.40, 0.10, 1.0),
		"color_mid_high": Color(0.95, 0.55, 0.12, 1.0),
		"color_high": Color(1.00, 0.10, 0.05, 1.0),
	},
	"warmth": {
		"color_low": Color(0.10, 0.10, 0.60, 1.0),
		"color_mid_low": Color(0.18, 0.30, 0.78, 1.0),
		"color_mid": Color(0.80, 0.50, 0.10, 1.0),
		"color_mid_high": Color(0.96, 0.66, 0.12, 1.0),
		"color_high": Color(1.00, 0.20, 0.05, 1.0),
	},
	"social": {
		"color_low": Color(0.05, 0.10, 0.30, 1.0),
		"color_mid_low": Color(0.12, 0.24, 0.55, 1.0),
		"color_mid": Color(0.20, 0.40, 0.80, 1.0),
		"color_mid_high": Color(0.28, 0.54, 0.95, 1.0),
		"color_high": Color(0.40, 0.60, 1.00, 1.0),
	},
	"knowledge": {
		"color_low": Color(0.15, 0.05, 0.20, 1.0),
		"color_mid_low": Color(0.28, 0.10, 0.38, 1.0),
		"color_mid": Color(0.50, 0.20, 0.70, 1.0),
		"color_mid_high": Color(0.66, 0.32, 0.88, 1.0),
		"color_high": Color(0.80, 0.40, 1.00, 1.0),
	},
	"resource": {
		"color_low": Color(0.20, 0.15, 0.05, 1.0),
		"color_mid_low": Color(0.42, 0.30, 0.08, 1.0),
		"color_mid": Color(0.70, 0.50, 0.20, 1.0),
		"color_mid_high": Color(0.88, 0.68, 0.24, 1.0),
		"color_high": Color(1.00, 0.80, 0.20, 1.0),
	},
	"authority": {
		"color_low": Color(0.15, 0.08, 0.25, 1.0),
		"color_mid_low": Color(0.28, 0.12, 0.45, 1.0),
		"color_mid": Color(0.50, 0.20, 0.65, 1.0),
		"color_mid_high": Color(0.70, 0.30, 0.85, 1.0),
		"color_high": Color(0.90, 0.45, 1.00, 1.0),
	},
}

var _sim_engine: RefCounted = null
var _world_renderer: Sprite2D = null
var _grid_size: Vector2i = Vector2i.ZERO
var _update_timer: float = 0.0
var _desat_shader_material: ShaderMaterial = null
var _current_zoom_level: int = -1
var _active_channels: Array[String] = []
var _channel_layers: Dictionary = {}


func init(sim_engine: RefCounted, reference_renderer: Sprite2D = null) -> void:
	_sim_engine = sim_engine
	_world_renderer = reference_renderer
	_grid_size = Vector2i.ZERO
	if _sim_engine != null and _sim_engine.has_method("get_influence_grid_size"):
		_grid_size = _sim_engine.get_influence_grid_size()
	if _grid_size.x <= 0 or _grid_size.y <= 0:
		_grid_size = GameConfig.WORLD_SIZE
	z_index = 1
	if reference_renderer != null:
		z_index = reference_renderer.z_index + 1
		position = reference_renderer.position


func _ready() -> void:
	if not SimulationBus.overlay_channels_changed.is_connected(_on_overlay_channels_changed):
		SimulationBus.overlay_channels_changed.connect(_on_overlay_channels_changed)


func sync_with_world_renderer(reference_renderer: Sprite2D) -> void:
	_world_renderer = reference_renderer
	if reference_renderer == null:
		position = Vector2.ZERO
		z_index = 5
		return
	position = reference_renderer.position
	z_index = reference_renderer.z_index + 1


func _process(delta: float) -> void:
	if _active_channels.is_empty():
		return
	var cam: Camera2D = get_viewport().get_camera_2d()
	if cam != null and cam.has_method("get_zoom_level"):
		var zl: int = int(cam.call("get_zoom_level"))
		if zl != _current_zoom_level:
			_current_zoom_level = zl
			_update_zoom_filter(zl)
	_update_timer += maxf(delta, 0.0)
	if _update_timer >= UPDATE_INTERVAL:
		_update_timer = 0.0
		_refresh_all_layers()


func _on_overlay_channels_changed(channels: Array) -> void:
	var new_channels: Array[String] = []
	for ch: Variant in channels:
		var s: String = str(ch)
		if not s.is_empty():
			new_channels.append(s)

	var to_remove: Array[String] = []
	for existing: String in _active_channels:
		if existing not in new_channels:
			to_remove.append(existing)
	for ch: String in to_remove:
		_remove_channel_layer(ch)

	for ch: String in new_channels:
		if ch not in _active_channels and _channel_layers.size() < MAX_SIMULTANEOUS:
			_add_channel_layer(ch)

	_active_channels = new_channels.duplicate()

	if _active_channels.is_empty():
		_remove_desaturation()
	else:
		_apply_desaturation()

	_update_timer = 0.0
	_refresh_all_layers()


func _add_channel_layer(channel: String) -> void:
	var layer := Sprite2D.new()
	layer.name = "overlay_%s" % channel
	layer.centered = false
	layer.position = Vector2.ZERO
	layer.scale = Vector2(float(GameConfig.TILE_SIZE), float(GameConfig.TILE_SIZE))
	layer.z_index = 0

	var shader: Shader = load(SHADER_PATH)
	if shader == null:
		layer.queue_free()
		return
	var mat := ShaderMaterial.new()
	mat.shader = shader
	layer.material = mat
	layer.texture_filter = CanvasItem.TEXTURE_FILTER_LINEAR

	var blank := Image.create(_grid_size.x, _grid_size.y, false, Image.FORMAT_L8)
	blank.fill(Color(0, 0, 0, 1))
	var tex := ImageTexture.create_from_image(blank)
	layer.texture = tex
	mat.set_shader_parameter("data_texture", tex)

	_apply_channel_colors_to_material(mat, channel)

	var count: int = _channel_layers.size() + 1
	var opacity: float = maxf(0.55 / sqrt(float(count)), 0.20)
	mat.set_shader_parameter("overlay_opacity", opacity)

	add_child(layer)
	_channel_layers[channel] = layer


func _remove_channel_layer(channel: String) -> void:
	if _channel_layers.has(channel):
		var layer: Sprite2D = _channel_layers[channel]
		layer.queue_free()
		_channel_layers.erase(channel)


func _refresh_all_layers() -> void:
	for channel: String in _active_channels:
		if not _channel_layers.has(channel):
			continue
		var layer: Sprite2D = _channel_layers[channel]
		_refresh_channel_data(channel, layer)

	var count: int = _active_channels.size()
	var opacity: float = maxf(0.55 / sqrt(float(maxi(count, 1))), 0.20)
	for channel: String in _channel_layers:
		var layer: Sprite2D = _channel_layers[channel]
		if layer.material is ShaderMaterial:
			(layer.material as ShaderMaterial).set_shader_parameter("overlay_opacity", opacity)


func _refresh_channel_data(channel: String, layer: Sprite2D) -> void:
	if _sim_engine == null or not _sim_engine.has_method("get_influence_texture"):
		return
	var bytes: PackedByteArray = _sim_engine.get_influence_texture(channel)
	var expected_size: int = _grid_size.x * _grid_size.y
	if bytes.is_empty() or bytes.size() != expected_size:
		return
	var image: Image = Image.create_from_data(_grid_size.x, _grid_size.y, false, Image.FORMAT_L8, bytes)
	if image == null:
		return
	var tex: ImageTexture = layer.texture as ImageTexture
	if tex != null:
		tex.update(image)
	else:
		tex = ImageTexture.create_from_image(image)
		layer.texture = tex
		if layer.material is ShaderMaterial:
			(layer.material as ShaderMaterial).set_shader_parameter("data_texture", tex)


func _update_zoom_filter(zoom_level: int) -> void:
	for channel: String in _channel_layers:
		var layer: Sprite2D = _channel_layers[channel]
		if zoom_level == 2:
			layer.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
		else:
			layer.texture_filter = CanvasItem.TEXTURE_FILTER_LINEAR


func _apply_channel_colors_to_material(mat: ShaderMaterial, channel: String) -> void:
	var preset: Dictionary = CHANNEL_PRESETS.get(channel, {})
	mat.set_shader_parameter("color_low", preset.get("color_low", Color(0.10, 0.20, 0.80, 1.0)))
	mat.set_shader_parameter("color_mid_low", preset.get("color_mid_low", Color(0.10, 0.70, 0.70, 1.0)))
	mat.set_shader_parameter("color_mid", preset.get("color_mid", Color(0.20, 0.80, 0.20, 1.0)))
	mat.set_shader_parameter("color_mid_high", preset.get("color_mid_high", Color(0.90, 0.90, 0.10, 1.0)))
	mat.set_shader_parameter("color_high", preset.get("color_high", Color(0.90, 0.15, 0.10, 1.0)))


func _apply_desaturation() -> void:
	if _world_renderer == null:
		return
	if _desat_shader_material == null:
		var desat_shader: Shader = load(DESAT_SHADER_PATH)
		if desat_shader == null:
			return
		_desat_shader_material = ShaderMaterial.new()
		_desat_shader_material.shader = desat_shader
	_desat_shader_material.set_shader_parameter("saturation", 0.30)
	_desat_shader_material.set_shader_parameter("brightness", 0.80)
	_world_renderer.material = _desat_shader_material


func _remove_desaturation() -> void:
	if _world_renderer != null:
		_world_renderer.material = null
