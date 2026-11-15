# Strict M-UX2 GPU visibility: eight actual saved scenes, every on-screen person, no occlusion censoring.
# M3D-SWITCH E2: measured on the 3D view. The ground control hides every World3D layer but the terrain, the built
#   walls/roofs, the sky and the camera (and the HUD and map labels); a person's bounds are an independent projection of
#   a box around their figure (feet ± BODY_HALF_WIDTH tiles, feet to head; World3D/People `bodies`); a person is removed
#   by scaling every instance of their figure to zero (People.figure_instances: full and LOD crowd of their bin,
#   contact shadow kept), so unchanged shadows and labels cannot count. Beyond `people.lod_distance` the figure is the
#   LOD bake (still a figure, not a marker).
# Bridge methods: load_world/get_time_snapshot/get_world_info/get_agent_snapshot; others via normal scene.
# Run windowed: --manifest /absolute/manifest.json --out /absolute/new-directory.
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
var _w3d: Node
var _people: Node
const BODY_HALF_WIDTH: float = 0.35
## World3D children kept in the ground control (the 2D control kept the ground chunks with their baked walls).
const GROUND_LAYERS: Array[String] = ["Terrain", "Structures", "Sky", "CameraRig"]


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
	var report: Dictionary = {"schema": 2, "renderer": "3d", "manifest_sha256": start_hashes.get(manifest_path, ""), "bridge_sha256": start_hashes.get(Provenance.BRIDGE, ""), "render_params_sha256": start_hashes.get("res://assets/v8art/render_params.json", ""), "provenance_start": provenance_start, "provenance_end": provenance_end, "changed_inputs": changed_inputs, "inputs_unchanged": changed_inputs.is_empty(), "scenes": _scenes, "failures": _failures, "pass": _failures.is_empty(), "ground_control": "same camera and sky; only World3D Terrain, Structures, Sky and CameraRig visible (walls and roofs included); no fabricated pixels", "attribution": "leave-one-person-out framebuffer difference (the person's crowd instance scaled to zero); unchanged contact shadows and labels cannot count"}
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
	_w3d = _main.get_node_or_null("World3D")
	_people = _w3d.get_node_or_null("People") if _w3d != null else null
	if _people == null:
		_failures.append("main scene has no World3D/People")
		_main.queue_free()
		await process_frame
		return
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
		# The 3D rig eases its height onto the ground: wait for it as well as the camera.
		for _i: int in 90:
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
	var hidden: Array[Node] = []
	for child: Node in _w3d.get_children():
		if str(child.name) in GROUND_LAYERS:
			continue
		if child is Node3D and (child as Node3D).visible:
			(child as Node3D).hide()
			hidden.append(child)
		elif child is CanvasLayer and (child as CanvasLayer).visible:
			(child as CanvasLayer).hide()
			hidden.append(child)
	var ui: CanvasLayer = _main.get_node("UI") as CanvasLayer
	ui.hide()
	var ground: Image = await _capture(directory.path_join("ground.png"))
	for child: Node in hidden:
		child.set("visible", true)
	ui.show()
	var snapshot: Dictionary = _world.get_agent_snapshot()
	var all_bits: PackedInt64Array = snapshot.get("ids", PackedInt64Array())
	var agent_ids: PackedInt64Array = snapshot.get("agent_ids", PackedInt64Array())
	var expected: Array[int] = []
	var rows: Array[Dictionary] = []
	var offscreen: Array[int] = []
	var partial: Array[Dictionary] = []
	var lod: bool = false
	var cam: Camera3D = _w3d.call("camera") as Camera3D
	for row_index: int in all_bits.size():
		var bits: int = all_bits[row_index]
		var body: Dictionary = _people.call("body_of", bits)
		var instances: Array = _people.call("figure_instances", bits) if not body.is_empty() else []
		if body.is_empty() or instances.is_empty():
			_failures.append("missing render ID %d in %s" % [bits, id])
			continue
		var bounds: Rect2i = Metrics.box_bounds(cam, body["feet"], float(body["h"]), BODY_HALF_WIDTH, 1.0)
		if not bounds.intersects(Rect2i(0, 0, 1920, 1080)):
			offscreen.append(bits)
			continue
		if not Metrics.judged(bounds, Rect2i(0, 0, 1920, 1080)):
			partial.append({"entity_bits": bits, "bounds": [bounds.position.x, bounds.position.y, bounds.size.x, bounds.size.y]})
			continue
		expected.append(bits)
		var before: Image = await _capture(directory.path_join("person-%d.png" % bits))
		var saved: Array[Transform3D] = []
		for inst: Array in instances:
			var transform: Transform3D = (inst[0] as MultiMesh).get_instance_transform(int(inst[1]))
			saved.append(transform)
			(inst[0] as MultiMesh).set_instance_transform(int(inst[1]), Transform3D(Basis().scaled(Vector3.ZERO), transform.origin))
		var removed: Image = await _capture(directory.path_join("without-%d.png" % bits))
		for k: int in instances.size():
			(instances[k][0] as MultiMesh).set_instance_transform(int(instances[k][1]), saved[k])
		var restored: Image = await _capture(directory.path_join("restored-%d.png" % bits))
		var result: Dictionary = Metrics.measure(before, removed, ground, bounds, 60 if zoom == 0.5 else 140, restored)
		var mask: Image = result.get("mask") as Image
		result.erase("mask")
		if mask != null:
			mask.save_png(directory.path_join("mask-%d.png" % bits))
		result["entity_bits"] = bits
		result["agent_id"] = agent_ids[row_index]
		result["activity"] = str((_people.call("state_of", bits) as Dictionary).get("clip", ""))
		result["full_sha256"] = FileAccess.get_sha256(directory.path_join("person-%d.png" % bits))
		result["removed_sha256"] = FileAccess.get_sha256(directory.path_join("without-%d.png" % bits))
		result["restored_sha256"] = FileAccess.get_sha256(directory.path_join("restored-%d.png" % bits))
		rows.append(result)
	var passed: bool = Metrics.coverage(expected, rows) and not lod
	if not passed:
		_failures.append("visibility/LOD failed " + id)
	var result: Dictionary = {"id": id, "clock": clock, "zoom": zoom, "lod": lod, "camera": [_camera.position.x, _camera.position.y], "viewport": [1920, 1080], "all_ids": Array(all_bits), "on_screen_ids": expected, "off_screen_ids": offscreen, "partial": partial, "rows": rows, "pass": passed, "full_sha256": FileAccess.get_sha256(directory.path_join("full.png")), "ground_sha256": FileAccess.get_sha256(directory.path_join("ground.png"))}
	_scenes.append(result)
	_write(directory.path_join("result.json"), result)
	print("MUX2_VISIBILITY_SCENE " + JSON.stringify(result))


func _write(path: String, value: Dictionary) -> void:
	var file: FileAccess = FileAccess.open(path, FileAccess.WRITE)
	if file == null:
		_failures.append("cannot write " + path)
		return
	file.store_string(JSON.stringify(value, "  "))
	file.close()
