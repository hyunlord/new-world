extends SceneTree

const Plans = preload("res://scripts/test/art/capture_plans.gd")


func _init() -> void:
	assert(Plans.ART.size() == 10)
	assert(Plans.ART_LONG.size() == 21)
	for plan: Array in [Plans.ART_LONG, Plans.ART_GOD]:
		var previous: int = -1
		for shot: Array in plan:
			var clock: PackedStringArray = str(shot[1]).split(":")
			var minute: int = (int(shot[0]) - 1) * 1440 + int(clock[0]) * 60 + int(clock[1])
			assert(minute >= previous, "capture plans must never rewind")
			previous = minute
	var snapshot: Dictionary = {
		"agent_ids": PackedInt64Array([12, 3, 1]),
		"carried_woods": PackedInt32Array([4, 2, 0]),
		"states": PackedByteArray([1, 1, 1]),
		"xs": PackedInt32Array([7, 9, 11]),
		"ys": PackedInt32Array([8, 10, 12]),
	}
	assert(Plans.wood_carrier_tile(snapshot) == Vector2(9, 10))
	snapshot["states"] = PackedByteArray([0, 0, 1])
	assert(Plans.wood_carrier_tile(snapshot) == Vector2(-1, -1))
	var kinds: Array[String] = []
	for shot: Array in Plans.ART_GOD:
		if not str(shot[3]).is_empty():
			kinds.append(str(shot[3]))
	assert(kinds == ["spoil_food", "cold_snap", "gift_wood", "windfall", "rain"])
	print("PASS capture_plans_p6: 21 natural frames, forward-only intervention schedule, deterministic FC selection")
	quit(0)
