# tools/v8/capture.gd — screenshots of the live game for pass reports (M1 colony view).
# Purpose: load res://scenes/main.tscn, advance the simulation to a tick through the bridge's tooling call,
#   and save a "near" frame (the camp at the default 2× zoom) and a "far" frame (1× zoom, the map centre).
#   Tooling only: the game never runs this script.
# Bridge methods called: advance_ticks (tooling), get_time_snapshot, get_agent_snapshot, get_building_snapshot,
#   get_colony_overlay (the --m1-set dumps).
# --panels adds three shots at the near framing: "inspect" (a click on the first agent through
#   OrderTool.click_world opens the inspector), "causal" (a click on an empty tile opens the causal "why?"
#   panel) and "chronicle" (the chronicle panel toggled as the C key does).
# --m1-set takes the V8-PLAY owner set instead (docs/v8/play/PLAY_GOAL.md 1.1), all on the live default seed:
#   a first pass without frames finds the work tick (before the hut is finished, the most people carrying),
#   then the world is reset to the same seed and shot: m1_01 start (tick 360) → the U-script orders
#   (tools/v8/u_script.json, resolved here as the colony probe resolves them) → m1_02 orders (361) → at the work
#   tick m1_03 work, m1_06 far (1×, camp centred), m1_07 inspector on the person carrying most, m1_08 "why?"
#   on an empty tile, m1_09 order bar in hut mode with a red ghost over a tree (synthetic F3 and 1 keys and mouse
#   motion, as m1_gui_test.gd drives it) → m1_04 hut done + 60 with the chronicle shown → m1_05 the first 23:00
#   after it → m1_10 the next 10:00 at the same framing (the finished camp by day, M1-ART); m1_03b is the literal
#   tick 900; m1_07 is the inspector scrolled to its job and carry lines,
#   m1_07a its top. Ticks advance one at a time; m1_set.json records them, and for every shot a dump ("dumps") of
#   what is on screen in image pixels: each person (centre of the drawn tile, stage, job, carry count, asleep, the
#   agent layer's frame and halo), buildings, log stacks, zone tiles and blueprint sites (M1-ART goal 8.2, 8.3).
#   Extra bridge methods: get_world_info, get_chronicle_since, get_trees, get_terrain_snapshot, can_place_blueprint,
#   get_colony_overlay; via SimReads.mutate_raw reset_world, cmd_designate_chop_tiles, cmd_add_stockpile,
#   cmd_place_blueprint (orders, as a player would give them).
# usage: Godot --path . --rendering-driver opengl3 --resolution 1920x1080 \
#          --script res://tools/v8/capture.gd -- --out <abs dir> [--tick 1440] [--panels] [--zoom 2] [--m1-set]
extends SceneTree

const TILE: float = 16.0

var _out: String = ""
var _tick: int = 1440
var _panels: bool = false
var _zoom: float = 2.0
var _m1_set: bool = false
var _record: Dictionary = {}


func _init() -> void:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	for i: int in args.size():
		if args[i] == "--out" and i + 1 < args.size():
			_out = args[i + 1]
		elif args[i] == "--tick" and i + 1 < args.size():
			_tick = int(args[i + 1])
		elif args[i] == "--zoom" and i + 1 < args.size():
			_zoom = float(args[i + 1])
		elif args[i] == "--panels":
			_panels = true
		elif args[i] == "--m1-set":
			_m1_set = true
	call_deferred("_run")


func _run() -> void:
	if _out.is_empty():
		push_error("capture: --out is required")
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
		push_error("capture: missing WorldSim or Camera2D")
		quit(1)
		return
	world.set("process_mode", Node.PROCESS_MODE_DISABLED)
	if _m1_set:
		await _m1_shots(main, world, camera)
		return
	var now: int = int((world.call("get_time_snapshot") as Dictionary).get("tick", 0))
	if _tick > now:
		world.call("advance_ticks", _tick - now)
	var camp: Vector2i = _hearth(world)
	var shots: Array[Array] = [["near", Vector2(camp), _zoom], ["far", Vector2(96, 96), 1.0]]
	for shot: Array in shots:
		_frame(camera, shot[1] as Vector2, float(shot[2]))
		for _i: int in 30:
			await process_frame
		await _save("%s_t%d.png" % [shot[0], _tick])
	if _panels:
		await _panel_shots(main, world, camera, camp)
	quit(0)


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


