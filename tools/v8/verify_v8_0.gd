# tools/v8/verify_v8_0.gd — M1 Part 0: V8-0 on screen (checks S1–S8 of docs/v8/m1/M1_GOAL.md 5.1).
# Purpose: load res://scenes/main.tscn in a window, freeze the live loop, advance the world with the
#   bridge's tooling call and check, at fixed ticks, that the V7 overlays and god hand are gone (S1, S2),
#   that the world equals the V7 game's (S3: the values the top bar shows and every agent's id and tile),
#   that a synthetic click on a person opens the inspector with a name, age and village line equal to the
#   bridge reads (S4), that a click on an empty tile opens the causal panel (S5) and that C shows the
#   chronicle (S6); then 120 s of the live loop at 3x speed for the mean FPS (S8). S7 (console errors) is
#   counted by the caller from this run's log. The same script runs unchanged against the V7 game at
#   v7-final-art (copied untracked into the base worktree; the scene's node paths are the same).
# Tooling only: the game never runs this script.
# Bridge methods called: advance_ticks (tooling), set_sim_speed, get_time_snapshot, get_agent_snapshot,
#   get_building_snapshot, get_settlement_snapshot, get_society_snapshot, get_agent_detail,
#   get_tile_causal_history.
# usage: Godot --path . --rendering-driver opengl3 --resolution 1920x1080 \
#          --script res://tools/v8/verify_v8_0.gd -- --out <abs dir> --label v8|v7 [--fps-seconds 120]
extends SceneTree

const ORIGIN: Vector2 = Vector2(448, 28)
const TILE_SIZE: float = 16.0
const DELETED_LIST: String = "res://tools/v8/v7_ui_deleted.txt"

var _out: String = ""
var _label: String = "v8"
var _fps_seconds: float = 120.0
var _deleted_inline: PackedStringArray = PackedStringArray()
var _result: Dictionary = {}


func _init() -> void:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	for i: int in args.size():
		if args[i] == "--out" and i + 1 < args.size():
			_out = args[i + 1]
		elif args[i] == "--label" and i + 1 < args.size():
			_label = args[i + 1]
		elif args[i] == "--fps-seconds" and i + 1 < args.size():
			_fps_seconds = float(args[i + 1])
		elif args[i] == "--deleted" and i + 1 < args.size():
			_deleted_inline = FileAccess.get_file_as_string(args[i + 1]).split("\n", false)
	call_deferred("_run")


func _run() -> void:
	if _out.is_empty():
		push_error("verify: --out is required")
		quit(2)
		return
	DirAccess.make_dir_recursive_absolute(_out)
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(main)
	for _i: int in 20:
		await process_frame
	var world: Node = main.get_node_or_null("WorldSim")
	var camera: Camera2D = main.get_node_or_null("Camera2D") as Camera2D
	if world == null or camera == null:
		push_error("verify: missing WorldSim or Camera2D")
		quit(1)
		return
	world.set("process_mode", Node.PROCESS_MODE_DISABLED)
	_result["label"] = _label
	_result["start_tick"] = _tick(world)
	_scan_tree(main)
	for target: int in [1440, 4320]:
		var now: int = _tick(world)
		if target > now:
			world.call("advance_ticks", target - now)
		for _i: int in 10:
			await process_frame
		_result["world_t%d" % target] = _world_values(world)
		await _shot("hud_t%d" % target)
		if target == 1440:
			await _person_click(main, world, camera)
			await _tile_click(main, world, camera)
	await _chronicle(main)
	await _fps(main, world)
	_scan_tree(main, "_end")
	var file: FileAccess = FileAccess.open(_out.path_join("verify_%s.json" % _label), FileAccess.WRITE)
	file.store_string(JSON.stringify(_result, "  ", true))
	file.close()
	print("VERIFY_DONE ", _label)
	quit(0)


func _tick(world: Node) -> int:
	return int((world.call("get_time_snapshot") as Dictionary).get("tick", 0))


