extends SceneTree
## Read-only, headless v19 calendar observations for Art Pass 8.
const OUTPUT_ROOT := "res://docs/art/production8/measurements"

func _initialize() -> void:
	call_deferred("run")

func run() -> void:
	var seed: int = 20261010
	for arg: String in OS.get_cmdline_user_args():
		if arg.begins_with("--seed="):
			seed = int(arg.trim_prefix("--seed="))
	var plan_path: String = OUTPUT_ROOT.path_join("probe_calendar_p8.json")
	var plan: Dictionary = JSON.parse_string(FileAccess.get_file_as_string(plan_path))
	var founding_ticks: Array[int] = []
	for history: Dictionary in plan["fission_histories"]:
		if int(history["seed"]) != seed:
			continue
		for fission: Dictionary in history["fissions"]:
			if fission["founded_tick"] != null:
				founding_ticks.append(int(fission["founded_tick"]) + 1)
	founding_ticks.sort()
	var founding_checks: Array[Dictionary] = []
	var first_three_alive_tick: int = -1
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
	var cold_last_seen: Dictionary = {}
	for day: int in range(1, 122):
		for minute: int in [120, 435, 900, 1155]:
			var target: int = (day - 1) * ticks_per_day + minute * ticks_per_day / 1440 - offset
			if target < tick:
				continue
			while tick < target:
				var step: int = mini(60, target - tick)
				for candidate: int in founding_ticks:
					if candidate > tick:
						step = mini(step, candidate - tick)
				var next_tick: int = int(world.call("advance_ticks", step))
				if next_tick <= tick:
					push_error("advance_ticks did not progress")
					world.free()
					quit(1)
					return
				tick = next_tick
				if founding_ticks.has(tick):
					var founded: Dictionary = world.call("get_settlement_snapshot")
					var alive: int = 0
					for members: int in founded["member_counts"]:
						if members > 0:
							alive += 1
					founding_checks.append({"tick": tick, "settlements_alive": alive, "snapshot": founded})
					if alive >= 3 and first_three_alive_tick < 0:
						first_three_alive_tick = tick
				var sampled: Dictionary = world.call("get_agent_snapshot")
				for index: int in sampled["agent_ids"].size():
					if int(sampled["colds"][index]) == 1:
						cold_last_seen[int(sampled["agent_ids"][index])] = tick
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
			var preceding_12h_cold_ids: Array[int] = []
			for id: int in cold_last_seen:
				if int(cold_last_seen[id]) >= tick - ticks_per_day / 2:
					preceding_12h_cold_ids.append(id)
			preceding_12h_cold_ids.sort()
			var row: Dictionary = {"tick": tick, "display_day": day, "minute": minute,
				"clock": world.call("get_time_snapshot"), "cold_ids": cold_ids,
				"preceding_12h_cold_ids": preceding_12h_cold_ids, "cold_sample_max_ticks": 60,
				"cold_ids_scope": "observed only; warm absence requires cumulative cold_member_hours equality",
				"wood_gatherer_ids": gatherers, "wood_carrier_ids": carriers,
				"settlements": settlements, "tallies": world.call("get_tallies")}
			if world.has_method("get_society_snapshot"):
				row["society"] = world.call("get_society_snapshot")
			rows.append(row)
		if day % 7 == 0:
			print("P8 MEASURE seed=%d day=%d tick=%d" % [seed, day, tick])
			await process_frame
	DirAccess.make_dir_recursive_absolute(ProjectSettings.globalize_path(OUTPUT_ROOT))
	var output: FileAccess = FileAccess.open(OUTPUT_ROOT.path_join("headless_%d.json" % seed), FileAccess.WRITE)
	if output == null:
		world.free()
		quit(1)
		return
	output.store_string(JSON.stringify({"scene": "game_v19", "seed": seed, "world_info": info,
		"source": "WorldSimNode.reset_world + advance_ticks; no interventions", "rows": rows,
		"founding_checks": founding_checks, "first_three_alive_tick": first_three_alive_tick}, "\t") + "\n")
	output.close()
	world.free()
	print("P8 MEASURE PASS seed=%d rows=%d" % [seed, rows.size()])
	quit(0)
