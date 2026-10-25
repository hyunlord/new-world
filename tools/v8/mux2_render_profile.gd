# Opt-in windowed attribution of the real N1000 half-zoom renderer, without changing render decisions.
# Bridge methods: reset_world_stress; get_world_info/get_agent_snapshot through SimReads. No manual ticks.
# Run with WORLDSIM_MUX2_RENDER_PROFILE=1; --out NEW_ABSOLUTE_DIR [--warmup 5 --seconds 15].
extends SceneTree

const View = preload("res://scripts/world/colony/view.gd")
const Store = preload("res://scripts/ui/mux/settings_store.gd")
const Provenance = preload("res://tools/v8/mux2_visibility_provenance.gd")
const SELF_PATH: String = "res://tools/v8/mux2_render_profile.gd"


func _init() -> void:
	call_deferred("_run")


func _arg(key: String, fallback: String = "") -> String:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	var index: int = args.find(key)
	return args[index + 1] if index >= 0 and index + 1 < args.size() else fallback


func _provenance() -> Dictionary:
	var result: Dictionary = Provenance.capture("res://project.godot")
	(result["hashes"] as Dictionary)[SELF_PATH] = FileAccess.get_sha256(SELF_PATH)
	return result


func _run() -> void:
	var out: String = _arg("--out").simplify_path().trim_suffix("/")
	var project: String = ProjectSettings.globalize_path("res://").simplify_path().trim_suffix("/")
	if OS.get_environment("WORLDSIM_MUX2_RENDER_PROFILE") != "1" or DisplayServer.get_name() == "headless" or not out.is_absolute_path() or out == project or out.begins_with(project + "/") or DirAccess.dir_exists_absolute(out):
		push_error("MUX2_RENDER_PROFILE requires opt-in, GPU and new absolute external output")
		quit(1)
		return
	if DirAccess.make_dir_recursive_absolute(out) != OK:
		quit(1)
		return
	var before: Dictionary = _provenance()
	_write(out.path_join("provenance-start.json"), before)
	DisplayServer.window_set_vsync_mode(DisplayServer.VSYNC_DISABLED)
	Engine.max_fps = 0
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	main.get_node("UI/MuxHud").set("store", Store.new(out.path_join("settings.cfg")))
	root.add_child(main)
	if not bool(main.get("boot_ok")) or not main.has_node("WorldSim"):
		push_error("MUX2_RENDER_PROFILE bootstrap failed")
		quit(1)
		return
	var world: Node = main.get_node("WorldSim")
	var camera: Camera2D = main.get_node("Camera2D") as Camera2D
	for _i: int in 5:
		await process_frame
	if not bool(SimReads.mutate(world, "reset_world_stress", [0, 1000])):
		push_error("MUX2_RENDER_PROFILE stress reset rejected")
		quit(1)
		return
	for _i: int in 30:
		await process_frame
	var info: Dictionary = SimReads.read(world, "get_world_info")
	camera.zoom = Vector2(0.5, 0.5)
	camera.position = View.tile_centre(Vector2i(int(info.get("width", 64)) >> 1, int(info.get("height", 64)) >> 1))
	camera.call("_clamp_position")
	var agents: Node = main.get_node("ColonyView/AgentLayer")
	var warmup: float = maxf(0.0, float(_arg("--warmup", "5")))
	var seconds: float = maxf(1.0, float(_arg("--seconds", "15")))
	var warming: int = Time.get_ticks_usec()
	while Time.get_ticks_usec() - warming < int(warmup * 1000000.0):
		await RenderingServer.frame_post_draw
	var file: FileAccess = FileAccess.open(out.path_join("frames.jsonl"), FileAccess.WRITE)
	if file == null:
		push_error("MUX2_RENDER_PROFILE cannot open frames")
		quit(1)
		return
	var started: int = Time.get_ticks_usec()
	var previous: int = started
	var count: int = 0
	var missing: int = 0
	var sums: Dictionary = {}
	var maxima: Dictionary = {}
	var initial_frames: int = Engine.get_frames_drawn()
	while Time.get_ticks_usec() - started < int(seconds * 1000000.0):
		await RenderingServer.frame_post_draw
		var now: int = Time.get_ticks_usec()
		var profile: Dictionary = (agents.get("mux2_render_profile") as Dictionary).duplicate(true)
		if not profile.has("update_total_usec"):
			missing += 1
		for key: String in profile.keys():
			if key.ends_with("_usec"):
				sums[key] = int(sums.get(key, 0)) + int(profile[key])
				maxima[key] = maxi(int(maxima.get(key, 0)), int(profile[key]))
		var row: Dictionary = {"sample": count, "wall_usec": now - previous, "process_ms": Performance.get_monitor(Performance.TIME_PROCESS) * 1000.0, "draw_calls": Performance.get_monitor(Performance.RENDER_TOTAL_DRAW_CALLS_IN_FRAME), "agent": profile}
		file.store_line(JSON.stringify(row))
		file.flush()
		count += 1
		previous = now
	file.close()
	var elapsed: float = float(Time.get_ticks_usec() - started) / 1000000.0
	var frames: int = Engine.get_frames_drawn() - initial_frames
	var people: int = (SimReads.read(world, "get_agent_snapshot").get("ids", PackedInt64Array()) as PackedInt64Array).size()
	# Readback follows the measured interval; retain the last actual stress frame for visual review.
	var final_frame: Dictionary = _visual_frame(out, "stress-final", agents as Node2D, world, camera)
	var visual_state: Dictionary = await _hover_evidence(out, agents as Node2D, world, camera)
	visual_state["measured_final"] = final_frame
	var after: Dictionary = _provenance()
	var changed: Array[String] = Provenance.changes(before["hashes"], after["hashes"])
	var valid: bool = count > 0 and missing == 0 and changed.is_empty() and people == 1000 and not bool(agents.call("lod_active"))
	var report: Dictionary = {"valid": valid, "samples": count, "missing_profile": missing, "people": people, "zoom": camera.zoom.x, "lod": agents.call("lod_active"), "seconds": elapsed, "frames_drawn": frames, "diagnostic_fps": float(frames) / elapsed, "phase_sums_usec": sums, "phase_maxima_usec": maxima, "changed_inputs": changed, "provenance_start": before, "provenance_end": after, "scope": "Opt-in phase attribution; nested spans overlap, profiling counters/copies/JSON add overhead. This is not the final FPS acceptance run."}
	report["visual_state"] = visual_state
	report["visual_valid"] = bool(visual_state["valid"]) and int(final_frame["screenshot_error"]) == OK and str(final_frame["screenshot_sha256"]).length() == 64
	_write(out.path_join("provenance-end.json"), after)
	_write(out.path_join("profile.json"), report)
	print("MUX2_RENDER_PROFILE_DONE " + JSON.stringify({"valid": valid, "visual_valid": report["visual_valid"], "samples": count, "diagnostic_fps": report["diagnostic_fps"], "phase_sums_usec": sums, "out": out}))
	quit(0 if valid and bool(report["visual_valid"]) else 1)


