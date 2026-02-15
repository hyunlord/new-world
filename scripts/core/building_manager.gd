extends RefCounted

const BuildingDataScript = preload("res://scripts/core/building_data.gd")

var _buildings: Dictionary = {}  # id -> BuildingData
var _tile_map: Dictionary = {}   # "x,y" -> building_id
var _next_id: int = 1


func _tile_key(x: int, y: int) -> String:
	return "%d,%d" % [x, y]


## Place a new building at the given tile (returns null if occupied)
func place_building(type: String, x: int, y: int) -> RefCounted:
	var key: String = _tile_key(x, y)
	if _tile_map.has(key):
		return null

	var building = BuildingDataScript.new()
	building.id = _next_id
	_next_id += 1
	building.building_type = type
	building.tile_x = x
	building.tile_y = y

	_buildings[building.id] = building
	_tile_map[key] = building.id

	SimulationBus.emit_event("building_placed", {
		"building_id": building.id,
		"building_type": type,
		"tile_x": x,
		"tile_y": y,
	})

	return building


## Get the building at a specific tile (or null)
func get_building_at(x: int, y: int) -> RefCounted:
	var key: String = _tile_key(x, y)
	var bid: int = _tile_map.get(key, -1)
	if bid < 0:
		return null
	return _buildings.get(bid, null)


## Get all buildings of a given type
func get_buildings_by_type(type: String) -> Array:
	var result: Array = []
	var all_buildings: Array = _buildings.values()
	for i in range(all_buildings.size()):
		var building = all_buildings[i]
		if building.building_type == type:
			result.append(building)
	return result


## Find the nearest building of a type (optionally only completed ones)
func get_nearest_building(x: int, y: int, type: String, built_only: bool = false) -> RefCounted:
	var nearest: RefCounted = null
	var best_dist: float = 999999.0
	var all_buildings: Array = _buildings.values()
	for i in range(all_buildings.size()):
		var building = all_buildings[i]
		if building.building_type != type:
			continue
		if built_only and not building.is_built:
			continue
		var dist: float = float(absi(building.tile_x - x) + absi(building.tile_y - y))
		if dist < best_dist:
			best_dist = dist
			nearest = building
	return nearest


## Remove a building by ID
func remove_building(id: int) -> void:
	if not _buildings.has(id):
		return
	var building = _buildings[id]
	var key: String = _tile_key(building.tile_x, building.tile_y)
	_tile_map.erase(key)
	_buildings.erase(id)
	SimulationBus.emit_event("building_destroyed", {
		"building_id": id,
		"building_type": building.building_type,
		"tile_x": building.tile_x,
		"tile_y": building.tile_y,
	})


## Get all buildings
func get_all_buildings() -> Array:
	return _buildings.values()


## Get total building count
func get_building_count() -> int:
	return _buildings.size()


## Serialization
func to_save_data() -> Array:
	var result: Array = []
	var all_buildings: Array = _buildings.values()
	for i in range(all_buildings.size()):
		result.append(all_buildings[i].to_dict())
	return result


func load_save_data(data: Array) -> void:
	_buildings.clear()
	_tile_map.clear()
	_next_id = 1
	for i in range(data.size()):
		var item = data[i]
		if item is Dictionary:
			var building = BuildingDataScript.from_dict(item)
			_buildings[building.id] = building
			_tile_map[_tile_key(building.tile_x, building.tile_y)] = building.id
			if building.id >= _next_id:
				_next_id = building.id + 1
