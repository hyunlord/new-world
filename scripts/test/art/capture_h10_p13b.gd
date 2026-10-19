extends "res://scripts/test/art/capture_life_p13.gd"
## Actual game_v26 H10 views; inherited hashing, exact-clock and failure evidence.

func _run() -> void:
	_case = OS.get_environment("P13B_H10_CASE")
	if _case not in ["road", "relations", "chronicle"]:
		_fail("P13B_H10_CASE must be road, relations, or chronicle")
		return
	_out = OS.get_environment("P13B_H10_OUT")
	if _out.is_empty() or DirAccess.dir_exists_absolute(_out):
		push_error("P13B_H10_OUT must name a new output directory")
		quit(1)
		return
	if DirAccess.make_dir_recursive_absolute(_out) != OK:
		_fail("cannot create evidence directory")
		return
	var git_output: Array = []
	if OS.execute("git", ["-C", ProjectSettings.globalize_path("res://"), "rev-parse", "HEAD"], git_output, true) != 0:
		_fail("cannot determine checkout HEAD")
		return
	var build_profile: String = OS.get_environment("P13B_H10_BUILD_PROFILE")
	if build_profile not in ["debug", "release"]:
		_fail("P13B_H10_BUILD_PROFILE must explicitly be debug or release")
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
		"loaded_bridge_sha256": loaded_sha, "sources": _source_hashes(), "seed": 19603, "case": _case, "scene": "game_v26"}
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
	var seed: int = 19603
	_expected_tick = int(OS.get_environment("P13B_H10_TICK")) if OS.has_environment("P13B_H10_TICK") else (122450 if _case == "road" else 122430)
	if _expected_tick <= 0 or _expected_tick >= 172320:
		_fail("capture must be within displayed day120")
		return
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
	var okay: bool = false
	match _case:
		"road": okay = await _road()
		"relations": okay = await _relations_view()
		"chronicle": okay = await _peace()
	if not okay:
		return
	var result: Dictionary = {"complete": true, "visual_acceptance": "NOT_VERIFIED", "provenance": _provenance,
		"images": _images, "captures": _captured, "reference_worlds": _references}
	var file := FileAccess.open(_out.path_join("reference-data.json"), FileAccess.WRITE)
	if file == null:
		_fail("cannot write H10 reference")
		return
	file.store_string(JSON.stringify(_json_ready(result), "\t") + "\n")
	file.close()
	quit(0)