# S1 / S2: every node's script path and scene path against the deleted V7 list.
func _scan_tree(main: Node, suffix: String = "") -> void:
	var deleted: PackedStringArray = _deleted_inline
	if deleted.is_empty():
		deleted = FileAccess.get_file_as_string(DELETED_LIST).split("\n", false)
	var paths: Array[String] = []
	for line: String in deleted:
		if not line.begins_with("#") and not line.strip_edges().is_empty():
			paths.append("res://" + line.strip_edges())
	var god_hits: Array[String] = []
	var overlay_hits: Array[String] = []
	var nodes: int = 0
	var stack: Array[Node] = [main]
	while not stack.is_empty():
		var node: Node = stack.pop_back()
		nodes += 1
		var found: Array[String] = []
		var script: Script = node.get_script() as Script
		if script != null:
			found.append(script.resource_path)
		if not node.scene_file_path.is_empty():
			found.append(node.scene_file_path)
		for path: String in found:
			var hit: bool = path.contains("god_hand") or paths.has(path)
			if not hit:
				continue
			var entry: String = "%s (%s)" % [str(main.get_path_to(node)), path]
			if path.contains("god_hand"):
				god_hits.append(entry)
			else:
				overlay_hits.append(entry)
		for child: Node in node.get_children():
			stack.append(child)
	_result["s1_god_hand_nodes" + suffix] = god_hits
	_result["s2_overlay_nodes" + suffix] = overlay_hits
	_result["scanned_nodes" + suffix] = nodes
	_result["deleted_list_size"] = paths.size()


# S3: the values the top bar shows, computed from the bridge as the top bar does, plus ids and tiles.
func _world_values(world: Node) -> Dictionary:
	var agents: Dictionary = world.call("get_agent_snapshot")
	var ids: PackedInt64Array = agents.get("ids", PackedInt64Array())
	var xs: PackedInt32Array = agents.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = agents.get("ys", PackedInt32Array())
	var rows: Array[String] = []
	for i: int in ids.size():
		rows.append("%d:%d,%d" % [ids[i], xs[i], ys[i]])
	var shelters: int = 0
	var kinds: PackedByteArray = (world.call("get_building_snapshot") as Dictionary).get("kinds", PackedByteArray())
	for kind: int in kinds:
		if kind == 1:
			shelters += 1
	var settlement: Dictionary = world.call("get_settlement_snapshot")
	var food: int = 0
	for amount: int in settlement.get("food_stocks", PackedInt32Array()):
		food += amount
	var wood: int = 0
	for amount: int in settlement.get("wood_stocks", PackedInt32Array()):
		wood += amount
	var clock: Dictionary = world.call("get_time_snapshot")
	var minutes: int = clampi(roundi(float(clock.get("time_of_day", 0.0)) * 60.0), 0, 1439)
	return {
		"tick": int(clock.get("tick", 0)),
		"population": ids.size(),
		"shelters": shelters,
		"food": food,
		"wood": wood,
		"day": int(clock.get("day_index", 0)) + 1,
		"time": "%02d:%02d" % [minutes / 60, minutes % 60],
		"agents_hash": str(hash(rows)),
		"agents": rows,
	}


func _focus(camera: Camera2D, tile: Vector2) -> void:
	camera.position = ORIGIN + tile * TILE_SIZE + Vector2(8, 8)
	camera.zoom = Vector2.ONE * 4.0
	camera.reset_smoothing()
	camera.force_update_scroll()


func _click(world_pos: Vector2) -> void:
	var screen: Vector2 = root.get_canvas_transform() * world_pos
	for pressed: bool in [true, false]:
		var event: InputEventMouseButton = InputEventMouseButton.new()
		event.button_index = MOUSE_BUTTON_LEFT
		event.pressed = pressed
		event.position = screen
		event.global_position = screen
		Input.parse_input_event(event)
		await process_frame


