extends SceneTree

const Plans = preload("res://scripts/test/art/capture_plans.gd")


func _init() -> void:
	assert(Plans.ART.size() == 10)
	const EXPECTED_LONG: Array[Array] = [
		[10, "12:00", "F2"], [10, "15:00", "F2"], [10, "15:00", "F5"],
		[15, "15:00", "F2"], [15, "15:30", "FC"], [16, "15:00", "FC"],
		[18, "12:00", "F2"], [23, "02:00", "F2N"],
		[24, "12:00", "F2"], [24, "12:00", "F1"], [24, "23:00", "F2N"],
		[25, "02:00", "F2N"], [27, "02:00", "F2N"], [27, "07:15", "F2"],
		[28, "02:00", "F2N"], [28, "02:00", "F2Z"], [29, "02:00", "F2N"],
		[29, "06:30", "F2"], [29, "07:15", "F2"],
		[32, "12:00", "F2"], [40, "12:00", "F2"],
		[56, "02:00", "F2N"], [57, "02:00", "F2N"], [57, "03:01", "DE"],
		[84, "02:00", "F2N"], [85, "02:00", "F2N"], [85, "04:01", "DE"],
	]
	assert(Plans.ART_LONG.size() == 27 and Plans.ART_LONG == EXPECTED_LONG)
	for plan: Array in [Plans.ART, Plans.ART_LONG, Plans.ART_SOCIETY, Plans.ART_GOD]:
		var previous: int = -1
		var resets: int = 0
		for shot: Array in plan:
			if str(shot[0]) == "RESET":
				assert(shot == ["RESET", 20261013], "only the measured society seed may reset")
				resets += 1
				assert(resets == 1, "a second reset cannot disguise a rewind")
				previous = -1
				continue
			var clock: PackedStringArray = str(shot[1]).split(":")
			var minute: int = (int(shot[0]) - 1) * 1440 + int(clock[0]) * 60 + int(clock[1])
			assert(minute >= previous, "capture plans must never rewind within a world")
			previous = minute
		var society_world: bool = plan == Plans.ART_SOCIETY or plan == Plans.ART_GOD
		assert(resets == (1 if society_world else 0))
	assert(Plans.ART_SOCIETY[0] == ["RESET", 20261013])
	assert(Plans.ART_GOD[10] == ["RESET", 20261013])
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
	var society_kinds: Array[String] = []
	var society: bool = false
	for shot: Array in Plans.ART_GOD:
		if str(shot[0]) == "RESET":
			society = true
			continue
		if not str(shot[3]).is_empty():
			if society:
				society_kinds.append(str(shot[3]))
				assert(shot == [50, "17:00", "F2", "rain", 13, 0.5, "soc_rain", "daughter"])
			else:
				kinds.append(str(shot[3]))
	assert(kinds == ["spoil_food", "cold_snap", "gift_wood", "windfall", "rain"])
	assert(society_kinds == ["rain"], "the second world adds only targeted daughter rain")
	assert([51, "02:00", "F2N", "", 0, 1.15, "soc_rain_night", "daughter"] in Plans.ART_GOD)
	assert([51, "02:00", "F2N", "", 0, 1.15, "soc_original_night", "orig"] in Plans.ART_GOD)
	assert([51, "02:00", "FV", "", 0, 1.15, "soc_both_night", "all"] in Plans.ART_GOD)
	print("PASS capture_plans_p6: v17 exact27 natural frames, explicit society reset, original five casts plus daughter rain, deterministic FC selection")
	quit(0)