func _road() -> bool:
	var porters: Dictionary = ReadRoute.read(_world, "get_porter_snapshot")
	var agents: Dictionary = ReadRoute.read(_world, "get_agent_snapshot")
	var settlements: Dictionary = ReadRoute.read(_world, "get_settlement_snapshot")
	var candidates: Array[Dictionary] = []
	for i: int in porters.get("ids", []).size():
		var agent_index: int = agents["ids"].find(int(porters["ids"][i]))
		if int(porters["stage"][i]) == 0 and agent_index >= 0:
			candidates.append({"porter_index": i, "agent_index": agent_index, "agent_id": int(agents["agent_ids"][agent_index])})
	candidates.sort_custom(func(a: Dictionary, b: Dictionary) -> bool: return int(a["agent_id"]) < int(b["agent_id"]))
	_captured["road_candidates"] = []
	_captured["saved_coverage"] = {}
	_inspector.hide()
	_set_zoom(4)
	var entity: int = -1
	for candidate: Dictionary in candidates:
		var i: int = int(candidate["porter_index"])
		var agent_index: int = int(candidate["agent_index"])
		var tile := Vector2(agents["xs"][agent_index], agents["ys"][agent_index])
		var source_index: int = settlements["settlement_ids"].find(int(porters["from"][i]))
		var destination_index: int = settlements["settlement_ids"].find(int(porters["to"][i]))
		candidate["road_eligible"] = false
		if source_index >= 0 and destination_index >= 0:
			var source := Vector2(settlements["formation_xs"][source_index], settlements["formation_ys"][source_index])
			var destination := Vector2(settlements["formation_xs"][destination_index], settlements["formation_ys"][destination_index])
			candidate["source_distance_tiles"] = tile.distance_to(source)
			candidate["destination_distance_tiles"] = tile.distance_to(destination)
			candidate["road_eligible"] = tile.distance_to(source) > 2.0 and tile.distance_to(destination) > 2.0 and Vector2(agents["target_xs"][agent_index], agents["target_ys"][agent_index]) == destination and (float(agents["carried_foods"][agent_index]) > 0.0 or int(agents["carried_woods"][agent_index]) > 0)
		if bool(candidate["road_eligible"]):
			_camera.position = Vector2(456, 36) + tile * 16.0
			_camera.force_update_scroll()
			await process_frame
			await process_frame
			candidate["coverage"] = _road_coverage(int(porters["ids"][i]))
		_captured["road_candidates"].append(candidate.duplicate(true))
		if bool(candidate["road_eligible"]) and bool(candidate["coverage"]["clear"]):
			entity = int(porters["ids"][i])
			_captured["outbound_porter"] = {"entity_bits": entity, "agent_id": candidate["agent_id"], "porter_snapshot": porters}
			break
	if entity < 0:
		_fail("no outbound porter has an unobscured road position at requested tick")
		return false
	for lang: String in ["en", "ko"]:
		root.get_node("Locale").call("set_language", lang)
		_page_context = {"language": lang, "section": "porter_road", "zoom": 4}
		if not await _save("porter_road_z4_" + lang):
			return false
		# No await after _save: collect the same scene state used by force_draw.
		var coverage: Dictionary = _road_coverage(entity)
		coverage["png_sha256"] = _images.back()["png_sha256"]
		_captured["saved_coverage"][lang] = coverage
		if not bool(coverage["clear"]):
			_fail("saved porter frame has foreign body/icon coverage")
			return false
		var sprite: Dictionary = _images.back()["page_context"]["porter_sprite"]
		var metadata: Dictionary = sprite.get("metadata", {})
		if not bool(sprite.get("visible", false)) or not bool(sprite.get("fully_inside_viewport", false)) or not bool(sprite.get("icon_visible", false)):
			_fail("porter body or glyph absent from saved frame")
			return false
		if str(sprite.get("icon_sprite_id", "")) not in ["p12_porter_food", "p12_porter_wood"] or str(metadata.get("pose", "")) not in ["carry", "carry_wood"]:
			_fail("saved porter lacks expected glyph and carry pose")
			return false
	return true

func _road_coverage(entity: int) -> Dictionary:
	var sprite: Dictionary = _sprite_evidence(entity)
	var ordering := SpriteOrder.new()
	var rows: Array[Dictionary] = ordering.collect(_main.get_node("WorldObjects") as Node2D)
	var result: Dictionary = {"clear": false, "sprite": sprite, "draw_rects": rows, "errors": ordering.errors.duplicate(), "body_blockers": [], "icon_blockers": []}
	if not ordering.errors.is_empty() or not bool(sprite.get("visible", false)) or not bool(sprite.get("fully_inside_viewport", false)) or not bool(sprite.get("icon_visible", false)):
		return result
	var body_rank: int = -1
	var icon_rank: int = -1
	for row: Dictionary in rows:
		if str(row["node_path"]) == str(sprite["node_path"]):
			body_rank = int(row["draw_rank"])
		if str(row["node_path"]) == str(sprite["icon_node_path"]):
			icon_rank = int(row["draw_rank"])
	result["body_draw_rank"] = body_rank
	result["icon_draw_rank"] = icon_rank
	var body := Rect2(sprite["rect"][0], sprite["rect"][1], sprite["rect"][2], sprite["rect"][3])
	var icon := Rect2(sprite["icon_rect"][0], sprite["icon_rect"][1], sprite["icon_rect"][2], sprite["icon_rect"][3])
	if body_rank < 0 or icon_rank < 0 or not Rect2(0, 0, 1920, 1080).encloses(icon):
		return result
	var owner: String = str(sprite["node_path"])
	for row: Dictionary in rows:
		var path: String = str(row["node_path"])
		if path == owner or path.begins_with(owner + "/"):
			continue
		var rect := Rect2(row["rect"][0], row["rect"][1], row["rect"][2], row["rect"][3])
		if int(row["draw_rank"]) > body_rank and body.intersects(rect):
			result["body_blockers"].append(row)
		if int(row["draw_rank"]) > icon_rank and icon.intersects(rect):
			result["icon_blockers"].append(row)
	result["clear"] = result["body_blockers"].is_empty() and result["icon_blockers"].is_empty()
	return result

