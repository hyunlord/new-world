extends RefCounted

const ART: Array[Array] = [
	[1, "08:30", "F2"],
	[2, "12:00", "F2"], [2, "12:00", "F2A"], [2, "12:00", "F2Z"],
	[2, "12:00", "F3"], [2, "12:00", "F4"], [2, "12:00", "F1"],
	[2, "19:30", "F2"], [2, "23:00", "F2N"], [6, "12:00", "F2"],
]

# Society dates are populated from Part 0 measurements, never from planner estimates.
const ART_SOCIETY: Array[Array] = []

const ART_LONG: Array[Array] = [
	[10, "12:00", "F2"], [10, "15:00", "F2"], [10, "15:00", "F5"],
	[15, "15:00", "F2"], [15, "15:30", "FC"], [16, "15:00", "FC"],
	[18, "12:00", "F2"],
	[23, "02:00", "F2N"],
	[24, "12:00", "F2"], [24, "12:00", "F1"], [24, "23:00", "F2N"],
	[25, "02:00", "F2N"], [27, "02:00", "F2N"], [27, "07:15", "F2"],
	[28, "02:00", "F2N"], [28, "02:00", "F2Z"], [29, "02:00", "F2N"],
	[29, "06:30", "F2"], [29, "07:15", "F2"],
	[32, "12:00", "F2"], [40, "12:00", "F2"],
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
# Extra fields: kind, amount, cumulative real seconds since cast, suffix.
const ART_GOD: Array[Array] = [
	[10, "12:00", "F2", "spoil_food", 60, 0.5, "spoil"],
	[16, "17:00", "CAST", "cold_snap", 1, 0.0, ""],
	[17, "02:00", "F2N", "", 0, 1.15, "cold_snap"],
	[23, "12:00", "F2", "gift_wood", 40, 1.0, "gift"],
	[23, "12:05", "F2", "windfall", 20, 0.2, "windfall_0200ms"],
	[23, "12:05", "F2", "", 0, 1.0, "windfall_1000ms"],
	[24, "17:00", "CAST", "rain", 13, 0.0, ""],
	[24, "18:30", "F2", "", 0, 1.15, "rain"],
	[25, "02:00", "F2N", "", 0, 1.15, "rain_night"],
	[25, "02:00", "F2", "", 0, 1.15, "panel_log"],
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
