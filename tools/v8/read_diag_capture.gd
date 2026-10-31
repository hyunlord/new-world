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


# Q1: 왜 이 사람이 굶어 죽었나 — no starvation death is reachable at HEAD, so Q1 shows the
# hungriest living person at the colony's hungriest moment: steward OFF, before wolves arrive.
# Pass 1 scans day by day for the lowest hunger; pass 2 replays to that tick (deterministic).
func _q1_starvation(main: Node, world: Node, reads: Node, camera: Camera2D, _tool_node: Node) -> Dictionary:
	var result: Dictionary = {"question": 1, "images": [], "seed": 0, "tick": -1}
	_reset_steward_off(reads, world)
	var best_tick: int = 0
	var lowest: float = 999.0
	for _step: int in 180:  # 30 days in 240-tick steps
		world.call("advance_ticks", 240)
		if int(SimReads.read(world, "get_ui_facts").get("wolves_spawned", 0)) > 0:
			print("Q1 scan: wolves at tick %d" % _now(world))
			break
		var hungers: PackedFloat32Array = (world.call("get_agent_snapshot") as Dictionary).get("hungers", PackedFloat32Array())
		print("Q1 scan tick=%d wolves=%d min_hunger=%.1f" % [_now(world), int(SimReads.read(world, "get_ui_facts").get("wolves_spawned", 0)), hungers[_lowest_index(hungers)] if not hungers.is_empty() else -1.0])
		for h: float in hungers:
			if h < lowest:
				lowest = h
				best_tick = _now(world)
	_reset_steward_off(reads, world)
	world.call("advance_ticks", best_tick)
	for _i: int in 10:
		await process_frame
	result["tick"] = _now(world)
	result["constructed"] = "seed 0, steward OFF, before wolves, tick of lowest hunger"
	var snap: Dictionary = world.call("get_agent_snapshot")
	var idx: int = _lowest_index(snap.get("hungers", PackedFloat32Array()))
	if idx < 0:
		return result
	var tile: Vector2i = Vector2i(snap["xs"][idx], snap["ys"][idx])
	result["note"] = "no starvation death reachable; hungriest living person hunger=%.0f" % snap["hungers"][idx]
	_frame(camera, Vector2(tile), 2.0)
	for _i: int in 30:
		await process_frame
	await _save_q("q1_overview", result)
	await _inspect_and_why(main, world, int(snap["ids"][idx]), "q1", result)
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


# Q3: 왜 이 사람이 춥나 / 왜 화덕이 꺼졌나 — constructed: steward OFF, mid-winter. With the
# steward off nobody stokes the hearth (fire_lit 0 from day 6). The hearth diagnosis is opened
# with the causal panel's own open_tile: a click on the camp tile hits the adjacent campfire workshop.
func _q3_cold(main: Node, world: Node, reads: Node, camera: Camera2D, _tool_node: Node) -> Dictionary:
	var result: Dictionary = {"question": 3, "images": [], "seed": 0, "tick": -1}
	# Steward-off colonies can die out before deep winter: take the first winter tick (season 3)
	# with the hearth out and people alive; fall back to the last tick anyone was alive.
	_reset_steward_off(reads, world)
	var target_tick: int = -1
	var last_alive: int = 0
	for _step: int in 180:  # 30 days in 240-tick steps
		world.call("advance_ticks", 240)
		if (world.call("get_agent_snapshot") as Dictionary).get("ids", PackedInt64Array()).is_empty():
			break
		last_alive = _now(world)
		var lit: Array = Array(SimReads.read(world, "get_settlement_snapshot").get("fire_lit", []))
		if int(SimReads.read(world, "get_ui_facts").get("season", -1)) == 3 and not lit.has(1):
			target_tick = last_alive
			break
	result["winter_alive"] = target_tick >= 0
	_reset_steward_off(reads, world)
	world.call("advance_ticks", target_tick if target_tick >= 0 else last_alive)
	for _i: int in 10:
		await process_frame
	result["tick"] = _now(world)
	result["constructed"] = "seed 0, steward OFF, first winter tick with hearth out and people alive"
	var settlements: Dictionary = SimReads.read(world, "get_settlement_snapshot")
	result["fire_lit"] = str(settlements.get("fire_lit", []))
	var camp: Vector2i = _hearth(world)
	_frame(camera, Vector2(camp), 2.0)
	for _i: int in 30:
		await process_frame
	await _save_q("q3_overview", result)
	var causal: Node = main.get_node_or_null("UI/CausalPanel")
	if causal != null:
		causal.call("open_tile", camp)
		(causal as CanvasItem).visible = true
		for _i: int in 30:
			await process_frame
		await _save_q("q3_hearth_diag", result)
		(causal as CanvasItem).visible = false
	# The cold person nearest the hearth, else the nearest person.
	var snap: Dictionary = world.call("get_agent_snapshot")
	var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
	var colds: PackedByteArray = snap.get("colds", PackedByteArray())
	var target: int = -1
	var best: int = 1 << 30
	for i: int in xs.size():
		var d: int = absi(xs[i] - camp.x) + absi(ys[i] - camp.y) - (100000 if i < colds.size() and colds[i] != 0 else 0)
		if d < best:
			best = d
			target = i
	if target >= 0:
		result["person_cold"] = target < colds.size() and colds[target] != 0
		await _inspect_and_why(main, world, int(snap["ids"][target]), "q3", result)
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
	# Click a stockpile zone tile (not a person) to show its diagnosis.
	var overlay: Dictionary = SimReads.read(world, "get_colony_overlay")
	var zxs: PackedInt32Array = overlay.get("zone_xs", PackedInt32Array())
	var zys: PackedInt32Array = overlay.get("zone_ys", PackedInt32Array())
	if not zxs.is_empty() and tool_node != null:
		var zone_tile: Vector2i = Vector2i(zxs[0], zys[0])
		tool_node.call("click_world", Vector2(zone_tile) * TILE + Vector2(8, 8), zone_tile)
		for _i: int in 30:
			await process_frame
		await _save_q("q4_stockpile", result)
	elif tool_node != null:
		# No zone yet: click a person as fallback.
		var snap: Dictionary = world.call("get_agent_snapshot")
		var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
		var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
		if not xs.is_empty():
			var tile: Vector2i = Vector2i(xs[0], ys[0])
			tool_node.call("click_world", Vector2(tile) * TILE + Vector2(8, 8), tile)
			for _i: int in 30:
				await process_frame
			await _save_q("q4_inspector", result)
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
	# Click the stockpile zone to show its diagnosis (stores vs winter need).
	var overlay: Dictionary = SimReads.read(world, "get_colony_overlay")
	var zxs: PackedInt32Array = overlay.get("zone_xs", PackedInt32Array())
	var zys: PackedInt32Array = overlay.get("zone_ys", PackedInt32Array())
	if not zxs.is_empty() and tool_node != null:
		var zone_tile: Vector2i = Vector2i(zxs[0], zys[0])
		tool_node.call("click_world", Vector2(zone_tile) * TILE + Vector2(8, 8), zone_tile)
		for _i: int in 30:
			await process_frame
		await _save_q("q6_stockpile_diag", result)
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

