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
var _case: String
var _expected_tick: int = 0
var _camera: Camera2D
var _chronicle: Array[Dictionary] = []
var _page_context: Dictionary = {}
const Chronicle = preload("res://scripts/ui/chronicle_text.gd")
const Selection = preload("res://scripts/core/settlement_selection.gd")
const CASES: Dictionary = {"live": [20261010, 85080], "multi": [19400, 121080], "estranged": [19414, 51840], "crowd1": [20261010, 41880], "crowd2": [20261018, 85080], "crowd3": [20261010, 42180]}
const SpriteOrder = preload("res://scripts/test/art/capture_sprite_order_p13.gd")

func _init() -> void:
	call_deferred("_run")

func _run() -> void:
	_case = OS.get_environment("P13_LIFE_CASE")
	if not CASES.has(_case):
		_fail("requires a registered P13_LIFE_CASE")
		return
	_out = OS.get_environment("P13_LIFE_OUT")
	if _out.is_empty() or DirAccess.dir_exists_absolute(_out):
		push_error("P13_LIFE_OUT must name a new output directory")
		quit(1)
		return
	if DirAccess.make_dir_recursive_absolute(_out) != OK:
		_fail("cannot create evidence directory")
		return
	var git_output: Array = []
	if OS.execute("git", ["-C", ProjectSettings.globalize_path("res://"), "rev-parse", "HEAD"], git_output, true) != 0:
		_fail("cannot determine checkout HEAD")
		return
	var build_profile: String = OS.get_environment("P13_LIFE_BUILD_PROFILE")
	if build_profile not in ["debug", "release"]:
		_fail("P13_LIFE_BUILD_PROFILE must explicitly be debug or release")
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
		"loaded_bridge_sha256": loaded_sha, "sources": _source_hashes(), "seed": CASES[_case][0], "case": _case, "scene": "game_v26"}
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
	_camera = _main.get_node("Camera2D") as Camera2D
	_camera.call("set_auto_track", false)
	_camera.position_smoothing_enabled = false
	_camera.process_mode = Node.PROCESS_MODE_DISABLED
	var seed: int = int(CASES[_case][0])
	_expected_tick = int(CASES[_case][1])
	ReadRoute.mutate(_world, "reset_world", [seed])
	if not _advance_exact(_expected_tick, _expected_tick):
		return
	await process_frame
	var info: Dictionary = ReadRoute.read(_world, "get_world_info")
	if int(info["seed"]) != seed or int(info["ticks_per_day"]) != 1440 or int(info["clock_offset_ticks"]) != 480:
		_fail("world clock contract changed")
		return
	Selection.set_layers_enabled(self, true)
	var settlements: Dictionary = ReadRoute.read(_world, "get_settlement_snapshot")
	if settlements["settlement_ids"].is_empty():
		_fail("capture world has no settlement")
		return
	_camera.position = Vector2(456, 36) + Vector2(settlements["formation_xs"][0], settlements["formation_ys"][0]) * 16.0
	_set_zoom(4)
	var cursor: int = 0
	var pages: Array[Dictionary] = []
	while true:
		var page: Dictionary = _world.call("get_chronicle_since", cursor, 512)
		pages.append(page)
		_chronicle.append_array(Chronicle.rows(page))
		var next: int = int(page["next"])
		if next >= int(page["total"]):
			break
		if next <= cursor:
			_fail("full chronicle cursor stalled")
			return
		cursor = next
	_references.append({"world_info": info, "clock": ReadRoute.read(_world, "get_time_snapshot"),
		"chronicle_pages": pages, "agents": ReadRoute.read(_world, "get_agent_snapshot"),
		"tallies": ReadRoute.read(_world, "get_tallies"),
		"settlements": ReadRoute.read(_world, "get_settlement_snapshot"),
		"buildings": ReadRoute.read(_world, "get_building_snapshot"),
		"activities": ReadRoute.read(_world, "get_activity_snapshot"),
		"society": ReadRoute.read(_world, "get_society_snapshot"),
		"porters": ReadRoute.read(_world, "get_porter_snapshot"), "relations": ReadRoute.read(_world, "get_settlement_relations")})
	var person_ids: Dictionary = {}
	for row: Dictionary in _chronicle:
		for key: String in ["subject", "other", "third"]:
			if int(row.get(key, -1)) >= 0:
				person_ids[int(row[key])] = true
	var persons: Array[Dictionary] = []
	for id: int in person_ids:
		persons.append(_world.call("get_person", id))
	_references[0]["persons"] = persons
	var okay: bool = false
	match _case:
		"live": okay = await _capture_family()
		"multi": okay = await _capture_multi()
		"estranged": okay = await _capture_estranged()
		"crowd1", "crowd2", "crowd3": okay = await _capture_crowd()
	if not okay:
		return
	if not _case.begins_with("crowd") and not await _chronicle_capture():
		return
	var result: Dictionary = {"complete": true, "provenance": _provenance, "images": _images,
		"captures": _captured, "reference_worlds": _references}
	var file := FileAccess.open(_out.path_join("reference-data.json"), FileAccess.WRITE)
	if file == null:
		_fail("cannot write full reference data")
		return
	file.store_string(JSON.stringify(_json_ready(result), "\t") + "\n")
	file.close()
	quit(0)

