extends SceneTree

var _main: Node
var _world: Node
var _camera: Camera2D
var _out: String = ".harness/evidence/art-production3/p4"
var _records: Array[Dictionary] = []
var _seen_reasons: Dictionary = {}
var _failed: bool = false


func _init() -> void:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	if args.size() == 2 and args[0] == "--out":
		_out = args[1]
	elif not args.is_empty():
		quit(2)
		return
	call_deferred("_run")


func _run() -> void:
	if DisplayServer.get_name() == "headless":
		push_error("Age consumer evidence requires a real window")
		quit(2)
		return
	if DirAccess.make_dir_recursive_absolute(ProjectSettings.globalize_path(_out)) != OK:
		quit(2)
		return
	var packed: PackedScene = load("res://scenes/main.tscn") as PackedScene
	_main = packed.instantiate()
	_world = _main.get_node("WorldSim")
	_camera = _main.get_node("Camera2D") as Camera2D
	_world.call("set_sim_speed", 0.0)
	_world.process_mode = Node.PROCESS_MODE_DISABLED
	root.add_child(_main)
	current_scene = _main
	_camera.call("set_auto_track", false)
	_camera.position_smoothing_enabled = false
	var info: Dictionary = _world.call("get_world_info")
	var offset: int = int(info.get("clock_offset_ticks", 0))
	if int(info.get("ticks_per_day", 0)) != 1440:
		quit(2)
		return
	for day: int in [2, 10, 18, 24, 40]:
		var target: int = (day - 1) * 1440 + 720 - offset
		var clock: Dictionary = _world.call("get_time_snapshot")
		var tick: int = int(clock.get("tick", -1))
		while tick < target:
			var previous: int = tick
			tick = int(_world.call("advance_ticks", mini(8, target - tick)))
			if tick <= previous:
				push_error("Natural tick advancement stalled")
				quit(1)
				return
			var deaths: Dictionary = _world.call("get_recent_deaths")
			var reasons: PackedInt32Array = deaths.get("reasons", PackedInt32Array())
			for index: int in reasons.size():
				var reason: int = reasons[index]
				if not _seen_reasons.has(reason):
					_seen_reasons[reason] = true
					await _capture("first_death_reason_%d" % reason, index)
			if tick % 1440 < 8:
				print("AGE_PROBE tick=", tick)
				await process_frame
		await _capture("d%d_noon_inspector" % day, -1)
	var output: FileAccess = FileAccess.open(_out.path_join("consumers.json"), FileAccess.WRITE)
	if output == null:
		quit(1)
		return
	output.store_string(JSON.stringify({"schema_version": 1, "simulation_injected": false, "advance_method": "advance_ticks", "captures": _records, "observed_death_reasons": _seen_reasons.keys()}, "\t") + "\n")
	print("AGE_PROBE captures=", _records.size(), " observed_reasons=", _seen_reasons.keys())
	quit(1 if _failed else 0)