func _reset_steward_off(reads: Node, world: Node) -> void:
	reads.call("mutate_raw", "reset_world", [0])
	SimReads.mutate(world, "cmd_set_steward", [false, 0])


func _lowest_index(values: PackedFloat32Array) -> int:
	var idx: int = -1
	for i: int in values.size():
		if idx < 0 or values[i] < values[idx]:
			idx = i
	return idx


## Opens the inspector on entity `bits` (no synthetic click), then its "왜?" view.
func _inspect_and_why(main: Node, world: Node, bits: int, prefix: String, result: Dictionary) -> void:
	var inspector: Node = main.get_node_or_null("UI/AgentInspectorPanel")
	var detail: Dictionary = world.call("get_agent_detail", bits)
	if inspector == null or not bool(detail.get("found", false)):
		return
	inspector.call("display_agent", detail)
	for _i: int in 30:
		await process_frame
	await _save_q(prefix + "_inspector", result)
	inspector.call("_open_tile_why")
	for _i: int in 30:
		await process_frame
	await _save_q(prefix + "_why", result)


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


## Load resolved commands from a JSON file (an array of {cmd, tick, ...} dicts).
func _load_commands(path: String) -> Array:
	if not FileAccess.file_exists(path):
		push_error("read_diag_capture: commands file not found: " + path)
		return []
	var parsed: Variant = JSON.parse_string(FileAccess.get_file_as_string(path))
	return parsed if parsed is Array else []


## Issue a single resolved command. Silently skips unknown or debug commands.
func _issue_command(world: Node, cmd: Dictionary) -> void:
	var name: String = str(cmd.get("cmd", ""))
	match name:
		"designate_chop_tiles":
			var tiles: Array = cmd.get("tiles", [])
			var xs: PackedInt32Array = PackedInt32Array()
			var ys: PackedInt32Array = PackedInt32Array()
			for t: Array in tiles:
				if t.size() >= 2:
					xs.append(int(t[0]))
					ys.append(int(t[1]))
			if not xs.is_empty():
				SimReads.mutate(world, "cmd_designate_chop_tiles", [xs, ys])
		"add_stockpile":
			SimReads.mutate(world, "cmd_add_stockpile", [int(cmd["x0"]), int(cmd["y0"]), int(cmd["x1"]), int(cmd["y1"])])
		"designate_quarry":
			SimReads.mutate(world, "cmd_designate_quarry", [int(cmd["x0"]), int(cmd["y0"]), int(cmd["x1"]), int(cmd["y1"])])
		"place_blueprint":
			SimReads.mutate(world, "cmd_place_blueprint", [str(cmd["kind"]), int(cmd["x"]), int(cmd["y"])])
		"place_rect":
			var hollow: bool = bool(cmd.get("hollow", false))
			SimReads.mutate(world, "cmd_place_rect", [str(cmd["kind"]), int(cmd["x0"]), int(cmd["y0"]), int(cmd["x1"]), int(cmd["y1"]), hollow])
		"add_bill":
			SimReads.mutate(world, "cmd_add_bill", [int(cmd["workshop"]), str(cmd["recipe"]), str(cmd["mode"]), int(cmd["count"])])
		"designate_dig":
			SimReads.mutate(world, "cmd_designate_dig", [int(cmd["x0"]), int(cmd["y0"]), int(cmd["x1"]), int(cmd["y1"])])
		"designate_clay":
			SimReads.mutate(world, "cmd_designate_clay", [int(cmd["x0"]), int(cmd["y0"]), int(cmd["x1"]), int(cmd["y1"])])
		"debug_kill":
			pass  # Not available via GDScript; the agent dies of old age at the same tick.
		_:
			pass  # Unknown command; skip silently.
