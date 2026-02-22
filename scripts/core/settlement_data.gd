extends RefCounted

var id: int = 0
var center_x: int = 0
var center_y: int = 0
var founding_tick: int = 0
var culture_id: String = "proto_syllabic"
var member_ids: Array = []
var building_ids: Array = []
## 정착지 공유 가치관 캐시 [Axelrod 1997] - value_system이 200 tick마다 재계산
var shared_values: Dictionary = {}


func to_dict() -> Dictionary:
	return {
		"id": id,
		"center_x": center_x,
		"center_y": center_y,
		"founding_tick": founding_tick,
		"culture_id": culture_id,
		"member_ids": member_ids.duplicate(),
		"building_ids": building_ids.duplicate(),
	}


static func from_dict(data: Dictionary) -> RefCounted:
	var script = load("res://scripts/core/settlement_data.gd")
	var settlement = script.new()
	settlement.id = data.get("id", 0)
	settlement.center_x = data.get("center_x", 0)
	settlement.center_y = data.get("center_y", 0)
	settlement.founding_tick = data.get("founding_tick", 0)
	settlement.culture_id = data.get("culture_id", "proto_nature")

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
