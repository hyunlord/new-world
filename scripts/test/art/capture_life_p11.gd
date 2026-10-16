extends SceneTree
## Real live-seed screenshots; reviewer receives PNGs only, never reference JSON.
const ReadRoute = preload("res://scripts/test/art/test_sim_reads_equivalence_p9.gd")
var _main: Node
var _world: Node
var _inspector: Control
var _out: String
var _captured: Dictionary = {}
var _references: Array[Dictionary] = []
var _provenance: Dictionary = {}
var _images: Array[Dictionary] = []

func _init() -> void:
	call_deferred("_run")

func _run() -> void:
	_out = OS.get_environment("P11_LIFE_OUT")
	if _out.is_empty() or DirAccess.dir_exists_absolute(_out):
		push_error("P11_LIFE_OUT must name a new output directory")
		quit(1)
		return
	if DirAccess.make_dir_recursive_absolute(_out) != OK:
		_fail("cannot create evidence directory")
		return
	var git_output: Array = []
	if OS.execute("git", ["-C", ProjectSettings.globalize_path("res://"), "rev-parse", "HEAD"], git_output, true) != 0:
		_fail("cannot determine checkout HEAD")
		return
	var build_profile: String = OS.get_environment("P11_LIFE_BUILD_PROFILE")
	if build_profile not in ["debug", "release"]:
		_fail("P11_LIFE_BUILD_PROFILE must explicitly be debug or release")
		return
	var release_sha: String = FileAccess.get_sha256("res://rust/target/release/libsim_bridge.dylib") if FileAccess.file_exists("res://rust/target/release/libsim_bridge.dylib") else ""
	var loaded_sha: String = FileAccess.get_sha256("res://rust/target/debug/libsim_bridge.dylib")
	if loaded_sha.is_empty():
		_fail("loaded debug-path bridge SHA is missing")
		return
	if build_profile == "release" and (release_sha.is_empty() or release_sha != loaded_sha):
		_fail("release and loaded debug-path bridge SHA differ")
		return
	_provenance = {"head": str(git_output[0]).strip_edges(), "build_profile": build_profile, "release_sha256": release_sha,
		"loaded_bridge_sha256": loaded_sha, "sources": _source_hashes(), "seed": 20261010, "scene": "game_v24"}
	root.size = Vector2i(1920, 1080)
	var scene: PackedScene = load("res://scenes/main.tscn") as PackedScene
	_main = scene.instantiate()
	_world = _main.get_node("WorldSim")
	_world.call("set_sim_speed", 0.0)
	_world.process_mode = Node.PROCESS_MODE_DISABLED
	root.add_child(_main)
	current_scene = _main
	_inspector = _main.get_node("UI/AgentInspectorPanel") as Control
	await process_frame
	ReadRoute.mutate(_world, "reset_world", [20261010])
	var info: Dictionary = ReadRoute.read(_world, "get_world_info")
	var tpd: int = int(info["ticks_per_day"])
	var offset: int = int(info["clock_offset_ticks"])
	var initial_clock: Dictionary = ReadRoute.read(_world, "get_time_snapshot")
	if int(initial_clock["tick"]) != 0:
		_fail("reset world tick is not zero")
		return
	var previous_tick: int = 0
	for day: int in range(60, 91):
		var tick: int = (day - 1) * tpd + floori(10.0 * tpd / 24.0) - offset
		if not _advance_exact(tick - previous_tick, tick):
			return
		previous_tick = tick
		await process_frame
		var agents: Dictionary = ReadRoute.read(_world, "get_agent_snapshot")
		var ids: Array = Array(agents["agent_ids"])
		ids.sort()
		for id_value: Variant in ids:
			var id: int = int(id_value)
			var person: Dictionary = _world.call("get_person", id)
			if not _captured.has("living_dead_parent"):
				for parent_key: String in ["mother", "father"]:
					var parent_id: int = int(person.get(parent_key, -1))
					if parent_id < 0:
						continue
					var parent: Dictionary = _world.call("get_person", parent_id)
					if bool(parent.get("found", false)) and not bool(parent.get("alive", false)):
						if not await _person_capture("living_dead_parent", id, day):
							return
						if not await _person_capture("dead_parent", parent_id, day):
							return
						break
		var page_start: int = 0
		var all_pages: Array[Dictionary] = []
		while true:
			var page: Dictionary = _world.call("get_chronicle_since", page_start, 512)
			all_pages.append(page)
			for i: int in page["kind"].size():
				if int(page["kind"][i]) == 5:
					var label: String = "depart_cause_%d" % int(page["cause"][i])
					if not _captured.has(label):
						if not await _person_capture(label, int(page["subject"][i]), day):
							return
			page_start = int(page["next"])
			if page_start >= int(page["total"]):
				break
		_references.append({"day": day, "world_info": ReadRoute.read(_world, "get_world_info"), "chronicle_pages": all_pages})
		if not _captured.has("chronicle"):
			var panel: Control = _main.get_node("UI/ChroniclePanel") as Control
			if not panel.visible:
				panel.call("toggle_visible")
			for _attempt: int in range(100):
				panel.call("_process", 0.0)
				if int(panel.get("_next")) >= int(panel.get("_total")):
					break
				if not _advance_exact(1, previous_tick + 1):
					return
				previous_tick += 1
				await process_frame
			if int(panel.get("_next")) < int(panel.get("_total")):
				_fail("chronicle feed backlog exceeded 100 actual ticks")
				return
			for lang: String in ["en", "ko"]:
				root.get_node("Locale").call("set_language", lang)
				panel.call("_process", 0.0)
				if not await _save("chronicle_" + lang):
					return
			_captured["chronicle"] = {"day": day, "tick": previous_tick, "next": panel.get("_next"), "total": panel.get("_total")}
			panel.hide()
		if _captured.size() >= 8:
			break
	var file: FileAccess = FileAccess.open(_out.path_join("reference-data.json"), FileAccess.WRITE)
	if file == null:
		_fail("cannot open reference-data.json")
		return
	file.store_string(JSON.stringify({"complete": true, "provenance": _provenance, "images": _images, "captures": _captured, "reference_days": _references}, "\t"))
	file.flush()
	if file.get_error() != OK:
		file.close()
		_fail("reference-data write failed")
		return
	file.close()
	quit(0)

