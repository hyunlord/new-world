extends RefCounted

var _food: PackedFloat32Array
var _wood: PackedFloat32Array
var _stone: PackedFloat32Array
var _width: int = 0
var _height: int = 0


func init_resources(w: int, h: int) -> void:
	_width = w
	_height = h
	var size: int = w * h
	_food.resize(size)
	_wood.resize(size)
	_stone.resize(size)


func _idx(x: int, y: int) -> int:
	return y * _width + x


func is_valid(x: int, y: int) -> bool:
	return x >= 0 and x < _width and y >= 0 and y < _height


## Fill resource arrays based on biome-resource mapping
func populate_from_biomes(world_data: RefCounted, rng: RandomNumberGenerator) -> void:
	for y in range(_height):
		for x in range(_width):
			var biome: int = world_data.get_biome(x, y)
			var res_def: Dictionary = GameConfig.BIOME_RESOURCES.get(biome, {})
			if res_def.is_empty():
				continue
			var idx: int = _idx(x, y)
			var food_min: float = res_def.get("food_min", 0.0)
			var food_max: float = res_def.get("food_max", 0.0)
			var wood_min: float = res_def.get("wood_min", 0.0)
			var wood_max: float = res_def.get("wood_max", 0.0)
			var stone_min: float = res_def.get("stone_min", 0.0)
			var stone_max: float = res_def.get("stone_max", 0.0)
			if food_max > 0.0:
				_food[idx] = food_min + rng.randf() * (food_max - food_min)
			if wood_max > 0.0:
				_wood[idx] = wood_min + rng.randf() * (wood_max - wood_min)
			if stone_max > 0.0:
				_stone[idx] = stone_min + rng.randf() * (stone_max - stone_min)


## Getters
func get_food(x: int, y: int) -> float:
	if not is_valid(x, y):
		return 0.0
	return _food[_idx(x, y)]


func get_wood(x: int, y: int) -> float:
	if not is_valid(x, y):
		return 0.0
	return _wood[_idx(x, y)]


func get_stone(x: int, y: int) -> float:
	if not is_valid(x, y):
		return 0.0
	return _stone[_idx(x, y)]


## Setters
func set_food(x: int, y: int, val: float) -> void:
	if is_valid(x, y):
		_food[_idx(x, y)] = val


func set_wood(x: int, y: int, val: float) -> void:
	if is_valid(x, y):
		_wood[_idx(x, y)] = val


func set_stone(x: int, y: int, val: float) -> void:
	if is_valid(x, y):
		_stone[_idx(x, y)] = val


## Generic accessors by resource type enum
func get_resource(x: int, y: int, resource_type: int) -> float:
	match resource_type:
		GameConfig.Resource.FOOD:
			return get_food(x, y)
		GameConfig.Resource.WOOD:
			return get_wood(x, y)
		GameConfig.Resource.STONE:
			return get_stone(x, y)
	return 0.0


func set_resource(x: int, y: int, resource_type: int, val: float) -> void:
	match resource_type:
		GameConfig.Resource.FOOD:
			set_food(x, y, val)
		GameConfig.Resource.WOOD:
			set_wood(x, y, val)
		GameConfig.Resource.STONE:
			set_stone(x, y, val)


## Harvest a resource from a tile, returns actual amount harvested
func harvest(x: int, y: int, resource_type: int, amount: float) -> float:
	if not is_valid(x, y):
		return 0.0
	var current: float = get_resource(x, y, resource_type)
	var actual: float = minf(amount, current)
	if actual > 0.0:
		set_resource(x, y, resource_type, current - actual)
	return actual


## Get the max resource value for a biome (used for regen cap)
func get_max_for_biome(biome: int, resource_type: int) -> float:
	var res_def: Dictionary = GameConfig.BIOME_RESOURCES.get(biome, {})
	if res_def.is_empty():
		return 0.0
	match resource_type:
		GameConfig.Resource.FOOD:
			return res_def.get("food_max", 0.0)
		GameConfig.Resource.WOOD:
			return res_def.get("wood_max", 0.0)
		GameConfig.Resource.STONE:
			return res_def.get("stone_max", 0.0)
	return 0.0


## Serialization
func to_save_data() -> Dictionary:
	return {
		"width": _width,
		"height": _height,
		"food": Array(_food),
		"wood": Array(_wood),
		"stone": Array(_stone),
	}


func load_save_data(data: Dictionary) -> void:
	_width = data.get("width", 0)
	_height = data.get("height", 0)
	var size: int = _width * _height
	_food.resize(size)
	_wood.resize(size)
	_stone.resize(size)
	var food_arr: Array = data.get("food", [])
	var wood_arr: Array = data.get("wood", [])
	var stone_arr: Array = data.get("stone", [])
	for i in range(size):
		if i < food_arr.size():
			_food[i] = food_arr[i]
		if i < wood_arr.size():
			_wood[i] = wood_arr[i]
		if i < stone_arr.size():
			_stone[i] = stone_arr[i]