func _capture_family() -> bool:
	var agents: Dictionary = ReadRoute.read(_world, "get_agent_snapshot")
	var ids: Array = Array(agents["agent_ids"])
	ids.sort()
	for value: Variant in ids:
		var id: int = int(value)
		var person: Dictionary = _world.call("get_person", id)
		for field: String in ["mother", "father"]:
			var parent_id: int = int(person.get(field, -1))
			if parent_id < 0:
				continue
			var parent: Dictionary = _world.call("get_person", parent_id)
			if bool(parent.get("found", false)) and not bool(parent.get("alive", false)):
				return await _person_capture("living_dead_parent", id) and await _person_capture("dead_parent", parent_id)
	_fail("no living person with dead parent at frozen tick")
	return false

func _capture_multi() -> bool:
	var causes: Dictionary = {}
	for row: Dictionary in _chronicle:
		if int(row["kind"]) == 5 and not causes.has(int(row["cause"])):
			causes[int(row["cause"])] = int(row["subject"])
	_captured["depart_causes_present"] = causes
	var absent: Array[int] = []
	for cause: int in range(5):
		if not causes.has(cause):
			absent.append(cause)
	_captured["depart_causes_absent"] = absent
	for cause: int in causes:
		if not await _person_capture("depart_cause_%d" % cause, int(causes[cause])):
			return false
	var porters: Dictionary = ReadRoute.read(_world, "get_porter_snapshot")
	var agents: Dictionary = ReadRoute.read(_world, "get_agent_snapshot")
	var index: int = -1
	for i: int in porters.get("ids", []).size():
		if int(porters["stage"][i]) == 0:
			index = i
			break
	if index < 0:
		_fail("W-multi has no actual outbound porter")
		return false
	var entity: int = int(porters["ids"][index])
	var agent_index: int = agents["ids"].find(entity)
	if agent_index < 0:
		_fail("porter has no live agent identity")
		return false
	var id: int = int(agents["agent_ids"][agent_index])
	if not await _person_capture("porter_life", id):
		return false
	_inspector.hide()
	_camera.position = Vector2(456, 36) + Vector2(agents["xs"][agent_index], agents["ys"][agent_index]) * 16.0
	_captured["outbound_porter"] = {"agent_id": id, "entity_bits": entity, "porter_snapshot": porters,
		"agent_detail": _world.call("get_agent_detail", entity)}
	for zoom: int in [4, 1]:
		_set_zoom(zoom)
		await process_frame
		_page_context = {"section": "porter_road", "zoom": zoom, "porter_sprite": _sprite_evidence(entity)}
		if not await _save("porter_road_z%d" % zoom):
			return false
	return true

