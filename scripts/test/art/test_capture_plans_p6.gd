extends SceneTree

const Plans = preload("res://scripts/test/art/capture_plans.gd")


func _init() -> void:
	assert(Plans.ART.size() == 10)
	const EXPECTED_LONG: Array[Array] = [
		[10, "12:00", "F2"], [10, "15:00", "F2"], [10, "15:00", "F5"],
		[15, "15:00", "F2"], [15, "15:00", "FC"], [16, "15:00", "FC"],
		[18, "12:00", "F2"], [23, "02:00", "F2N"],
		[24, "12:00", "F2"], [24, "12:00", "F1"], [24, "23:00", "F2N"],
		[26, "07:15", "F2"],
		[28, "02:00", "F2N"], [28, "02:00", "F2Z"], [28, "07:15", "F2"],
		[29, "02:00", "F2N"], [29, "06:30", "F2"], [29, "07:15", "F2"],
		[32, "12:00", "F2"], [40, "12:00", "F2"], [43, "15:00", "FC"],
		[55, "02:00", "F2N"], [56, "02:00", "F2N"],
		[57, "02:00", "F2N"], [57, "04:01", "DE"], [57, "07:15", "F2"],
		[71, "15:00", "FC"],
		[84, "02:00", "F2N"], [85, "02:00", "F2N"], [85, "07:15", "F2"],
	]
	assert(Plans.ART_LONG.size() == 30 and Plans.ART_LONG == EXPECTED_LONG)
	for plan: Array in [Plans.ART, Plans.ART_LONG, Plans.ART_SOCIETY, Plans.ART_GOD]:
		var previous: int = -1
		var resets: int = 0
		for shot: Array in plan:
			if str(shot[0]) == "RESET":
				assert(shot == ["RESET", 20261013], "only the measured society seed may reset")
				resets += 1
				previous = -1
				continue
			var clock: PackedStringArray = str(shot[1]).split(":")
			var minute: int = (int(shot[0]) - 1) * 1440 + int(clock[0]) * 60 + int(clock[1])
			assert(minute >= previous, "capture plans must never rewind within a world")
			previous = minute
		var expected_resets: int = 3 if plan == Plans.ART_GOD else (1 if plan == Plans.ART_SOCIETY else 0)
		assert(resets == expected_resets, "only explicit legacy/P/G worlds may reset")
	assert(Plans.ART_SOCIETY[0] == ["RESET", 20261013])
	assert(Plans.ART_GOD[12] == ["RESET", 20261013])
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
	var world_index: int = 0
	var paired_roles: Dictionary = {"P": [], "G": []}
	var stems: Array[String] = []
	for shot: Array in Plans.ART_GOD:
		if str(shot[0]) == "RESET":
			world_index += 1
			continue
		if world_index >= 2:
			var label: String = "P" if world_index == 2 else "G"
			var kind: String = "spoil_food" if label == "P" else "gift_food"
			var prefix: String = "p_spoil" if label == "P" else "p_gift"
			assert(shot.size() == 9)
			var metadata: Dictionary = shot[8]
			assert(metadata["paired_world"] == label and metadata["case_id"] == "a33")
			var role: String = metadata["role"]
			paired_roles[label].append([shot[0], role])
			var final: bool = role == "final"
			assert(shot[1] == ("12:00" if final else "06:00"))
			assert(shot[2] == ("FV" if final else "F2") and shot[7] == ("all" if final else "orig"))
			assert(shot[3] == (kind if role == "cast" else "") and shot[4] == (60 if role == "cast" else 0))
			var expected_stem: String = "%s_d%d_%s_%s_%s_%s" % [prefix, shot[0], str(shot[1]).replace(":", ""), shot[2], role, shot[7]]
			assert(metadata["stem_override"] == expected_stem and not stems.has(expected_stem))
			stems.append(expected_stem)
			continue
		if not str(shot[3]).is_empty():
			if world_index == 1:
				society_kinds.append(str(shot[3]))
				assert(shot == [50, "17:00", "F2", "rain", 13, 0.5, "soc_rain", "daughter"])
			else:
				kinds.append(str(shot[3]))
	assert(kinds == ["spoil_food", "rot_wood", "gift_food", "cold_snap", "gift_wood", "windfall", "rain"])
	assert(society_kinds == ["rain"], "the second world adds only targeted daughter rain")
	assert([51, "02:00", "F2N", "", 0, 1.15, "soc_rain_night", "daughter"] in Plans.ART_GOD)
	assert([51, "02:00", "F2N", "", 0, 1.15, "soc_original_night", "orig"] in Plans.ART_GOD)
	assert([51, "02:00", "FV", "", 0, 1.15, "soc_both_night", "all"] in Plans.ART_GOD)
	for label: String in ["P", "G"]:
		assert(paired_roles[label] == [[30, "cast"], [31, "morning"], [31, "cast"], [32, "morning"], [32, "cast"], [33, "morning"], [120, "final"]])
	print("PASS capture_plans_p6: v19 exact30 natural frames, explicit society/P/G resets, prior and added casts, paired roles/stems, deterministic FC selection")
	quit(0)
