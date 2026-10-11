extends SceneTree
const PRESETS: Dictionary = {"gift_food": 60, "spoil_food": 60, "gift_wood": 40, "rot_wood": 40, "rain": 13, "cold_snap": 1, "windfall": 20}

func _initialize() -> void:
	call_deferred("_run")

func _run() -> void:
	var world := WorldSimNode.new()
	root.add_child(world)
	world.set_process(false)
	var settlements: Dictionary = world.get_settlement_snapshot()
	for attempt: int in 120:
		if not settlements.get("settlement_ids", PackedInt32Array()).is_empty():
			break
		world.advance_ticks(60)
		settlements = world.get_settlement_snapshot()
	if settlements.get("settlement_ids", PackedInt32Array()).is_empty():
		push_error("No real settlement after 7200 ticks")
		quit(1)
		return
	for kind: String in PRESETS:
		var reply: Dictionary = world.apply_intervention(kind, -1, PRESETS[kind])
		if not bool(reply.get("ok", false)) or int(reply.get("applied", -1)) < 0:
			push_error("Real intervention failed: " + kind + " " + str(reply))
			quit(1)
			return
		print("REAL_INTERVENTION ", kind, " ", reply)
	var state: Dictionary = world.call("get_intervention_state")
	if state.get("log_kinds", PackedStringArray()).size() != 7 or state.get("rain_settlement_ids", PackedInt32Array()).is_empty():
		push_error("Real getter lost applied records or active rain")
		quit(1)
		return
	world.free()
	print("INTERVENTIONS_REAL_P6 PASS: seven real commands and read-only state")
	quit(0)