func _save(name: String) -> void:
	await RenderingServer.frame_post_draw
	var image: Image = root.get_texture().get_image()
	var path: String = _out.path_join(name)
	if image == null or image.save_png(path) != OK:
		push_error("capture: could not save " + path)
	else:
		print("capture: saved ", path)


func _panel_shots(main: Node, world: Node, camera: Camera2D, camp: Vector2i) -> void:
	_frame(camera, Vector2(camp), _zoom)
	for _i: int in 10:
		await process_frame
	var tool: Node = main.get_node_or_null("ColonyView/OrderTool")
	var snap: Dictionary = world.call("get_agent_snapshot")
	var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
	var steps: Array[Array] = []
	if tool != null and not xs.is_empty():
		steps.append(["inspect", Vector2i(xs[0], ys[0])])
	if tool != null:
		steps.append(["causal", camp + Vector2i(1, 1)])
	steps.append(["chronicle", Vector2i.ZERO])
	for step: Array in steps:
		if step[0] == "chronicle":
			var chronicle: Node = main.get_node_or_null("UI/ChroniclePanel")
			if chronicle != null and chronicle.has_method("toggle_visible"):
				chronicle.call("toggle_visible")
		else:
			var tile: Vector2i = step[1]
			tool.call("click_world", Vector2(tile) * TILE + Vector2(8, 8), tile)
		for _i: int in 30:
			await process_frame
		await _save("%s_t%d.png" % [step[0], _tick])


# --- --m1-set (V8-PLAY) -------------------------------------------------------------------------------------

