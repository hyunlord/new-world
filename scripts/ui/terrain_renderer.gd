extends Node2D

const TILE_SIZE: int = 16
const SPRITE_ORIGIN: Vector2 = Vector2(448, 28)
const ART = preload("res://scripts/ui/art_sprite_library.gd")
const MANIFEST: String = "res://assets/art/terrain/terrain_manifest.json"
const LAYERS: Array[String] = ["water", "grass", "forest_floor", "camp_earth", "hill", "rock"]
const SEASON = preload("res://scripts/ui/art_season_tint.gd")
const LAYOUT = preload("res://scripts/ui/nature_layout.gd")
const RIVER: int = 4
var _world_sim: WorldSimNode
var _layers: Dictionary = {}
var _terrain: PackedByteArray = PackedByteArray()
var _width: int = 0
var _height: int = 0
var _camp: Dictionary = {}
var _building_fingerprint: int = -1
var _decal_fingerprint: int = -1
var _decals: Node2D
var _frame: int = 0

func _ready() -> void:
	z_index = 0
	_world_sim = get_node("../WorldSim") as WorldSimNode
	var manifest: Dictionary = JSON.parse_string(FileAccess.get_file_as_string(MANIFEST))
	for description: Dictionary in manifest["layers"]:
		var atlas: TileSetAtlasSource = TileSetAtlasSource.new()
		atlas.texture = load(str(description["path"])) as Texture2D
		atlas.texture_region_size = Vector2i(TILE_SIZE, TILE_SIZE)
		for tile: Dictionary in description["tiles"]:
			var coords: Array = tile["atlas_coords"]
			var cell: Vector2i = Vector2i(int(coords[0]), int(coords[1]))
			atlas.create_tile(cell)
			if int(tile["frames"]) > 1:
				atlas.set_tile_animation_columns(cell, int(tile["frames"]))
				atlas.set_tile_animation_frames_count(cell, int(tile["frames"]))
				atlas.set_tile_animation_speed(cell, float(tile["fps"]))
		var tiles: TileSet = TileSet.new()
		tiles.tile_size = Vector2i(TILE_SIZE, TILE_SIZE)
		tiles.add_source(atlas, 0)
		var layer: TileMapLayer = TileMapLayer.new()
		layer.name = str(description["name"])
		layer.tile_set = tiles
		layer.position = SPRITE_ORIGIN - Vector2(8, 8)
		layer.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
		add_child(layer)
		_layers[str(description["name"])] = layer
	_decals = Node2D.new()
	_decals.name = "Decals"
	_decals.z_index = 1
	add_child(_decals)
	_world_sim.connect("world_reset", refresh_snapshot)
	refresh_snapshot()

func _process(_delta: float) -> void:
	_update_season()
	_frame += 1
	if _frame % 30 == 0:
		_refresh_camp()
		_refresh_decals()

func refresh_snapshot() -> void:
	var snapshot: Dictionary = SimReads.read(_world_sim, "get_terrain_snapshot")
	_width = int(snapshot.get("width", 0))
	_height = int(snapshot.get("height", 0))
	_terrain = snapshot.get("terrain", PackedByteArray())
	_building_fingerprint = -1
	_decal_fingerprint = -1
	_refresh_camp()
	for layer_name: String in LAYERS:
		_fill_layer(layer_name)
	_refresh_decals()

func _code(x: int, y: int) -> int:
	if x < 0 or y < 0 or x >= _width or y >= _height:
		return -1
	return _terrain[y * _width + x]

func _member(layer_name: String, x: int, y: int) -> bool:
	var code: int = _code(x, y)
	if code < 0:
		return false
	match layer_name:
		"water": return code == RIVER
		"grass": return code != RIVER
		"forest_floor": return code == 1
		"camp_earth": return _camp.has(Vector2i(x, y))
		"hill": return code == 2 or code == 3
		"rock": return code == 2
	return false

func _mask(layer_name: String, x: int, y: int) -> int:
	return int(_member(layer_name, x-1, y-1)) + 2*int(_member(layer_name, x, y-1)) + 4*int(_member(layer_name, x-1, y)) + 8*int(_member(layer_name, x, y))

func _fill_layer(layer_name: String) -> void:
	var layer: TileMapLayer = _layers[layer_name] as TileMapLayer
	layer.clear()
	for y: int in range(_height + 1):
		for x: int in range(_width + 1):
			var mask: int = _mask(layer_name, x, y)
			if mask == 0:
				continue
			var count: int = 4 if layer_name == "water" else (8 if mask == 15 else 2)
			var variant: int = (((x * 73856093) ^ (y * 19349663)) & 0x7fffffff) % count
			var column: int = variant * 4 if layer_name == "water" else variant
			layer.set_cell(Vector2i(x, y), 0, Vector2i(column, mask))