func _person_capture(label: String, id: int, day: int) -> bool:
	_inspector.call("show_person", id)
	await process_frame
	var person: Dictionary = _world.call("get_person", id)
	var entries: Dictionary = _world.call("get_person_chronicle", id)
	_captured[label] = {"agent_id": id, "day": day, "person": person, "chronicle": entries}
	for lang: String in ["en", "ko"]:
		root.get_node("Locale").call("set_language", lang)
		_inspector.call("_process", 0.0)
		var scroll: ScrollContainer = _inspector.get("_scroll") as ScrollContainer
		scroll.scroll_vertical = 0
		if not await _save(label + "_" + lang + "_top"):
			return false
		_inspector.call("_expand_life")
		await process_frame
		var bar: VScrollBar = scroll.get_v_scroll_bar()
		var offset: int = maxi(1, int(scroll.size.y) - 48)
		while scroll.scroll_vertical < int(bar.max_value - bar.page):
			scroll.scroll_vertical += offset
			if not await _save(label + "_" + lang + "_scroll_%d" % scroll.scroll_vertical):
				return false
	return true

func _save(stem: String) -> bool:
	await process_frame
	await process_frame
	if _source_hashes() != _provenance["sources"]:
		_fail("source files changed during capture")
		return false
	var clock: Dictionary = ReadRoute.read(_world, "get_time_snapshot")
	RenderingServer.force_draw(false)
	var image: Image = root.get_texture().get_image()
	if image.get_size() != Vector2i(1920, 1080) or image.save_png(_out.path_join(stem + ".png")) != OK:
		_fail("life capture image write or dimensions failed: " + stem)
		return false
	var evidence: Dictionary = {"file": stem + ".png", "tick": int(clock["tick"]), "clock": clock,
		"provenance": _provenance, "png_sha256": FileAccess.get_sha256(_out.path_join(stem + ".png"))}
	var file: FileAccess = FileAccess.open(_out.path_join(stem + ".json"), FileAccess.WRITE)
	if file == null:
		_fail("cannot write capture sidecar: " + stem)
		return false
	file.store_string(JSON.stringify(evidence, "\t") + "\n")
	file.flush()
	var error: Error = file.get_error()
	file.close()
	if error != OK:
		_fail("capture sidecar write failed: " + stem)
		return false
	_images.append(evidence)
	return true

func _advance_exact(amount: int, expected_tick: int) -> bool:
	var advanced: int = int(ReadRoute.mutate(_world, "advance_ticks", [amount]))
	var clock: Dictionary = ReadRoute.read(_world, "get_time_snapshot")
	if advanced != amount or int(clock["tick"]) != expected_tick:
		_fail("advance mismatch: requested %d returned %d expected tick %d actual %d" % [amount, advanced, expected_tick, int(clock["tick"])])
		return false
	return true

func _source_hashes() -> Dictionary:
	var hashes: Dictionary = {}
	for folder: String in ["scripts", "scenes", "shaders", "localization/compiled"]:
		_hash_folder("res://" + folder, hashes)
	for path: String in ["project.godot", "sim_bridge.gdextension", "assets/art/atlas/world_atlas.json", "rust/target/debug/libsim_bridge.dylib", "rust/target/release/libsim_bridge.dylib"]:
		hashes[path] = FileAccess.get_sha256("res://" + path) if FileAccess.file_exists("res://" + path) else "ABSENT"
	return hashes

func _hash_folder(folder: String, hashes: Dictionary) -> void:
	for name: String in DirAccess.get_files_at(folder):
		if name.get_extension() in ["gd", "tscn", "gdshader", "json"]:
			var path: String = folder.path_join(name)
			hashes[path] = FileAccess.get_sha256(path)
	for name: String in DirAccess.get_directories_at(folder):
		_hash_folder(folder.path_join(name), hashes)

func _fail(message: String) -> void:
	push_error(message)
	if not _out.is_empty() and DirAccess.dir_exists_absolute(_out):
		var file: FileAccess = FileAccess.open(_out.path_join("failure.json"), FileAccess.WRITE)
		if file != null:
			file.store_string(JSON.stringify({"complete": false, "error": message, "provenance": _provenance, "images": _images}))
			file.close()
	quit(1)
