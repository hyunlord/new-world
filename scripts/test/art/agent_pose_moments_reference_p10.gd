extends RefCounted

var _previous: Dictionary = {}
var _pickup: Dictionary = {}
var _meal: Dictionary = {}

func update(agent_id: int, index: int, snapshot: Dictionary, tile: Vector2i, stationary: bool, deadwood: bool, hearths: Array, delta: float) -> Dictionary:
	var woods: Variant = snapshot.get("carried_woods")
	var hungers: Variant = snapshot.get("hungers")
	var has_wood: bool = woods is PackedInt32Array and index >= 0 and index < woods.size()
	var has_hunger: bool = hungers is PackedFloat32Array and index >= 0 and index < hungers.size()
	var previous: Dictionary = _previous.get(agent_id, {})
	if _pickup.has(agent_id):
		var pickup: Dictionary = _pickup[agent_id]
		pickup["remaining"] = maxf(0.0, float(pickup["remaining"]) - maxf(delta, 0.0))
		if tile != Vector2i(pickup["tile"]) or float(pickup["remaining"]) <= 0.0 or not has_wood:
			_pickup.erase(agent_id)
	var remaining: float = maxf(0.0, float(_meal.get(agent_id, 0.0)) - maxf(delta, 0.0))
	if not stationary or not has_hunger or not has_wood:
		remaining = 0.0
	if has_wood and previous.has("wood") and int(woods[index]) > int(previous["wood"]) and deadwood:
		_pickup[agent_id] = {"tile": tile, "remaining": 1.0}
	if has_wood and has_hunger and previous.has("hunger") and float(previous["hunger"]) - float(hungers[index]) >= 20.0 and stationary:
		for hearth: Vector2i in hearths:
			if maxi(absi(tile.x - hearth.x), absi(tile.y - hearth.y)) <= 8:
				remaining = 3.0
				break
	_meal[agent_id] = remaining
	var current: Dictionary = {}
	if has_wood:
		current["wood"] = int(woods[index])
	if has_hunger:
		current["hunger"] = float(hungers[index])
	_previous[agent_id] = current
	return {"pickup": _pickup.has(agent_id), "meal": remaining > 0.0}

func retire(agent_id: int) -> void:
	_previous.erase(agent_id)
	_pickup.erase(agent_id)
	_meal.erase(agent_id)

func clear() -> void:
	_previous.clear()
	_pickup.clear()
	_meal.clear()