func _relations_view() -> bool:
	_inspector.hide()
	_set_zoom(1)
	var settlements: Dictionary = ReadRoute.read(_world, "get_settlement_snapshot")
	var centre := Vector2.ZERO
	var count: int = 0
	for i: int in settlements["formation_xs"].size():
		if int(settlements["formation_xs"][i]) >= 0 and int(settlements["formation_ys"][i]) >= 0:
			centre += Vector2(settlements["formation_xs"][i], settlements["formation_ys"][i])
			count += 1
	if count < 2:
		_fail("fewer than two actual village anchors")
		return false
	_camera.position = Vector2(456, 36) + centre / float(count) * 16.0
	_camera.force_update_scroll()
	for lang: String in ["en", "ko"]:
		root.get_node("Locale").call("set_language", lang)
		_page_context = {"language": lang, "section": "relations", "zoom": 1}
		if not await _save("relations_z1_" + lang):
			return false
		var relations: Dictionary = _images.back()["page_context"]["village_layers"]["relations"]
		var colors: Dictionary = {}
		for row: Dictionary in relations.get("lines", []):
			var start := Vector2(row["from_screen"][0], row["from_screen"][1])
			var end := Vector2(row["to_screen"][0], row["to_screen"][1])
			if Rect2(0, 0, 1920, 1080).has_point(start) and Rect2(0, 0, 1920, 1080).has_point(end) and start.distance_to(end) > 2.0:
				colors[str(row["color"])] = true
		if not bool(relations.get("visible", false)) or colors.size() < 2:
			_fail("saved Z1 frame lacks two fully onscreen relation colours")
			return false
	return true

func _peace() -> bool:
	var peace: Dictionary = {}
	for row: Dictionary in _chronicle:
		if int(row["kind"]) == 21:
			peace = row
	if peace.is_empty():
		_fail("actual current-build chronicle has no kind21 at requested tick")
		return false
	_captured["reconciliation_entry"] = peace
	if not await _chronicle_capture():
		return false
	var info: Dictionary = ReadRoute.read(_world, "get_world_info")
	for lang: String in ["en", "ko"]:
		root.get_node("Locale").call("set_language", lang)
		var sentence: String = Chronicle.sentence(peace, -1, root.get_node("Locale"), info)
		var found: bool = false
		for image_row: Dictionary in _images:
			var context: Dictionary = image_row["page_context"]
			if context.get("language", "") == lang and sentence.length() > 0 and str(context.get("rendered_text", "")).contains(sentence):
				found = true
		if not found:
			_fail("kind21 sentence absent from saved chronicle pages for " + lang)
			return false
	return true

func _save(stem: String) -> bool:
	var head_output: Array = []
	if OS.execute("git", ["-C", ProjectSettings.globalize_path("res://"), "rev-parse", "HEAD"], head_output, true) != 0 or str(head_output[0]).strip_edges() != str(_provenance["head"]):
		_fail("HEAD changed during H10 capture")
		return false
	return await super._save(stem)

func _fail(message: String) -> void:
	push_error(message)
	if not _out.is_empty() and DirAccess.dir_exists_absolute(_out):
		var file := FileAccess.open(_out.path_join("failure.json"), FileAccess.WRITE)
		if file != null:
			file.store_string(JSON.stringify(_json_ready({"complete": false, "error": message,
				"provenance": _provenance, "images": _images, "reference_worlds": _references, "captures": _captured}), "\t"))
			file.close()
	quit(1)
