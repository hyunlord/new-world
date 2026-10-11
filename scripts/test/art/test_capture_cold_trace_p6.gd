extends SceneTree

const Trace = preload("res://scripts/test/art/capture_cold_trace.gd")


func _init() -> void:
	var trace: RefCounted = Trace.new()
	trace.ingest({"agent_ids": PackedInt64Array([2, 1]), "colds": PackedByteArray([1, 0])}, 16)
	trace.ingest({"agent_ids": PackedInt64Array([2, 1]), "colds": PackedByteArray([1, 0])}, 24)
	trace.ingest({"agent_ids": PackedInt64Array([2, 1]), "colds": PackedByteArray([0, 0])}, 32)
	trace.ingest({"agent_ids": PackedInt64Array([2, 1]), "colds": PackedByteArray([0, 0])}, 40)
	assert(trace.rows() == [{"id": 2, "last_cold_tick": 24, "first_clear_tick": 32}])
	trace.ingest({"agent_ids": PackedInt64Array([2]), "colds": PackedByteArray([1])}, 48)
	assert(trace.rows()[0]["first_clear_tick"] == -1)
	print("PASS capture cold truth trace: bounded clearing, never-cold exclusion, later cold reset")
	quit(0)
