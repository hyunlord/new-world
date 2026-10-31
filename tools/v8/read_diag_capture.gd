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
const WORK_MIND_KEYS: Array[String] = ["MIND_BEHAVIOUR_DO_JOB", "MIND_BEHAVIOUR_SLEEP"]

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


# Q1: 왜 이 사람이 굶어 죽었나 — constructed: seed 0, steward OFF. People starve on day 19
# (first at tick ~27550). Scan forward to the first starvation death (chronicle kind 11, cause 0),
# run ~250 ticks more so a few deaths are listed, then: overview, chronicle, the dead person's
# record, and the death entry's "왜?" chain.
func _q1_starvation(main: Node, world: Node, reads: Node, camera: Camera2D, _tool_node: Node) -> Dictionary:
	var result: Dictionary = {"question": 1, "images": [], "seed": 0, "tick": -1}
	_reset_steward_off(reads, world)
	var death: Dictionary = {}
	for _step: int in 180:  # 30 days in 240-tick steps
		world.call("advance_ticks", 240)
		death = _first_starvation(world)
		if not death.is_empty():
			break
	if death.is_empty():
		result["note"] = "no starvation death within 30 days"
		return result
	var extra: int = int(death["tick"]) + 250 - _now(world)
	if extra > 0:
		world.call("advance_ticks", extra)
	for _i: int in 10:
		await process_frame
	result["tick"] = _now(world)
	result["constructed"] = "seed 0, steward OFF, ~250 ticks after the first starvation death"
	result["death"] = death
	_frame(camera, Vector2(_hearth(world)), 2.0)
	for _i: int in 30:
		await process_frame
	await _save_q("q1_overview", result)
	var chronicle_panel: Node = main.get_node_or_null("UI/ChroniclePanel")
	if chronicle_panel != null:
		chronicle_panel.call("toggle_visible")
		for _i: int in 30:
			await process_frame
		await _save_q("q1_chronicle", result)
		chronicle_panel.call("toggle_visible")
	var inspector: Node = main.get_node_or_null("UI/AgentInspectorPanel")
	if inspector != null:
		inspector.call("show_person", int(death["subject"]))
		for _i: int in 30:
			await process_frame
		await _save_q("q1_record", result)
	if chronicle_panel != null and bool(chronicle_panel.call("open_why", int(death["index"]))):
		for _i: int in 30:
			await process_frame
		await _save_q("q1_why", result)
	return result


## The first chronicle death by starvation (kind 11, cause 0) as {index, tick, subject}, or {}.
func _first_starvation(world: Node) -> Dictionary:
	var next: int = 0
	while true:
		var page: Dictionary = world.call("get_chronicle_since", next, 512)
		var kinds: PackedInt32Array = page.get("kind", PackedInt32Array())
		var causes: PackedInt64Array = page.get("cause", PackedInt64Array())
		for i: int in kinds.size():
			if kinds[i] == 11 and causes[i] == 0:
				return {"index": int(page["index"][i]), "tick": int(page["tick"][i]), "subject": int(page["subject"][i])}
		var after: int = int(page.get("next", next))
		if kinds.is_empty() or after <= next:
			return {}
		next = after
	return {}


# Q2: 왜 이 어른이 일을 안 하나 — steward OFF (with the steward on nobody is idle through day 15),
# scan from tick 0 to the first adult the "할 일 없는 어른" alert counts (assets/ui/alerts.json
# idle_adults predicates); overview, then their inspector and "왜?" (no synthetic click).
func _q2_not_working(main: Node, world: Node, reads: Node, camera: Camera2D, _tool_node: Node) -> Dictionary:
	var result: Dictionary = {"question": 2, "images": [], "seed": 0, "tick": -1}
	_reset_steward_off(reads, world)
	var bits: int = -1
	for _step: int in 90:  # up to 15 days in 240-tick steps
		world.call("advance_ticks", 240)
		bits = _idle_adult(world)
		if bits >= 0:
			break
	for _i: int in 10:
		await process_frame
	result["tick"] = _now(world)
	result["idle_adult_found"] = bits >= 0
	_frame(camera, Vector2(_hearth(world)), 2.0)
	for _i: int in 30:
		await process_frame
	await _save_q("q2_overview", result)
	if bits >= 0:
		await _inspect_and_why(main, world, bits, "q2", result)
	return result


## The entity bits of the first person the idle-adults alert counts (an adult with no job who is
## idle, not sleeping, drafted, sick or in breakdown) who also has a person diagnosis; -1 when none.
func _idle_adult(world: Node) -> int:
	var people: Dictionary = SimReads.read(world, "get_ui_facts").get("people", {})
	var entities: PackedInt64Array = people.get("entity_ids", PackedInt64Array())
	for i: int in entities.size():
		var flags: Array = ["children", "has_job", "sleeping", "drafted", "sick", "breakdown"].map(
			func(key: String) -> bool: return i < (people.get(key, PackedByteArray()) as PackedByteArray).size() and people[key][i] != 0)
		var idle: bool = i < (people.get("idle", PackedByteArray()) as PackedByteArray).size() and people["idle"][i] != 0
		if idle and not flags.has(true) and _has_diagnosis(world, int(entities[i])) and not _mind_working(world, int(entities[i])):
			return int(entities[i])
	return -1


## Whether person `bits`'s mind has chosen work or sleep ("지금: 일하기" / "잠자기"), which would not read
## as an idle adult in the screenshot.
func _mind_working(world: Node, bits: int) -> bool:
	var mind: Dictionary = world.call("get_agent_mind", bits)
	return str(mind.get("name_key", "")) in WORK_MIND_KEYS


