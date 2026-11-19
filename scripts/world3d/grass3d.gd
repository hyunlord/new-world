# grass3d.gd (Terrain/Grass): camera-budgeted batches of small wind-swaying ground-cover cards.
# Bridge methods called: none. Terrain supplies its tile heights and the built/field/path presentation masks.
extends Node3D

const Data3D = preload("res://scripts/world3d/data3d.gd")
const View = preload("res://scripts/world/colony/view.gd")
const GRASS_SHADER = preload("res://shaders/world3d/grass.gdshader")
const CARD_TEXTURE = "res://assets/3d/textures/grass_card.png"

var _settings: Dictionary
var _material: ShaderMaterial
var _mesh: QuadMesh
var _width: int
var _height: int
var _chunk: int
var _per_tile: int
var _terrain_codes: Array
var _density: Array
var _radii: Array
var _heights: Array
var _zooms: Array
var _size_variation: float
var _terrain: PackedByteArray
var _seed: int
var _mask: ImageTexture
var _batches: Dictionary = {}
var _pending: Array[Vector2i] = []
var _desired: Dictionary = {}
var _focus_cell: Vector2i = Vector2i(-999, -999)
var _lod: int = -1


func _ready() -> void:
	_settings = Data3D.mood("detail.grass", {})
	_chunk = maxi(4, int(_settings.get("chunk_tiles", 12)))
	_per_tile = maxi(1, int(_settings.get("tufts_per_tile", 5)))
	_terrain_codes = _settings.get("terrain_codes", [0, 1, 3])
	_density = _settings.get("density", [0.10, 0.23, 0.65, 1.0, 1.0])
	_radii = _settings.get("radius", [39.0, 32.0, 24.0, 20.0, 18.0])
	_heights = _settings.get("height", [0.22, 0.38, 0.66, 0.91, 0.95])
	_zooms = _settings.get("zooms", [0.25, 0.5, 1.0, 2.0, 4.0])
	_size_variation = float(_settings.get("size_variation", 0.28))
	_material = ShaderMaterial.new()
	_material.shader = GRASS_SHADER
	_material.set_shader_parameter("cards", load(CARD_TEXTURE) as Texture2D)
	for key: String in ["clearing_limit", "wind_strength", "wind_speed", "cutoff", "winter_height"]:
		if _settings.has(key):
			_material.set_shader_parameter(key, float(_settings[key]))
	for key: String in ["shade", "root_shade", "winter_shade"]:
		var colour: Variant = _settings.get(key, [])
		if OS.get_cmdline_user_args().has("--style=s2"):
			colour = Data3D.mood("style2.grass_" + key, colour)
		_material.set_shader_parameter(key, Data3D.colour(colour, Color.WHITE))
	_mesh = QuadMesh.new()
	_mesh.material = _material
	_mesh.size = Vector2(float(_settings.get("width", 0.48)), 1.0)
	_mesh.center_offset = Vector3(0.0, 0.5, 0.0)


func reset_map(width: int, height: int, terrain: PackedByteArray, seed: int) -> void:
	_width = width
	_height = height
	_terrain = terrain
	_seed = seed
	_material.set_shader_parameter("map_size", Vector2(width, height))
	_mask = null
	_lod = -1
	_desired.clear()
	_pending.clear()
	for batch: MultiMeshInstance3D in _batches.values():
		batch.queue_free()
	_batches.clear()


func set_maps(paths: ImageTexture, biome: ImageTexture) -> void:
	_material.set_shader_parameter("path_tex", paths)
	_material.set_shader_parameter("biome_tex", biome)


func set_built(tiles: Dictionary) -> void:
	var bytes: PackedByteArray = PackedByteArray()
	bytes.resize(_width * _height)
	for tile: Vector2i in tiles:
		if tile.x >= 0 and tile.x < _width and tile.y >= 0 and tile.y < _height:
			bytes[tile.y * _width + tile.x] = 255
	var image: Image = Image.create_from_data(_width, _height, false, Image.FORMAT_R8, bytes)
	if _mask == null:
		_mask = ImageTexture.create_from_image(image)
		_material.set_shader_parameter("built_tex", _mask)
	else:
		_mask.update(image)


func refresh(focus: Vector2, zoom: float) -> void:
	var lod: int = 0
	for i: int in _zooms.size():
		if zoom >= float(_zooms[i]):
			lod = i
	var coarse: Vector2i = Vector2i((focus / (float(_chunk) * 0.5)).floor())
	if coarse != _focus_cell or lod != _lod:
		_focus_cell = coarse
		_lod = lod
		_select_chunks(focus, lod)
	# Leave every other frame available to the other presentation layers during initial streaming.
	if _pending.is_empty() or Engine.get_process_frames() % 2 != 0 or not bool(get_parent().call("_claim")):
		return
	var key: Vector2i = _pending.pop_front()
	var batch: MultiMeshInstance3D = _build_chunk(key)
	_apply_budget(batch, key)