# S4: a synthetic click on the first agent's drawn sprite opens the inspector.
func _person_click(main: Node, world: Node, camera: Camera2D) -> void:
	var snap: Dictionary = world.call("get_agent_snapshot")
	var bits: PackedInt64Array = snap.get("ids", PackedInt64Array())
	var agent_ids: PackedInt64Array = snap.get("agent_ids", PackedInt64Array())
	var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
	var ages: PackedInt32Array = snap.get("ages_days", PackedInt32Array())
	var check: Dictionary = {"opened": false}
	if bits.is_empty():
		_result["s4"] = check
		return
	var tile: Vector2 = Vector2(xs[0], ys[0])
	_focus(camera, tile)
	for _i: int in 20:
		await process_frame
	var agents: Node = main.get_node_or_null("AgentRenderer")
	var world_pos: Vector2 = ORIGIN + tile * TILE_SIZE + Vector2(8, 4)
	if agents != null and agents.has_method("drawn_entities_at"):
		# Aim at the drawn sprite when the renderer exposes its rectangles.
		for dy: int in range(-24, 9, 2):
			var probe: Vector2 = ORIGIN + tile * TILE_SIZE + Vector2(8, 8 + dy)
			var hits: PackedInt64Array = agents.call("drawn_entities_at", probe)
			if not hits.is_empty() and hits[0] == bits[0]:
				world_pos = probe
				break
	await _click(world_pos)
	for _i: int in 20:
		await process_frame
	var panel: Node = main.get_node_or_null("UI/AgentInspectorPanel")
	var detail: Dictionary = world.call("get_agent_detail", panel.call("get_selected_entity_bits") if panel != null else -1)
	var agent_id: int = int(detail.get("agent_id", -1))
	var age: int = -1
	for i: int in agent_ids.size():
		if agent_ids[i] == agent_id and i < ages.size():
			age = ages[i]
	var society: Dictionary = world.call("get_society_snapshot")
	var home: int = -1
	var homes_a: PackedInt64Array = society.get("home_agent_ids", PackedInt64Array())
	var homes_s: PackedInt32Array = society.get("home_settlement_ids", PackedInt32Array())
	for i: int in mini(homes_a.size(), homes_s.size()):
		if homes_a[i] == agent_id:
			home = homes_s[i]
	var selection: GDScript = load("res://scripts/core/settlement_selection.gd") as GDScript
	var expected_village: String = ""
	if home >= 0 and selection != null:
		expected_village = str(selection.call("name_for", home, root.get_node_or_null("Locale")))
	if panel != null:
		var id_label: Label = panel.get("_label_id") as Label
		var age_label: Label = panel.get("_label_age") as Label
		var village_label: Label = panel.get("_village_label") as Label
		check = {
			"opened": (panel as Control).visible,
			"clicked_entity": bits[0],
			"selected_entity": int(panel.call("get_selected_entity_bits")),
			"agent_id": agent_id,
			"name_line": id_label.text if id_label != null else "",
			"age_line": age_label.text if age_label != null else "",
			"village_line": village_label.text if village_label != null else "",
			"bridge_age_days": age,
			"bridge_home": home,
			"expected_village_line": expected_village,
		}
		check["name_ok"] = not str(check["name_line"]).is_empty() and str(check["name_line"]).contains(str(agent_id))
		check["age_ok"] = age >= 0 and str(check["age_line"]).contains(str(age))
		var shown: String = str(check["village_line"]).replace(char(0x2060), "")
		check["village_ok"] = not expected_village.is_empty() and shown == expected_village.replace(char(0x2060), "")
	_result["s4"] = check
	await _shot("s4_inspector")
	if panel != null and panel.has_method("close"):
		panel.call("close")


