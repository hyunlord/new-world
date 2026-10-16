extends SceneTree

const Trace = preload("res://scripts/test/art/capture_society_trace.gd")
const Plans = preload("res://scripts/test/art/capture_plans.gd")


func _init() -> void:
	var trace: RefCounted = Trace.new()
	var snapshot: Dictionary = {"current_tick": 10, "xs": PackedInt32Array([2, 2]), "ys": PackedInt32Array([3, 3]), "ticks": PackedInt64Array([9, 9]), "reasons": PackedInt32Array([4, 4])}
	trace.ingest(snapshot)
	trace.ingest(snapshot)
	assert(trace.rows().size() == 2, "coincident deaths are distinct; repeated snapshots are not")
	assert(trace.rows()[0]["death_id"] == "9:2:3:4:0")
	assert(trace.rows()[1]["death_id"] == "9:2:3:4:1")
	snapshot["current_tick"] = 18
	trace.ingest(snapshot)
	assert(trace.max_interval_ticks == 8)
	var copied: Array = trace.rows()
	copied.clear()
	assert(trace.rows().size() == 2)
	trace.reset()
	assert(trace.rows().is_empty() and trace.max_interval_ticks == 0)
	trace.ingest(snapshot)
	snapshot["current_tick"] = 1
	snapshot["xs"] = PackedInt32Array()
	snapshot["ys"] = PackedInt32Array()
	snapshot["ticks"] = PackedInt64Array()
	snapshot["reasons"] = PackedInt32Array()
	trace.ingest(snapshot)
	assert(trace.rows().is_empty(), "clock reversal clears the independent census")
	for plan: Array in [Plans.ART, Plans.ART_LONG, Plans.ART_SOCIETY, Plans.ART_GOD]:
		var previous: int = -1
		for row: Array in plan:
			if str(row[0]) == "RESET":
				previous = -1
				continue
			var clock: PackedStringArray = str(row[1]).split(":")
			var tick: int = (int(row[0]) - 1) * 1440 + int(clock[0]) * 60 + int(clock[1]) - 480
			assert(tick >= previous, "plan must advance monotonically between resets")
			previous = tick
	print("PASS capture independent death census and forward-only measured plans")
	quit(0)
