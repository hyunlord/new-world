extends RefCounted

var id: int = 0
var center_x: int = 0
var center_y: int = 0
var founding_tick: int = 0
var member_ids: Array[int] = []
var building_ids: Array[int] = []


func to_dict() -> Dictionary:
	return {
		"id": id,
		"center_x": center_x,
		"center_y": center_y,
		"founding_tick": founding_tick,
		"member_ids": Array(member_ids),
		"building_ids": Array(building_ids),
	}


static func from_dict(data: Dictionary) -> RefCounted:
	var script = load("res://scripts/core/settlement_data.gd")
	var settlement = script.new()
	settlement.id = data.get("id", 0)
	settlement.center_x = data.get("center_x", 0)
	settlement.center_y = data.get("center_y", 0)
	settlement.founding_tick = data.get("founding_tick", 0)

	settlement.member_ids.clear()
	var raw_member_ids = data.get("member_ids", [])
	if raw_member_ids is Array:
		for i in range(raw_member_ids.size()):
			settlement.member_ids.append(int(raw_member_ids[i]))

	settlement.building_ids.clear()
	var raw_building_ids = data.get("building_ids", [])
	if raw_building_ids is Array:
		for i in range(raw_building_ids.size()):
			settlement.building_ids.append(int(raw_building_ids[i]))

	return settlement
