class_name WorldData
extends RefCounted

var width: int = 0
var height: int = 0
var biomes: PackedInt32Array
var elevation: PackedFloat32Array
var moisture: PackedFloat32Array
var temperature: PackedFloat32Array
var _entity_map: Dictionary = {}  # Vector2i → Array[int]


## Initialize world arrays
func init_world(w: int, h: int) -> void:
	width = w
	height = h
	var size: int = w * h
	biomes.resize(size)
	elevation.resize(size)
	moisture.resize(size)
	temperature.resize(size)
	_entity_map.clear()


## Convert 2D coords to 1D index
func _idx(x: int, y: int) -> int:
	return y * width + x


## Check if coordinates are within bounds
func is_valid(x: int, y: int) -> bool:
	return x >= 0 and x < width and y >= 0 and y < height


## Tile getters
func get_biome(x: int, y: int) -> int:
	return biomes[_idx(x, y)]


func get_elevation(x: int, y: int) -> float:
	return elevation[_idx(x, y)]


func get_moisture(x: int, y: int) -> float:
	return moisture[_idx(x, y)]


func get_temperature(x: int, y: int) -> float:
	return temperature[_idx(x, y)]


## Set all tile data at once
func set_tile(x: int, y: int, b: int, e: float, m: float, t: float) -> void:
	var idx: int = _idx(x, y)
	biomes[idx] = b
	elevation[idx] = e
	moisture[idx] = m
	temperature[idx] = t


## Check if a tile is walkable (move cost > 0)
func is_walkable(x: int, y: int) -> bool:
	if not is_valid(x, y):
		return false
	var biome: int = biomes[_idx(x, y)]
	return GameConfig.BIOME_MOVE_COST.get(biome, 0.0) > 0.0


## Get movement cost for a tile
func get_move_cost(x: int, y: int) -> float:
	if not is_valid(x, y):
		return 0.0
	return GameConfig.BIOME_MOVE_COST.get(biomes[_idx(x, y)], 0.0)


## Get walkable neighbors (8-directional)
func get_walkable_neighbors(x: int, y: int) -> Array[Vector2i]:
	var neighbors: Array[Vector2i] = []
	for dy in range(-1, 2):
		for dx in range(-1, 2):
			if dx == 0 and dy == 0:
				continue
			var nx: int = x + dx
			var ny: int = y + dy
			if is_walkable(nx, ny):
				neighbors.append(Vector2i(nx, ny))
	return neighbors


## Register an entity at a position
func register_entity(pos: Vector2i, entity_id: int) -> void:
	if not _entity_map.has(pos):
		_entity_map[pos] = []
	_entity_map[pos].append(entity_id)


## Unregister an entity from a position
func unregister_entity(pos: Vector2i, entity_id: int) -> void:
	if _entity_map.has(pos):
		_entity_map[pos].erase(entity_id)
		if _entity_map[pos].is_empty():
			_entity_map.erase(pos)


## Move an entity between positions
func move_entity(from_pos: Vector2i, to_pos: Vector2i, entity_id: int) -> void:
	unregister_entity(from_pos, entity_id)
	register_entity(to_pos, entity_id)


## Get entity IDs at a position
func get_entities_at(pos: Vector2i) -> Array:
	return _entity_map.get(pos, [])
