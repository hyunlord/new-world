# Controlled actual in-game D4 rebakes, separate from the fixed-window FPS run.
# Bridge methods: reset_world_stress; get_world_info/get_agent_snapshot/get_time_snapshot through SimReads.
# Windowed release bridge mapped to debug; --out NEW_EXTERNAL_DIRECTORY. All residents, exactly three complete passes.
extends SceneTree

const View = preload("res://scripts/world/colony/view.gd")
const Store = preload("res://scripts/ui/mux/settings_store.gd")
const Provenance = preload("res://tools/v8/mux2_visibility_provenance.gd")
const Fps = preload("res://tools/v8/mart2_fps.gd")
var _out: String
var _ground: Node
var _checks: Array[Dictionary] = []
var _rows: Array[Dictionary] = []
var _images: Array[Dictionary] = []


func _init() -> void:
	call_deferred("_run")


func _arg(name: String) -> String:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	var at: int = args.find(name)
	return args[at + 1] if at >= 0 and at + 1 < args.size() else ""


func _check(label: String, valid: bool) -> void:
	_checks.append({"label": label, "pass": valid})
	print("MUX2_REBAKE_%s %s" % ["PASS" if valid else "FAIL", label])


func _write(name: String, value: Variant) -> bool:
	var file: FileAccess = FileAccess.open(_out.path_join(name), FileAccess.WRITE)
	if file == null:
		_check("write_" + name, false)
		return false
	file.store_string(JSON.stringify(value, "  ") + "\n")
	file.flush()
	var valid: bool = file.get_error() == OK
	file.close()
	if not valid:
		_check("write_" + name, false)
	return valid


func _provenance() -> Dictionary:
	var result: Dictionary = Provenance.capture("res://project.godot")
	var hashes: Dictionary = result["hashes"]
	for path: String in [str(get_script().resource_path), "res://tools/v8/mart2_fps.gd", "res://rust/target/release/libsim_bridge.dylib"]:
		hashes[path] = FileAccess.get_sha256(path)
	return result


func _image_record(image: Image) -> Dictionary:
	if image == null:
		return {"valid": false}
	var hash: HashingContext = HashingContext.new()
	hash.start(HashingContext.HASH_SHA256)
	hash.update(image.get_data())
	return {"valid": true, "width": image.get_width(), "height": image.get_height(), "format": image.get_format(), "sha256": hash.finish().hex_encode()}


func _texture_record(texture: Texture2D) -> Dictionary:
	return _image_record(texture.get_image()) if texture != null else {"valid": false}


func _fingerprint(chunk: Vector2i) -> Dictionary:
	var chunks: Dictionary = _ground.get("_chunks")
	if not chunks.has(chunk):
		return {"valid": false, "reason": "missing resident"}
	var sprite: Sprite2D = chunks[chunk]
	var material: ShaderMaterial = sprite.material as ShaderMaterial
	var params: Dictionary = {}
	for key: String in ["water_gain", "snow_alpha", "snow_feature_px", "snow_drift_strength", "snow_colour", "chunk_tiles", "map_tint_strength"]:
		params[key] = str(material.get_shader_parameter(key))
	var strips: Array[Dictionary] = []
	for strip: Sprite2D in (_ground.get("_strips") as Dictionary).get(chunk, []):
		strips.append({"texture": _texture_record(strip.texture), "position": str(strip.position), "scale": str(strip.scale), "offset": str(strip.offset), "modulate": str(strip.modulate)})
	var result: Dictionary = {"valid": true, "cpu_rgba": _image_record(_ground.call("chunk_image", chunk)),
		"ground_texture": _texture_record(sprite.texture), "position": str(sprite.position), "scale": str(sprite.scale),
		"tint_texture": _texture_record(material.get_shader_parameter("terrain_tint_tex") as Texture2D),
		"snow_texture": _texture_record(material.get_shader_parameter("snow_mask_tex") as Texture2D),
		"shader_parameters": params, "strip_textures": strips}
	for key: String in ["cpu_rgba", "ground_texture", "tint_texture", "snow_texture"]:
		result["valid"] = bool(result["valid"]) and bool(result[key]["valid"])
	for strip: Dictionary in strips:
		result["valid"] = bool(result["valid"]) and bool(strip["texture"]["valid"])
	return result