func _capture(stem: String, death_index: int) -> void:
	var inspector: Node = _main.get_node("UI/AgentInspectorPanel")
	inspector.call("close")
	var agents: Dictionary = _world.call("get_agent_snapshot")
	var ids: PackedInt64Array = agents.get("ids", PackedInt64Array())
	var agent_ids: PackedInt64Array = agents.get("agent_ids", PackedInt64Array())
	var stages: PackedByteArray = agents.get("life_stages", PackedByteArray())
	var ages: Variant = agents.get("ages_days", [])
	var selected: int = -1
	for index: int in ids.size():
		if selected < 0 or (index < stages.size() and int(stages[index]) == 2):
			selected = index
			if index < stages.size() and int(stages[index]) == 2:
				break
	var tile: Vector2 = Vector2(32, 32)
	var selected_record: Dictionary = {}
	if selected >= 0:
		tile = Vector2(agents["xs"][selected], agents["ys"][selected])
		selected_record = {"entity_bits": ids[selected], "agent_id": agent_ids[selected], "life_stage": int(stages[selected]) if selected < stages.size() else -1, "age_days": int(ages[selected]) if selected < ages.size() else -1}
		if death_index < 0:
			var detail: Dictionary = _world.call("get_agent_detail", ids[selected])
			detail["entity_bits"] = ids[selected]
			inspector.call("display_agent", detail)
	var deaths: Dictionary = _world.call("get_recent_deaths")
	if death_index >= 0:
		tile = Vector2(deaths["xs"][death_index], deaths["ys"][death_index])
	_camera.position = Vector2(448, 28) + tile * 16.0 + Vector2(8, 8)
	_camera.zoom = Vector2(4, 4)
	_camera.reset_smoothing()
	_camera.force_update_scroll()
	for node_name: String in ["TerrainRenderer", "BuildingRenderer", "AgentRenderer"]:
		var renderer: Node = _main.get_node(node_name)
		if renderer.has_method("refresh_snapshot"):
			renderer.call("refresh_snapshot")
	await create_timer(1.2).timeout
	await RenderingServer.frame_post_draw
	var image: Image = root.get_texture().get_image()
	if image.get_width() < 1920 or image.get_height() < 1080:
		_failed = true
		return
	if image.save_png(ProjectSettings.globalize_path(_out.path_join(stem + ".png"))) != OK:
		_failed = true
	var renderer: Node = _main.get_node("AgentRenderer")
	var sprites: Dictionary = renderer.get("_sprites")
	var looks: Array[Dictionary] = []
	for index: int in agent_ids.size():
		var sprite: Sprite2D = sprites.get(int(agent_ids[index])) as Sprite2D
		if sprite != null:
			looks.append({"agent_id": agent_ids[index], "stage": int(stages[index]) if index < stages.size() else -1, "age_days": int(ages[index]) if index < ages.size() else -1, "variant": sprite.get_meta("variant", {}), "modulate": sprite.modulate.to_html(), "sprite_id": sprite.get_meta("sprite_id", "")})
	var topbar: Node = _main.get_node("UI/HudTopbar")
	var status: Node = _main.get_node("UI/HudStatusPanel")
	var death_viz: Node = _main.get_node("DeathVizRenderer")
	var reason_colors: Array[Dictionary] = []
	var visible_reasons: PackedInt32Array = death_viz.get("_reasons")
	for reason: int in visible_reasons:
		var color: Color = death_viz.call("_reason_color", reason)
		reason_colors.append({"reason": reason, "rgb": [color.r, color.g, color.b]})
	var terrain: Node = _main.get_node("TerrainRenderer")
	var terrain_layers: Dictionary = terrain.get("_layers")
	var terrain_tints: Dictionary = {}
	for key: Variant in terrain_layers:
		var layer: CanvasItem = terrain_layers[key] as CanvasItem
		terrain_tints[str(key)] = layer.modulate.to_html()
	_records.append({"file": stem + ".png", "time_snapshot": _world.call("get_time_snapshot"), "tallies": _world.call("get_tallies"), "selected": selected_record, "inspector_labels": _labels(inspector), "hud_labels": _labels(topbar), "news_labels": _labels(status), "news_entries": (status.get("_entries") as Array).duplicate(true), "death_snapshot": deaths, "death_renderer_colors": reason_colors, "agents": looks, "terrain_tints": terrain_tints, "world_object_tints": _object_tints()})


func _labels(node: Node) -> Array[String]:
	var result: Array[String] = []
	if node is Label and (node as Label).is_visible_in_tree():
		result.append((node as Label).text)
	for child: Node in node.get_children():
		result.append_array(_labels(child))
	return result


func _object_tints() -> Array[Dictionary]:
	var result: Array[Dictionary] = []
	for child: Node in _main.get_node("WorldObjects").get_children():
		if child is Sprite2D and child.has_meta("art_kind"):
			var sprite: Sprite2D = child as Sprite2D
			result.append({"kind": str(sprite.get_meta("art_kind")), "modulate": sprite.modulate.to_html()})
	return result
