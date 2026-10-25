# Strict M-UX2 GPU visibility: eight actual saved scenes, every on-screen person, no occlusion censoring.
# Bridge methods: load_world/get_time_snapshot/get_world_info/get_agent_snapshot; others via normal scene.
# Run windowed: --manifest /absolute/manifest.json --out /absolute/new-directory. No product hooks.
extends SceneTree

const Provenance = preload("res://tools/v8/mux2_visibility_provenance.gd")
const Metrics = preload("res://tools/v8/mux2_visibility_metrics.gd")
const Store = preload("res://scripts/ui/mux/settings_store.gd")
var _out: String
var _failures: Array[String] = []
var _scenes: Array[Dictionary] = []
var _main: Node
var _world: Node
var _camera: Camera2D
var _agents: Node2D


func _init() -> void:
	call_deferred("_run")


func _arg(key: String) -> String:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	var at: int = args.find(key)
	return args[at + 1] if at >= 0 and at + 1 < args.size() else ""


func _run() -> void:
	_out = _arg("--out").simplify_path().trim_suffix("/")
	var manifest_path: String = _arg("--manifest")
	var project_path: String = ProjectSettings.globalize_path("res://").simplify_path().trim_suffix("/")
	if _out == project_path or _out.begins_with(project_path + "/"):
		push_error("MUX2_VISIBILITY raw evidence must be outside the repository")
		quit(1)
		return
	if DisplayServer.get_name() == "headless" or not _out.is_absolute_path() or not manifest_path.is_absolute_path() or DirAccess.dir_exists_absolute(_out):
		push_error("MUX2_VISIBILITY requires GPU, absolute manifest and NEW output directory")
		quit(1)
		return
	if DirAccess.make_dir_recursive_absolute(_out) != OK:
		quit(1)
		return
	var provenance_start: Dictionary = Provenance.capture(manifest_path)
	_write(_out.path_join("provenance-start.json"), provenance_start)
	var start_hashes: Dictionary = provenance_start["hashes"]
	for missing: String in Provenance.changes(start_hashes, start_hashes):
		_failures.append("missing/unreadable input at start " + missing)
	var value: Variant = JSON.parse_string(FileAccess.get_file_as_string(manifest_path))
	if not value is Dictionary or int(value.get("schema", 0)) != 1 or not value.get("scenes") is Array:
		push_error("MUX2_VISIBILITY invalid manifest")
		quit(1)
		return
	var expected: Array[String] = ["day1_noon", "day1_night", "day20_noon", "day20_night"]
	var seen: Array[String] = []
	for entry: Variant in value["scenes"]:
		if not entry is Dictionary:
			_failures.append("non-object scene")
			continue
		var row: Dictionary = entry
		var id: String = str(row.get("id", ""))
		if id not in expected or id in seen:
			_failures.append("unknown/duplicate scene " + id)
			continue
		seen.append(id)
		await _save_scenes(row)
	if seen.size() != 4 or _scenes.size() != 8:
		_failures.append("incomplete eight-scene coverage")
	var provenance_end: Dictionary = Provenance.capture(manifest_path)
	_write(_out.path_join("provenance-end.json"), provenance_end)
	var changed_inputs: Array[String] = Provenance.changes(start_hashes, provenance_end["hashes"])
	for changed: String in changed_inputs:
		_failures.append("input mutated/unreadable during capture " + changed)
	var report: Dictionary = {"schema": 2, "manifest_sha256": start_hashes.get(manifest_path, ""), "bridge_sha256": start_hashes.get(Provenance.BRIDGE, ""), "render_params_sha256": start_hashes.get("res://assets/v8art/render_params.json", ""), "provenance_start": provenance_start, "provenance_end": provenance_end, "changed_inputs": changed_inputs, "inputs_unchanged": changed_inputs.is_empty(), "scenes": _scenes, "failures": _failures, "pass": _failures.is_empty(), "ground_control": "same camera/daylight; only GroundLayer visible; includes actual baked walls; no fabricated pixels", "attribution": "leave-one-person-out framebuffer difference; unchanged shadows and activity glyphs cannot count"}
	_write(_out.path_join("visibility.json"), report)
	print("MUX2_VISIBILITY_DONE " + JSON.stringify({"scenes": _scenes.size(), "failures": _failures.size(), "pass": _failures.is_empty()}))
	quit(0 if _failures.is_empty() else 1)


