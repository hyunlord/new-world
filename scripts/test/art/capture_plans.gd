extends RefCounted

const ART: Array[Array] = [
	[1, "08:30", "F2"],
	[2, "12:00", "F2"], [2, "12:00", "F2A"], [2, "12:00", "F2Z"],
	[2, "12:00", "F3"], [2, "12:00", "F4"], [2, "12:00", "F1"],
	[2, "19:30", "F2"], [2, "23:00", "F2N"], [6, "12:00", "F2"],
]

const ART_LONG: Array[Array] = [
	[10, "12:00", "F2"], [10, "15:00", "F2"], [10, "15:00", "F5"],
	[18, "12:00", "F2"],
	[23, "02:00", "F2N"],
	[24, "12:00", "F2"], [24, "12:00", "F1"], [24, "23:00", "F2N"],
	[25, "02:00", "F2N"], [27, "02:00", "F2N"], [29, "02:00", "F2N"],
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
	file.store_string(JSON.stringify(data, "\t") + "\n")
	file.close()
	return true