func _required_chunks() -> Array[Vector2i]:
	var map: RefCounted = _ground.call("ground_map")
	var baker: RefCounted = _ground.call("baker")
	var tiles: int = int(baker.get("chunk_tiles"))
	var span: int = tiles * View.TILE
	var viewport: Viewport = _ground.get_viewport()
	var view: Rect2 = viewport.get_canvas_transform().affine_inverse() * viewport.get_visible_rect()
	var first: Vector2i = Vector2i(floori(view.position.x / span) - 1, floori(view.position.y / span) - 1)
	var last: Vector2i = Vector2i(floori(view.end.x / span) + 1, floori(view.end.y / span) + 1)
	var maximum: Vector2i = Vector2i(ceili(float(map.get("width")) / tiles) - 1, ceili(float(map.get("height")) / tiles) - 1)
	var result: Array[Vector2i] = []
	for y: int in range(maxi(0, first.y), mini(maximum.y, last.y) + 1):
		for x: int in range(maxi(0, first.x), mini(maximum.x, last.x) + 1):
			result.append(Vector2i(x, y))
	return result


func _coordinates(chunks: Array[Vector2i]) -> Array:
	var result: Array = []
	for chunk: Vector2i in chunks:
		result.append([chunk.x, chunk.y])
	return result


func _capture(name: String) -> void:
	var image: Image = root.get_texture().get_image()
	var path: String = _out.path_join(name)
	var error: Error = image.save_png(path)
	_check("framebuffer_" + name, error == OK)
	_images.append({"path": path, "png_sha256": FileAccess.get_sha256(path), "rgba": _image_record(image)})


