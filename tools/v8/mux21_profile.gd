# Quarter-zoom phase attribution on the real scene. Diagnostic only, not acceptance FPS.
# Bridge: reset_world_stress; SimReads get_world_info/get_agent_snapshot. No manual ticks.
extends SceneTree

const View = preload("res://scripts/world/colony/view.gd")

func _init() -> void:
	call_deferred("_run")

func _run() -> void:
	if OS.get_environment("WORLDSIM_MUX2_PROFILE") != "1" or OS.get_environment("WORLDSIM_MUX2_RENDER_PROFILE") != "1":
		quit(2)
		return
	DisplayServer.window_set_vsync_mode(DisplayServer.VSYNC_DISABLED)
	Engine.max_fps = 0
	root.always_on_top = true
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(main)
	for _i: int in 5:
		await process_frame
	var world: Node = main.get_node("WorldSim")
	world.call("reset_world_stress", 0, 200)
	for _i: int in 30:
		await process_frame
	var camera: Camera2D = main.get_node("Camera2D")
	var info: Dictionary = SimReads.read(world, "get_world_info")
	camera.zoom = Vector2(0.25, 0.25)
	camera.position = View.tile_centre(Vector2i(int(info["width"]) >> 1, int(info["height"]) >> 1))
	camera.call("_clamp_position")
	var start: int = Time.get_ticks_usec()
	while Time.get_ticks_usec() - start < 30000000:
		await RenderingServer.frame_post_draw
	var agents: Node = main.get_node("ColonyView/AgentLayer")
	var minimap: Node = main.get_node("UI/MuxHud/MapDock/Minimap")
	var previous_map: int = int(minimap.get("_total_usec"))
	var nodes: Dictionary = {"ground": main.get_node("ColonyView/GroundLayer"), "buildings": main.get_node("ColonyView/BuildingLayer"), "alerts": main.get_node("UI/MuxHud/Alerts"), "hud": main.get_node("UI/MuxHud")}
	var sums: Dictionary = {}
	var samples: int = 0
	start = Time.get_ticks_usec()
	while Time.get_ticks_usec() - start < 60000000:
		await RenderingServer.frame_post_draw
		var row: Dictionary = {"agents_total_usec": agents.get("last_process_usec"), "markers_total_usec": agents.get("last_draw_usec")}
		var profile: Dictionary = agents.get("mux2_render_profile")
		for key: String in profile:
			if key.ends_with("_usec"):
				row["agent/" + key] = profile[key]
		for key: String in nodes:
			var timing: Dictionary = nodes[key].call("mux2_frame_timings")
			if str(timing.get("status")) != "MEASURED" or int(timing.get("frame", -1)) != Engine.get_process_frames():
				push_error("Missing current timing for " + key)
				quit(2)
				return
			for metric: String in timing["usec"]:
				row[key + "/" + metric + "_usec"] = timing["usec"][metric]
		var map_total: int = int(minimap.get("_total_usec"))
		row["minimap_usec"] = map_total - previous_map
		previous_map = map_total
		row["process_usec"] = Performance.get_monitor(Performance.TIME_PROCESS) * 1000000.0
		row["draw_calls"] = Performance.get_monitor(Performance.RENDER_TOTAL_DRAW_CALLS_IN_FRAME)
		row["objects_drawn"] = Performance.get_monitor(Performance.RENDER_TOTAL_OBJECTS_IN_FRAME)
		for key: String in row:
			sums[key] = float(sums.get(key, 0.0)) + float(row[key])
		samples += 1
	var means: Dictionary = {}
	for key: String in sums:
		means[key] = float(sums[key]) / maxi(samples, 1)
	print("MUX21_PROFILE " + JSON.stringify({"samples": samples, "means": means, "seconds": float(Time.get_ticks_usec() - start) / 1000000.0, "zoom": camera.zoom.x, "people": (SimReads.read(world, "get_agent_snapshot")["ids"] as PackedInt64Array).size(), "scope": "Nested spans overlap. Diagnostic counters enabled; not FPS acceptance."}))
	quit(0)