func _m1_shots(main: Node, world: Node, camera: Camera2D) -> void:
	var reads: Node = main.get_node_or_null("SimReads")
	var tool: Node = main.get_node_or_null("ColonyView/OrderTool")
	if reads == null or tool == null:
		push_error("capture: --m1-set needs Main/SimReads and ColonyView/OrderTool")
		quit(1)
		return
	camera.set_process(false)
	var seed: int = int((world.call("get_world_info") as Dictionary).get("seed", -1))
	_record["seed"] = seed
	# Pass 1 (no frames): find the work tick, the tick before the hut is finished with the most people carrying.
	_advance_to(world, 360)
	_m1_orders(world, reads)
	var best: int = -1
	var work_tick: int = 361
	var scan_hut: int = -1
	while scan_hut < 0 and _now(world) < 360 + 4 * 1440:
		world.call("advance_ticks", 1)
		var carrying: int = _carriers(world).size()
		if carrying > best:
			best = carrying
			work_tick = _now(world)
		if _hut_tile(world).x >= 0:
			scan_hut = _now(world)
	_record["scan"] = {"hut_completed_tick": scan_hut, "work_tick": work_tick, "carrying_at_work_tick": best}
	# Pass 2: the same world again from tick 0 (reset to the same seed), with frames.
	reads.call("mutate_raw", "reset_world", [seed])
	for _i: int in 10:
		await process_frame
	_advance_to(world, 360)
	var hearth: Vector2i = _hearth(world)
	_record["hearth"] = [hearth.x, hearth.y]
	await _near(camera, hearth)
	await _save_m1("m1_01_start_2x.png", world)
	_m1_orders(world, reads)
	_advance_to(world, 361)
	await _near(camera, hearth)
	await _save_m1("m1_02_orders_2x.png", world)
	var hut_tick: int = -1
	var p4_tick: int = -1
	var done: Dictionary = {}
	var limit: int = 360 + 4 * 1440
	while done.size() < 4:
		var now: int = _now(world)
		if now >= limit:
			push_error("capture: --m1-set ran past tick %d (hut tick %d)" % [limit, hut_tick])
			break
		if hut_tick < 0 and _hut_tile(world).x >= 0:
			hut_tick = now
			_record["hut_completed_tick"] = hut_tick
		if now == work_tick and not done.has("work"):
			await _m1_at_work(main, world, camera, tool, hearth)
			done["work"] = true
		if now == 900 and not done.has("t900"):
			await _near(camera, _work_centre(world, hearth))
			await _save_m1("m1_03b_t900_2x.png", world)
			done["t900"] = true
		if hut_tick >= 0 and p4_tick < 0 and now == hut_tick + 60:
			p4_tick = now
			var chronicle: Node = main.get_node_or_null("UI/ChroniclePanel")
			await _near(camera, _mid(hearth, _hut_tile(world) + Vector2i(1, 1)))
			if chronicle != null:
				chronicle.call("toggle_visible")
			await _save_m1("m1_04_hut_done_2x.png", world)
			var page: Dictionary = world.call("get_chronicle_since", 0, 512)
			var kinds: Dictionary = {}
			for k: int in page.get("kind", PackedInt32Array()):
				kinds[str(k)] = int(kinds.get(str(k), 0)) + 1
			_record["chronicle_kinds_at_p4"] = kinds
			if chronicle != null:
				chronicle.call("toggle_visible")
		if p4_tick >= 0 and now > p4_tick and _minute_of_day(world) == 23 * 60 and not done.has("night"):
			await _near(camera, _mid(hearth, _hut_tile(world) + Vector2i(1, 1)))
			await _save_m1("m1_05_night_2x.png", world)
			done["night"] = true
		if done.has("night") and _minute_of_day(world) == 10 * 60 and not done.has("morning"):
			await _near(camera, _mid(hearth, _hut_tile(world) + Vector2i(1, 1)))
			await _save_m1("m1_10_morning_2x.png", world)
			done["morning"] = true
		world.call("advance_ticks", 1)
	var file: FileAccess = FileAccess.open(_out.path_join("m1_set.json"), FileAccess.WRITE)
	if file != null:
		file.store_string(JSON.stringify(_record, "  ", true))
		file.close()
	print("CAPTURE_M1_SET_DONE ", JSON.stringify(_record))
	quit(0)


## The clock minute the top bar shows (hud_topbar.gd's formula).
func _minute_of_day(world: Node) -> int:
	return clampi(roundi(float((world.call("get_time_snapshot") as Dictionary).get("time_of_day", 0.0)) * 60.0), 0, 1439)


## Snapshot rows of people carrying something (carry count > 0).
func _carriers(world: Node) -> Array[int]:
	var snap: Dictionary = world.call("get_agent_snapshot")
	var counts: PackedInt32Array = snap.get("carry_counts", PackedInt32Array())
	var rows: Array[int] = []
	for i: int in counts.size():
		if counts[i] > 0:
			rows.append(i)
	return rows


## Between the hearth and the stockpile zone, so the work area and the camp share the frame.
func _work_centre(world: Node, hearth: Vector2i) -> Vector2i:
	var overlay: Dictionary = world.call("get_colony_overlay")
	var zxs: PackedInt32Array = overlay.get("zone_xs", PackedInt32Array())
	var zys: PackedInt32Array = overlay.get("zone_ys", PackedInt32Array())
	if zxs.is_empty():
		return hearth
	return _mid(hearth, Vector2i(zxs[0], zys[0]))


## The tile halfway between `a` and `b` (rounded toward zero).
func _mid(a: Vector2i, b: Vector2i) -> Vector2i:
	@warning_ignore("integer_division")
	return (a + b) / 2


func _now(world: Node) -> int:
	return int((world.call("get_time_snapshot") as Dictionary).get("tick", 0))


func _advance_to(world: Node, tick: int) -> void:
	var now: int = _now(world)
	if tick > now:
		world.call("advance_ticks", tick - now)


func _near(camera: Camera2D, tile: Vector2i, zoom: float = 2.0) -> void:
	camera.zoom = Vector2.ONE * zoom
	camera.call("center_on_tile", tile)
	camera.reset_smoothing()
	camera.force_update_scroll()
	for _i: int in 30:
		await process_frame