func _run() -> void:
	_out = _arg("--out").simplify_path().trim_suffix("/")
	var project: String = ProjectSettings.globalize_path("res://").simplify_path().trim_suffix("/")
	if DisplayServer.get_name() == "headless" or not _out.is_absolute_path() or _out == project or _out.begins_with(project + "/") or DirAccess.dir_exists_absolute(_out):
		push_error("MUX2_REBAKE requires windowed rendering and a new external output directory")
		quit(2)
		return
	if DirAccess.make_dir_recursive_absolute(_out) != OK:
		quit(2)
		return
	var before: Dictionary = _provenance()
	_write("provenance-start.json", before)
	var release_hash: String = FileAccess.get_sha256("res://rust/target/release/libsim_bridge.dylib")
	var release_mapped: bool = release_hash.length() == 64 and release_hash == FileAccess.get_sha256(Provenance.BRIDGE)
	_check("release_bridge_mapped", release_mapped)
	if not release_mapped:
		_write("failed-preconditions.json", _checks)
		quit(2)
		return
	DisplayServer.window_set_vsync_mode(DisplayServer.VSYNC_DISABLED)
	Engine.max_fps = 0
	_check("window1920x1080_vsync_off_uncapped", root.size == Vector2i(1920, 1080) and DisplayServer.window_get_vsync_mode() == DisplayServer.VSYNC_DISABLED and Engine.max_fps == 0)
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	main.get_node("UI/MuxHud").set("store", Store.new(_out.path_join("settings.cfg")))
	_ground = main.get_node("ColonyView/GroundLayer")
	_ground.set("keep_images", true)
	_ground.set("capture_bake_events", true)
	_ground.set("bake_capture_context", "supplement/bootstrap")
	root.add_child(main)
	if not bool(main.get("boot_ok")) or not main.has_node("WorldSim"):
		_check("bootstrap", false)
		_write("failed-preconditions.json", _checks)
		quit(2)
		return
	var world: Node = main.get_node("WorldSim")
	var camera: Camera2D = main.get_node("Camera2D") as Camera2D
	for unused: int in 5:
		await RenderingServer.frame_post_draw
	_ground.set("bake_capture_context", "supplement/stress_setup")
	var reset: bool = bool(SimReads.mutate(world, "reset_world_stress", [0, 1000]))
	_check("stress_reset_accepted", reset)
	var setup_start: int = Time.get_ticks_usec()
	for unused: int in 30:
		await RenderingServer.frame_post_draw
		if unused % 5 == 4:
			print("MUX2_REBAKE_SETUP " + JSON.stringify({"frames": unused + 1, "elapsed_usec": Time.get_ticks_usec() - setup_start, "zoom": camera.zoom.x}))
	var info: Dictionary = SimReads.read(world, "get_world_info")
	camera.zoom = Vector2(0.5, 0.5)
	camera.position = View.tile_centre(Vector2i(int(info.get("width", 64)) >> 1, int(info.get("height", 64)) >> 1))
	camera.call("_clamp_position")
	print("MUX2_REBAKE_STAGE camera_half_ready")
	_ground.set("bake_capture_context", "supplement/warmup")
	var warm_start: int = Time.get_ticks_usec()
	while Time.get_ticks_usec() - warm_start < 30000000:
		await RenderingServer.frame_post_draw
	print("MUX2_REBAKE_STAGE warmup_complete")
	# Controlled content-equivalence scope begins only after the normal automatic setup/warm-up is retained.
	world.process_mode = Node.PROCESS_MODE_DISABLED
	camera.call("set_paused", true)
	await RenderingServer.frame_post_draw
	print("MUX2_REBAKE_STAGE world_frozen_frame_drawn")
	main.process_mode = Node.PROCESS_MODE_DISABLED
	await RenderingServer.frame_post_draw
	print("MUX2_REBAKE_STAGE main_frozen_frame_drawn")
	var frozen_time: Dictionary = SimReads.read(world, "get_time_snapshot").duplicate(true)
	var population: int = (SimReads.read(world, "get_agent_snapshot").get("ids", PackedInt64Array()) as PackedInt64Array).size()
	_check("stress_population1000_half_zoom", population == 1000 and camera.zoom == Vector2(0.5, 0.5))
	var chunks: Array[Vector2i] = []
	for chunk: Vector2i in (_ground.get("_chunks") as Dictionary):
		chunks.append(chunk)
	chunks.sort_custom(func(a: Vector2i, b: Vector2i) -> bool: return a.y < b.y or (a.y == b.y and a.x < b.x))
	var required: Array[Vector2i] = _required_chunks()
	var coverage: bool = not required.is_empty() and not chunks.is_empty()
	for chunk: Vector2i in required:
		coverage = coverage and chunks.has(chunk)
	_check("all_visible_plus_margin_chunks_resident", coverage)
	var baseline: Dictionary = {}
	var baseline_rows: Array[Dictionary] = []
	for chunk: Vector2i in chunks:
		var fingerprint: Dictionary = _fingerprint(chunk)
		baseline[chunk] = fingerprint
		baseline_rows.append({"chunk": [chunk.x, chunk.y], "fingerprint": fingerprint})
		_check("baseline_%s_rgba_and_factor" % str(chunk), bool(fingerprint.get("valid", false)) and bool((fingerprint.get("cpu_rgba", {}) as Dictionary).get("valid", false)) and int((_ground.get("_res_of") as Dictionary).get(chunk, 0)) == 2)
	_write("baseline-textures.json", baseline_rows)
	_capture("before.png")
	var legacy_before: Dictionary = {"last_bake_usec": _ground.get("last_bake_usec"), "worst_bake": str(_ground.get("worst_bake")), "event_count": (_ground.get("bake_events") as Array).size()}
	var controlled_start: int = Time.get_ticks_usec()
	for pass_index: int in 3:
		_ground.set("bake_capture_context", "supplement/controlled_pass_%d" % pass_index)
		for chunk: Vector2i in chunks:
			var event_count: int = (_ground.get("bake_events") as Array).size()
			var prior: int = int((_ground.get("bake_counts") as Dictionary).get(chunk, 0))
			_ground.call("bake_chunk_now", chunk)
			var events: Array = _ground.get("bake_events")
			var captured: bool = events.size() == event_count + 1
			var event: Dictionary = events.back().duplicate(true) if captured else {}
			_check("pass%d_%s_real_rebake_event" % [pass_index, str(chunk)], captured and str(event.get("kind", "")) == "rebake" and bool(event.get("resident_before", false)) and int(event.get("previous_bakes", -1)) == prior and event.get("chunk", []) == [chunk.x, chunk.y] and int(event.get("factor", 0)) == 2)
			await RenderingServer.frame_post_draw
			var fingerprint: Dictionary = _fingerprint(chunk)
			var equal: bool = fingerprint == baseline[chunk]
			_check("pass%d_%s_identical_cpu_gpu_textures_and_strips" % [pass_index, str(chunk)], equal)
			_rows.append({"pass": pass_index, "chunk": [chunk.x, chunk.y], "event": event, "identical": equal, "fingerprint": fingerprint})
		_capture("after-pass-%d.png" % pass_index)
	var controlled_end: int = Time.get_ticks_usec()
	_ground.set("capture_bake_events", false)
	_check("controlled_content_clock_unchanged", SimReads.read(world, "get_time_snapshot") == frozen_time)
	_check("three_complete_passes_no_chunk_omissions", _rows.size() == chunks.size() * 3)
	var all_events: Array[Dictionary] = _ground.get("bake_events")
	var raw: FileAccess = FileAccess.open(_out.path_join("bake-events.jsonl"), FileAccess.WRITE)
	var raw_ok: bool = raw != null
	if raw != null:
		for event: Dictionary in all_events:
			raw.store_line(JSON.stringify(event))
		raw.flush()
		raw_ok = raw.get_error() == OK
		raw.close()
	_check("all_raw_events_written", raw_ok and FileAccess.get_sha256(_out.path_join("bake-events.jsonl")).length() == 64)
	var summary: Dictionary = Fps.summarize_bakes(all_events, controlled_start, controlled_end)
	_check("controlled_rebakes_fulltotal_at_most4000", summary["measured_rebake_status"] == "PASS")
	_check("summary_covers_exactly_all_controlled_calls", int((summary["groups"] as Dictionary).get("measurement/rebake", {}).get("count", -1)) == chunks.size() * 3)
	var after: Dictionary = _provenance()
	var changed: Array[String] = Provenance.changes(before["hashes"], after["hashes"])
	_check("immutable_sources_and_release_bridge", changed.is_empty())
	var passed: bool = true
	for check: Dictionary in _checks:
		passed = passed and bool(check["pass"])
	_write("provenance-end.json", after)
	var report_written: bool = _write("rebake-supplement.json", {"pass": passed, "scope": "Controlled actual in-game rebakes, not FPS; all startup/setup/warm-up events retained", "checks": _checks, "people": population, "zoom": 0.5, "passes": 3, "resident_chunks": _coordinates(chunks), "required_view_plus_margin": _coordinates(required), "frozen_time": frozen_time, "legacy_before_controlled": legacy_before, "controlled_start_usec": controlled_start, "controlled_end_usec": controlled_end, "summary": summary, "rows": _rows, "images": _images, "raw_event_sha256": FileAccess.get_sha256(_out.path_join("bake-events.jsonl")), "changed_inputs": changed,
		"comparison_scope": "Exact CPU chunk bytes, uploaded ground/tint/snow/strip texture bytes and shader parameters; full framebuffer images retained but shader TIME prevents a whole-scene byte-identity claim"})
	passed = passed and report_written
	print("MUX2_REBAKE_DONE " + JSON.stringify({"pass": passed, "chunks": chunks.size(), "passes": 3, "events": all_events.size(), "out": _out}))
	quit(0 if passed else 1)