func _capture_estranged() -> bool:
	var quarrel: Dictionary = {}
	var estranged: Dictionary = {}
	for row: Dictionary in _chronicle:
		if int(row["kind"]) == 19 and quarrel.is_empty():
			quarrel = row
		if int(row["kind"]) == 20 and estranged.is_empty():
			estranged = row
	if quarrel.is_empty() or estranged.is_empty():
		_fail("validated estrangement/quarrel missing at fixed tick")
		return false
	_captured["estrangement_entry"] = estranged
	if not await _person_capture("quarreller_life", int(quarrel["subject"])):
		return false
	_inspector.hide()
	_set_zoom(1)
	_page_context = {"section": "estranged_villages", "zoom": 1, "relations": ReadRoute.read(_world, "get_settlement_relations")}
	return await _save("estranged_villages_z1")

func _capture_crowd() -> bool:
	var settings: Dictionary = {"crowd1": [1, 22, 20], "crowd2": [29, 39, 35], "crowd3": [30, 24, 21]}
	var spec: Array = settings[_case]
	var agents: Dictionary = ReadRoute.read(_world, "get_agent_snapshot")
	var index: int = agents["agent_ids"].find(int(spec[0]))
	if index < 0:
		_fail("registered actual crowd subject no longer alive")
		return false
	var entity: int = int(agents["ids"][index])
	_camera.position = Vector2(456, 36) + Vector2(spec[1], spec[2]) * 16.0
	_set_zoom(4)
	for lang: String in ["en", "ko"]:
		root.get_node("Locale").call("set_language", lang)
		_inspector.call("show_person", int(spec[0]))
		for frame: int in range(4):
			await process_frame
		var selected: int = int(_inspector.call("get_selected_entity_bits"))
		var evidence: Dictionary = _sprite_evidence(entity)
		if selected != entity or not bool(evidence.get("icon_visible", false)):
			_fail("crowd actual selection or visible icon missing")
			return false
		var ordering := SpriteOrder.new()
		var rows: Array[Dictionary] = ordering.collect(_main.get_node("WorldObjects") as Node2D)
		var body_rank: int = -1
		var icon_rank: int = -1
		for row: Dictionary in rows:
			if row["node_path"] == evidence["node_path"]:
				body_rank = int(row["draw_rank"])
			if row["node_path"] == evidence["icon_node_path"]:
				icon_rank = int(row["draw_rank"])
		if not ordering.errors.is_empty() or body_rank < 0 or icon_rank < 0:
			_fail("crowd draw ordering incomplete")
			return false
		var holder: Dictionary = {"entity_bits": entity, "node_path": evidence["node_path"],
			"viewport_rect": evidence["rect"], "icon_viewport_rect": evidence["icon_rect"],
			"draw_rank": body_rank, "icon_draw_rank": icon_rank, "icon_visible": true,
			"visible_in_tree": evidence["visible"], "fully_inside_viewport": evidence["fully_inside_viewport"]}
		_page_context = {"section": _case, "language": lang, "crowd": {"complete": true, "errors": [],
			"zoom": 4, "draw_order_schema": 3, "draw_order_complete": true, "draw_rects": rows,
			"crowd_centre_tile": [spec[1], spec[2]], "raw_agent_snapshot": agents,
			"selected_entity_bits": entity, "activity_holders": [holder]}, "selected_sprite": evidence}
		if not await _save(_case + "_" + lang):
			return false
	return true

