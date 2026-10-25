# Generate real seed0 day1/day20 noon/night saves for M-UX2 visibility, without construction orders.
# Bridge methods: reset_world, cmd_set_steward, advance_ticks, save_world, get_time_snapshot, get_world_info.
extends SceneTree

const Store = preload("res://scripts/ui/mux/settings_store.gd")


func _init() -> void:
	call_deferred("_run")


func _run() -> void:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	if args.size() != 1 or not args[0].is_absolute_path():
		_fail("expected one absolute output directory")
		return
	var out: String = args[0].simplify_path().trim_suffix("/")
	var project_path: String = ProjectSettings.globalize_path("res://").simplify_path().trim_suffix("/")
	if out == project_path or out.begins_with(project_path + "/"):
		_fail("raw evidence must be outside the repository")
		return
	if DirAccess.dir_exists_absolute(out):
		_fail("refusing existing output directory")
		return
	if DirAccess.make_dir_recursive_absolute(out) != OK:
		_fail("cannot create output directory")
		return
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	main.get_node("UI/MuxHud").set("store", Store.new(out.path_join("settings.cfg")))
	root.add_child(main)
	if not bool(main.get("boot_ok")) or not main.has_node("WorldSim"):
		_fail("main scene bootstrap failed")
		return
	var world: Node = main.get_node("WorldSim")
	world.process_mode = Node.PROCESS_MODE_DISABLED
	if not bool(SimReads.mutate(world, "reset_world", [0])):
		_fail("reset failed")
		return
	var command: Dictionary = SimReads.mutate(world, "cmd_set_steward", [true, 0])
	if not bool(command.get("accepted", false)):
		_fail("steward ON rejected")
		return
	var info: Dictionary = world.get_world_info()
	var clock: Dictionary = world.get_time_snapshot()
	var per_day: int = int(clock.get("ticks_per_day", 0))
	if per_day <= 0 or int(info.get("seed", -1)) != 0 or not info.has("clock_offset_ticks"):
		_fail("invalid world calendar/seed")
		return
	# collect_world_info/world_info_to_dict expose this authoritative offset (snapshot.rs).
	var offset: int = int(info["clock_offset_ticks"])
	var scenes: Array[Dictionary] = []
	for day: int in [1, 20]:
		for hour: int in [12, 22]:
			var target: int = (day - 1) * per_day + floori(float(hour * per_day) / 24.0) - offset
			while int(world.get_time_snapshot().get("tick", 0)) < target:
				var current: int = int(world.get_time_snapshot().get("tick", 0))
				var count: int = mini(240, target - current)
				var reached: int = int(world.advance_ticks(count))
				if reached != current + count:
					_fail("advance did not reach requested tick")
					return
				await process_frame
			clock = world.get_time_snapshot()
			var id: String = "day%d_%s" % [day, "noon" if hour == 12 else "night"]
			if int(clock.get("tick", -1)) != target or int(clock.get("day_index", -1)) + 1 != day or absf(float(clock.get("time_of_day", -100.0)) - float(hour)) > 0.02:
				_fail("actual clock does not match requested scene " + id + ": " + JSON.stringify(clock))
				return
			var path: String = out.path_join(id + ".wsave")
			if not bool(world.save_world(path)):
				_fail("save failed: " + id)
				return
			var row: Dictionary = {"id": id, "save": path, "sha256": FileAccess.get_sha256(path), "day": day, "hour": hour, "tick": int(clock.get("tick", -1)), "clock": clock}
			scenes.append(row)
			print("MUX2_VISIBILITY_SAVE " + JSON.stringify(row))
	var result: Dictionary = {"schema": 1, "seed": 0, "steward": "on", "source": "actual debug bridge; no player construction/draft orders; batched advance only for preparation", "bridge_sha256": FileAccess.get_sha256("res://rust/target/debug/libsim_bridge.dylib"), "script_sha256": FileAccess.get_sha256("res://tools/v8/mux2_visibility_saves.gd"), "scenes": scenes}
	var file: FileAccess = FileAccess.open(out.path_join("manifest.json"), FileAccess.WRITE)
	if file == null:
		_fail("manifest write failed")
		return
	file.store_string(JSON.stringify(result, "  "))
	file.close()
	print("MUX2_VISIBILITY_SAVES_DONE scenes=4")
	quit(0)


func _fail(reason: String) -> void:
	push_error("MUX2_VISIBILITY_SAVES_FAIL " + reason)
	quit(1)