## Whether person `bits` has a person-diagnosis line (any failing check).
func _has_diagnosis(world: Node, bits: int) -> bool:
	if not world.has_method("get_person_diagnosis"):
		return true
	var diag: Dictionary = world.call("get_person_diagnosis", bits)
	return not (diag.get("text_keys", PackedStringArray()) as PackedStringArray).is_empty()


# Q3: 왜 이 사람이 춥나 / 왜 화덕이 꺼졌나 — constructed: seed 0, steward OFF, winter (tick 26400,
# day 19; winter starts day 18), before the day-19 starvation deaths. The hearth diagnosis is opened with the
# causal panel's own open_tile (a camp-tile click hits the adjacent campfire workshop), then the
# inspector and "왜?" of a cold person, else the hungriest.
func _q3_cold(main: Node, world: Node, reads: Node, camera: Camera2D, _tool_node: Node) -> Dictionary:
	var result: Dictionary = {"question": 3, "images": [], "seed": 0, "tick": -1}
	_reset_steward_off(reads, world)
	world.call("advance_ticks", 26400)  # winter day 2 (day 19), before the first starvation at ~27550
	for _i: int in 10:
		await process_frame
	result["tick"] = _now(world)
	result["constructed"] = "seed 0, steward OFF, winter tick 26400, before starvation deaths"
	result["fire_lit"] = str(SimReads.read(world, "get_settlement_snapshot").get("fire_lit", []))
	var camp: Vector2i = _hearth(world)
	_frame(camera, Vector2(camp), 2.0)
	for _i: int in 30:
		await process_frame
	var causal: Node = main.get_node_or_null("UI/CausalPanel")
	if causal != null:
		causal.call("open_tile", camp)
		(causal as CanvasItem).visible = true
		for _i: int in 30:
			await process_frame
		await _save_q("q3_hearth_diag", result)
		(causal as CanvasItem).visible = false
	var snap: Dictionary = world.call("get_agent_snapshot")
	var ids: PackedInt64Array = snap.get("ids", PackedInt64Array())
	var colds: PackedByteArray = snap.get("colds", PackedByteArray())
	result["alive"] = ids.size()
	var target: int = _lowest_index(snap.get("hungers", PackedFloat32Array()))
	for i: int in mini(ids.size(), colds.size()):
		if colds[i] != 0:
			target = i
			break
	if target >= 0:
		result["person_cold"] = target < colds.size() and colds[target] != 0
		result["person_hunger"] = snap["hungers"][target]
		await _inspect_and_why(main, world, int(ids[target]), "q3", result)
	return result


# Q4: 왜 비축 구역에 물건이 안 오나 — day 8 (tick 11520), when a stockpile zone is full; overview
# on that zone, then the causal panel's own open_tile on it with its stockpile diagnosis line.
func _q4_stockpile(main: Node, world: Node, reads: Node, camera: Camera2D, _tool_node: Node) -> Dictionary:
	var result: Dictionary = {"question": 4, "images": [], "seed": 0, "tick": -1}
	reads.call("mutate_raw", "reset_world", [0])
	for _i: int in 10:
		await process_frame
	world.call("advance_ticks", 11520)  # day 8
	for _i: int in 10:
		await process_frame
	result["tick"] = _now(world)
	var tile: Vector2i = _full_stockpile_tile(world)
	result["full_zone"] = tile.x >= 0
	if tile.x < 0:
		tile = _hearth(world)
	_frame(camera, Vector2(tile), 2.0)
	for _i: int in 30:
		await process_frame
	await _save_q("q4_overview", result)
	var causal: Node = main.get_node_or_null("UI/CausalPanel")
	if causal != null:
		causal.call("open_tile", tile)
		(causal as CanvasItem).visible = true
		for _i: int in 30:
			await process_frame
		await _save_q("q4_stockpile", result)
		(causal as CanvasItem).visible = false
	return result


## A tile of the first stockpile zone whose subject diagnosis says it is full, or (-1, -1).
func _full_stockpile_tile(world: Node) -> Vector2i:
	if not world.has_method("get_subject_diagnosis"):
		return Vector2i(-1, -1)
	var overlay: Dictionary = SimReads.read(world, "get_colony_overlay")
	var xs: PackedInt32Array = overlay.get("zone_xs", PackedInt32Array())
	var ys: PackedInt32Array = overlay.get("zone_ys", PackedInt32Array())
	var ids: PackedInt32Array = overlay.get("zone_ids", PackedInt32Array())
	for i: int in mini(ids.size(), mini(xs.size(), ys.size())):
		var diag: Dictionary = world.call("get_subject_diagnosis", "stockpile", int(ids[i]))
		if Array(diag.get("text_keys", PackedStringArray())).has("DIAG_STOCKPILE_FULL"):
			return Vector2i(xs[i], ys[i])
	return Vector2i(-1, -1)


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
	# Hover the plaque so its tooltip shows the request's full reason chain.
	if plaque != null and plaque.has_method("plaque_tooltip"):
		result["plaque_tooltip"] = str(plaque.call("plaque_tooltip"))
		var panel: Control = plaque.get("_panel")
		var rect: Rect2 = panel.get_global_rect()
		var spot: Vector2 = rect.position + Vector2(16, rect.size.y - 6)  # lower-left: the tooltip opens below the plaque
		Input.warp_mouse(spot)
		var motion: InputEventMouseMotion = InputEventMouseMotion.new()
		motion.position = spot
		motion.global_position = spot
		root.push_input(motion)
		for _i: int in 90:
			await process_frame
	await _save_q("q5_alerts", result)  # the plaque hover with the full chain
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
