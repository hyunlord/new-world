extends RefCounted

# Capture-only truth derived directly from FFI snapshots, independent of cue memory.
var _rows: Dictionary = {}


func ingest(snapshot: Dictionary, tick: int) -> void:
	var ids: PackedInt64Array = snapshot.get("agent_ids", PackedInt64Array())
	var colds: PackedByteArray = snapshot.get("colds", PackedByteArray())
	for index: int in mini(ids.size(), colds.size()):
		var id: int = ids[index]
		if colds[index] == 1:
			_rows[id] = {"id": id, "last_cold_tick": tick, "first_clear_tick": -1}
		elif _rows.has(id) and int(_rows[id]["first_clear_tick"]) < 0:
			_rows[id]["first_clear_tick"] = tick


func rows() -> Array[Dictionary]:
	var result: Array[Dictionary] = []
	var ids: Array = _rows.keys()
	ids.sort()
	for id: int in ids:
		result.append(_rows[id].duplicate())
	return result