func _sprite_evidence(entity: int) -> Dictionary:
	for child: Node in _main.get_node("WorldObjects").get_children():
		if not child is Sprite2D or int(child.get_meta("entity_bits", -1)) != entity:
			continue
		var sprite: Sprite2D = child as Sprite2D
		var rect: Rect2 = sprite.get_global_transform_with_canvas() * sprite.get_rect()
		var metadata: Dictionary = {}
		for key: StringName in sprite.get_meta_list():
			metadata[str(key)] = sprite.get_meta(key)
		var result: Dictionary = {"node_path": str(sprite.get_path()), "metadata": metadata,
			"rect": [rect.position.x, rect.position.y, rect.size.x, rect.size.y],
			"visible": sprite.is_visible_in_tree(), "fully_inside_viewport": Rect2(0, 0, 1920, 1080).encloses(rect), "icon_visible": false}
		for item: Node in sprite.get_children():
			if item is Sprite2D and str(item.get_meta("art_kind", "")) == "activity_icon":
				var icon: Sprite2D = item as Sprite2D
				var icon_rect: Rect2 = icon.get_global_transform_with_canvas() * icon.get_rect()
				result.merge({"icon_node_path": str(icon.get_path()), "icon_visible": icon.is_visible_in_tree(),
					"icon_rect": [icon_rect.position.x, icon_rect.position.y, icon_rect.size.x, icon_rect.size.y],
					"icon_sprite_id": icon.get_meta("sprite_id", ""), "icon_scale": [icon.scale.x, icon.scale.y],
					"icon_alpha": icon.modulate.a}, true)
		return result
	return {}

func _person_capture(label: String, id: int) -> bool:
	var person: Dictionary = _world.call("get_person", id)
	if not bool(person.get("found", false)):
		_fail("actual capture subject not found")
		return false
	_captured[label] = {"agent_id": id, "person": person, "chronicle": _world.call("get_person_chronicle", id)}
	if bool(person.get("alive", false)):
		var agents: Dictionary = ReadRoute.read(_world, "get_agent_snapshot")
		var agent_index: int = agents["agent_ids"].find(id)
		if agent_index >= 0:
			var entity: int = int(agents["ids"][agent_index])
			_captured[label]["entity_bits"] = entity
			_captured[label]["agent_detail"] = _world.call("get_agent_detail", entity)
	for lang: String in ["en", "ko"]:
		root.get_node("Locale").call("set_language", lang)
		_inspector.call("show_person", id)
		_inspector.call("_process", 0.0)
		_inspector.call("_expand_life")
		await process_frame
		var scroll: ScrollContainer = _inspector.get("_scroll") as ScrollContainer
		_page_context = {"subject": id, "language": lang, "section": label,
			"life_text": (_inspector.get("_life_text") as RichTextLabel).text,
			"record_mode": _inspector.get("_record_mode"),
			"porter_status": {"visible": (_inspector.get("_porter_status") as Label).is_visible_in_tree(),
				"text": (_inspector.get("_porter_status") as Label).text}}
		if not await _scroll_pages(label + "_" + lang, scroll.get_v_scroll_bar()):
			return false
	return true

func _chronicle_capture() -> bool:
	_inspector.hide()
	var panel: Control = _main.get_node("UI/ChroniclePanel") as Control
	panel.show()
	# Drain the actual UI cursor at a fixed simulation tick; never inject entries.
	var drains: int = 0
	while drains == 0 or int(panel.get("_next")) < int(panel.get("_total")):
		var before: int = int(panel.get("_next"))
		panel.set("_tick", -1)
		panel.call("_process", 0.0)
		drains += 1
		if int(panel.get("_next")) <= before and int(panel.get("_next")) < int(panel.get("_total")):
			_fail("chronicle UI cursor stalled")
			return false
	for lang: String in ["en", "ko"]:
		root.get_node("Locale").call("set_language", lang)
		panel.call("_refresh")
		await process_frame
		var body: RichTextLabel = panel.get("_body") as RichTextLabel
		_page_context = {"language": lang, "section": "chronicle", "rendered_text": body.text,
			"cursor": panel.get("_next"), "total": panel.get("_total"), "fixed_tick_cursor_drains": drains}
		if not await _scroll_pages("chronicle_" + lang, body.get_v_scroll_bar()):
			return false
	panel.hide()
	return true

func _scroll_pages(stem: String, bar: VScrollBar) -> bool:
	bar.value = 0
	await process_frame
	var page: int = 0
	while true:
		_page_context["scroll_value"] = bar.value
		_page_context["scroll_max"] = bar.max_value
		_page_context["scroll_page"] = bar.page
		if not await _save(stem + "_page_%03d" % page):
			return false
		if bar.value >= bar.max_value - bar.page:
			break
		var before: float = bar.value
		bar.value = minf(bar.max_value - bar.page, bar.value + maxf(1.0, bar.page - 48.0))
		await process_frame
		if bar.value <= before or page >= 100:
			_fail("scroll page traversal stalled or exceeded bound")
			return false
		page += 1
	return true