func _save_m1(name: String, world: Node) -> void:
	await _save(name)
	var shots: Dictionary = _record.get("shots", {})
	shots[name] = _now(world)
	_record["shots"] = shots
	var dumps: Dictionary = _record.get("dumps", {})
	dumps[name] = _dump(world)
	_record["dumps"] = dumps


## What the shot shows, in image pixels (the window is the 1920×1080 canvas): people, buildings, stacks, zones, sites.
func _dump(world: Node) -> Dictionary:
	var xf: Transform2D = root.get_canvas_transform()
	var view: Rect2 = Rect2(Vector2.ZERO, Vector2(1920, 1080))
	var agents: Node = root.get_node_or_null("Main/ColonyView/AgentLayer")
	var snap: Dictionary = world.call("get_agent_snapshot")
	var ids: PackedInt64Array = snap.get("ids", PackedInt64Array())
	var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
	var stages: PackedByteArray = snap.get("life_stages", PackedByteArray())
	var jobs: PackedByteArray = snap.get("jobs", PackedByteArray())
	var carries: PackedInt32Array = snap.get("carry_counts", PackedInt32Array())
	var states: PackedByteArray = snap.get("states", PackedByteArray())
	var targets: PackedByteArray = snap.get("target_kinds", PackedByteArray())
	var people: Array = []
	for i: int in mini(ids.size(), mini(xs.size(), ys.size())):
		var drawn: Vector2 = Vector2(xs[i], ys[i]) * TILE
		var look: Dictionary = {}
		if agents != null:
			var d: Vector2 = agents.call("drawn_position", ids[i])
			if d.x >= 0.0:
				drawn = d
			look = agents.call("person_look", ids[i])
		var centre: Vector2 = xf * (drawn + Vector2(8, 8))
		people.append({"tile": [xs[i], ys[i]], "screen": [roundi(centre.x), roundi(centre.y)],
			"on_screen": view.has_point(centre), "stage": int(stages[i]) if i < stages.size() else 0,
			"job": int(jobs[i]) if i < jobs.size() else 0, "carry": int(carries[i]) if i < carries.size() else 0,
			"asleep": i < states.size() and i < targets.size() and states[i] == 3 and targets[i] == 3,
			"look": look})
	var out: Dictionary = {"zoom": xf.x.x, "people": people}
	var b: Dictionary = world.call("get_building_snapshot")
	var bk: PackedByteArray = b.get("kinds", PackedByteArray())
	var bx: PackedInt32Array = b.get("xs", PackedInt32Array())
	var by: PackedInt32Array = b.get("ys", PackedInt32Array())
	var buildings: Array = []
	for i: int in mini(bk.size(), mini(bx.size(), by.size())):
		var c: Vector2 = xf * (Vector2(bx[i], by[i]) * TILE + Vector2(8, 8))
		buildings.append({"kind": int(bk[i]), "tile": [bx[i], by[i]], "screen": [roundi(c.x), roundi(c.y)]})
	out["buildings"] = buildings
	var o: Dictionary = world.call("get_colony_overlay")
	for pair: Array in [["stacks", "item_xs", "item_ys"], ["zones", "zone_xs", "zone_ys"], ["sites", "site_xs", "site_ys"], ["chops", "designation_xs", "designation_ys"]]:
		var pxs: PackedInt32Array = o.get(pair[1], PackedInt32Array())
		var pys: PackedInt32Array = o.get(pair[2], PackedInt32Array())
		var rows: Array = []
		for i: int in mini(pxs.size(), pys.size()):
			var c: Vector2 = xf * (Vector2(pxs[i], pys[i]) * TILE + Vector2(8, 8))
			rows.append({"tile": [pxs[i], pys[i]], "screen": [roundi(c.x), roundi(c.y)]})
		out[pair[0]] = rows
	var counts: PackedInt32Array = o.get("item_counts", PackedInt32Array())
	for i: int in mini(counts.size(), (out["stacks"] as Array).size()):
		(out["stacks"][i] as Dictionary)["count"] = counts[i]
	var trees: Node = root.get_node_or_null("Main/ColonyView/TreeLayer")
	if trees != null and trees.has_method("stump_tiles"):
		var stumps: Array = []
		for t: Vector2i in trees.call("stump_tiles"):
			var c: Vector2 = xf * (Vector2(t) * TILE + Vector2(8, 8))
			stumps.append({"tile": [t.x, t.y], "screen": [roundi(c.x), roundi(c.y)]})
		out["stumps"] = stumps
	return out


