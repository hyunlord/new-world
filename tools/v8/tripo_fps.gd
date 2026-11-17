# tripo_fps.gd — real 500-agent Tripo crowd acceptance, windowed under the machine gate lock.
# Purpose: same stress world and z1 framing for both bindings arms; count actual people/model rows,
#   discard 15 seconds, then measure 30 seconds of frame_post_draw at uncapped 1x simulation speed.
#   --model MODEL is required, with at least 100 matching people. No render-only replication.
# Bridge methods called: reset_world_stress; via SimReads get_agent_snapshot, get_world_info.
extends "res://tools/v8/world3d_fps.gd"


func _accepted(fps: float, population: Vector2i, model_count: int) -> bool:
	return fps >= 60.0 and population == Vector2i(500, 500) and model_count >= 100


func _run() -> void:
	if "--self-test" in OS.get_cmdline_user_args():
		assert(_accepted(60.0, Vector2i(500, 500), 100))
		assert(not _accepted(59.9, Vector2i(500, 500), 100))
		assert(not _accepted(60.0, Vector2i(499, 500), 100))
		assert(not _accepted(60.0, Vector2i(500, 501), 100))
		assert(not _accepted(60.0, Vector2i(500, 500), 99))
		print("TRIPO_FPS_SELF_TEST PASS")
		quit(0)
		return
	var model: String = _arg("--model", "")
	if model == "" or DisplayServer.get_name() == "headless":
		push_error("tripo_fps: --model and a windowed renderer required")
		quit(1)
		return
	DisplayServer.window_set_vsync_mode(DisplayServer.VSYNC_DISABLED)
	Engine.max_fps = 0
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(main)
	for _i: int in 60:
		await process_frame
	var world: Node = main.get_node("WorldSim")
	world.call("reset_world_stress", 0, 500)
	for _i: int in 30:
		await process_frame
	var people: Node = main.get_node("World3D/People")
	var camera: Camera2D = main.get_node("Camera2D")
	var info: Dictionary = SimReads.read(world, "get_world_info")
	camera.zoom = Vector2.ONE
	camera.position = View.tile_centre(Vector2i(int(info.get("width", 64)) >> 1, int(info.get("height", 64)) >> 1))
	camera.call("_clamp_position")
	var warmup_end: int = Time.get_ticks_msec() + 15000
	while Time.get_ticks_msec() < warmup_end:
		await process_frame
	var population_min: int = 500
	var population_max: int = 500
	var model_min: int = 500
	var started: int = Time.get_ticks_usec()
	var first_frame: int = _drawn_frames
	while Time.get_ticks_usec() - started < 30000000:
		await process_frame
		var population: int = (SimReads.read(world, "get_agent_snapshot").get("ids", PackedInt64Array()) as PackedInt64Array).size()
		population_min = mini(population_min, population)
		population_max = maxi(population_max, population)
		var model_count: int = 0
		var states: Dictionary = people.get("_state")
		for state: Dictionary in states.values():
			if str(state.get("model", "")) == model:
				model_count += 1
		model_min = mini(model_min, model_count)
	var elapsed: float = float(Time.get_ticks_usec() - started) / 1000000.0
	var fps: float = float(_drawn_frames - first_frame) / elapsed
	var models: Dictionary = people.get("_models")
	var vat_ok: bool = models.has(model) and bool(models[model]["vat"].get("ok"))
	var passed: bool = _accepted(fps, Vector2i(population_min, population_max), model_min) and vat_ok and int(people.get("replicate")) == 1
	var result: Dictionary = {"passed": passed, "fps": fps, "frames": _drawn_frames - first_frame,
		"seconds": elapsed, "zoom": 1, "population_min": population_min, "population_max": population_max,
		"model": model, "model_min": model_min, "vat_ok": vat_ok, "replicate": people.get("replicate"),
		"bindings": OS.get_environment("WORLDSIM_3D_BINDINGS"), "size": [root.size.x, root.size.y]}
	print("TRIPO_FPS " + JSON.stringify(result))
	quit(0 if passed else 1)