func _write(path: String, value: Dictionary) -> void:
	var file: FileAccess = FileAccess.open(path, FileAccess.WRITE)
	if file == null:
		push_error("MUX2_RENDER_PROFILE cannot write " + path)
		return
	file.store_string(JSON.stringify(value, "  "))
	file.close()


func _point(value: Vector2) -> Array[float]:
	return [value.x, value.y]


func _visual_frame(out: String, name: String, agents: Node2D, world: Node, camera: Camera2D) -> Dictionary:
	var path: String = out.path_join(name + ".png")
	var error: Error = root.get_texture().get_image().save_png(path)
	var badges: Array[Dictionary] = []
	var canvas: Transform2D = agents.get_global_transform_with_canvas()
	for badge: Dictionary in agents.get("_coincident_badges"):
		var rectangle: Rect2 = canvas * (badge["rect"] as Rect2)
		var scope: Array = []
		for endpoint: Vector2 in badge.get("screen_scope", PackedVector2Array()):
			scope.append(_point(endpoint))
		badges.append({"text": str(badge["text"]), "member_count": int(badge.get("member_count", (badge["members"] as Array).size())), "members": (badge["members"] as Array).duplicate(), "source_groups": (badge["source_groups"] as Array).duplicate(true), "aggregate": bool(badge.get("aggregate", false)), "active": bool(badge.get("active", false)), "exhausted": bool(badge.get("exhausted", false)), "screen_rect": [_point(rectangle.position), _point(rectangle.size)], "screen_scope_endpoint_pairs": scope})
	var drawn: Dictionary = {}
	var drawer: Node2D = agents.get("_badge_leaders")
	for field: String in ["segments", "aggregate_segments", "active_segments"]:
		var endpoints: Array = []
		for endpoint: Vector2 in drawer.get(field):
			endpoints.append(_point(endpoint))
		drawn[field] = endpoints
	var clock: Dictionary = SimReads.read(world, "get_time_snapshot")
	var row: Dictionary = {"image": name + ".png", "capture_directory": out, "screenshot_error": error, "screenshot_sha256": FileAccess.get_sha256(path), "tick": int(clock.get("tick", -1)), "clock": clock, "badge_count": badges.size(), "badges": badges, "exhausted": int(agents.get("badge_layout_exhausted")), "selected_bits": int(agents.get("_selected_bits")), "pointer_viewport": _point(root.get_mouse_position()), "drawn_local_endpoint_pairs": drawn, "draw_scale_width": drawer.get("scale_width"), "camera_position": _point(camera.position), "camera_zoom": _point(camera.zoom), "camera_process_enabled": camera.is_processing(), "simulation_paused": bool(camera.call("is_paused")), "after_measured_interval": true}
	_write(out.path_join(name + ".json"), row)
	return row