## The top-left tile of the first finished hut (building kind 3), or (-1, -1).
func _hut_tile(world: Node) -> Vector2i:
	var b: Dictionary = world.call("get_building_snapshot")
	var kinds: PackedByteArray = b.get("kinds", PackedByteArray())
	var xs: PackedInt32Array = b.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = b.get("ys", PackedInt32Array())
	for i: int in mini(kinds.size(), mini(xs.size(), ys.size())):
		if kinds[i] == 3:
			return Vector2i(xs[i], ys[i])
	return Vector2i(-1, -1)


func _cheb(a: Vector2i, b: Vector2i) -> int:
	return maxi(absi(a.x - b.x), absi(a.y - b.y))


## The U-script of tools/v8/u_script.json, resolved as rust/crates/sim-test/src/probe/colony.rs does
## (nearest_trees, stockpile_block, hut_spot), sent as orders at the current tick (360).
func _m1_orders(world: Node, reads: Node) -> void:
	var hearth: Vector2i = _hearth(world)
	var trees: Dictionary = world.call("get_trees")
	var txs: PackedInt32Array = trees.get("xs", PackedInt32Array())
	var tys: PackedInt32Array = trees.get("ys", PackedInt32Array())
	var ranked: Array[Vector3i] = []
	for i: int in mini(txs.size(), tys.size()):
		ranked.append(Vector3i(_cheb(Vector2i(txs[i], tys[i]), hearth), tys[i], txs[i]))
	ranked.sort()
	var cxs: PackedInt32Array = PackedInt32Array()
	var cys: PackedInt32Array = PackedInt32Array()
	for k: int in mini(15, ranked.size()):
		cxs.append(ranked[k].z)
		cys.append(ranked[k].y)
	_record["chop"] = reads.call("mutate_raw", "cmd_designate_chop_tiles", [cxs, cys])
	var terrain: Dictionary = world.call("get_terrain_snapshot")
	var blocked: Dictionary = {}
	var b: Dictionary = world.call("get_building_snapshot")
	var bxs: PackedInt32Array = b.get("xs", PackedInt32Array())
	var bys: PackedInt32Array = b.get("ys", PackedInt32Array())
	for i: int in mini(bxs.size(), bys.size()):
		blocked[Vector2i(bxs[i], bys[i])] = true
	var start: Vector2i = hearth + Vector2i(3, 0)
	var corner: Vector2i = Vector2i(-1, -1)
	for ring: int in 41:
		for y: int in range(start.y - ring, start.y + ring + 1):
			for x: int in range(start.x - ring, start.x + ring + 1):
				if corner.x >= 0 or x < 0 or y < 0 or _cheb(Vector2i(x, y), start) != ring:
					continue
				var ok: bool = true
				for dy: int in 4:
					for dx: int in 4:
						ok = ok and _zoneable(Vector2i(x + dx, y + dy), terrain, blocked)
				if ok:
					corner = Vector2i(x, y)
	var avoid: Dictionary = {}
	if corner.x >= 0:
		_record["stockpile"] = reads.call("mutate_raw", "cmd_add_stockpile", [corner.x, corner.y, corner.x + 3, corner.y + 3])
		_record["stockpile_rect"] = [corner.x, corner.y, corner.x + 3, corner.y + 3]
		for dy: int in 4:
			for dx: int in 4:
				avoid[corner + Vector2i(dx, dy)] = true
	var best: Vector3i = Vector3i(1 << 30, 0, 0)
	for y: int in range(maxi(0, hearth.y - 40), hearth.y + 40):
		for x: int in range(maxi(0, hearth.x - 40), hearth.x + 40):
			var centre: Vector2i = Vector2i(x + 1, y + 1)
			if centre.y <= hearth.y:
				continue
			var key: Vector3i = Vector3i(_cheb(centre, hearth), y, x)
			if key >= best or not bool(world.call("can_place_blueprint", "hut", x, y)):
				continue
			var covers: bool = false
			for dy: int in 3:
				for dx: int in 3:
					covers = covers or avoid.has(Vector2i(x + dx, y + dy))
			if not covers:
				best = key
	if best.x < (1 << 30):
		_record["hut"] = reads.call("mutate_raw", "cmd_place_blueprint", ["hut", best.z, best.y])
		_record["hut_top_left"] = [best.z, best.y]