func _select_chunks(focus: Vector2, lod: int) -> void:
	var radius: float = float(_radii[lod])
	_material.set_shader_parameter("radius", radius)
	_material.set_shader_parameter("height", float(_heights[lod]))
	var reach: float = radius + _chunk * 0.85
	var lo: Vector2i = Vector2i(((focus - Vector2.ONE * reach) / float(_chunk)).floor()).max(Vector2i.ZERO)
	var hi: Vector2i = Vector2i(((focus + Vector2.ONE * reach) / float(_chunk)).floor())
	hi = hi.min(Vector2i((Vector2(_width - 1, _height - 1) / float(_chunk)).floor()))
	var candidates: Array[Vector2i] = []
	for z: int in range(lo.y, hi.y + 1):
		for x: int in range(lo.x, hi.x + 1):
			var key: Vector2i = Vector2i(x, z)
			if focus.distance_squared_to(_centre(key)) <= reach * reach:
				candidates.append(key)
	candidates.sort_custom(func(a: Vector2i, b: Vector2i) -> bool:
		return focus.distance_squared_to(_centre(a)) < focus.distance_squared_to(_centre(b)))
	var cap: int = clampi(int(_settings.get("cap", 150000)), 0, 150000)
	var upper_bound: int = _chunk * _chunk * _per_tile
	var quota: int = ceili(float(upper_bound) * float(_density[lod]))
	_desired.clear()
	_pending.clear()
	for key: Vector2i in candidates:
		if cap <= 0 or quota <= 0:
			break
		_desired[key] = min(quota, cap)
		cap -= int(_desired[key])
		if not _batches.has(key):
			_pending.append(key)
	for key: Vector2i in _batches.keys():
		var batch: MultiMeshInstance3D = _batches[key]
		if _desired.has(key):
			_apply_budget(batch, key)
		else:
			batch.queue_free()
			_batches.erase(key)


func _centre(key: Vector2i) -> Vector2:
	return (Vector2(key) + Vector2(0.5, 0.5)) * _chunk


func _apply_budget(batch: MultiMeshInstance3D, key: Vector2i) -> void:
	var density_count: int = ceili(batch.multimesh.instance_count * float(_density[_lod]))
	batch.multimesh.visible_instance_count = mini(density_count, int(_desired[key]))


func _build_chunk(key: Vector2i) -> MultiMeshInstance3D:
	var start: Vector2i = key * _chunk
	var tiles: Array[Vector2i] = []
	for z: int in range(start.y, mini(start.y + _chunk, _height)):
		for x: int in range(start.x, mini(start.x + _chunk, _width)):
			if _terrain_codes.any(func(code: Variant) -> bool: return int(code) == _terrain[z * _width + x]):
				tiles.append(Vector2i(x, z))
	tiles.sort_custom(func(a: Vector2i, b: Vector2i) -> bool:
		return View.key_hash(_seed, a.x, a.y, 59) < View.key_hash(_seed, b.x, b.y, 59))
	var mm: MultiMesh = MultiMesh.new()
	mm.transform_format = MultiMesh.TRANSFORM_3D
	mm.use_colors = true
	mm.mesh = _mesh
	mm.instance_count = tiles.size() * _per_tile
	var i: int = 0
	var terrain: Node3D = get_parent()
	# Interleave one sample per tile in each tier. Reducing visible_instance_count thins every tile in the batch.
	for tier: int in _per_tile:
		for t: Vector2i in tiles:
			var h: int = View.key_hash(_seed, t.x, t.y, 73 + tier)
			var x: float = t.x + 0.10 + 0.80 * float(h & 1023) / 1024.0
			var z: float = t.y + 0.10 + 0.80 * float((h >> 10) & 1023) / 1024.0
			var y: float = float(terrain.call("height_at", x, z))
			var v: float = float((h >> 20) & 1023) / 1024.0
			var yaw: float = TAU * v
			var scale_factor: float = 1.0 + (2.0 * v - 1.0) * _size_variation
			var basis: Basis = Basis(Vector3.UP, yaw).scaled(Vector3.ONE * scale_factor)
			mm.set_instance_transform(i, Transform3D(basis, Vector3(x, y, z)))
			mm.set_instance_color(i, Color(0.75 + 0.25 * v, 1.0, 1.0))
			i += 1
	var batch: MultiMeshInstance3D = MultiMeshInstance3D.new()
	batch.name = "Grass_%d_%d" % [key.x, key.y]
	batch.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	batch.multimesh = mm
	add_child(batch)
	_batches[key] = batch
	return batch