func _move_pointer(point: Vector2) -> void:
	var previous: Vector2 = root.get_mouse_position()
	Input.warp_mouse(point)
	var motion: InputEventMouseMotion = InputEventMouseMotion.new()
	motion.position = point
	motion.global_position = point
	motion.relative = point - previous
	Input.parse_input_event(motion)


func _hover_attempt(out: String, agents: Node2D, world: Node, camera: Camera2D) -> Dictionary:
	var saved_pointer: Vector2 = root.get_mouse_position()
	var was_processing: bool = camera.is_processing()
	var was_paused: bool = bool(camera.call("is_paused"))
	# Only after elapsed/frames have been frozen: prevent edge-pan and hold the tick for this visual pair.
	camera.set_process(false)
	camera.call("set_paused", true)
	_move_pointer(Vector2(4, 4))
	var settling: Dictionary = await _settle_badges(agents, world)
	var normal_geometry: Dictionary = _badge_geometry(agents)
	var normal: Dictionary = _visual_frame(out, "stress-default", agents, world, camera)
	var largest: Dictionary = {}
	for badge: Dictionary in agents.get("_coincident_badges"):
		if bool(badge.get("aggregate", false)) and (largest.is_empty() or (badge["members"] as Array).size() > (largest["members"] as Array).size()):
			largest = badge.duplicate(true)
	var failures: Array[String] = []
	if not bool(settling["valid"]):
		failures.append("natural interpolation/geometry settling timed out")
	var hovered: Dictionary = {}
	var target: Vector2 = Vector2.ZERO
	if largest.is_empty():
		failures.append("no multi-source aggregate available")
	else:
		if bool(largest.get("active", false)):
			failures.append("target already active in default frame; hover-only attribution unavailable")
		target = agents.get_global_transform_with_canvas() * (largest["rect"] as Rect2).get_center()
		if not root.get_visible_rect().has_point(target):
			failures.append("largest aggregate center outside viewport")
		else:
			_move_pointer(target)
			await RenderingServer.frame_post_draw
			hovered = _visual_frame(out, "stress-hover", agents, world, camera)
			if _badge_geometry(agents) != normal_geometry:
				failures.append("complete badge geometry changed during hover pair")
			if _pending_interpolations(agents) != 0:
				failures.append("interpolation pending during hover frame")
			var matched: bool = false
			for badge: Dictionary in agents.get("_coincident_badges"):
				if badge["members"] == largest["members"] and badge["source_groups"] == largest["source_groups"]:
					matched = true
					if not bool(badge.get("active", false)):
						failures.append("largest aggregate did not become active through real mouse input")
					var rectangle: Rect2 = agents.get_global_transform_with_canvas() * (badge["rect"] as Rect2)
					if not rectangle.has_point(root.get_mouse_position()):
						failures.append("actual pointer is not inside matched badge")
			if not matched:
				failures.append("aggregate membership changed during hover pair")
			if int(normal["tick"]) != int(hovered["tick"]):
				failures.append("simulation tick changed during paused visual pair")
	for frame: Dictionary in [normal, hovered]:
		if not frame.is_empty() and (int(frame["screenshot_error"]) != OK or str(frame["screenshot_sha256"]).length() != 64):
			failures.append("screenshot write/hash failed: " + str(frame["image"]))
	_move_pointer(saved_pointer)
	await RenderingServer.frame_post_draw
	var restored_pointer: Vector2 = root.get_mouse_position()
	if restored_pointer.distance_to(saved_pointer) > 1.0:
		failures.append("pointer restore differs by more than one viewport pixel")
	camera.call("set_paused", was_paused)
	camera.set_process(was_processing)
	var result: Dictionary = {"valid": failures.is_empty(), "failures": failures, "capture_directory": out, "settling": settling, "default": normal, "hover": hovered, "target_screen": _point(target), "target_members": largest.get("members", []), "target_source_groups": largest.get("source_groups", []), "pointer_before": _point(saved_pointer), "pointer_after_restore": _point(restored_pointer), "camera_processing_before": was_processing, "simulation_paused_before": was_paused, "scope": "VISUAL_ONLY after completed measurement: camera processing disabled and simulation paused through production camera API, then restored. Real pointer warp and InputEventMouseMotion; no manual active flag, inspect click, simulation tick or image editing. This pair supplies visual evidence, not FPS acceptance."}
	_write(out.path_join("aggregate-hover.json"), result)
	return result


