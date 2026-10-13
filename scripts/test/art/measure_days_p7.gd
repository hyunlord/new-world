extends SceneTree
## Read-only, headless v17 calendar observations for Art Pass 7.
const OUTPUT_ROOT := "res://docs/art/production7/measurements"

func _initialize() -> void:
	call_deferred("run")

func run() -> void:
	var seed: int = 20261010
	for arg: String in OS.get_cmdline_user_args():
		if arg.begins_with("--seed="):
			seed = int(arg.trim_prefix("--seed="))
	var world: Node = ClassDB.instantiate("WorldSimNode") as Node
	world.process_mode = Node.PROCESS_MODE_DISABLED
	root.add_child(world)
	world.call("set_sim_speed", 0.0)
	world.call("reset_world", seed)
	await process_frame
	await process_frame
	var info: Dictionary = world.call("get_world_info")
	var rows: Array[Dictionary] = []
	var ticks_per_day: int = int(info["ticks_per_day"])
	var offset: int = int(info["clock_offset_ticks"])
	var tick: int = 0
	for day: int in range(1, 122):
		for minute: int in [120, 435, 900]:
			var target: int = (day - 1) * ticks_per_day + minute * ticks_per_day / 1440 - offset
			if target < tick:
				continue
			while tick < target:
				var next_tick: int = int(world.call("advance_ticks", mini(60, target - tick)))
				if next_tick <= tick:
					push_error("advance_ticks did not progress")
					world.free()
					quit(1)
					return
				tick = next_tick
			var agents: Dictionary = world.call("get_agent_snapshot")
			var settlements: Dictionary = world.call("get_settlement_snapshot")
			var cold_ids: Array[int] = []
			var gatherers: Array[int] = []
			var carriers: Array[int] = []
			for i: int in agents["agent_ids"].size():
				var id: int = int(agents["agent_ids"][i])
				if int(agents["colds"][i]) == 1:
					cold_ids.append(id)
				if int(agents["states"][i]) == 3 and int(agents["target_kinds"][i]) == 7:
					gatherers.append(id)
				if int(agents["states"][i]) == 1 and int(agents["carried_woods"][i]) > 0:
					carriers.append(id)
			var row: Dictionary = {"tick": tick, "display_day": day, "minute": minute,
				"clock": world.call("get_time_snapshot"), "cold_ids": cold_ids,
				"wood_gatherer_ids": gatherers, "wood_carrier_ids": carriers,
				"settlements": settlements, "tallies": world.call("get_tallies")}
			if world.has_method("get_society_snapshot"):
				row["society"] = world.call("get_society_snapshot")
			rows.append(row)
		if day % 7 == 0:
			print("P7 MEASURE seed=%d day=%d tick=%d" % [seed, day, tick])
			await process_frame
	DirAccess.make_dir_recursive_absolute(ProjectSettings.globalize_path(OUTPUT_ROOT))
	var output: FileAccess = FileAccess.open(OUTPUT_ROOT.path_join("headless_%d.json" % seed), FileAccess.WRITE)
	if output == null:
		world.free()
		quit(1)
		return
	output.store_string(JSON.stringify({"scene": "game_v17", "seed": seed, "world_info": info,
		"source": "WorldSimNode.reset_world + advance_ticks; no interventions", "rows": rows}, "\t") + "\n")
	output.close()
	world.free()
	print("P7 MEASURE PASS seed=%d rows=%d" % [seed, rows.size()])
	quit(0)
