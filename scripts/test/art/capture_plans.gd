extends RefCounted

const ART: Array[Array] = [
	[1, "08:30", "F2"],
	[2, "12:00", "F2"], [2, "12:00", "F2A"], [2, "12:00", "F2Z"],
	[2, "12:00", "F3"], [2, "12:00", "F4"], [2, "12:00", "F1"],
	[2, "19:30", "F2"], [2, "23:00", "F2N"], [6, "12:00", "F2"],
]

# Society dates are populated from Part 0 measurements, never from planner estimates.
const ART_SOCIETY: Array[Array] = [
	["RESET", 20261013],
	[32, "22:00", "F2W", "orig"],
	[33, "06:30", "FD", "leader"], [33, "06:30", "FT", "site"],
	[33, "08:00", "FD", "leader"], [33, "08:59", "FS", "daughter"],
	[33, "09:00", "F2W", "orig"], [33, "09:49", "FV", "all"],
	[33, "12:00", "FV", "all"],
	[33, "12:00", "FV", "all", "orig"], [33, "12:00", "FV", "all", "daughter"],
	[51, "02:00", "F2N", "daughter"], [51, "12:00", "FV", "all"],
	[84, "02:00", "F2N", "daughter"], [84, "12:00", "FV", "all"],
	[89, "06:30", "FD", "leader"], [89, "12:34", "FV", "all"],
	[120, "12:00", "FV", "all"],
]

# The v19 first walk lasts through the required +2h frame; no rejected walk is needed.
const ART_SOCIETY_REJECTED_WALK: Array[Array] = []

