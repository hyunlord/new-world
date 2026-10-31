# tools/v8/read_diag_capture.gd — M-READ 30-second diagnosis test capture script.
# Purpose: for each of the six owner questions (Q1–Q6), produce at most 4 KO UI-scale-1.0 images:
#   the problem-visible screen, then after at most 2 clicks (subject, then "왜?" or plaque).
#   Works on BASE (where plaque/why-not do not exist — captures what exists) and on M-READ.
# Bridge methods: reset_world, advance_ticks (tooling); get_agent_snapshot, get_time_snapshot,
#   get_building_snapshot, get_settlement_snapshot, get_chronicle_since, get_agent_mind,
#   get_ui_facts, get_agent_detail (direct); via SimReads.
# usage: $GODOT_BIN --headless --path . --rendering-driver opengl3 --resolution 1920x1080 \
#          --script res://tools/v8/read_diag_capture.gd -- --out <abs dir> [--question 1]
extends SceneTree

const TILE: float = 64.0

var _out: String = ""
var _question: int = 0  # 0 = all


func _init() -> void:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	for i: int in args.size():
		if args[i] == "--out" and i + 1 < args.size():
			_out = args[i + 1]
		elif args[i] == "--question" and i + 1 < args.size():
			_question = int(args[i + 1])
	call_deferred("_run")


func _run() -> void:
	if _out.is_empty():
		push_error("read_diag_capture: --out is required")
		quit(2)
		return
	DirAccess.make_dir_recursive_absolute(_out)
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(main)
	for _i: int in 10:
		await process_frame
	var world: Node = main.get_node_or_null("WorldSim")
	var camera: Camera2D = main.get_node_or_null("Camera2D") as Camera2D
	if world == null or camera == null:
		push_error("read_diag_capture: missing WorldSim or Camera2D")
		quit(1)
		return
	world.set("process_mode", Node.PROCESS_MODE_DISABLED)
	var reads: Node = main.get_node_or_null("SimReads")
	var tool_node: Node = main.get_node_or_null("ColonyView/OrderTool")
	if reads == null:
		push_error("read_diag_capture: missing SimReads")
		quit(1)
		return
	var questions: Array = [1, 2, 3, 4, 5, 6] if _question == 0 else [_question]
	var results: Dictionary = {}
	for q: int in questions:
		var result: Dictionary = await _run_question(q, main, world, reads, camera, tool_node)
		results["Q%d" % q] = result
		print("Q%d: %s" % [q, JSON.stringify(result)])
	# Write the manifest.
	var manifest_path: String = _out.path_join("manifest.json")
	var file: FileAccess = FileAccess.open(manifest_path, FileAccess.WRITE)
	if file != null:
		file.store_string(JSON.stringify(results, "\t"))
		file.close()
		print("read_diag_capture: manifest at ", manifest_path)
	print("READ_DIAG_CAPTURE_DONE")
	quit(0)


func _run_question(q: int, main: Node, world: Node, reads: Node, camera: Camera2D, tool_node: Node) -> Dictionary:
	var result: Dictionary = {"question": q, "images": []}
	# Each question: reset world, advance, capture.
	match q:
		1: result = await _q1_starvation(main, world, reads, camera, tool_node)
		2: result = await _q2_not_working(main, world, reads, camera, tool_node)
		3: result = await _q3_cold(main, world, reads, camera, tool_node)
		4: result = await _q4_stockpile(main, world, reads, camera, tool_node)
		5: result = await _q5_what_to_do(main, world, reads, camera, tool_node)
		6: result = await _q6_winter_prep(main, world, reads, camera, tool_node)
	return result


