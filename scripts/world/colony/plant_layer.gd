# PlantLayer draws real tile plants in phase batches, using art bindings or data-colored placeholders.
# Bridge: get_plants(viewport rectangle) directly at a bounded cadence; get_time_snapshot via SimReads.
extends Node2D

const View = preload("res://scripts/world/colony/view.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
const Content = preload("res://scripts/ui/content_context.gd")
var _world: Node
var _elapsed: float = 0.0
var _tick: int = -1
var _window: Rect2i
var _signature: int = -1
var _params_version: int = -1
var _batches: Array[MultiMeshInstance2D] = []
var _textures: Dictionary = {}

func _ready() -> void:
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_world = get_node("/root/Main/WorldSim")
	_world.connect("world_reset", _reset)

func _reset() -> void:
	_tick = -1
	_signature = -1
	_textures.clear()
	for batch: MultiMeshInstance2D in _batches:
		batch.queue_free()
	_batches.clear()

func _process(delta: float) -> void:
	_elapsed += delta
	if _elapsed < float(ArtData.param("plants.poll_seconds", 0.5)):
		return
	_elapsed = 0.0
	var camera: Camera2D = get_viewport().get_camera_2d()
	if camera == null:
		return
	var half: Vector2 = get_viewport_rect().size / camera.zoom / 2.0
	var centre: Vector2 = camera.get_screen_center_position()
	var first: Vector2i = View.world_to_tile(centre - half) - Vector2i.ONE
	var last: Vector2i = View.world_to_tile(centre + half) + Vector2i.ONE
	var window := Rect2i(first, last - first + Vector2i.ONE)
	var tick: int = int(SimReads.read(_world, "get_time_snapshot").get("tick", 0))
	var params_version: int = ArtData.params_version()
	if tick == _tick and window == _window and params_version == _params_version:
		return
	_tick = tick
	_window = window
	if params_version != _params_version:
		_params_version = params_version
		_signature = -1
		_textures.clear()
	var rows: Dictionary = _world.get_plants(first.x, first.y, last.x, last.y)
	var signature: int = hash(rows)
	if signature == _signature:
		return
	_signature = signature
	_rebuild(rows)

func _rebuild(rows: Dictionary) -> void:
	for batch: MultiMeshInstance2D in _batches:
		batch.queue_free()
	_batches.clear()
	var xs: PackedInt64Array = rows.get("xs", PackedInt64Array())
	var ys: PackedInt64Array = rows.get("ys", PackedInt64Array())
	var kinds: PackedInt64Array = rows.get("kinds", PackedInt64Array())
	var phases: PackedInt64Array = rows.get("phases", PackedInt64Array())
	var groups: Dictionary = {}
	for index: int in mini(mini(xs.size(), ys.size()), mini(kinds.size(), phases.size())):
		var phase: int = clampi(phases[index], 0, 3)
		var definition: Dictionary = Content.row("plants", kinds[index])
		var asset: String = ArtData.plant_asset(definition, phase)
		var key: String = asset if not asset.is_empty() and ArtData.has(asset) else "phase:%d" % phase
		if not groups.has(key):
			groups[key] = {"points": [], "phase": phase, "asset": asset}
		groups[key]["points"].append(View.base_point(Vector2i(xs[index], ys[index])))
	for key: String in groups:
		var group: Dictionary = groups[key]
		var texture: Texture2D = ArtData.texture(str(group["asset"])) if ArtData.has(str(group["asset"])) else _placeholder(int(group["phase"]))
		if texture == null:
			continue
		var batch := MultiMeshInstance2D.new()
		var mesh: ArrayMesh = _quad(texture.get_size())
		var instances := MultiMesh.new()
		instances.transform_format = MultiMesh.TRANSFORM_2D
		instances.mesh = mesh
		var anchor: Vector2 = Vector2(ArtData.anchor(str(group["asset"]))) if ArtData.has(str(group["asset"])) else Vector2(texture.get_width() / 2.0, texture.get_height())
		var offset: Vector2 = texture.get_size() / 2.0 - anchor
		var points: Array = group["points"]
		instances.instance_count = points.size()
		for index: int in points.size():
			instances.set_instance_transform_2d(index, Transform2D(0.0, points[index] + offset))
		batch.multimesh = instances
		batch.texture = texture
		add_child(batch)
		_batches.append(batch)

## Native pixel bounds with top-left UV origin, matching the existing 2D marker mesh convention.
func _quad(size: Vector2) -> ArrayMesh:
	var half: Vector2 = size / 2.0
	var arrays: Array = []
	arrays.resize(Mesh.ARRAY_MAX)
	arrays[Mesh.ARRAY_VERTEX] = PackedVector2Array([Vector2(-half.x, -half.y), Vector2(half.x, -half.y), Vector2(half.x, half.y), Vector2(-half.x, half.y)])
	arrays[Mesh.ARRAY_TEX_UV] = PackedVector2Array([Vector2(0, 0), Vector2(1, 0), Vector2(1, 1), Vector2(0, 1)])
	arrays[Mesh.ARRAY_INDEX] = PackedInt32Array([0, 1, 2, 0, 2, 3])
	var mesh := ArrayMesh.new()
	mesh.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, arrays)
	return mesh

func _placeholder(phase: int) -> Texture2D:
	if _textures.has(phase):
		return _textures[phase]
	var style: Dictionary = ArtData.params().get("plants", {})
	var pixels: int = maxi(12, int(style.get("placeholder_px", 16)))
	var image := Image.create(pixels, pixels, false, Image.FORMAT_RGBA8)
	image.fill(Color.TRANSPARENT)
	var outline := Color(str(style.get("outline", "141210")))
	var colors: Array = style.get("phase_colors", ["777461", "718253", "9c9b56", "77704f"])
	var color := Color(str(colors[clampi(phase, 0, colors.size() - 1)]))
	var middle: int = floori(pixels / 2.0)
	image.fill_rect(Rect2i(middle - 1, 3, 3, pixels - 3), outline)
	image.fill_rect(Rect2i(middle, 4, 1, pixels - 5), color)
	if phase == 1 or phase == 2:
		image.fill_rect(Rect2i(2, 3, pixels - 4, pixels - 7), outline)
		image.fill_rect(Rect2i(3, 4, pixels - 6, pixels - 9), color)
	if phase == 2:
		var ripe := Color(str(style.get("ripe_accent", "c6a26a")))
		image.fill_rect(Rect2i(4, 5, 2, 2), ripe)
		image.fill_rect(Rect2i(pixels - 6, 7, 2, 2), ripe)
	var texture := ImageTexture.create_from_image(image)
	_textures[phase] = texture
	return texture