const ART_LONG: Array[Array] = [
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


static func moving_agent(world: Node) -> int:
	var snap: Dictionary = world.call("get_agent_snapshot")
	var ids: PackedInt64Array = snap.get("agent_ids", PackedInt64Array())
	var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
	var txs: PackedInt32Array = snap.get("target_xs", PackedInt32Array())
	var tys: PackedInt32Array = snap.get("target_ys", PackedInt32Array())
	var states: PackedByteArray = snap.get("states", PackedByteArray())
	var best: int = -1
	for i: int in ids.size():
		if i >= txs.size() or i >= states.size():
			continue
		if states[i] == 1 and (xs[i] != txs[i] or ys[i] != tys[i]) and (best < 0 or ids[i] < best):
			best = ids[i]
	return best


static func agent_position(main: Node, id: int) -> Vector2:
	var objects: Node = main.get_node_or_null("WorldObjects")
	if objects != null:
		for child: Node in objects.get_children():
			if child is Node2D and str(child.get_meta("art_kind", "")) == "agent" and int(child.get_meta("art_id", -1)) == id:
				return (child as Node2D).global_position
	return Vector2.INF


static func write_json(path: String, data: Variant) -> bool:
	var file: FileAccess = FileAccess.open(path, FileAccess.WRITE)
	if file == null:
		return false
	file.store_string(JSON.stringify(data, "	") + "\n")
	file.close()
	return true


# Cast-only entries and images share one chronological, forward-only world.
# Extra fields: kind, amount, cumulative real seconds since cast, suffix, target.
# Optional index8 metadata: paired_world, case_id, stem_override, role.
# At equal06:00 ticks, morning precedes the next cast to preserve the pre-dose reading.
const ART_GOD: Array[Array] = [
	[10, "12:00", "F2", "spoil_food", 60, 0.5, "spoil"],
	[10, "12:05", "F2", "rot_wood", 40, 0.5, "rot_wood"],
	[10, "12:10", "F2", "gift_food", 60, 0.5, "gift_food"],
	[16, "17:00", "CAST", "cold_snap", 1, 0.0, ""],
	[17, "02:00", "F2N", "", 0, 1.15, "cold_snap"],
	[23, "12:00", "F2", "gift_wood", 40, 1.0, "gift"],
	[23, "12:05", "F2", "windfall", 20, 0.2, "windfall_0200ms"],
	[23, "12:05", "F2", "", 0, 1.0, "windfall_1000ms"],
	[24, "17:00", "CAST", "rain", 13, 0.0, ""],
	[24, "18:30", "F2", "", 0, 1.15, "rain"],
	[25, "02:00", "F2N", "", 0, 1.15, "rain_night"],
	[25, "02:00", "F2", "", 0, 1.15, "panel_log"],
	["RESET", 20261013],
	[50, "17:00", "F2", "rain", 13, 0.5, "soc_rain", "daughter"],
	[51, "02:00", "F2N", "", 0, 1.15, "soc_rain_night", "daughter"],
	[51, "02:00", "F2N", "", 0, 1.15, "soc_original_night", "orig"],
	[51, "02:00", "FV", "", 0, 1.15, "soc_both_night", "all"],
	["RESET", 20261013],
	[30, "06:00", "F2", "spoil_food", 60, 0.5, "p_spoil_cast", "orig", {"paired_world": "P", "case_id": "a33", "stem_override": "p_spoil_d30_0600_F2_cast_orig", "role": "cast"}],
	[31, "06:00", "F2", "", 0, 1.15, "p_spoil_morning", "orig", {"paired_world": "P", "case_id": "a33", "stem_override": "p_spoil_d31_0600_F2_morning_orig", "role": "morning"}],
	[31, "06:00", "F2", "spoil_food", 60, 0.5, "p_spoil_cast", "orig", {"paired_world": "P", "case_id": "a33", "stem_override": "p_spoil_d31_0600_F2_cast_orig", "role": "cast"}],
	[32, "06:00", "F2", "", 0, 1.15, "p_spoil_morning", "orig", {"paired_world": "P", "case_id": "a33", "stem_override": "p_spoil_d32_0600_F2_morning_orig", "role": "morning"}],
	[32, "06:00", "F2", "spoil_food", 60, 0.5, "p_spoil_cast", "orig", {"paired_world": "P", "case_id": "a33", "stem_override": "p_spoil_d32_0600_F2_cast_orig", "role": "cast"}],
	[33, "06:00", "F2", "", 0, 1.15, "p_spoil_morning", "orig", {"paired_world": "P", "case_id": "a33", "stem_override": "p_spoil_d33_0600_F2_morning_orig", "role": "morning"}],
	[120, "12:00", "FV", "", 0, 1.15, "p_spoil_final", "all", {"paired_world": "P", "case_id": "a33", "stem_override": "p_spoil_d120_1200_FV_final_all", "role": "final"}],
	["RESET", 20261013],
	[30, "06:00", "F2", "gift_food", 60, 0.5, "p_gift_cast", "orig", {"paired_world": "G", "case_id": "a33", "stem_override": "p_gift_d30_0600_F2_cast_orig", "role": "cast"}],
	[31, "06:00", "F2", "", 0, 1.15, "p_gift_morning", "orig", {"paired_world": "G", "case_id": "a33", "stem_override": "p_gift_d31_0600_F2_morning_orig", "role": "morning"}],
	[31, "06:00", "F2", "gift_food", 60, 0.5, "p_gift_cast", "orig", {"paired_world": "G", "case_id": "a33", "stem_override": "p_gift_d31_0600_F2_cast_orig", "role": "cast"}],
	[32, "06:00", "F2", "", 0, 1.15, "p_gift_morning", "orig", {"paired_world": "G", "case_id": "a33", "stem_override": "p_gift_d32_0600_F2_morning_orig", "role": "morning"}],
	[32, "06:00", "F2", "gift_food", 60, 0.5, "p_gift_cast", "orig", {"paired_world": "G", "case_id": "a33", "stem_override": "p_gift_d32_0600_F2_cast_orig", "role": "cast"}],
	[33, "06:00", "F2", "", 0, 1.15, "p_gift_morning", "orig", {"paired_world": "G", "case_id": "a33", "stem_override": "p_gift_d33_0600_F2_morning_orig", "role": "morning"}],
	[120, "12:00", "FV", "", 0, 1.15, "p_gift_final", "all", {"paired_world": "G", "case_id": "a33", "stem_override": "p_gift_d120_1200_FV_final_all", "role": "final"}],
]


static func wood_carrier_tile(snapshot: Dictionary) -> Vector2:
	var ids: PackedInt64Array = snapshot.get("agent_ids", PackedInt64Array())
	var woods: PackedInt32Array = snapshot.get("carried_woods", PackedInt32Array())
	var states: PackedByteArray = snapshot.get("states", PackedByteArray())
	var xs: PackedInt32Array = snapshot.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snapshot.get("ys", PackedInt32Array())
	var best_id: int = -1
	var tile: Vector2 = Vector2(-1, -1)
	for index: int in ids.size():
		if index >= woods.size() or index >= states.size() or index >= xs.size() or index >= ys.size():
			continue
		if woods[index] > 0.0 and states[index] == 1 and (best_id < 0 or ids[index] < best_id):
			best_id = ids[index]
			tile = Vector2(xs[index], ys[index])
	return tile