# Q1: 왜 이 사람이 굶어 죽었나
func _q1_starvation(main: Node, world: Node, reads: Node, camera: Camera2D, tool_node: Node) -> Dictionary:
	var result: Dictionary = {"question": 1, "images": [], "seed": -1, "tick": -1}
	# Seed 1 first (design doc), then seed 3. Advance day by day checking for starvation.
	for try_seed: int in [1, 3, 0]:
		reads.call("mutate_raw", "reset_world", [try_seed])
		for _i: int in 10:
			await process_frame
		# Jump to day 8 quickly, then check daily.
		world.call("advance_ticks", 11520)
		for _day: int in range(9, 19):
			world.call("advance_ticks", 1440)
			var chronicle: Dictionary = world.call("get_chronicle_since", 0, 512)
			var kinds: PackedInt32Array = chronicle.get("kind", PackedInt32Array())
			var causes: PackedInt64Array = chronicle.get("cause", PackedInt64Array())
			var found: bool = false
			for i: int in mini(kinds.size(), causes.size()):
				if kinds[i] == 0 and causes[i] == 0:
					found = true
					break
			if found:
				result["seed"] = try_seed
				result["tick"] = _now(world)
				break
		if result["seed"] >= 0:
			break
	if result["seed"] < 0:
		# Fallback: capture the hungriest person in winter (seed 1, day 12).
		reads.call("mutate_raw", "reset_world", [1])
		for _i: int in 10:
			await process_frame
		world.call("advance_ticks", 17280)
		result["seed"] = 1
		result["tick"] = _now(world)
		result["note"] = "no starvation death; showing hungriest person instead"
	var camp: Vector2i = _hearth(world)
	_frame(camera, Vector2(camp), 2.0)
	for _i: int in 30:
		await process_frame
	await _save_q("q1_overview", result)
	# Click the hungriest person (lowest hunger value).
	if tool_node != null:
		var snap: Dictionary = world.call("get_agent_snapshot")
		var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
		var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
		var hungers: PackedFloat32Array = snap.get("hungers", PackedFloat32Array())
		var target_idx: int = 0
		var lowest: float = 999.0
		for i: int in mini(xs.size(), hungers.size()):
			if hungers[i] < lowest:
				lowest = hungers[i]
				target_idx = i
		if not xs.is_empty():
			var tile: Vector2i = Vector2i(xs[target_idx], ys[target_idx])
			tool_node.call("click_world", Vector2(tile) * TILE + Vector2(8, 8), tile)
			for _i: int in 30:
				await process_frame
			await _save_q("q1_inspector", result)
			var inspector: Node = main.get_node_or_null("UI/AgentInspectorPanel")
			if inspector != null and inspector.has_method("_open_tile_why"):
				inspector.call("_open_tile_why")
				for _i2: int in 30:
					await process_frame
				await _save_q("q1_why", result)
	return result


# Q2: 왜 이 어른이 일을 안 하나
func _q2_not_working(main: Node, world: Node, reads: Node, camera: Camera2D, tool_node: Node) -> Dictionary:
	var result: Dictionary = {"question": 2, "images": [], "seed": 0, "tick": -1}
	reads.call("mutate_raw", "reset_world", [0])
	for _i: int in 10:
		await process_frame
	world.call("advance_ticks", 7200)  # day 5
	result["tick"] = _now(world)
	var camp: Vector2i = _hearth(world)
	# Capture 1: camp overview.
	_frame(camera, Vector2(camp), 2.0)
	for _i: int in 30:
		await process_frame
	await _save_q("q2_overview", result)
	# Find a non-working adult and click them.
	if tool_node != null:
		var snap: Dictionary = world.call("get_agent_snapshot")
		var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
		var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
		var jobs: PackedByteArray = snap.get("jobs", PackedByteArray())
		var target_idx: int = -1
		for i: int in mini(xs.size(), jobs.size()):
			if jobs[i] == 0 or jobs[i] == 7:  # idle or eating
				target_idx = i
				break
		if target_idx < 0 and not xs.is_empty():
			target_idx = 0
		if target_idx >= 0:
			var tile: Vector2i = Vector2i(xs[target_idx], ys[target_idx])
			tool_node.call("click_world", Vector2(tile) * TILE + Vector2(8, 8), tile)
			for _i: int in 30:
				await process_frame
			await _save_q("q2_inspector", result)
			# Capture 3: click "왜?" button.
			var inspector: Node = main.get_node_or_null("UI/AgentInspectorPanel")
			if inspector != null and inspector.has_method("_open_tile_why"):
				inspector.call("_open_tile_why")
				for _i2: int in 30:
					await process_frame
				await _save_q("q2_why", result)
	return result


