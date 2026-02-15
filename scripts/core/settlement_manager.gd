extends RefCounted

const SettlementDataScript = preload("res://scripts/core/settlement_data.gd")

var _settlements: Dictionary = {}  # id -> settlement
var _next_id: int = 1


func create_settlement(cx: int, cy: int, tick: int) -> RefCounted:
	var settlement = SettlementDataScript.new()
	settlement.id = _next_id
	settlement.center_x = cx
	settlement.center_y = cy
	settlement.founding_tick = tick
	_next_id += 1
	_settlements[settlement.id] = settlement
	return settlement


func get_settlement(id: int) -> RefCounted:
	return _settlements.get(id, null)


func get_all_settlements() -> Array:
	return _settlements.values()


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


func add_member(settlement_id: int, entity_id: int) -> void:
	var settlement = _settlements.get(settlement_id, null)
	if settlement == null:
		return
	if not settlement.member_ids.has(entity_id):
		settlement.member_ids.append(entity_id)


func remove_member(settlement_id: int, entity_id: int) -> void:
	var settlement = _settlements.get(settlement_id, null)
	if settlement == null:
		return
	var idx: int = settlement.member_ids.find(entity_id)
	if idx >= 0:
		settlement.member_ids.remove_at(idx)


func add_building(settlement_id: int, building_id: int) -> void:
	var settlement = _settlements.get(settlement_id, null)
	if settlement == null:
		return
	if not settlement.building_ids.has(building_id):
		settlement.building_ids.append(building_id)


func get_settlement_population(id: int) -> int:
	var settlement = _settlements.get(id, null)
	if settlement == null:
		return 0
	return settlement.member_ids.size()


func to_save_data() -> Array:
	var result: Array = []
	var all_settlements: Array = _settlements.values()
	for i in range(all_settlements.size()):
		result.append(all_settlements[i].to_dict())
	return result


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
