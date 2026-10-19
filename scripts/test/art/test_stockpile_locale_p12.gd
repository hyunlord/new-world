extends SceneTree
var failures: int = 0

func _initialize() -> void:
	_run.call_deferred()

func check(ok: bool, message: String) -> void:
	if not ok:
		failures += 1
		push_error(message)

func _run() -> void:
	var locale: Node = root.get_node("Locale")
	locale.call("set_language", "ko")
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	var world: Node = main.get_node("WorldSim")
	world.call("set_sim_speed", 0.0)
	world.process_mode = Node.PROCESS_MODE_DISABLED
	root.add_child(main)
	main.set_process(false)
	var reads: SimReads = main.get_node("SimReads") as SimReads
	reads.reset_world_stress(15100, 200)
	reads.advance_ticks(1440)
	var buildings: Node = main.get_node("BuildingRenderer")
	buildings.set_process(false)
	buildings.call("refresh_snapshot")
	var labels: Dictionary = buildings.get("_labels")
	check(not labels.is_empty(), "stress world supplies actual stockpile labels")
	var tick: int = int(SimReads.read(world, "get_time_snapshot")["tick"])
	for language: String in ["en", "ko", "en"]:
		locale.call("set_language", language)
		# No process frame, snapshot refresh or simulated tick is allowed here.
		for label: Label in labels.values():
			check(label.text.begins_with(str(locale.call("ltr", "HUD_STOCKPILE_FOOD"))), "frozen food caption follows locale immediately")
			var wood: Label = label.get_node("Wood") as Label
			check(wood.text.begins_with(str(locale.call("ltr", "HUD_STOCKPILE_WOOD"))), "frozen wood caption follows locale immediately")
		check(int(SimReads.read(world, "get_time_snapshot")["tick"]) == tick, "locale refresh does not advance simulation")
	main.free()
	print("STOCKPILE_LOCALE_P12 failures=", failures)
	quit(1 if failures else 0)
