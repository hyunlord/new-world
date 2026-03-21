extends Sprite2D

const GameConfig = preload("res://scripts/core/simulation/game_config.gd")

const SHADER_PATH: String = "res://shaders/heatmap_overlay.gdshader"
const UPDATE_INTERVAL: float = 0.5

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
}

var _sim_engine: RefCounted = null
var _grid_size: Vector2i = Vector2i.ZERO
var _active_channel: String = ""
var _update_timer: float = 0.0
var _shader_material: ShaderMaterial = null
var _data_texture: ImageTexture = null


func init(sim_engine: RefCounted, reference_renderer: Sprite2D = null) -> void:
	_sim_engine = sim_engine
	_grid_size = Vector2i.ZERO
	if _sim_engine != null and _sim_engine.has_method("get_influence_grid_size"):
		_grid_size = _sim_engine.get_influence_grid_size()
	if _grid_size.x <= 0 or _grid_size.y <= 0:
		_grid_size = GameConfig.WORLD_SIZE

	var shader: Shader = load(SHADER_PATH)
	if shader == null:
		push_warning("[OverlayRenderer] Failed to load heatmap shader")
		return

	_shader_material = ShaderMaterial.new()
	_shader_material.shader = shader
	material = _shader_material
	centered = false
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	z_index = 1
	_create_blank_texture()
	sync_with_world_renderer(reference_renderer)
	visible = false


func _ready() -> void:
	if not SimulationBus.overlay_channels_changed.is_connected(_on_overlay_channels_changed):
		SimulationBus.overlay_channels_changed.connect(_on_overlay_channels_changed)


func sync_with_world_renderer(reference_renderer: Sprite2D) -> void:
	if reference_renderer == null:
		scale = Vector2(GameConfig.TILE_SIZE, GameConfig.TILE_SIZE)
		position = Vector2.ZERO
		return
	centered = reference_renderer.centered
	position = reference_renderer.position
	scale = reference_renderer.scale
	z_index = reference_renderer.z_index + 1


func set_active_channel(channel: String) -> void:
	if channel == _active_channel and visible:
		return
	_active_channel = channel
	_update_timer = 0.0
	if _active_channel.is_empty():
		clear_overlay()
		return
	_apply_channel_colors(_active_channel)
	visible = true
	_refresh_data()


func clear_overlay() -> void:
	_active_channel = ""
	_update_timer = 0.0
	visible = false
	_create_blank_texture()


func _process(delta: float) -> void:
	if not visible or _active_channel.is_empty():
		return
	_update_timer += maxf(delta, 0.0)
	if _update_timer >= UPDATE_INTERVAL:
		_update_timer = 0.0
		_refresh_data()


func _on_overlay_channels_changed(channels: Array) -> void:
	if channels.is_empty():
		clear_overlay()
	else:
		set_active_channel(str(channels[0]))


func _refresh_data() -> void:
	if _sim_engine == null or not _sim_engine.has_method("get_influence_texture"):
		_create_blank_texture()
		return
	var bytes: PackedByteArray = _sim_engine.get_influence_texture(_active_channel)
	var expected_size: int = _grid_size.x * _grid_size.y
	if bytes.is_empty() or bytes.size() != expected_size:
		_create_blank_texture()
		return
	var image: Image = Image.create_from_data(_grid_size.x, _grid_size.y, false, Image.FORMAT_L8, bytes)
	if image == null:
		_create_blank_texture()
		return
	_update_texture_from_image(image)


func _create_blank_texture() -> void:
	var image: Image = Image.create(_grid_size.x, _grid_size.y, false, Image.FORMAT_L8)
	image.fill(Color(0.0, 0.0, 0.0, 1.0))
	_update_texture_from_image(image)


func _update_texture_from_image(image: Image) -> void:
	if _data_texture == null:
		_data_texture = ImageTexture.create_from_image(image)
	else:
		_data_texture.update(image)
	texture = _data_texture
	if _shader_material != null:
		_shader_material.set_shader_parameter("data_texture", _data_texture)


func _apply_channel_colors(channel: String) -> void:
	if _shader_material == null:
		return
	var preset: Dictionary = CHANNEL_PRESETS.get(channel, {})
	_shader_material.set_shader_parameter("color_low", preset.get("color_low", Color(0.10, 0.20, 0.80, 1.0)))
	_shader_material.set_shader_parameter("color_mid_low", preset.get("color_mid_low", Color(0.10, 0.70, 0.70, 1.0)))
	_shader_material.set_shader_parameter("color_mid", preset.get("color_mid", Color(0.20, 0.80, 0.20, 1.0)))
	_shader_material.set_shader_parameter("color_mid_high", preset.get("color_mid_high", Color(0.90, 0.90, 0.10, 1.0)))
	_shader_material.set_shader_parameter("color_high", preset.get("color_high", Color(0.90, 0.15, 0.10, 1.0)))
