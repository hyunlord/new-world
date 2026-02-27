extends RefCounted

const SettlementDataScript = preload("res://scripts/core/settlement/settlement_data.gd")

var _settlements: Dictionary = {}  # id -> settlement
var _next_id: int = 1


## Create and register a new settlement centered at (cx, cy), founded at the given tick.
func create_settlement(cx: int, cy: int, tick: int) -> RefCounted:
	var settlement = SettlementDataScript.new()
	settlement.id = _next_id
	settlement.center_x = cx
	settlement.center_y = cy
	settlement.founding_tick = tick
	_next_id += 1
	_settlements[settlement.id] = settlement
	return settlement


## Return the settlement with the given ID, or null if not found.
func get_settlement(id: int) -> RefCounted:
	return _settlements.get(id, null)


## Return all registered settlements.
func get_all_settlements() -> Array:
	return _settlements.values()


## Return the settlement whose center is closest (Manhattan distance) to (x, y).
func get_nearest_settlement(x: int, y: int) -> RefCounted:
	var nearest: RefCounted = null
	var best_dist: int = 2147483647
	var all_settlements: Array = _settlements.values()
	for i in range(all_settlements.size()):
		var settlement = all_settlements[i]
		var dist: int = absi(settlement.center_x - x) + absi(settlement.center_y - y)
		if dist < best_dist:
			best_dist = dist
			nearest = settlement
	return nearest


## Add an entity to a settlement's member list (no-op if already a member).
func add_member(settlement_id: int, entity_id: int) -> void:
	var settlement = _settlements.get(settlement_id, null)
	if settlement == null:
		return
	if not settlement.member_ids.has(entity_id):
		settlement.member_ids.append(entity_id)


## Remove an entity from a settlement's member list.
func remove_member(settlement_id: int, entity_id: int) -> void:
	var settlement = _settlements.get(settlement_id, null)
	if settlement == null:
		return
	var idx: int = settlement.member_ids.find(entity_id)
	if idx >= 0:
		settlement.member_ids.remove_at(idx)


## Register a building ID under a settlement (no-op if already registered).
func add_building(settlement_id: int, building_id: int) -> void:
	var settlement = _settlements.get(settlement_id, null)
	if settlement == null:
		return
	if not settlement.building_ids.has(building_id):
		settlement.building_ids.append(building_id)


## Return the current member count of a settlement (0 if not found).
func get_settlement_population(id: int) -> int:
	var settlement = _settlements.get(id, null)
	if settlement == null:
		return 0
	return settlement.member_ids.size()


## Serialize all settlements to an array of dictionaries for saving.
func to_save_data() -> Array:
	var result: Array = []
	var all_settlements: Array = _settlements.values()
	for i in range(all_settlements.size()):
		result.append(all_settlements[i].to_dict())
	return result


## Return the total number of registered settlements.
func get_settlement_count() -> int:
	return _settlements.size()


## Return settlements that have at least 1 member
func get_active_settlements() -> Array:
	var result: Array = []
	var all_settlements: Array = _settlements.values()
	for i in range(all_settlements.size()):
		if all_settlements[i].member_ids.size() > 0:
			result.append(all_settlements[i])
	return result


## Remove settlements with 0 members
func cleanup_empty_settlements() -> void:
	var to_remove: Array = []
	var all_settlements: Array = _settlements.values()
	for i in range(all_settlements.size()):
		if all_settlements[i].member_ids.size() == 0:
			to_remove.append(all_settlements[i].id)
	for i in range(to_remove.size()):
		_settlements.erase(to_remove[i])


## Remove a settlement by ID from the registry.
func remove_settlement(id: int) -> void:
	_settlements.erase(id)


## Restore all settlements from a saved array of dictionaries.
func load_save_data(data: Array) -> void:
	_settlements.clear()
	_next_id = 1
	for i in range(data.size()):
		var item = data[i]
		if item is Dictionary:
			var settlement = SettlementDataScript.from_dict(item)
			_settlements[settlement.id] = settlement
			if settlement.id >= _next_id:
				_next_id = settlement.id + 1