func _refresh_camp() -> void:
	var snapshot: Dictionary = SimReads.read(_world_sim, "get_building_snapshot")
	var fingerprint: int = hash(snapshot)
	if fingerprint == _building_fingerprint:
		return
	_building_fingerprint = fingerprint
	_camp.clear()
	var xs: PackedInt32Array = snapshot.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snapshot.get("ys", PackedInt32Array())
	var kinds: PackedByteArray = snapshot.get("kinds", PackedByteArray())
	for i: int in range(xs.size()):
		var radius: int = 2 if kinds[i] == 0 else 1
		for dy: int in range(-radius, radius+1):
			for dx: int in range(-radius, radius+1):
				var p: Vector2i = Vector2i(xs[i]+dx, ys[i]+dy)
				if _code(p.x, p.y) == 0:
					_camp[p] = true
	_fill_layer("camp_earth")

func _shore(x: int, y: int) -> bool:
	for d: Vector2i in [Vector2i.LEFT, Vector2i.RIGHT, Vector2i.UP, Vector2i.DOWN]:
		if _code(x+d.x, y+d.y) == RIVER:
			return true
	return false

func _refresh_decals() -> void:
	var buildings: Dictionary = SimReads.read(_world_sim, "get_building_snapshot")
	var resources: Dictionary = SimReads.read(_world_sim, "get_resource_snapshot")
	var sites: Dictionary = SimReads.read(_world_sim, "get_construction_snapshot")
	var settlements: Dictionary = SimReads.read(_world_sim, "get_settlement_snapshot")
	var fingerprint: int = hash([buildings, sites, resources.get("xs"), resources.get("ys"), resources.get("kinds"), settlements.get("formation_xs"), settlements.get("formation_ys")])
	if fingerprint == _decal_fingerprint:
		return
	_decal_fingerprint = fingerprint
	for child: Node in _decals.get_children():
		child.queue_free()
	var excluded: Dictionary = {}
	for snapshot: Dictionary in [buildings, sites, resources]:
		var xs: PackedInt32Array = snapshot.get("xs", PackedInt32Array())
		var ys: PackedInt32Array = snapshot.get("ys", PackedInt32Array())
		for i: int in range(xs.size()):
			excluded[Vector2i(xs[i], ys[i])] = true
	var cleared: Dictionary = LAYOUT.clearings(buildings,sites,resources,settlements,3)
	var grass: Array[Vector2i] = []
	var forest: Array[Vector2i] = []
	for y: int in range(_height):
		for x: int in range(_width):
			var tile: Vector2i = Vector2i(x,y)
			if cleared.has(tile) or excluded.has(tile) or _shore(x,y):
				continue
			if _code(x,y) == 0:
				grass.append(tile)
			elif _code(x,y) == 1:
				forest.append(tile)
	var patches: Dictionary = LAYOUT.patches(grass,floori(_terrain.count(0)*0.04),0x5DEC)
	patches.merge(LAYOUT.patches(forest,floori(_terrain.count(1)*0.03),0x5DEC))
	for y: int in range(_height):
		for x: int in range(_width):
			var code: int = _code(x, y)
			var h: int = ((x*73856093) ^ (y*19349663)) & 0x7fffffff
			var tile: Vector2i = Vector2i(x,y)
			var selected: bool = h % 100 < 6 if code == RIVER else patches.has(tile)
			if not selected or cleared.has(tile) or excluded.has(tile):
				continue
			var sprite_id: String = "N3_%02d" % (16 if code == RIVER else 1+(h>>8)%15)
			if ART.frame(sprite_id).is_empty():
				continue
			var sprite: Sprite2D = Sprite2D.new()
			ART.apply(sprite, sprite_id)
			sprite.position = SPRITE_ORIGIN + Vector2(x*16+8+(h>>12)%5-2, y*16+8+(h>>16)%5-2)
			sprite.set_meta("art_kind", "decal")
			sprite.set_meta("tile_x", x)
			sprite.set_meta("tile_y", y)
			_decals.add_child(sprite)

func _update_season() -> void:
	var tint: Color = SEASON.resolve(SimReads.read(_world_sim, "get_time_snapshot"))
	for layer: TileMapLayer in _layers.values():
		layer.modulate = tint
	_decals.modulate = tint
