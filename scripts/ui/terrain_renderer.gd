extends Node2D

const TILE_SIZE: int = 16
const SPRITE_ORIGIN: Vector2 = Vector2(448, 28)
const ATLAS_PATH: String = "res://assets/sprites/terrain/terrain_v8.png"
# Rust codes: Plain 0, Forest 1, Mountain 2, Hill 3, River 4.
# Atlas rows: Plain, Forest, Hill, Mountain, River.
const TERRAIN_ROWS: Dictionary = {0: 0, 1: 1, 2: 3, 3: 2, 4: 4}
const RIVER: int = 4
const SHORE_LAND: Color = Color("c9b98a")
const SHORE_WATER: Color = Color("2f566f")
var _world_sim: WorldSimNode
var _layer: TileMapLayer
var _terrain: PackedByteArray = PackedByteArray()
var _width: int = 0
var _height: int = 0

func _ready() -> void:
	z_index = 0
	_world_sim = get_node("../WorldSim") as WorldSimNode
	var atlas: TileSetAtlasSource = TileSetAtlasSource.new()
	atlas.texture = load(ATLAS_PATH) as Texture2D
	atlas.texture_region_size = Vector2i(TILE_SIZE, TILE_SIZE)
	for row: int in range(5):
		for variant: int in range(4):
			atlas.create_tile(Vector2i(variant, row))
	var tiles: TileSet = TileSet.new()
	tiles.tile_size = Vector2i(TILE_SIZE, TILE_SIZE)
	tiles.add_source(atlas, 0)
	_layer = TileMapLayer.new()
	_layer.tile_set = tiles
	_layer.position = SPRITE_ORIGIN
	_layer.show_behind_parent = true
	_layer.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	add_child(_layer)
	_world_sim.connect("world_reset", refresh_snapshot)
	refresh_snapshot()

func refresh_snapshot() -> void:
	var snapshot: Dictionary = _world_sim.get_terrain_snapshot()
	_width = int(snapshot.get("width", 0))
	_height = int(snapshot.get("height", 0))
	_terrain = snapshot.get("terrain", PackedByteArray())
	_layer.clear()
	for y: int in range(_height):
		for x: int in range(_width):
			var code: int = _terrain[y * _width + x]
			var row: int = int(TERRAIN_ROWS.get(code, 0))
			var variant: int = (((x * 73856093) ^ (y * 19349663)) & 0x7fffffff) % 4
			_layer.set_cell(Vector2i(x, y), 0, Vector2i(variant, row))
	queue_redraw()

func _is_land(x: int, y: int) -> bool:
	return x >= 0 and y >= 0 and x < _width and y < _height and _terrain[y * _width + x] != RIVER

func _draw() -> void:
	for y: int in range(_height):
		for x: int in range(_width):
			if _terrain[y * _width + x] != RIVER:
				continue
			var p: Vector2 = SPRITE_ORIGIN + Vector2(x, y) * TILE_SIZE
			if _is_land(x, y - 1):
				draw_rect(Rect2(p + Vector2(0, -2), Vector2(16, 2)), SHORE_LAND)
				draw_rect(Rect2(p, Vector2(16, 1)), SHORE_WATER)
			if _is_land(x, y + 1):
				draw_rect(Rect2(p + Vector2(0, 16), Vector2(16, 2)), SHORE_LAND)
				draw_rect(Rect2(p + Vector2(0, 15), Vector2(16, 1)), SHORE_WATER)
			if _is_land(x - 1, y):
				draw_rect(Rect2(p + Vector2(-2, 0), Vector2(2, 16)), SHORE_LAND)
				draw_rect(Rect2(p, Vector2(1, 16)), SHORE_WATER)
			if _is_land(x + 1, y):
				draw_rect(Rect2(p + Vector2(16, 0), Vector2(2, 16)), SHORE_LAND)
				draw_rect(Rect2(p + Vector2(15, 0), Vector2(1, 16)), SHORE_WATER)