func _zoneable(t: Vector2i, terrain: Dictionary, blocked: Dictionary) -> bool:
	var w: int = int(terrain.get("width", 0))
	var codes: PackedByteArray = terrain.get("terrain", PackedByteArray())
	if t.x < 0 or t.y < 0 or t.x >= w or t.y * w + t.x >= codes.size():
		return false
	var code: int = codes[t.y * w + t.x]
	return code != 2 and code != 4 and not blocked.has(t)


## The work tick: work (P3), the far view (P6), the inspector on a carrying person (P7), "why?" (P8), the order bar (P9).
func _m1_at_work(main: Node, world: Node, camera: Camera2D, tool: Node, hearth: Vector2i) -> void:
	await _near(camera, _work_centre(world, hearth))
	await _save_m1("m1_03_work_2x.png", world)
	await _near(camera, hearth, 1.0)
	await _save_m1("m1_06_far_1x.png", world)
	var snap: Dictionary = world.call("get_agent_snapshot")
	var ids: PackedInt64Array = snap.get("ids", PackedInt64Array())
	var counts: PackedInt32Array = snap.get("carry_counts", PackedInt32Array())
	var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
	var pick: int = -1
	for i: int in _carriers(world):
		if pick < 0 or counts[i] > counts[pick]:
			pick = i
	_record["carrying_at_work_tick"] = _carriers(world).size()
	var inspector: Node = main.get_node_or_null("UI/AgentInspectorPanel")
	if pick >= 0:
		var tile: Vector2i = Vector2i(xs[pick], ys[pick])
		await _near(camera, tile)
		var agents: Node = main.get_node_or_null("ColonyView/AgentLayer")
		var drawn: Vector2 = Vector2(tile) * TILE
		if agents != null and agents.has_method("drawn_position"):
			var d: Vector2 = agents.call("drawn_position", ids[pick])
			if d.x >= 0.0:
				drawn = d
		tool.call("click_world", drawn + Vector2(8, 8), tile)
		_record["inspected"] = {"tile": [tile.x, tile.y], "carry_count": counts[pick], "bits": ids[pick]}
		for _i: int in 30:
			await process_frame
		await _save_m1("m1_07a_inspector_top.png", world)
		var scroll: ScrollContainer = inspector.get("_scroll") as ScrollContainer if inspector != null else null
		if scroll != null:
			var job_line: Control = inspector.get("_job_label") as Control
			scroll.scroll_vertical = maxi(0, int(job_line.position.y) - 120) if job_line != null else int(scroll.get_v_scroll_bar().max_value)
			for _i: int in 10:
				await process_frame
			await _save_m1("m1_07_inspector.png", world)
	else:
		push_error("capture: nobody carries at tick %d" % _now(world))
	if inspector != null and inspector.has_method("close"):
		inspector.call("close")
	var empty: Vector2i = _empty_tile_near(world, hearth)
	await _near(camera, hearth)
	tool.call("click_world", Vector2(empty) * TILE + Vector2(8, 8), empty)
	_record["why_tile"] = [empty.x, empty.y]
	for _i: int in 30:
		await process_frame
	await _save_m1("m1_08_why.png", world)
	var causal: CanvasItem = main.get_node_or_null("UI/CausalPanel") as CanvasItem
	if causal != null:
		causal.visible = false
	await _m1_order_bar(main, world, camera, hearth)