func _save_scenes(row: Dictionary) -> void:
	var path: String = str(row.get("save", ""))
	if not path.is_absolute_path() or not FileAccess.file_exists(path) or str(row.get("sha256", "")).length() != 64 or FileAccess.get_sha256(path) != str(row.get("sha256", "missing")):
		_failures.append("save missing/hash mismatch " + path)
		return
	_main = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	_main.get_node("UI/MuxHud").set("store", Store.new(_out.path_join(str(row["id"]) + "-settings.cfg")))
	root.add_child(_main)
	if not bool(_main.get("boot_ok")) or not _main.has_node("WorldSim"):
		_failures.append("main scene bootstrap failed")
		_main.queue_free()
		await process_frame
		return
	_world = _main.get_node("WorldSim")
	_world.process_mode = Node.PROCESS_MODE_DISABLED
	_camera = _main.get_node("Camera2D") as Camera2D
	_agents = _main.get_node("ColonyView/AgentLayer") as Node2D
	if not bool(SimReads.mutate(_world, "load_world", [path])):
		_failures.append("load failed " + path)
		_main.queue_free()
		await process_frame
		return
	_camera.call("set_paused", true)
	var clock: Dictionary = _world.get_time_snapshot()
	var info: Dictionary = _world.get_world_info()
	var day: int = 1 if str(row["id"]).begins_with("day1_") else 20
	var hour: int = 12 if str(row["id"]).ends_with("noon") else 22
	if int(info.get("seed", -1)) != 0 or int(clock.get("day_index", -1)) + 1 != day or absf(float(clock.get("time_of_day", -100)) - hour) > 0.02 or int(clock.get("tick", -1)) != int(row.get("tick", -2)):
		_failures.append("wrong live seed/calendar " + str(row["id"]))
		_main.queue_free()
		await process_frame
		return
	for zoom: float in [0.5, 0.25]:
		_main.process_mode = Node.PROCESS_MODE_INHERIT
		_camera.zoom = Vector2.ONE * zoom
		_camera.call("center_on_camp")
		for _i: int in 45:
			await process_frame
		_main.process_mode = Node.PROCESS_MODE_DISABLED
		await _measure_scene(row, clock, zoom)
	_main.queue_free()
	await process_frame


func _capture(path: String) -> Image:
	await RenderingServer.frame_post_draw
	await RenderingServer.frame_post_draw
	var image: Image = root.get_texture().get_image()
	if image == null or image.get_size() != Vector2i(1920, 1080) or image.save_png(path) != OK:
		_failures.append("readback failed/wrong dimensions " + path)
	return image