# Q3: 왜 이 사람이 춥나 / 왜 화덕이 꺼졌나
# Must be a real cold scene: winter with hearth out or no firewood, NOT a flee scene.
func _q3_cold(main: Node, world: Node, reads: Node, camera: Camera2D, tool_node: Node) -> Dictionary:
	var result: Dictionary = {"question": 3, "images": [], "seed": -1, "tick": -1}
	# 6 days/season: winter = days 18-23. Advance to day 17, check daily through day 23.
	reads.call("mutate_raw", "reset_world", [0])
	for _i: int in 10:
		await process_frame
	world.call("advance_ticks", 24480)  # jump to day 17
	for _day: int in range(18, 24):
		world.call("advance_ticks", 1440)
		var snap: Dictionary = world.call("get_agent_snapshot")
		var cold_flags: PackedByteArray = snap.get("colds", PackedByteArray())
		var fleeing: PackedByteArray = snap.get("fleeing", PackedByteArray())
		var cold_idx: int = -1
		for i: int in mini(cold_flags.size(), fleeing.size()):
			if cold_flags[i] != 0 and fleeing[i] == 0:
				cold_idx = i
				break
		if cold_idx < 0:
			continue
		var settlements: Dictionary = SimReads.read(world, "get_settlement_snapshot")
		var fire_lit: PackedByteArray = settlements.get("fire_lit", PackedByteArray())
		var wood_stocks: PackedInt32Array = settlements.get("wood_stocks", PackedInt32Array())
		var hearth_out: bool = false
		for fi: int in fire_lit.size():
			if fire_lit[fi] == 0:
				hearth_out = true
				break
		var no_firewood: bool = false
		for wi: int in wood_stocks.size():
			if wood_stocks[wi] == 0:
				no_firewood = true
				break
		if not hearth_out and not no_firewood:
			continue
		result["seed"] = 0
		result["tick"] = _now(world)
		result["hearth_out"] = hearth_out
		result["no_firewood"] = no_firewood
		break
	if result["seed"] < 0:
		# Fallback: capture mid-winter scene (cold from being outdoors, no hearth-out required).
		reads.call("mutate_raw", "reset_world", [0])
		for _i: int in 10:
			await process_frame
		world.call("advance_ticks", 28800)  # day 20, mid-winter
		result["seed"] = 0
		result["tick"] = _now(world)
		result["note"] = "no hearth-out scene; showing winter mid-point"
	var camp: Vector2i = _hearth(world)
	_frame(camera, Vector2(camp), 2.0)
	for _i: int in 30:
		await process_frame
	await _save_q("q3_overview", result)
	# Click a cold (non-fleeing) person; fall back to any person if none is cold.
	if tool_node != null:
		var snap: Dictionary = world.call("get_agent_snapshot")
		var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
		var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
		var cold_flags: PackedByteArray = snap.get("colds", PackedByteArray())
		var fleeing: PackedByteArray = snap.get("fleeing", PackedByteArray())
		var target_idx: int = -1
		for i: int in mini(xs.size(), mini(cold_flags.size(), fleeing.size())):
			if cold_flags[i] != 0 and fleeing[i] == 0:
				target_idx = i
				break
		if target_idx < 0 and not xs.is_empty():
			target_idx = 0
		if target_idx >= 0:
			var tile: Vector2i = Vector2i(xs[target_idx], ys[target_idx])
			tool_node.call("click_world", Vector2(tile) * TILE + Vector2(8, 8), tile)
			for _i: int in 30:
				await process_frame
			await _save_q("q3_inspector", result)
			var inspector: Node = main.get_node_or_null("UI/AgentInspectorPanel")
			if inspector != null and inspector.has_method("_open_tile_why"):
				inspector.call("_open_tile_why")
				for _i2: int in 30:
					await process_frame
				await _save_q("q3_why", result)
	return result


# Q4: 왜 비축 구역에 물건이 안 오나
func _q4_stockpile(main: Node, world: Node, reads: Node, camera: Camera2D, tool_node: Node) -> Dictionary:
	var result: Dictionary = {"question": 4, "images": [], "seed": 0, "tick": -1}
	reads.call("mutate_raw", "reset_world", [0])
	for _i: int in 10:
		await process_frame
	world.call("advance_ticks", 4320)  # day 3
	result["tick"] = _now(world)
	var camp: Vector2i = _hearth(world)
	_frame(camera, Vector2(camp), 2.0)
	for _i: int in 30:
		await process_frame
	await _save_q("q4_overview", result)
	# Click a person who is not hauling.
	if tool_node != null:
		var snap: Dictionary = world.call("get_agent_snapshot")
		var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
		var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
		if not xs.is_empty():
			var tile: Vector2i = Vector2i(xs[0], ys[0])
			tool_node.call("click_world", Vector2(tile) * TILE + Vector2(8, 8), tile)
			for _i: int in 30:
				await process_frame
			await _save_q("q4_inspector", result)
			var inspector: Node = main.get_node_or_null("UI/AgentInspectorPanel")
			if inspector != null and inspector.has_method("_open_tile_why"):
				inspector.call("_open_tile_why")
				for _i2: int in 30:
					await process_frame
				await _save_q("q4_why", result)
	return result


