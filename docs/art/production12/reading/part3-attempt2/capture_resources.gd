extends SceneTree
## Separate deterministic corroboration; never replaces original captures.
const ReadRoute = preload("res://scripts/test/art/test_sim_reads_equivalence_p9.gd")
const CAPTURE_ROOT = "/Users/rexxa/work/art-production12-evidence/p3/live-attempt2"
const CASES = {"live": [20261010, 85080], "multi": [19400, 121080], "estranged": [19414, 34680], "crowd1": [20261010, 41880], "crowd2": [20261018, 85080], "crowd3": [20261010, 42180]}

func _init() -> void:
	call_deferred("_run")

func _run() -> void:
	var output: String = OS.get_environment("P12_RESOURCE_OUT")
	if output.is_empty() or FileAccess.file_exists(output):
		push_error("P12_RESOURCE_OUT must be a new JSON path")
		quit(1)
		return
	var registration: Dictionary = JSON.parse_string(FileAccess.get_file_as_string(CAPTURE_ROOT + "/registration.json"))
	var before: Dictionary = _source_hashes()
	var git: Array = []
	if OS.execute("git", ["-C", ProjectSettings.globalize_path("res://"), "rev-parse", "HEAD"], git, true) != 0:
		quit(1)
		return
	var loaded_sha: String = FileAccess.get_sha256("res://rust/target/debug/libsim_bridge.dylib")
	if loaded_sha != registration["loaded_bridge_sha256"]:
		push_error("bridge differs from original attempt 2; refusing replay")
		quit(1)
		return
	var scene: PackedScene = load("res://scenes/main.tscn") as PackedScene
	var main: Node = scene.instantiate()
	var world: Node = main.get_node("WorldSim")
	world.call("set_sim_speed", 0.0)
	world.process_mode = Node.PROCESS_MODE_DISABLED
	root.add_child(main)
	current_scene = main
	await process_frame
	var rows: Array[Dictionary] = []
	for case: String in CASES:
		var seed: int = int(CASES[case][0])
		var tick: int = int(CASES[case][1])
		if not bool(ReadRoute.mutate(world, "reset_world", [seed])):
			quit(1)
			return
		var advanced: int = int(ReadRoute.mutate(world, "advance_ticks", [tick]))
		await process_frame
		var info: Dictionary = ReadRoute.read(world, "get_world_info")
		var clock: Dictionary = ReadRoute.read(world, "get_time_snapshot")
		if advanced != tick or int(clock["tick"]) != tick or int(info["seed"]) != seed or int(info["ticks_per_day"]) != 1440 or int(info["clock_offset_ticks"]) != 480:
			push_error("replay clock mismatch: " + case)
			quit(1)
			return
		var resources: Dictionary = ReadRoute.read(world, "get_resource_snapshot")
		var tallies: Dictionary = ReadRoute.read(world, "get_tallies")
		var berries: int = 0
		if not resources.has("kinds") or not resources.has("amounts") or resources["kinds"].size() != resources["amounts"].size():
			quit(1)
			return
		for index: int in resources["kinds"].size():
			if int(resources["kinds"][index]) == 0 and int(resources["amounts"][index]) < 255:
				berries += int(resources["amounts"][index])
		var original_path: String = CAPTURE_ROOT.path_join(case).path_join("reference-data.json")
		var original: Dictionary = JSON.parse_string(FileAccess.get_file_as_string(original_path))
		var original_world: Dictionary = original["reference_worlds"][0]
		rows.append({"case": case, "seed": seed, "tick": tick, "resources": resources, "tallies": tallies, "clock": clock, "world_info": info,
			"derived_hud_berries": berries, "derivation": "sum(amounts[i]) where kinds[i] == 0 and amounts[i] < 255; hud_topbar.gd",
			"original_reference_sha256": FileAccess.get_sha256(original_path),
			"original_tallies_equal": tallies == original_world["tallies"], "original_clock_equal": clock == original_world["clock"],
			"original_sources_equal": before == original["provenance"]["sources"]})
	var after: Dictionary = _source_hashes()
	var git_after: Array = []
	OS.execute("git", ["-C", ProjectSettings.globalize_path("res://"), "rev-parse", "HEAD"], git_after, true)
	if before != after or git != git_after:
		push_error("source or HEAD changed during replay")
		quit(1)
		return
	var file := FileAccess.open(output, FileAccess.WRITE)
	if file == null:
		quit(1)
		return
	file.store_string(JSON.stringify(_plain({"purpose": "Separate post-capture berry corroboration, not original capture-time evidence", "injected_facts": false,
		"head": str(git[0]).strip_edges(), "original_head": registration["head"], "loaded_bridge_sha256": loaded_sha,
		"sources_before": before, "sources_after": after, "probe_sha256": FileAccess.get_sha256(get_script().resource_path), "worlds": rows}), "\t") + "\n")
	file.close()
	quit(0)

func _plain(value: Variant) -> Variant:
	if value is Dictionary:
		var result: Dictionary = {}
		for key: Variant in value:
			result[key] = _plain(value[key])
		return result
	if value is Array:
		var result: Array = []
		for item: Variant in value:
			result.append(_plain(item))
		return result
	if value is PackedByteArray or value is PackedInt32Array or value is PackedInt64Array or value is PackedFloat32Array or value is PackedFloat64Array or value is PackedStringArray:
		return Array(value)
	return value

func _source_hashes() -> Dictionary:
	var hashes: Dictionary = {}
	for folder: String in ["scripts", "scenes", "shaders", "localization/compiled", "assets/fonts", "assets/art/atlas"]:
		_hash_folder("res://" + folder, hashes)
	for path: String in ["project.godot", "sim_bridge.gdextension", "assets/art/atlas/world_atlas.json", "rust/target/debug/libsim_bridge.dylib", "rust/target/release/libsim_bridge.dylib"]:
		hashes[path] = FileAccess.get_sha256("res://" + path) if FileAccess.file_exists("res://" + path) else "ABSENT"
	return hashes

func _hash_folder(folder: String, hashes: Dictionary) -> void:
	for name: String in DirAccess.get_files_at(folder):
		if name.get_extension() in ["gd", "tscn", "gdshader", "json", "png", "ttf", "otf"]:
			var path: String = folder.path_join(name)
			hashes[path] = FileAccess.get_sha256(path)
	for name: String in DirAccess.get_directories_at(folder):
		_hash_folder(folder.path_join(name), hashes)