func _measure_scene(input: Dictionary, clock: Dictionary, zoom: float) -> void:
	var id: String = "%s_%s" % [input["id"], "half" if zoom == 0.5 else "quarter"]
	var directory: String = _out.path_join(id)
	DirAccess.make_dir_recursive_absolute(directory)
	var _full: Image = await _capture(directory.path_join("full.png"))
	var hidden: Array[CanvasItem] = []
	for child: Node in _main.get_node("ColonyView").get_children():
		if child is CanvasItem and child.name != "GroundLayer":
			var item: CanvasItem = child as CanvasItem
			if item.visible:
				hidden.append(item)
				item.hide()
	var ui: CanvasLayer = _main.get_node("UI") as CanvasLayer
	ui.hide()
	var ground: Image = await _capture(directory.path_join("ground.png"))
	for child: CanvasItem in hidden:
		child.show()
	ui.show()
	var persons: Dictionary = _agents.get("_persons")
	var snapshot: Dictionary = _world.get_agent_snapshot()
	var all_bits: PackedInt64Array = snapshot.get("ids", PackedInt64Array())
	var expected: Array[int] = []
	var rows: Array[Dictionary] = []
	var offscreen: Array[int] = []
	var lod: bool = bool(_agents.call("lod_active"))
	var markers: Node = _agents.get_node("LodMarkers")
	var discs: MultiMeshInstance2D = markers.get("_discs") as MultiMeshInstance2D
	var keys: Array = persons.keys()
	for bits: int in all_bits:
		if not persons.has(bits):
			_failures.append("missing render ID %d in %s" % [bits, id])
			continue
		var person: Node2D = persons[bits] as Node2D
		var index: int = keys.find(bits)
		if lod and (index < 0 or index >= discs.multimesh.instance_count):
			_failures.append("missing marker ID %d in %s" % [bits, id])
			continue
		var bounds: Rect2i = _bounds(person, discs, index, lod)
		if not bounds.intersects(Rect2i(0, 0, 1920, 1080)):
			offscreen.append(bits)
			continue
		expected.append(bits)
		var before: Image = await _capture(directory.path_join("person-%d.png" % bits))
		var body: Sprite2D = person.get("body") as Sprite2D
		var previous: bool = body.visible
		var transform: Transform2D = Transform2D.IDENTITY
		if lod:
			transform = discs.multimesh.get_instance_transform_2d(index)
			discs.multimesh.set_instance_transform_2d(index, Transform2D(Vector2.ZERO, Vector2.ZERO, transform.origin))
		else:
			body.hide()
		var removed: Image = await _capture(directory.path_join("without-%d.png" % bits))
		if lod:
			discs.multimesh.set_instance_transform_2d(index, transform)
		else:
			body.visible = previous
		var restored: Image = await _capture(directory.path_join("restored-%d.png" % bits))
		var result: Dictionary = Metrics.measure(before, removed, ground, bounds, 60 if zoom == 0.5 else 140, restored)
		var mask: Image = result.get("mask") as Image
		result.erase("mask")
		if mask != null:
			mask.save_png(directory.path_join("mask-%d.png" % bits))
		result["entity_bits"] = bits
		result["agent_id"] = int(person.get("agent_id"))
		result["activity"] = str(person.get("activity"))
		result["full_sha256"] = FileAccess.get_sha256(directory.path_join("person-%d.png" % bits))
		result["removed_sha256"] = FileAccess.get_sha256(directory.path_join("without-%d.png" % bits))
		result["restored_sha256"] = FileAccess.get_sha256(directory.path_join("restored-%d.png" % bits))
		rows.append(result)
	var passed: bool = Metrics.coverage(expected, rows) and ((zoom == 0.25) == lod)
	if not passed:
		_failures.append("visibility/LOD failed " + id)
	var result: Dictionary = {"id": id, "clock": clock, "zoom": zoom, "lod": lod, "camera": [_camera.position.x, _camera.position.y], "viewport": [1920, 1080], "all_ids": Array(all_bits), "on_screen_ids": expected, "off_screen_ids": offscreen, "rows": rows, "pass": passed, "full_sha256": FileAccess.get_sha256(directory.path_join("full.png")), "ground_sha256": FileAccess.get_sha256(directory.path_join("ground.png"))}
	_scenes.append(result)
	_write(directory.path_join("result.json"), result)
	print("MUX2_VISIBILITY_SCENE " + JSON.stringify(result))


func _bounds(person: Node2D, discs: MultiMeshInstance2D, index: int, lod: bool) -> Rect2i:
	var body: Sprite2D = person.get("body") as Sprite2D
	var rectangle: Rect2 = body.get_rect()
	var transform: Transform2D = body.get_global_transform_with_canvas()
	var margin: float = 1.0
	if lod:
		rectangle = Rect2(-0.5, -0.5, 1, 1)
		transform = discs.get_global_transform_with_canvas() * discs.multimesh.get_instance_transform_2d(index)
		margin = 3.0
	elif body.texture != null:
		var texture: Image = body.texture.get_image()
		if texture.is_compressed():
			texture.decompress()
		var used: Rect2i = texture.get_used_rect()
		if body.flip_h:
			used.position.x = texture.get_width() - used.end.x
		rectangle = Rect2(rectangle.position + Vector2(used.position), Vector2(used.size))
	var projected: Rect2 = transform * rectangle
	projected = projected.grow(margin)
	return Rect2i(Vector2i(floori(projected.position.x), floori(projected.position.y)), Vector2i(ceili(projected.size.x), ceili(projected.size.y)))


func _write(path: String, value: Dictionary) -> void:
	var file: FileAccess = FileAccess.open(path, FileAccess.WRITE)
	if file == null:
		_failures.append("cannot write " + path)
		return
	file.store_string(JSON.stringify(value, "  "))
	file.close()