# Q5: 지금 내가 뭘 해야 하나
# M-READ: capture the "지금 할 일" plaque (TaskPlaque) which is the primary Q5 surface.
func _q5_what_to_do(main: Node, world: Node, reads: Node, camera: Camera2D, _tool_node: Node) -> Dictionary:
	var result: Dictionary = {"question": 5, "images": [], "seed": 0, "tick": -1}
	reads.call("mutate_raw", "reset_world", [0])
	for _i: int in 10:
		await process_frame
	world.call("advance_ticks", 1440)  # day 1
	result["tick"] = _now(world)
	var camp: Vector2i = _hearth(world)
	_frame(camera, Vector2(camp), 2.0)
	for _i: int in 30:
		await process_frame
	# M-READ: the plaque should be visible now with the current task.
	var plaque: Node = main.get_node_or_null("UI/TaskPlaque")
	if plaque != null:
		result["plaque_visible"] = (plaque as Control).visible
		if plaque.has_method("plaque_text"):
			result["plaque_text"] = str(plaque.call("plaque_text"))
	await _save_q("q5_overview", result)
	# Open the goals panel.
	var goals: Node = main.get_node_or_null("UI/Mux/GoalsPanel")
	if goals != null and goals.has_method("toggle_panel"):
		goals.call("toggle_panel")
		for _i: int in 30:
			await process_frame
		await _save_q("q5_goals", result)
		goals.hide()
	# Show alerts with plaque still visible.
	for _i: int in 10:
		await process_frame
	await _save_q("q5_alerts", result)
	return result


# Q6: 왜 겨울 준비가 모자라나
func _q6_winter_prep(main: Node, world: Node, reads: Node, camera: Camera2D, tool_node: Node) -> Dictionary:
	var result: Dictionary = {"question": 6, "images": [], "seed": 0, "tick": -1}
	reads.call("mutate_raw", "reset_world", [0])
	for _i: int in 10:
		await process_frame
	world.call("advance_ticks", 11520)  # day 8 (late autumn)
	result["tick"] = _now(world)
	var camp: Vector2i = _hearth(world)
	_frame(camera, Vector2(camp), 2.0)
	for _i: int in 30:
		await process_frame
	await _save_q("q6_overview", result)
	# Click a person to show inspector.
	if tool_node != null:
		var snap: Dictionary = world.call("get_agent_snapshot")
		var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
		var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
		if not xs.is_empty():
			var tile: Vector2i = Vector2i(xs[0], ys[0])
			tool_node.call("click_world", Vector2(tile) * TILE + Vector2(8, 8), tile)
			for _i: int in 30:
				await process_frame
			await _save_q("q6_inspector", result)
	# Open chronicle to show winter entries.
	var chronicle_panel: Node = main.get_node_or_null("UI/ChroniclePanel")
	if chronicle_panel != null and chronicle_panel.has_method("toggle_visible"):
		chronicle_panel.call("toggle_visible")
		for _i: int in 30:
			await process_frame
		await _save_q("q6_chronicle", result)
		chronicle_panel.call("toggle_visible")
	return result


# --- Helpers ---

func _now(world: Node) -> int:
	return int((world.call("get_time_snapshot") as Dictionary).get("tick", 0))


func _hearth(world: Node) -> Vector2i:
	var b: Dictionary = world.call("get_building_snapshot")
	var kinds: PackedByteArray = b.get("kinds", PackedByteArray())
	var xs: PackedInt32Array = b.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = b.get("ys", PackedInt32Array())
	for i: int in mini(kinds.size(), mini(xs.size(), ys.size())):
		if kinds[i] == 0:
			return Vector2i(xs[i], ys[i])
	return Vector2i(96, 96)


func _frame(camera: Camera2D, tile: Vector2, zoom: float) -> void:
	camera.set_process(false)
	camera.zoom = Vector2.ONE * zoom
	camera.position = tile * TILE + Vector2(8, 8)
	camera.reset_smoothing()
	camera.force_update_scroll()


func _save_q(name: String, result: Dictionary) -> void:
	await RenderingServer.frame_post_draw
	var image: Image = root.get_texture().get_image()
	var path: String = _out.path_join(name + ".png")
	if image == null or image.save_png(path) != OK:
		push_error("read_diag_capture: could not save " + path)
	else:
		print("read_diag_capture: saved ", path)
		var images: Array = result.get("images", [])
		images.append(path)
		result["images"] = images