# S5: a synthetic click on an empty tile next to the village opens the causal panel with that tile's events.
func _tile_click(main: Node, world: Node, camera: Camera2D) -> void:
	var snap: Dictionary = world.call("get_settlement_snapshot")
	var fx: PackedInt32Array = snap.get("formation_xs", PackedInt32Array())
	var fy: PackedInt32Array = snap.get("formation_ys", PackedInt32Array())
	var village: Vector2 = Vector2(fx[0], fy[0]) if not fx.is_empty() else Vector2(32, 32)
	var agents: Dictionary = world.call("get_agent_snapshot")
	var occupied: Dictionary = {}
	var xs: PackedInt32Array = agents.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = agents.get("ys", PackedInt32Array())
	for i: int in xs.size():
		for dx: int in range(-1, 2):
			for dy: int in range(-1, 2):
				occupied[Vector2i(xs[i] + dx, ys[i] + dy)] = true
	var tile: Vector2i = Vector2i(int(village.x), int(village.y))
	for ring: int in range(1, 12):
		var found: bool = false
		for dy: int in range(-ring, ring + 1):
			for dx: int in range(-ring, ring + 1):
				var t: Vector2i = Vector2i(int(village.x) + dx, int(village.y) + dy)
				if not occupied.has(t) and not (world.call("get_tile_causal_history", t.x, t.y) as Array).is_empty():
					tile = t
					found = true
					break
			if found:
				break
		if found:
			break
	_focus(camera, Vector2(tile))
	for _i: int in 20:
		await process_frame
	await _click(ORIGIN + Vector2(tile) * TILE_SIZE + Vector2(8, 8))
	for _i: int in 20:
		await process_frame
	var panel: Node = main.get_node_or_null("UI/CausalPanel")
	var expected: int = (world.call("get_tile_causal_history", tile.x, tile.y) as Array).size()
	var rows: int = -1
	var title: String = ""
	if panel != null:
		var box: Node = panel.get("_history_container") as Node
		rows = box.get_child_count() if box != null else -1
		var title_label: Label = panel.get("_title_label") as Label
		title = title_label.text if title_label != null else ""
	var opened: bool = false
	if panel != null:
		opened = bool(panel.call("is_panel_visible")) if panel.has_method("is_panel_visible") else (panel as Control).visible
	_result["s5"] = {
		"tile": [tile.x, tile.y],
		"opened": opened,
		"bridge_events": expected,
		"panel_rows": rows,
		"title": title,
	}
	await _shot("s5_causal")


# S6: the C key shows the chronicle with at least one entry.
func _chronicle(main: Node) -> void:
	var event: InputEventKey = InputEventKey.new()
	event.keycode = KEY_C
	event.physical_keycode = KEY_C
	event.pressed = true
	Input.parse_input_event(event)
	await process_frame
	var release: InputEventKey = event.duplicate() as InputEventKey
	release.pressed = false
	Input.parse_input_event(release)
	for _i: int in 30:
		await process_frame
	var panel: Node = main.get_node_or_null("UI/ChroniclePanel")
	var entries: Array = panel.get("_entries") as Array if panel != null else []
	_result["s6"] = {
		"visible": panel != null and (panel as Control).visible,
		"entries": entries.size(),
		"total": int(panel.get("_total")) if panel != null else -1,
	}
	await _shot("s6_chronicle")
	if panel != null and (panel as Control).visible and panel.has_method("toggle_visible"):
		panel.call("toggle_visible")


# S8: the live loop at 3x for _fps_seconds; mean FPS = frames / wall seconds.
func _fps(main: Node, world: Node) -> void:
	var camera: Node = main.get_node_or_null("Camera2D")
	if camera != null and camera.has_method("_on_world_reset"):
		camera.call("_on_world_reset")
	world.call("set_sim_speed", 3.0)
	world.set("process_mode", Node.PROCESS_MODE_INHERIT)
	for _i: int in 30:
		await process_frame
	var start_tick: int = _tick(world)
	var started: int = Time.get_ticks_usec()
	var frames: int = 0
	var samples: Array[float] = []
	var next_sample: int = started + 1000000
	while Time.get_ticks_usec() - started < int(_fps_seconds * 1000000.0):
		await process_frame
		frames += 1
		if Time.get_ticks_usec() >= next_sample:
			samples.append(Engine.get_frames_per_second())
			next_sample += 1000000
	var seconds: float = float(Time.get_ticks_usec() - started) / 1000000.0
	world.set("process_mode", Node.PROCESS_MODE_DISABLED)
	_result["s8"] = {
		"seconds": seconds,
		"frames": frames,
		"mean_fps": float(frames) / seconds,
		"min_sampled_fps": samples.min() if not samples.is_empty() else 0.0,
		"ticks": _tick(world) - start_tick,
		"speed": 3.0,
	}
	await _shot("s8_end")


func _shot(name: String) -> void:
	for _i: int in 5:
		await process_frame
	RenderingServer.force_draw(false)
	await process_frame
	var image: Image = root.get_texture().get_image()
	var path: String = _out.path_join("%s_%s.png" % [_label, name])
	if image == null or image.save_png(path) != OK:
		push_error("verify: could not save " + path)
	else:
		print("verify: saved ", path)