# Read-only evidence: no forced elapsed/clock/position or renderer updates.
func _pending_interpolations(agents: Node2D) -> int:
	var pending: int = 0
	var persons: Dictionary = agents.get("_persons")
	for person: Node2D in persons.values():
		if str(person.get("activity")) != "sleep" and float(person.get("elapsed")) < float(person.get("duration")):
			pending += 1
	return pending


func _badge_geometry(agents: Node2D) -> Dictionary:
	var result: Array[Dictionary] = []
	for badge: Dictionary in agents.get("_coincident_badges"):
		var geometry: Dictionary = badge.duplicate(true)
		# Only the hover accent may differ across an otherwise identical pair.
		geometry.erase("active")
		result.append(geometry)
	var drawer: Node2D = agents.get("_badge_leaders")
	var segments: Dictionary = {}
	for field: String in ["segments", "aggregate_segments", "active_segments"]:
		var points: PackedVector2Array = drawer.get(field)
		for i: int in range(0, points.size(), 2):
			var key: Rect2 = Rect2(points[i], points[i + 1] - points[i])
			segments[key] = int(segments.get(key, 0)) + 1
	return {"badges": result, "drawn_endpoint_multiset": segments, "style": (drawer.get("extent_style") as Dictionary).duplicate(true), "width": drawer.get("scale_width"), "ordinary_light": drawer.get("light")}


func _settle_badges(agents: Node2D, world: Node) -> Dictionary:
	var started: int = Time.get_ticks_usec()
	var previous: Dictionary = {}
	var consecutive: int = 0
	var rows: Array[Dictionary] = []
	while Time.get_ticks_usec() - started < 8000000:
		await RenderingServer.frame_post_draw
		var geometry: Dictionary = _badge_geometry(agents)
		var pending: int = _pending_interpolations(agents)
		consecutive = consecutive + 1 if geometry == previous and pending == 0 else 0
		rows.append({"frame": Engine.get_frames_drawn(), "elapsed_usec": Time.get_ticks_usec() - started, "tick": int(SimReads.read(world, "get_time_snapshot").get("tick", -1)), "pending_interpolations": pending, "geometry_equal_previous": geometry == previous, "consecutive_stable": consecutive})
		previous = geometry
		if consecutive >= 2:
			return {"valid": true, "rows": rows, "elapsed_usec": Time.get_ticks_usec() - started, "condition": "No nonsleeping interpolation pending; complete badge geometry equal over three consecutive post-draw samples."}
	return {"valid": false, "rows": rows, "elapsed_usec": Time.get_ticks_usec() - started, "condition": "Natural settling timed out after eight seconds; no renderer state was forced."}


func _hover_evidence(out: String, agents: Node2D, world: Node, camera: Camera2D) -> Dictionary:
	var attempts: Array[Dictionary] = []
	var result: Dictionary = {"valid": false, "failures": ["no capture attempt completed"]}
	for attempt: int in 3:
		var directory: String = out.path_join("hover-attempt-%02d" % (attempt + 1))
		if DirAccess.make_dir_recursive_absolute(directory) != OK:
			result = {"valid": false, "failures": ["cannot create raw attempt directory " + directory]}
			attempts.append(result)
			break
		result = await _hover_attempt(directory, agents, world, camera)
		attempts.append(result.duplicate(true))
		if bool(result["valid"]):
			break
	result["attempts"] = attempts
	result["attempt_limit"] = 3
	result["failed_attempts_retained"] = true
	_write(out.path_join("aggregate-hover.json"), result)
	return result