func _set_zoom(zoom: int) -> void:
	_camera.zoom = Vector2.ONE * zoom
	_camera.reset_smoothing()
	_camera.force_update_scroll()

func _json_ready(value: Variant) -> Variant:
	if value is PackedByteArray:
		return Array(value)
	if value is Dictionary:
		var result: Dictionary = {}
		for key: Variant in value:
			result[key] = _json_ready(value[key])
		return result
	if value is Array:
		var result: Array = []
		for item: Variant in value:
			result.append(_json_ready(item))
		return result
	return value

func _save(stem: String) -> bool:
	await process_frame
	await process_frame
	if _source_hashes() != _provenance["sources"]:
		_fail("source files changed during capture")
		return false
	var clock: Dictionary = ReadRoute.read(_world, "get_time_snapshot")
	if int(clock["tick"]) != _expected_tick:
		_fail("simulation tick advanced during frozen capture")
		return false
	if _page_context.has("crowd") and not _refresh_crowd_evidence():
		return false
	if _page_context.get("section", "") == "porter_road":
		_page_context["porter_sprite"] = _sprite_evidence(int(_captured["outbound_porter"]["entity_bits"]))
	_page_context["village_layers"] = _main.get_node("SettlementLabels").call("capture_metadata")
	RenderingServer.force_draw(false)
	var image: Image = root.get_texture().get_image()
	if image.get_size() != Vector2i(1920, 1080) or image.save_png(_out.path_join(stem + ".png")) != OK:
		_fail("life capture image write or dimensions failed: " + stem)
		return false
	var evidence: Dictionary = {"page_context": _page_context.duplicate(true), "file": stem + ".png", "tick": int(clock["tick"]), "clock": clock,
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
	if advanced != expected_tick or int(clock["tick"]) != expected_tick:
		_fail("advance mismatch: requested %d returned %d expected tick %d actual %d" % [amount, advanced, expected_tick, int(clock["tick"])])
		return false
	return true

func _refresh_crowd_evidence() -> bool:
	var crowd: Dictionary = _page_context["crowd"]
	var entity: int = int(crowd["selected_entity_bits"])
	var evidence: Dictionary = _sprite_evidence(entity)
	var ordering := SpriteOrder.new()
	var rows: Array[Dictionary] = ordering.collect(_main.get_node("WorldObjects") as Node2D)
	var body_rank: int = -1
	var icon_rank: int = -1
	for row: Dictionary in rows:
		if row["node_path"] == evidence.get("node_path", ""):
			body_rank = int(row["draw_rank"])
		if row["node_path"] == evidence.get("icon_node_path", ""):
			icon_rank = int(row["draw_rank"])
	if not ordering.errors.is_empty() or body_rank < 0 or icon_rank < 0 or not bool(evidence.get("icon_visible", false)):
		_fail("actual screenshot frame lacks complete selected body/icon geometry")
		return false
	var holder: Dictionary = crowd["activity_holders"][0]
	holder.merge({"viewport_rect": evidence["rect"], "icon_viewport_rect": evidence["icon_rect"],
		"draw_rank": body_rank, "icon_draw_rank": icon_rank, "visible_in_tree": evidence["visible"],
		"fully_inside_viewport": evidence["fully_inside_viewport"]}, true)
	crowd["draw_rects"] = rows
	_page_context["selected_sprite"] = evidence
	return true

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

func _fail(message: String) -> void:
	push_error(message)
	if not _out.is_empty() and DirAccess.dir_exists_absolute(_out):
		var file: FileAccess = FileAccess.open(_out.path_join("failure.json"), FileAccess.WRITE)
		if file != null:
			file.store_string(JSON.stringify({"complete": false, "error": message, "provenance": _provenance, "images": _images}))
			file.close()
	quit(1)