## An empty tile near the hearth: no tree, building, person or zone, not water or mountain (ring order).
func _empty_tile_near(world: Node, hearth: Vector2i) -> Vector2i:
	var taken: Dictionary = {}
	for getter: String in ["get_trees", "get_building_snapshot", "get_agent_snapshot"]:
		var d: Dictionary = world.call(getter)
		var xs: PackedInt32Array = d.get("xs", PackedInt32Array())
		var ys: PackedInt32Array = d.get("ys", PackedInt32Array())
		for i: int in mini(xs.size(), ys.size()):
			taken[Vector2i(xs[i], ys[i])] = true
	var overlay: Dictionary = world.call("get_colony_overlay")
	for pair: Array in [["zone_xs", "zone_ys"], ["item_xs", "item_ys"], ["site_xs", "site_ys"]]:
		var oxs: PackedInt32Array = overlay.get(pair[0], PackedInt32Array())
		var oys: PackedInt32Array = overlay.get(pair[1], PackedInt32Array())
		for i: int in mini(oxs.size(), oys.size()):
			taken[Vector2i(oxs[i], oys[i])] = true
	var terrain: Dictionary = world.call("get_terrain_snapshot")
	for ring: int in range(2, 12):
		for y: int in range(hearth.y - ring, hearth.y + ring + 1):
			for x: int in range(hearth.x - ring, hearth.x + ring + 1):
				var t: Vector2i = Vector2i(x, y)
				if _cheb(t, hearth) == ring and _zoneable(t, terrain, taken):
					return t
	return hearth + Vector2i(2, 2)


## P9: F3 then 1 (the build menu's hut) by synthetic keys, then the mouse moved onto the standing tree nearest the hearth.
func _m1_order_bar(main: Node, world: Node, camera: Camera2D, hearth: Vector2i) -> void:
	await _near(camera, hearth)
	var trees: Dictionary = world.call("get_trees")
	var txs: PackedInt32Array = trees.get("xs", PackedInt32Array())
	var tys: PackedInt32Array = trees.get("ys", PackedInt32Array())
	var target: Vector2i = hearth + Vector2i(0, -3)
	var best: int = 1 << 30
	for i: int in mini(txs.size(), tys.size()):
		var t: Vector2i = Vector2i(txs[i], tys[i])
		if _cheb(t, hearth) < best and absi(t.y - hearth.y) <= 10:
			best = _cheb(t, hearth)
			target = t
	# M2: F3 opens the build menu; 1 picks the hut.
	for code: Key in [KEY_F3, KEY_1]:
		for pressed: bool in [true, false]:
			var key: InputEventKey = InputEventKey.new()
			key.keycode = code
			key.physical_keycode = code
			key.pressed = pressed
			Input.parse_input_event(key)
			await process_frame
	var screen: Vector2 = root.get_canvas_transform() * (Vector2(target) * TILE + Vector2(8, 8))
	for at: Vector2 in [screen + Vector2(TILE * 2.0, 0.0), screen]:
		var motion: InputEventMouseMotion = InputEventMouseMotion.new()
		motion.position = at
		motion.global_position = at
		Input.parse_input_event(motion)
		await process_frame
	for _i: int in 10:
		await process_frame
	var tool: Node = main.get_node("ColonyView/OrderTool")
	_record["order_bar"] = {"tree": [target.x, target.y], "mode": int(tool.get("mode")), "ghost_ok": bool(tool.get("_ghost_ok"))}
	await _save_m1("m1_09_orderbar.png", world)
	var esc: InputEventKey = InputEventKey.new()
	esc.keycode = KEY_ESCAPE
	esc.physical_keycode = KEY_ESCAPE
	esc.pressed = true
	Input.parse_input_event(esc)
	await process_frame
