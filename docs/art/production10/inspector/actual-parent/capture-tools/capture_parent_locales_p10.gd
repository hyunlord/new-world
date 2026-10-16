extends SceneTree

const ReadRoute = preload("res://scripts/test/art/test_sim_reads_equivalence_p9.gd")
const Selection = preload("res://scripts/core/settlement_selection.gd")
const FixedDelta = preload("res://scripts/test/art/test_fixed_delta_p9.gd")
signal settled
var _delta_valid: bool = true
var _delta_samples: int = 0

func _init() -> void:
	call_deferred("_run")

func _process(delta: float) -> bool:
	_delta_samples += 1
	_delta_valid = _delta_valid and FixedDelta.accepts_delta(delta)
	return false

func _wait(frames: int) -> void:
	for index: int in frames:
		await process_frame
		call_deferred("_settle")
		await settled

func _settle() -> void:
	settled.emit()

func _fail(message: String) -> void:
	push_error("parent-p10: " + message)
	quit(1)

func _run() -> void:
	var output: String = OS.get_environment("P10_PARENT_OUT")
	if output.is_empty():
		_fail("P10_PARENT_OUT required; installed Part3 inspector required")
		return
	root.size = Vector2i(1920, 1080)
	var scene: PackedScene = load("res://scenes/main.tscn")
	var main: Node = scene.instantiate()
	var world: Node = main.get_node("WorldSim")
	world.call("set_sim_speed", 0.0)
	world.process_mode = Node.PROCESS_MODE_DISABLED
	root.add_child(main)
	current_scene = main
	var camera: Camera2D = main.get_node("Camera2D")
	camera.call("set_auto_track", false)
	camera.position_smoothing_enabled = false
	camera.process_mode = Node.PROCESS_MODE_DISABLED
	ReadRoute.resolve(world)
	ReadRoute.mutate(world, "reset_world", [20261010])
	var info: Dictionary = ReadRoute.read(world, "get_world_info")
	var ticks_per_day: int = int(info["ticks_per_day"])
	var target_tick: int = 29 * ticks_per_day + int(10.0 * ticks_per_day / 24.0) - int(info["clock_offset_ticks"])
	if int(ReadRoute.mutate(world, "advance_ticks", [target_tick])) != target_tick:
		_fail("exact day30 advance failed")
		return
	Selection.set_layers_enabled(self, true)
	await _wait(120)
	var snapshot: Dictionary = ReadRoute.read(world, "get_agent_snapshot")
	var live: Dictionary = {}
	for index: int in snapshot["agent_ids"].size():
		live[int(snapshot["agent_ids"][index])] = int(snapshot["ids"][index])
	var ids: Array = live.keys()
	ids.sort()
	var renderer: Node = main.get_node("AgentRenderer")
	var sprites: Dictionary = renderer.get("_sprites")
	var rings_installed: bool = false
	for property: Dictionary in renderer.get_property_list():
		if property["name"] == "_household_rings":
			rings_installed = renderer.get("_household_rings") != null
	var chosen: int = -1
	var family: Dictionary = {}
	var framing_center := Vector2.ZERO
	var census: Array[Dictionary] = []
	for agent_id: int in ids:
		var row: Dictionary = world.call("get_agent_family", live[agent_id])
		census.append({"agent_id": agent_id, "entity_bits": live[agent_id], "family": row})
		if chosen >= 0 or not bool(row.get("found", false)) or not live.has(int(row.get("partner", -1))):
			continue
		var sprite: Sprite2D = sprites.get(agent_id) as Sprite2D
		if sprite == null or not sprite.visible:
			continue
		var partner: Sprite2D = sprites.get(int(row.get("partner", -1))) as Sprite2D
		if rings_installed and (partner == null or not partner.visible):
			continue
		var bounds := Rect2(sprite.position, Vector2.ZERO)
		if partner != null and partner.visible:
			bounds = bounds.expand(partner.position)
		var visible_children: int = 0
		var living_children: int = 0
		for child_id: int in row.get("children", PackedInt64Array()):
			if live.has(child_id):
				living_children += 1
			var child: Sprite2D = sprites.get(child_id) as Sprite2D
			if live.has(child_id) and child != null and child.visible:
				visible_children += 1
				bounds = bounds.expand(child.position)
		if living_children > 0 and (not rings_installed or (visible_children > 0 and bounds.size.x <= 330.0 and bounds.size.y <= 230.0)):
			chosen = agent_id
			family = row
			framing_center = bounds.get_center() if rings_installed else sprite.position
	if chosen < 0:
		_fail("baseline has no visible living partnered parent; do not invent a family")
		return
	var inspector: Control = main.get_node("UI/AgentInspectorPanel")
	if inspector.get("_scroll") == null:
		_fail("Part3 scroll inspector is not installed")
		return
	var detail: Dictionary = world.call("get_agent_detail", live[chosen])
	detail["entity_bits"] = live[chosen]
	inspector.call("display_agent", detail)
	camera.zoom = Vector2(4, 4)
	# Keep selected parent left of the right-hand inspector.
	camera.position = framing_center + Vector2(45, 0)
	camera.reset_smoothing()
	camera.force_update_scroll()
	await _wait(30)
	if DirAccess.make_dir_recursive_absolute(output) != OK:
		_fail("cannot create output")
		return
	var locale: Node = root.get_node("Locale")
	var original_language: String = str(locale.get("_current_lang"))
	var captures: Array[Dictionary] = []
	var clicks: Array[Dictionary] = []
	var scroll: ScrollContainer = inspector.get("_scroll")
	for language: String in ["en", "ko"]:
		locale.call("set_language", language)
		inspector.call("display_agent", detail)
		await _wait(3)
		var family_buttons: Array = inspector.get("_family_buttons")
		for index: int in [0, 3]:
			var button: Button = family_buttons[index]
			var target_agent: int = int(button.get_meta("family_agent_id", -1))
			var allowed: bool = target_agent == int(family["partner"]) if index == 0 else target_agent in family["children"]
			if button.disabled or not allowed or not live.has(target_agent):
				_fail("actual partner/child button has invalid live identity")
				return
			button.pressed.emit()
			await _wait(2)
			var selected_bits: int = int(inspector.call("get_selected_entity_bits"))
			var clicked_detail: Dictionary = world.call("get_agent_detail", selected_bits)
			if selected_bits != int(live[target_agent]) or not bool(clicked_detail.get("found", false)) or int(clicked_detail.get("agent_id", -1)) != target_agent:
				_fail("family signal click did not select the verified live entity")
				return
			clicks.append({"locale": language, "button_index": index, "target_agent_id": target_agent, "selected_entity_bits": selected_bits, "detail": clicked_detail, "method": "actual_button_pressed_signal"})
			inspector.call("display_agent", detail)
			await _wait(2)
		for view: String in ["top", "temperament", "family"]:
			scroll.scroll_vertical = 0 if view == "top" else 100000
			if view == "temperament":
				var temperament_box: Control = inspector.get("_temperament_box")
				scroll.scroll_vertical = int(temperament_box.position.y)
			await _wait(3)
			RenderingServer.force_draw(false)
			var filename: String = "parent-" + language + "-" + view + ".png"
			var screenshot: Image = root.get_texture().get_image()
			if screenshot.get_size() != Vector2i(1920, 1080) or screenshot.save_png(output.path_join(filename)) != OK:
				_fail("cannot save full framebuffer screenshot")
				return
			var clock: Dictionary = ReadRoute.read(world, "get_time_snapshot")
			if str(locale.get("_current_lang")) != language or int(clock["tick"]) != target_tick or not inspector.visible or inspector.size.y > 620.0 or int(inspector.call("get_selected_entity_bits")) != int(live[chosen]):
				_fail("locale, frozen tick, parent selection, visibility or620px height invariant")
				return
			captures.append({"locale": language, "view": view, "file": filename, "frame": Engine.get_process_frames(), "tick": clock["tick"], "scroll_offset": scroll.scroll_vertical, "panel_height": inspector.size.y, "labels": _label_evidence(inspector), "rings": _ring_evidence(renderer)})
	locale.call("set_language", original_language)
	var time: Dictionary = ReadRoute.read(world, "get_time_snapshot")
	if not _delta_valid or _delta_samples == 0 or int(time["tick"]) != target_tick:
		_fail("fixed-fps or frozen tick invariant")
		return
	var file := FileAccess.open(output.path_join("parent.json"), FileAccess.WRITE)
	if file == null:
		_fail("cannot save metadata")
		return
	file.store_string(JSON.stringify(_json_ready({"seed": 20261010, "tick": target_tick, "display_day": 30, "hour": 10, "world_info": info, "fixed_fps": 60, "agent_id": chosen, "entity_bits": live[chosen], "family": family, "census": census, "captures": captures, "clicks": clicks, "delta_valid": _delta_valid, "delta_samples": _delta_samples, "temperament": world.call("get_agent_temperament", live[chosen]), "panel_height": inspector.size.y, "camera": [camera.position.x, camera.position.y], "zoom": 4, "optional_rings_installed": rings_installed, "optional_rings_status": "installed_pending_visual_review" if rings_installed else "dropped_absent"}), "\t") + "\n")
	quit(0)


# Godot serializes PackedByteArray as a string. Copy the evidence tree so
# normalized JSON arrays never mutate cached bridge snapshots or raw census.
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


func _label_evidence(node: Node) -> Array[Dictionary]:
	var rows: Array[Dictionary] = []
	if node is Label or node is Button:
		var control: Control = node as Control
		var rect: Rect2 = control.get_global_rect()
		rows.append({"path": str(node.get_path()), "text": str(node.get("text")), "visible_in_tree": control.is_visible_in_tree(), "rect": [rect.position.x, rect.position.y, rect.size.x, rect.size.y]})
	for child: Node in node.get_children():
		rows.append_array(_label_evidence(child))
	return rows

func _ring_evidence(renderer: Node) -> Array[Dictionary]:
	var rows: Array[Dictionary] = []
	for child: Node in renderer.get("_shadow_layer").get_children():
		if child.get_meta("art_kind", "") == "household_ring" and child.visible:
			rows.append({"agent_id": child.get_meta("art_id"), "position": [child.position.x, child.position.y], "alpha": child.default_color.a})
	return rows
