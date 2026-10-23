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
# --m3-set --load <abs save file> takes the M3 set (docs/v8/m3, goal 12.3) on a saved colony world (the colony
#   probe's `--save-at TICK --save-out FILE`, e.g. the M3-U script on a seed with a hill): the world is loaded
#   through SimReads.mutate_raw load_world, the clock stays stopped (WorldSim's processing is off; only
#   advance_ticks moves it), and as a player would: R shows the room overlay (m3_01 rooms, after a room's role was
#   fixed through the room panel), m3_02 the dug pocket close up (overlay off), m3_03 F3 then 5 (log wall) and a
#   mouse drag held across a built wall (the preview ghost with refused tiles in red; right click leaves the mode,
#   nothing is placed), m3_04 the inspector on a person whose bed is in the bedroom (OrderTool.click_world, the bed
#   line in view), m3_05 the mouse over a stack in the storehouse (the hover's "방: 창고 n"; the overlay's hover
#   follows the OS cursor, so Input.warp_mouse moves it there and parks it in a corner otherwise), m3_06 a click on
#   the dug pocket with R shown opens the room panel and 작업장 is chosen in its dropdown (m3_06b the dropdown
#   open), m3_07 far (1×, camp centred), m3_08 the next 23:00 at 3× (the bedroom with its sleepers). m3_01 is
#   taken at 2× when every room fits the frame, else at 1× plus m3_01b (the camp's rooms at 2×). m3_set.json
#   records the rooms, the ticks and the dumps.
#   Extra bridge methods: get_rooms, get_space, get_agent_economy, get_place_preview, get_room_detail (via the room
#   panel); via SimReads.mutate_raw load_world; the room panel sends cmd_set_room_role.
# --m4-set --load <abs save file> takes the M4 set (docs/v8/m4) on a saved colony world (the colony probe's
#   `--save-at TICK --save-out FILE` with tools/v8/m4_u_script.json, a tick after the first clay pots were fired): the
#   world is loaded as for --m3-set (clock stopped, the OS cursor parked in a corner) and, as a player would: m4_01
#   the camp at 2× with the HUD's era line ("구석기 n %" / "신석기 전환기 n %") and the "⚠ 기술 위험 N" badge; m4_02 T
#   opens the tech map (미지 / 위험 / 안정 cards); m4_03 a mouse click on the first at-risk card (else the first known
#   one) shows its detail (holders, learners, conditions, gates, 가르치기); m4_04 a click on 가르치기 opens the
#   teach dialog (K) on that node, the first listed teacher and learner are chosen and a click on 명령 sends the
#   order, so the open order shows in the list (this changes the world: one cmd_order_teach); K and T close both;
#   m4_05 a click on the person with the most kinds of knowledge source (전승/구전/관찰/발견/도제; one with a 도제
#   entry among equals), then on the inspector's 지식 tab (m4_05b the list scrolled to its first 관찰 or 도제 entry); m4_06 the mouse over a fired clay pot stack (item kind 9) in the storehouse at 3× (the
#   hover line "토기 ×n · 방: 창고 n"; Input.warp_mouse moves the OS cursor there, see --m3-set); m4_07 a river ford
#   at 3×; m4_08 far (1×) over the camp and its clay banks and pits; m4_09 C shows the chronicle at the camp (2×).
#   m4_set.json records the tick, the tech read, what each shot shows and the dumps.
#   Extra bridge methods: get_tech, get_agent_knowledge (direct); via the panels get_tech_node, get_teach_orders,
#   get_knowledge_waits, get_chronicle_since; the teach dialog sends cmd_order_teach.
# --m5-set --load <abs save file> [--m5-shots 1,2,…] takes the M5 set (docs/v8/m5, goal §10) on a saved colony world
#   (the colony probe's `--save-at TICK --save-out FILE` with tools/v8/m5_u_script.json); a save serves the shots that
#   fit its moment and several runs share one --out (m5_set.json merges their notes; each note names its save). The
#   world is loaded as for --m3-set (clock stopped, OS cursor parked) and, as a player would: m5_01 the clock is
#   advanced an hour at a time to the first tick with wolves on the map, then until they are within 30 tiles of the
#   hearth (or 6 hours more): the camp and the wolves framed together with the HUD's banner "늑대 무리가 다가온다 (n)";
#   m5_02 four adults (spear holders first) are drafted by a click on each and D, then a right click on the nearest
#   wolf orders all four (the inspector closed), the clock runs 40 ticks: the party with the drafted list (this changes
#   the world: four cmd_draft and four cmd_draft_attack); m5_03 the clock is advanced (≤ 3 days) to someone not 보통,
#   framed with the camp, and M shows the mood overlay; m5_04 C
#   opens the chronicle and the "→ 왜?" of the latest quarrel with a cause opens its chain (UI/WhyPanel; the link's
#   meta_clicked, as a click on it sends); m5_05 the camp at 2× with the cold-snap banner "한파 n일째 · 장작 n (하루 n
#   필요)" on the snap's second evening when it lasts that long; m5_06 a click on the person with the most chronicle lines, then on the inspector's 이야기 tab; m5_07 the
#   clock is advanced half an hour at a time (≤ 3 days) to a new deer Hunted entry, then the hunter and the meat stack
#   nearest them are framed at 3× (else the latest hunt before the save); when that meat lies under a crown (the
#   canopy hides ground stacks, as it hides logs) m5_07b is the first meat stack in the open within a day (a hauled
#   carcass) at 3× with its hover line; m5_08 the clock is advanced to someone wearing a hide wrap (get_agent_snapshot `tops`), who is
#   clicked: the inspector scrolled to "옷: 가죽 랩 (추위 −30 %)". Each note records its tick and what it shows.
#   Extra bridge methods: get_story, get_animals, get_drafted, get_agent_m5, get_agent_economy, get_chronicle_since,
#   get_chronicle_why, get_person_chronicle (direct or via the panels); D, right clicks and the panels send cmd_draft,
#   cmd_draft_attack.
# --m55-set [<abs dir>] takes the M5.5 set (docs/v8/m5_5, goal §9 and §11 12.3) on the live default world (the
#   directory may follow the flag or come as --out): the clock is stopped and the OS cursor parked; a 3×3 stockpile
#   near the hearth and chop marks on the ten nearest trees give the colony stock; the clock is advanced 240 ticks at
#   a time (≤ 30 days) to the first caravan trading and the first wave waiting at the hearth. m55_02 a click on the
#   caravan opens the trade dialog; the cheapest unknown artifact (else the cheapest goods) is picked and logs given
#   through the rows' + buttons until the traders would accept: the two columns and the balance bar; then 거래 is
#   clicked (this changes the world: one cmd_trade) so the colony holds a clue. m55_01 the migrant dialog that opened
#   by itself (camp at 2× behind it); then 받아들이기 is clicked (one cmd_accept_migrants). m55_03 N shows the
#   neighbours panel. m55_06 T opens the tech map on a rumoured node ("이웃 ○○가 앎 (소문)"; a clue node's ★ shows on
#   its card). m55_04 the clock is advanced to the first wolves, then 30 ticks at a time (≤ 2 days) to an undrafted
#   person fleeing (snapshot `fleeing`): framed at 3× with the "!" and clicked, the inspector showing "도망 중: 늑대".
#   m55_05 the clock is advanced (≤ day 60) to a Eureka chronicle entry (kind 44; else the newest M5.5 entry, noted)
#   and C shows the chronicle filtered to 지식. m55_set.json records each shot's tick and what it shows.
#   Extra bridge methods: get_neighbours, get_wave, get_caravan, get_story, get_trees, get_chronicle_since (direct); the
#   dialogs send cmd_trade and cmd_accept_migrants; cmd_add_stockpile, cmd_designate_chop_tiles via SimReads.
# usage: Godot --path . --rendering-driver opengl3 --resolution 1920x1080 \
#          --script res://tools/v8/capture.gd -- --out <abs dir> [--tick 1440] [--panels] [--zoom 2] [--m1-set]
#          [--m3-set --load <abs save file>] [--m4-set --load <abs save file>]
#          [--m5-set --load <abs save file> [--m5-shots 1,2,3,4,5,6,7,8]] [--m55-set [<abs dir>]]
extends SceneTree

const TILE: float = 16.0
const OverlaySuffix = preload("res://scripts/world/colony/overlay_layer.gd")
const ChronicleText = preload("res://scripts/ui/chronicle_text.gd")
const NeighbourText = preload("res://scripts/ui/neighbour_text.gd")

var _out: String = ""
var _tick: int = 1440
var _panels: bool = false
var _zoom: float = 2.0
var _m1_set: bool = false
var _m3_set: bool = false
var _m4_set: bool = false
var _m5_set: bool = false
var _m55_set: bool = false
var _m5_shots: PackedStringArray = PackedStringArray(["1", "2", "3", "4", "5", "6", "7", "8"])
var _load: String = ""
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
		elif args[i] == "--m3-set":
			_m3_set = true
		elif args[i] == "--m4-set":
			_m4_set = true
		elif args[i] == "--m5-set":
			_m5_set = true
		elif args[i] == "--m55-set":
			_m55_set = true
			if i + 1 < args.size() and not args[i + 1].begins_with("--"):
				_out = args[i + 1]
		elif args[i] == "--m5-shots" and i + 1 < args.size():
			_m5_shots = args[i + 1].split(",")
		elif args[i] == "--load" and i + 1 < args.size():
			_load = args[i + 1]
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
	if _m3_set:
		await _m3_shots(main, world, camera)
		return
	if _m4_set:
		await _m4_shots(main, world, camera)
		return
	if _m5_set:
		await _m5_shots_run(main, world, camera)
		return
	if _m55_set:
		await _m55_shots(main, world, camera)
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


# --- --m3-set (M3, goal 12.3) -------------------------------------------------------------------------------

func _key(code: Key) -> void:
	for pressed: bool in [true, false]:
		var key: InputEventKey = InputEventKey.new()
		key.keycode = code
		key.physical_keycode = code
		key.pressed = pressed
		Input.parse_input_event(key)
		await process_frame


## The screen point of a tile's centre under the current camera.
func _screen_of(tile: Vector2i) -> Vector2:
	return root.get_canvas_transform() * (Vector2(tile) * TILE + Vector2(8, 8))


func _motion(at: Vector2) -> void:
	var motion: InputEventMouseMotion = InputEventMouseMotion.new()
	motion.position = at
	motion.global_position = at
	Input.parse_input_event(motion)
	await process_frame


func _button(at: Vector2, index: MouseButton, pressed: bool) -> void:
	var click: InputEventMouseButton = InputEventMouseButton.new()
	click.button_index = index
	click.pressed = pressed
	click.position = at
	click.global_position = at
	Input.parse_input_event(click)
	await process_frame


## Rooms by id from get_rooms: {id: {role, overridden, tiles, beds, roofed, dug, centre, cells}}.
func _rooms_of(world: Node) -> Dictionary:
	var snap: Dictionary = world.call("get_rooms")
	var ids: PackedInt32Array = snap.get("ids", PackedInt32Array())
	var roles: PackedInt32Array = snap.get("roles", PackedInt32Array())
	var over: PackedByteArray = snap.get("overridden", PackedByteArray())
	var tiles: PackedInt32Array = snap.get("tiles", PackedInt32Array())
	var beds: PackedInt32Array = snap.get("beds", PackedInt32Array())
	var roofed: PackedByteArray = snap.get("roofed", PackedByteArray())
	var dug: PackedByteArray = snap.get("dug", PackedByteArray())
	var cxs: PackedInt32Array = snap.get("centre_xs", PackedInt32Array())
	var cys: PackedInt32Array = snap.get("centre_ys", PackedInt32Array())
	var out: Dictionary = {}
	for i: int in mini(ids.size(), mini(roles.size(), mini(cxs.size(), cys.size()))):
		out[ids[i]] = {"id": ids[i], "role": roles[i], "overridden": i < over.size() and over[i] != 0,
			"tiles": tiles[i] if i < tiles.size() else 0, "beds": beds[i] if i < beds.size() else 0,
			"roofed": i < roofed.size() and roofed[i] != 0, "dug": i < dug.size() and dug[i] != 0,
			"centre": Vector2i(cxs[i], cys[i]), "cells": []}
	var txs: PackedInt32Array = snap.get("tile_xs", PackedInt32Array())
	var tys: PackedInt32Array = snap.get("tile_ys", PackedInt32Array())
	var trs: PackedInt32Array = snap.get("tile_rooms", PackedInt32Array())
	for i: int in mini(txs.size(), mini(tys.size(), trs.size())):
		if out.has(trs[i]):
			(out[trs[i]]["cells"] as Array).append(Vector2i(txs[i], tys[i]))
	return out


## The first room with role `role` (the dug pocket when `dug`), largest first; -1 when none.
func _room_of_role(rooms: Dictionary, role: int, dug: bool) -> int:
	var best: int = -1
	for id: int in rooms:
		var r: Dictionary = rooms[id]
		if bool(r["dug"]) != dug or (role >= 0 and int(r["role"]) != role):
			continue
		if best < 0 or int(r["tiles"]) > int(rooms[best]["tiles"]):
			best = id
	return best


func _rooms_json(rooms: Dictionary) -> Array:
	var rows: Array = []
	for id: int in rooms:
		var r: Dictionary = rooms[id]
		rows.append({"id": id, "role": r["role"], "overridden": r["overridden"], "tiles": r["tiles"], "beds": r["beds"],
			"roofed": r["roofed"], "dug": r["dug"], "centre": [(r["centre"] as Vector2i).x, (r["centre"] as Vector2i).y]})
	return rows


## The tile framing tiles `points` (their bounding-box centre).
func _centre_of(points: Array[Vector2i]) -> Vector2i:
	var lo: Vector2i = points[0]
	var hi: Vector2i = points[0]
	for p: Vector2i in points:
		lo = Vector2i(mini(lo.x, p.x), mini(lo.y, p.y))
		hi = Vector2i(maxi(hi.x, p.x), maxi(hi.y, p.y))
	return _mid(lo, hi)


func _span(points: Array[Vector2i]) -> Vector2i:
	var lo: Vector2i = points[0]
	var hi: Vector2i = points[0]
	for p: Vector2i in points:
		lo = Vector2i(mini(lo.x, p.x), mini(lo.y, p.y))
		hi = Vector2i(maxi(hi.x, p.x), maxi(hi.y, p.y))
	return hi - lo


func _m3_shots(main: Node, world: Node, camera: Camera2D) -> void:
	var reads: Node = main.get_node_or_null("SimReads")
	var tool: Node = main.get_node_or_null("ColonyView/OrderTool")
	var room_layer: Node = main.get_node_or_null("ColonyView/OverlayLayer/Rooms")
	var room_panel: Node = main.get_node_or_null("UI/RoomPanel")
	var inspector: Node = main.get_node_or_null("UI/AgentInspectorPanel")
	if reads == null or tool == null or room_layer == null or room_panel == null or inspector == null or _load.is_empty():
		push_error("capture: --m3-set needs --load <file>, Main/SimReads, OrderTool, the room layer, panels")
		quit(1)
		return
	camera.set_process(false)
	if not bool(reads.call("mutate_raw", "load_world", [_load])):
		push_error("capture: load_world failed for " + _load)
		quit(1)
		return
	for _i: int in 20:
		await process_frame
	_record["loaded"] = _load
	_record["loaded_tick"] = _now(world)
	_record["seed"] = int((world.call("get_world_info") as Dictionary).get("seed", -1))
	var hearth: Vector2i = _hearth(world)
	_record["hearth"] = [hearth.x, hearth.y]
	var rooms: Dictionary = _rooms_of(world)
	_record["rooms_at_load"] = _rooms_json(rooms)
	var bedroom: int = _room_of_role(rooms, 1, false)
	var store: int = _room_of_role(rooms, 2, false)
	var pocket: int = _room_of_role(rooms, -1, true)
	_record["picked"] = {"bedroom": bedroom, "storehouse": store, "pocket": pocket}
	if bedroom < 0 or store < 0:
		push_error("capture: --m3-set needs a bedroom and a storehouse (rooms %s)" % str(_record["rooms_at_load"]))
		quit(1)
		return
	var camp_points: Array[Vector2i] = [hearth, rooms[bedroom]["centre"], rooms[store]["centre"]]
	var all_points: Array[Vector2i] = camp_points.duplicate()
	if pocket >= 0:
		all_points.append(rooms[pocket]["centre"])
	# The overlay's hover follows the OS cursor (the root viewport reads it, not the synthetic events): park it in
	# the bottom-right corner so no stray hover label shows; m3_05 moves it onto a stack.
	Input.warp_mouse(Vector2(1919, 1079))
	# m3_06 first (it fixes the pocket's role): R, a click on the pocket (or the storehouse) opens the room panel;
	# 작업장 is chosen in its dropdown (item_selected, as a click on the entry), then the panel is shot with the role
	# fixed, and again with the dropdown opened.
	var panel_room: int = pocket if pocket >= 0 else store
	var panel_tile: Vector2i = (rooms[panel_room]["cells"] as Array)[0] if not (rooms[panel_room]["cells"] as Array).is_empty() else rooms[panel_room]["centre"]
	await _near(camera, rooms[panel_room]["centre"])
	await _key(KEY_R)
	tool.call("click_world", Vector2(panel_tile) * TILE + Vector2(8, 8), panel_tile)
	for _i: int in 10:
		await process_frame
	var choice: OptionButton = room_panel.get("_role") as OptionButton
	if choice != null and pocket >= 0:
		choice.select(3)
		choice.item_selected.emit(3)
		_record["role_set"] = "workshop (dropdown entry 3, item_selected)"
		world.call("advance_ticks", 1)
	for _i: int in 10:
		await process_frame
	_record["room_panel"] = {"room": int(room_panel.call("current_room")), "visible": (room_panel as Control).visible,
		"detail": room_panel.call("detail")}
	await _save_m1("m3_06_room_panel.png", world)
	if choice != null:
		choice.show_popup()
		for _i: int in 10:
			await process_frame
		await _save_m1("m3_06b_role_dropdown.png", world)
		choice.get_popup().hide()
	room_panel.call("close")
	for _i: int in 10:
		await process_frame
	# m3_01: the room overlay over the camp (2× when the rooms fit the frame, else 1×).
	var span: Vector2i = _span(all_points)
	var zoom: float = 2.0 if span.x <= 48 and span.y <= 24 else 1.0
	_record["rooms_after_role"] = _rooms_json(_rooms_of(world))
	await _near(camera, _centre_of(all_points), zoom)
	await _save_m1("m3_01_rooms_%dx.png" % int(zoom), world)
	if zoom < 2.0:
		await _near(camera, _centre_of(camp_points), 2.0)
		await _save_m1("m3_01b_rooms_camp_2x.png", world)
	await _key(KEY_R)
	# m3_02: the dug pocket close up (dug rock floor), overlay off.
	if pocket >= 0:
		await _near(camera, rooms[pocket]["centre"], 3.0)
		await _save_m1("m3_02_pocket_3x.png", world)
	else:
		push_error("capture: no dug pocket in this world")
	# m3_04: the inspector on a person whose bed is in the bedroom.
	var snap: Dictionary = world.call("get_agent_snapshot")
	var bits: PackedInt64Array = snap.get("ids", PackedInt64Array())
	var agent_ids: PackedInt64Array = snap.get("agent_ids", PackedInt64Array())
	var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
	var pick: int = -1
	for i: int in mini(agent_ids.size(), bits.size()):
		var eco: Dictionary = world.call("get_agent_economy", agent_ids[i])
		if int(eco.get("bed_room", -1)) == bedroom:
			pick = i
			break
	if pick >= 0:
		var tile: Vector2i = Vector2i(xs[pick], ys[pick])
		await _near(camera, tile)
		var agents: Node = main.get_node_or_null("ColonyView/AgentLayer")
		var drawn: Vector2 = Vector2(tile) * TILE
		if agents != null:
			var d: Vector2 = agents.call("drawn_position", bits[pick])
			if d.x >= 0.0:
				drawn = d
		tool.call("click_world", drawn + Vector2(8, 8), tile)
		for _i: int in 30:
			await process_frame
		var bed_label: Label = inspector.get("_bed_label") as Label
		var scroll: ScrollContainer = inspector.get("_scroll") as ScrollContainer
		if scroll != null and bed_label != null:
			scroll.scroll_vertical = maxi(0, int(bed_label.position.y) - 240)
			for _i: int in 10:
				await process_frame
		_record["inspected"] = {"agent": agent_ids[pick], "tile": [tile.x, tile.y],
			"bed_line": bed_label.text if bed_label != null else "", "bed_visible": bed_label != null and bed_label.visible}
		await _save_m1("m3_04_inspector_bed.png", world)
		inspector.call("close")
	else:
		push_error("capture: nobody has a bed in bedroom %d" % bedroom)
	# m3_05: the mouse over a stack inside the storehouse.
	var overlay: Dictionary = world.call("get_colony_overlay")
	var ixs: PackedInt32Array = overlay.get("item_xs", PackedInt32Array())
	var iys: PackedInt32Array = overlay.get("item_ys", PackedInt32Array())
	var stack: Vector2i = Vector2i(-1, -1)
	for i: int in mini(ixs.size(), iys.size()):
		if int(room_layer.call("room_at", Vector2i(ixs[i], iys[i]))) == store:
			stack = Vector2i(ixs[i], iys[i])
			break
	if stack.x >= 0:
		await _near(camera, rooms[store]["centre"], 3.0)
		var at: Vector2 = _screen_of(stack)
		await _motion(at + Vector2(TILE, 0.0))
		Input.warp_mouse(at)
		await _motion(at)
		for _i: int in 10:
			await process_frame
		_record["hover"] = {"tile": [stack.x, stack.y], "screen": [roundi(at.x), roundi(at.y)],
			"suffix": OverlaySuffix.room_suffix(room_layer, stack, true, root.get_node_or_null("Locale"))}
		await _save_m1("m3_05_hover_store_3x.png", world)
	else:
		push_error("capture: no stack in storehouse %d" % store)
	# m3_03: F3, 5 (log wall), a drag held from outside the bedroom across its west wall (refused: built).
	var cells: Array = rooms[bedroom]["cells"]
	var west: Vector2i = rooms[bedroom]["centre"]
	for c: Vector2i in cells:
		if c.y == west.y and c.x < west.x:
			west = c
	var from: Vector2i = west + Vector2i(-6, 0)
	var to: Vector2i = west + Vector2i(1, 0)
	await _near(camera, _mid(from, to))
	await _key(KEY_F3)
	await _key(KEY_5)
	await _motion(_screen_of(from))
	await _button(_screen_of(from), MOUSE_BUTTON_LEFT, true)
	for x: int in range(from.x + 1, to.x + 1):
		await _motion(_screen_of(Vector2i(x, from.y)))
	for _i: int in 10:
		await process_frame
	var preview: Dictionary = tool.call("preview")
	var refused: int = int(tool.call("refused_count", preview))
	_record["drag"] = {"from": [from.x, from.y], "to": [to.x, to.y], "kind": str(tool.get("build_kind")), "mode": int(tool.get("mode")),
		"tiles": (preview.get("xs", PackedInt32Array()) as PackedInt32Array).size(), "refused": refused,
		"reasons": Array(preview.get("reasons", PackedStringArray()))}
	await _save_m1("m3_03_build_drag.png", world)
	await _button(_screen_of(to), MOUSE_BUTTON_RIGHT, true)
	await _button(_screen_of(to), MOUSE_BUTTON_RIGHT, false)
	await _button(_screen_of(to), MOUSE_BUTTON_LEFT, false)
	Input.warp_mouse(Vector2(1919, 1079))
	await _motion(Vector2(1919, 1079))
	_record["drag_after_cancel_mode"] = int(tool.get("mode"))
	# m3_07: far, 1×, the camp centred.
	await _near(camera, _centre_of(camp_points), 1.0)
	await _save_m1("m3_07_far_1x.png", world)
	# m3_08: the next 23:00, the bedroom at 2×.
	var guard: int = 0
	while _minute_of_day(world) != 23 * 60 and guard < 1440:
		world.call("advance_ticks", 1)
		guard += 1
	for _i: int in 5:
		await process_frame
	var sleepers: int = 0
	var after: Dictionary = world.call("get_agent_snapshot")
	var axs: PackedInt32Array = after.get("xs", PackedInt32Array())
	var ays: PackedInt32Array = after.get("ys", PackedInt32Array())
	var states: PackedByteArray = after.get("states", PackedByteArray())
	var targets: PackedByteArray = after.get("target_kinds", PackedByteArray())
	for i: int in mini(axs.size(), mini(ays.size(), mini(states.size(), targets.size()))):
		if states[i] == 3 and targets[i] == 3 and int(room_layer.call("room_at", Vector2i(axs[i], ays[i]))) == bedroom:
			sleepers += 1
	_record["night"] = {"tick": _now(world), "asleep_in_bedroom": sleepers}
	await _near(camera, rooms[bedroom]["centre"], 3.0)
	await _save_m1("m3_08_night_3x.png", world)
	var file: FileAccess = FileAccess.open(_out.path_join("m3_set.json"), FileAccess.WRITE)
	if file != null:
		file.store_string(JSON.stringify(_record, "  ", true))
		file.close()
	print("CAPTURE_M3_SET_DONE ", JSON.stringify({"picked": _record.get("picked"), "drag": _record.get("drag"), "night": _record.get("night"),
		"inspected": _record.get("inspected"), "hover": _record.get("hover"), "room_panel": _record.get("room_panel"), "role_set": _record.get("role_set")}))
	quit(0)


# --- --m4-set (M4) -------------------------------------------------------------------------------------------

## A left click at `at` (screen point) as a player gives it: motion, press, release.
func _click(at: Vector2) -> void:
	await _motion(at)
	await _button(at, MOUSE_BUTTON_LEFT, true)
	await _button(at, MOUSE_BUTTON_LEFT, false)


func _wait(frames: int) -> void:
	for _i: int in frames:
		await process_frame


## Node ids by status (0 미지, 1 위험, 2 알려짐, 3 안정, 4 소실) of a get_tech read.
func _by_status(tech: Dictionary) -> Dictionary:
	var out: Dictionary = {0: [], 1: [], 2: [], 3: [], 4: []}
	var ids: PackedInt64Array = tech.get("ids", PackedInt64Array())
	var statuses: PackedInt64Array = tech.get("statuses", PackedInt64Array())
	for i: int in mini(ids.size(), statuses.size()):
		if out.has(int(statuses[i])):
			(out[int(statuses[i])] as Array).append(int(ids[i]))
	return out


## The visible label texts under `node`.
func _labels_under(node: Node) -> Array[String]:
	var out: Array[String] = []
	if node == null:
		return out
	for label: Node in node.find_children("*", "Label", true, false):
		if (label as Label).is_visible_in_tree() and (label as Label).text != "":
			out.append((label as Label).text)
	return out


func _m4_shots(main: Node, world: Node, camera: Camera2D) -> void:
	var reads: Node = main.get_node_or_null("SimReads")
	var tool: Node = main.get_node_or_null("ColonyView/OrderTool")
	var hud: Node = main.get_node_or_null("UI/HudTopbar")
	var tech_panel: Node = main.get_node_or_null("UI/TechPanel")
	var teach_panel: Node = main.get_node_or_null("UI/TeachPanel")
	var inspector: Node = main.get_node_or_null("UI/AgentInspectorPanel")
	var room_layer: Node = main.get_node_or_null("ColonyView/OverlayLayer/Rooms")
	var chronicle: Node = main.get_node_or_null("UI/ChroniclePanel")
	var bar: Node = main.get_node_or_null("UI/OrderBar")
	if reads == null or tool == null or hud == null or tech_panel == null or teach_panel == null or inspector == null \
			or room_layer == null or chronicle == null or bar == null or _load.is_empty():
		push_error("capture: --m4-set needs --load <file>, Main/SimReads, OrderTool, the HUD, the M4 panels")
		quit(1)
		return
	camera.set_process(false)
	if not bool(reads.call("mutate_raw", "load_world", [_load])):
		push_error("capture: load_world failed for " + _load)
		quit(1)
		return
	await _wait(20)
	_record["loaded"] = _load
	_record["loaded_tick"] = _now(world)
	_record["clock_minute"] = _minute_of_day(world)
	_record["seed"] = int((world.call("get_world_info") as Dictionary).get("seed", -1))
	var hearth: Vector2i = _hearth(world)
	_record["hearth"] = [hearth.x, hearth.y]
	var tech: Dictionary = world.call("get_tech")
	var by_status: Dictionary = _by_status(tech)
	_record["tech"] = {"era": int(tech.get("era", -1)), "progress": int(tech.get("progress", -1)), "at_risk": int(tech.get("at_risk", -1)),
		"by_status": by_status}
	var notes: Dictionary = {}
	# The overlay's hover follows the OS cursor (see --m3-set): parked in a corner except for m4_06.
	Input.warp_mouse(Vector2(1919, 1079))
	# m4_01: the camp at 2×, the HUD's era line and badge.
	await _near(camera, hearth)
	notes["m4_01_hud_era_2x.png"] = {"era": str(hud.call("era_text")), "badge": str(hud.call("risk_text")), "warning": str(hud.call("warning_text"))}
	await _save_m1("m4_01_hud_era_2x.png", world)
	# m4_02: T, the tech map.
	await _key(KEY_T)
	await _wait(10)
	var shown: Dictionary = {}
	for id: int in 24:
		var c: Button = tech_panel.call("card", id)
		if c != null and c.is_visible_in_tree():
			var s: String = str(int(c.get_meta("status", -1)))
			shown[s] = int(shown.get(s, 0)) + 1
	notes["m4_02_tech_map.png"] = {"visible": (tech_panel as Control).visible, "cards_by_status": shown}
	await _save_m1("m4_02_tech_map.png", world)
	# m4_03: a mouse click on the first at-risk card (else the first known or stable one): the detail.
	var pick_node: int = -1
	for s: int in [1, 2, 3]:
		if pick_node < 0 and not (by_status[s] as Array).is_empty():
			pick_node = int((by_status[s] as Array)[0])
	var card: Button = tech_panel.call("card", pick_node) if pick_node >= 0 else null
	if card != null:
		await _click(card.get_global_rect().get_center())
		await _wait(10)
		if int(tech_panel.call("selected_node")) != pick_node:
			push_error("capture: the card click did not select node %d" % pick_node)
	var detail: Dictionary = tech_panel.call("detail")
	var detail_box: Node = (tech_panel as Control).find_child("Detail", true, false)
	notes["m4_03_tech_node.png"] = {"node": int(tech_panel.call("selected_node")), "holders": Array(detail.get("holder_ids", PackedInt64Array())),
		"learners": (detail.get("learner_ids", PackedInt64Array()) as PackedInt64Array).size(), "lines": _labels_under(detail_box)}
	await _save_m1("m4_03_tech_node.png", world)
	# m4_04: 가르치기 opens the teach dialog on the node; the first teacher and learner, then a click on 명령.
	var teach_button: Button = detail_box.find_child("Teach", true, false) as Button if detail_box != null else null
	if teach_button != null and teach_button.is_visible_in_tree():
		await _click(teach_button.get_global_rect().get_center())
		await _wait(10)
	var listed: Dictionary = teach_panel.call("listed")
	var order: Dictionary = {}
	if (listed["teachers"] as Array).size() > 0 and (listed["learners"] as Array).size() > 0:
		order = {"teacher": int((listed["teachers"] as Array)[0]), "learner": int((listed["learners"] as Array)[0])}
		teach_panel.call("choose_people", int(order["teacher"]), int(order["learner"]))
		var order_button: Button = (teach_panel as Control).find_child("Order", true, false) as Button
		if order_button != null:
			await _click(order_button.get_global_rect().get_center())
			await _wait(10)
	var toast: Label = bar.get("_toast") as Label
	notes["m4_04_teach_dialog.png"] = {"visible": (teach_panel as Control).visible, "node": int(teach_panel.call("current_node")), "listed": listed,
		"order": order, "order_rows": int(teach_panel.call("order_rows")), "toast": toast.text if toast != null else ""}
	await _save_m1("m4_04_teach_dialog.png", world)
	await _key(KEY_K)
	await _key(KEY_T)
	await _wait(5)
	# m4_05: a click on the person with the most kinds of knowledge source (fewest entries among equals, so the list
	# fits the panel), then on the inspector's 지식 tab.
	var snap: Dictionary = world.call("get_agent_snapshot")
	var bits: PackedInt64Array = snap.get("ids", PackedInt64Array())
	var agent_ids: PackedInt64Array = snap.get("agent_ids", PackedInt64Array())
	var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
	var best: int = -1
	var best_kinds: int = -1
	var best_entries: int = 0
	var source_kinds: Dictionary = {}
	for i: int in mini(agent_ids.size(), mini(bits.size(), mini(xs.size(), ys.size()))):
		var read: Dictionary = world.call("get_agent_knowledge", agent_ids[i])
		var kinds: Dictionary = {}
		var sources: PackedInt64Array = read.get("sources", PackedInt64Array())
		for src: int in sources:
			kinds[src] = true
		source_kinds[str(agent_ids[i])] = kinds.keys()
		# Most kinds of source first; among equals one with a 도제 (apprenticed, 4) entry, then fewer entries.
		var score: int = kinds.size() * 2 + (1 if kinds.has(4) else 0)
		if score > best_kinds or (score == best_kinds and sources.size() < best_entries):
			best = i
			best_kinds = score
			best_entries = sources.size()
	_record["source_kinds_by_agent"] = source_kinds
	if best >= 0:
		var tile: Vector2i = Vector2i(xs[best], ys[best])
		await _near(camera, tile)
		var agents: Node = main.get_node_or_null("ColonyView/AgentLayer")
		var drawn: Vector2 = Vector2(tile) * TILE
		if agents != null:
			var d: Vector2 = agents.call("drawn_position", bits[best])
			if d.x >= 0.0:
				drawn = d
		tool.call("click_world", drawn + Vector2(8, 8), tile)
		await _wait(20)
		var tab: Button = (inspector as Control).find_child("Tab_1", true, false) as Button
		if tab != null:
			await _click(tab.get_global_rect().get_center())
		await _wait(20)
		var read_best: Dictionary = world.call("get_agent_knowledge", agent_ids[best])
		notes["m4_05_inspector_knowledge.png"] = {"agent": agent_ids[best], "tile": [tile.x, tile.y], "score": best_kinds,
			"sources": Array(read_best.get("sources", PackedInt64Array())), "tab": int(inspector.call("current_tab")),
			"lines": inspector.call("knowledge_texts")}
		await _save_m1("m4_05_inspector_knowledge.png", world)
		# m4_05b: the same list scrolled down to the first 관찰 (3) or 도제 (4) entry, placed about 240 px below the top
		# so the entries above it show too (else to the end).
		var scroll: ScrollContainer = inspector.get("_scroll") as ScrollContainer
		var box: Control = inspector.get("_knowledge_box") as Control
		if scroll != null and box != null:
			var entry: int = -1
			var src_list: PackedInt64Array = read_best.get("sources", PackedInt64Array())
			for k: int in src_list.size():
				if entry < 0 and src_list[k] >= 3:
					entry = k
			var target: int = int(scroll.get_v_scroll_bar().max_value)
			var seen: int = 0
			for child: Node in box.get_children():
				if child is Label and child.name != "Lesson":
					if seen == entry:
						target = maxi(0, int(box.position.y + (child as Label).position.y) - 240)
					seen += 1
			scroll.scroll_vertical = target
			await _wait(10)
			notes["m4_05b_inspector_knowledge_more.png"] = {"entry": entry, "scroll": scroll.scroll_vertical, "max": int(scroll.get_v_scroll_bar().max_value)}
			await _save_m1("m4_05b_inspector_knowledge_more.png", world)
		inspector.call("select_tab", 0)
		inspector.call("close")
		await _wait(5)
	# m4_06: the mouse over a fired clay pot stack (item kind 9), in the storehouse when one lies there.
	var store: int = _room_of_role(_rooms_of(world), 2, false)
	var overlay: Dictionary = world.call("get_colony_overlay")
	var ixs: PackedInt32Array = overlay.get("item_xs", PackedInt32Array())
	var iys: PackedInt32Array = overlay.get("item_ys", PackedInt32Array())
	var iks: PackedInt32Array = overlay.get("item_kinds", PackedInt32Array())
	var ics: PackedInt32Array = overlay.get("item_counts", PackedInt32Array())
	var pot: Vector2i = Vector2i(-1, -1)
	var pot_in_store: bool = false
	var pots: Array = []
	for i: int in mini(ixs.size(), mini(iys.size(), iks.size())):
		if iks[i] != 9:
			continue
		var t: Vector2i = Vector2i(ixs[i], iys[i])
		var room: int = int(room_layer.call("room_at", t))
		pots.append({"tile": [t.x, t.y], "count": ics[i] if i < ics.size() else -1, "room": room})
		if pot.x < 0 or (not pot_in_store and store >= 0 and room == store):
			pot = t
			pot_in_store = store >= 0 and room == store
	if pot.x >= 0:
		await _near(camera, pot, 3.0)
		var at: Vector2 = _screen_of(pot)
		await _motion(at + Vector2(TILE, 0.0))
		Input.warp_mouse(at)
		await _motion(at)
		await _wait(10)
		notes["m4_06_pots_store_3x.png"] = {"tile": [pot.x, pot.y], "in_storehouse": pot_in_store, "storehouse": store, "pots": pots,
			"screen": [roundi(at.x), roundi(at.y)], "suffix": OverlaySuffix.room_suffix(room_layer, pot, true, root.get_node_or_null("Locale"))}
		await _save_m1("m4_06_pots_store_3x.png", world)
		Input.warp_mouse(Vector2(1919, 1079))
		await _motion(Vector2(1919, 1079))
	else:
		push_error("capture: no clay pot stack in this world")
	# m4_07: the river's fords at 3× (get_space ford_xs/ford_ys, framed on their centre).
	var space: Dictionary = world.call("get_space")
	var fxs: PackedInt32Array = space.get("ford_xs", PackedInt32Array())
	var fys: PackedInt32Array = space.get("ford_ys", PackedInt32Array())
	var fords: Array[Vector2i] = []
	var ford_rows: Array = []
	for i: int in mini(fxs.size(), fys.size()):
		fords.append(Vector2i(fxs[i], fys[i]))
		ford_rows.append([fxs[i], fys[i]])
	if not fords.is_empty():
		await _near(camera, _centre_of(fords), 3.0)
		notes["m4_07_ford_3x.png"] = {"fords": ford_rows}
		await _save_m1("m4_07_ford_3x.png", world)
	else:
		push_error("capture: no ford in this world")
	# m4_08: far, 1×, over the camp and the clay banks (node kind 2) within 40 tiles and the clay-pit marks.
	var points: Array[Vector2i] = [hearth]
	var nxs: PackedInt32Array = overlay.get("node_xs", PackedInt32Array())
	var nys: PackedInt32Array = overlay.get("node_ys", PackedInt32Array())
	var nks: PackedInt32Array = overlay.get("node_kinds", PackedInt32Array())
	var banks: Array = []
	for i: int in mini(nxs.size(), mini(nys.size(), nks.size())):
		if nks[i] == 2 and _cheb(Vector2i(nxs[i], nys[i]), hearth) <= 40:
			points.append(Vector2i(nxs[i], nys[i]))
			banks.append([nxs[i], nys[i]])
	var dks: PackedInt32Array = overlay.get("designation_kinds", PackedInt32Array())
	var pits: int = 0
	for k: int in dks:
		if k == 4:
			pits += 1
	var far: Vector2i = _centre_of(points)
	await _near(camera, far, 1.0)
	notes["m4_08_far_1x.png"] = {"centre": [far.x, far.y], "clay_banks_near": banks, "clay_pit_marks": pits}
	await _save_m1("m4_08_far_1x.png", world)
	# m4_09: C, the chronicle at the camp (2×).
	await _near(camera, hearth)
	await _key(KEY_C)
	await _wait(20)
	var page: Dictionary = world.call("get_chronicle_since", 0, 4096)
	var m4_kinds: Dictionary = {}
	for k: int in page.get("kind", PackedInt32Array()):
		if k >= 24:
			m4_kinds[str(k)] = int(m4_kinds.get(str(k), 0)) + 1
	var body: RichTextLabel = chronicle.get("_body") as RichTextLabel
	notes["m4_09_chronicle.png"] = {"visible": (chronicle as Control).visible, "m4_kinds": m4_kinds,
		"text_tail": body.get_parsed_text().right(800) if body != null else ""}
	await _save_m1("m4_09_chronicle.png", world)
	await _key(KEY_C)
	_record["notes"] = notes
	var file: FileAccess = FileAccess.open(_out.path_join("m4_set.json"), FileAccess.WRITE)
	if file != null:
		file.store_string(JSON.stringify(_record, "  ", true))
		file.close()
	print("CAPTURE_M4_SET_DONE ", JSON.stringify({"tick": _record.get("loaded_tick"), "tech": _record.get("tech"), "notes": notes}))
	quit(0)


# --- --m5-set (M5, goal §10) ---------------------------------------------------------------------------------

## Advance the stopped clock `step` ticks at a time until `done` returns true or `limit` ticks passed; true when done.
func _advance_until(world: Node, done: Callable, step: int, limit: int) -> bool:
	var spent: int = 0
	while spent <= limit:
		if bool(done.call()):
			return true
		world.call("advance_ticks", step)
		spent += step
		await process_frame
	return bool(done.call())


## The drawn top-left of the person with AgentId `agent` (its tile when not drawn), and its tile: {drawn, tile, bits}.
func _person_at(main: Node, world: Node, agent: int) -> Dictionary:
	var snap: Dictionary = world.call("get_agent_snapshot")
	var ids: PackedInt64Array = snap.get("agent_ids", PackedInt64Array())
	var bits: PackedInt64Array = snap.get("ids", PackedInt64Array())
	var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
	var i: int = ids.find(agent)
	if i < 0:
		return {}
	var tile: Vector2i = Vector2i(xs[i], ys[i])
	var drawn: Vector2 = Vector2(tile) * TILE
	var agents: Node = main.get_node_or_null("ColonyView/AgentLayer")
	if agents != null:
		var d: Vector2 = agents.call("drawn_position", bits[i])
		if d.x >= 0.0:
			drawn = d
	return {"drawn": drawn, "tile": tile, "bits": bits[i]}


## A click on person `agent` as a player gives it (camera on them first when `frame`); true when the inspector shows them.
func _click_person(main: Node, world: Node, camera: Camera2D, agent: int, zoom: float, frame: bool = true) -> bool:
	var at: Dictionary = _person_at(main, world, agent)
	if at.is_empty():
		return false
	if frame:
		await _near(camera, at["tile"], zoom)
		at = _person_at(main, world, agent)
	await _click(root.get_canvas_transform() * ((at["drawn"] as Vector2) + Vector2(8, 8)))
	await _wait(5)
	var inspector: Node = main.get_node_or_null("UI/AgentInspectorPanel")
	return inspector != null and (inspector as Control).visible and int(inspector.get("_selected_person")) == agent


## The shot's note: the save, the tick, the clock and `extra`.
func _m5_note(notes: Dictionary, shot: String, world: Node, extra: Dictionary) -> void:
	var note: Dictionary = extra.duplicate()
	note["save"] = _load.get_file()
	note["tick"] = _now(world)
	note["clock_minute"] = _minute_of_day(world)
	note["display_day"] = floori(float(_now(world) + int((world.call("get_world_info") as Dictionary).get("clock_offset_ticks", 0))) / 1440.0) + 1
	notes[shot] = note


func _m5_shots_run(main: Node, world: Node, camera: Camera2D) -> void:
	var reads: Node = main.get_node_or_null("SimReads")
	var banner: Node = main.get_node_or_null("UI/StoryBanner")
	var inspector: Node = main.get_node_or_null("UI/AgentInspectorPanel")
	var chronicle: Node = main.get_node_or_null("UI/ChroniclePanel")
	if reads == null or banner == null or inspector == null or chronicle == null or _load.is_empty():
		push_error("capture: --m5-set needs --load <file>, Main/SimReads, the story banner, the inspector, the chronicle")
		quit(1)
		return
	camera.set_process(false)
	if not bool(reads.call("mutate_raw", "load_world", [_load])):
		push_error("capture: load_world failed for " + _load)
		quit(1)
		return
	await _wait(20)
	var path: String = _out.path_join("m5_set.json")
	var previous: Variant = JSON.parse_string(FileAccess.get_file_as_string(path)) if FileAccess.file_exists(path) else null
	_record = previous if previous is Dictionary else {}
	var runs: Array = _record.get("runs", [])
	runs.append({"save": _load, "loaded_tick": _now(world), "shots": Array(_m5_shots)})
	_record["runs"] = runs
	var notes: Dictionary = _record.get("notes", {})
	Input.warp_mouse(Vector2(1919, 1079))
	var hearth: Vector2i = _hearth(world)
	for shot: String in _m5_shots:
		match shot:
			"1": await _m5_wolves(main, world, camera, banner, hearth, notes)
			"2": await _m5_draft(main, world, camera, banner, hearth, notes)
			"3": await _m5_mood(main, world, camera, banner, hearth, notes)
			"4": await _m5_why(main, world, camera, chronicle, hearth, notes)
			"5": await _m5_cold(world, camera, banner, hearth, notes)
			"6": await _m5_story_tab(main, world, camera, inspector, notes)
			"7": await _m5_hunt(main, world, camera, hearth, notes)
			"8": await _m5_wrap(main, world, camera, inspector, notes)
	_record["notes"] = notes
	var file: FileAccess = FileAccess.open(path, FileAccess.WRITE)
	if file != null:
		file.store_string(JSON.stringify(_record, "  ", true))
		file.close()
	print("CAPTURE_M5_SET_DONE ", JSON.stringify({"save": _load.get_file(), "shots": Array(_m5_shots), "notes": notes.keys()}))
	quit(0)


func _wolves(world: Node) -> Array[Vector2i]:
	var read: Dictionary = world.call("get_animals")
	var out: Array[Vector2i] = []
	var kinds: PackedInt32Array = read.get("kinds", PackedInt32Array())
	var xs: PackedInt32Array = read.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = read.get("ys", PackedInt32Array())
	for i: int in mini(kinds.size(), mini(xs.size(), ys.size())):
		if kinds[i] == 0:
			out.append(Vector2i(xs[i], ys[i]))
	return out


## Frame `points` at 2× when they fit 1920×1080 (with a margin), else at 1×.
func _frame_points(camera: Camera2D, points: Array[Vector2i]) -> float:
	var span: Vector2i = _span(points)
	var zoom: float = 2.0 if span.x * TILE * 2.0 < 1700.0 and span.y * TILE * 2.0 < 860.0 else 1.0
	await _near(camera, _centre_of(points), zoom)
	return zoom


func _m5_wolves(_main: Node, world: Node, camera: Camera2D, banner: Node, hearth: Vector2i, notes: Dictionary) -> void:
	var arrived: bool = await _advance_until(world, func() -> bool: return not _wolves(world).is_empty(), 60, 4 * 1440)
	if not arrived:
		push_error("capture: m5_01 no wolves within 4 days of the save")
		return
	await _advance_until(world, func() -> bool:
		for w: Vector2i in _wolves(world):
			if _cheb(w, hearth) <= 30:
				return true
		return false, 30, 360)
	var points: Array[Vector2i] = [hearth]
	points.append_array(_wolves(world))
	var zoom: float = await _frame_points(camera, points)
	await _wait(10)
	var story: Dictionary = world.call("get_story")
	_m5_note(notes, "m5_01_wolves_banner.png", world, {"banner": banner.call("banner_texts"), "wolves": int(story.get("wolves", 0)),
		"wolves_at": [int(story.get("wolves_x", -1)), int(story.get("wolves_y", -1))], "pulse": str(banner.call("pulse")), "zoom": zoom})
	await _save_m1("m5_01_wolves_banner.png", world)


func _m5_draft(main: Node, world: Node, camera: Camera2D, banner: Node, hearth: Vector2i, notes: Dictionary) -> void:
	if _wolves(world).is_empty():
		await _advance_until(world, func() -> bool: return not _wolves(world).is_empty(), 60, 4 * 1440)
	var snap: Dictionary = world.call("get_agent_snapshot")
	var ids: PackedInt64Array = snap.get("agent_ids", PackedInt64Array())
	var stages: PackedByteArray = snap.get("life_stages", PackedByteArray())
	var picks: Array[int] = []
	var others: Array[int] = []
	for i: int in mini(ids.size(), stages.size()):
		if stages[i] != 0:
			continue
		var economy: Dictionary = world.call("get_agent_economy", ids[i])
		if int(economy.get("tool_kind", -1)) == 15:
			picks.append(ids[i])
		else:
			others.append(ids[i])
	picks.append_array(others)
	picks = picks.slice(0, 4)
	var tool: Node = main.get_node_or_null("ColonyView/OrderTool")
	var inspector: Node = main.get_node_or_null("UI/AgentInspectorPanel")
	for agent: int in picks:
		if await _click_person(main, world, camera, agent, 2.0):
			await _key(KEY_D)
			await _wait(3)
	inspector.call("close")
	await _wait(3)
	var wolves: Array[Vector2i] = _wolves(world)
	var target: Vector2i = hearth
	if not wolves.is_empty():
		wolves.sort_custom(func(a: Vector2i, b: Vector2i) -> bool: return _cheb(a, hearth) < _cheb(b, hearth))
		target = wolves[0]
	await _near(camera, target, 2.0)
	var animals: Node = main.get_node_or_null("ColonyView/AnimalLayer")
	var read: Dictionary = world.call("get_animals")
	var wolf_id: int = -1
	var xs: PackedInt32Array = read.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = read.get("ys", PackedInt32Array())
	var aids: PackedInt64Array = read.get("ids", PackedInt64Array())
	for i: int in mini(aids.size(), mini(xs.size(), ys.size())):
		if Vector2i(xs[i], ys[i]) == target:
			wolf_id = aids[i]
	if wolf_id >= 0 and animals != null:
		var drawn: Vector2 = animals.call("drawn_position", wolf_id)
		var at: Vector2 = root.get_canvas_transform() * (drawn + Vector2(8, 8))
		await _motion(at)
		await _button(at, MOUSE_BUTTON_RIGHT, true)
		await _button(at, MOUSE_BUTTON_RIGHT, false)
	else:
		tool.call("draft_order_at", Vector2(target) * TILE + Vector2(8, 8), target)
	world.call("advance_ticks", 40)
	await _wait(10)
	var drafted: Dictionary = world.call("get_drafted")
	var points: Array[Vector2i] = []
	var dxs: PackedInt32Array = drafted.get("xs", PackedInt32Array())
	var dys: PackedInt32Array = drafted.get("ys", PackedInt32Array())
	for i: int in mini(dxs.size(), dys.size()):
		points.append(Vector2i(dxs[i], dys[i]))
	points.append_array(_wolves(world))
	if points.is_empty():
		points.append(hearth)
	var zoom: float = await _frame_points(camera, points)
	await _wait(10)
	_m5_note(notes, "m5_02_drafted_party.png", world, {"drafted": Array(drafted.get("ids", PackedInt64Array())), "weapons": Array(drafted.get("weapons", PackedInt32Array())),
		"orders": Array(drafted.get("orders", PackedInt32Array())), "wolf": wolf_id, "rows": banner.call("drafted_texts"), "zoom": zoom})
	await _save_m1("m5_02_drafted_party.png", world)


func _m5_mood(main: Node, world: Node, camera: Camera2D, banner: Node, hearth: Vector2i, notes: Dictionary) -> void:
	# A moment when someone is not 보통 (우울, 흡족 or in a breakdown), within three days; framed on them with the camp.
	var marked: Array[Vector2i] = []
	var mixed: Callable = func() -> bool:
		var snap: Dictionary = world.call("get_agent_snapshot")
		var bands: PackedByteArray = snap.get("mood_bands", PackedByteArray())
		var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
		var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
		marked.clear()
		for i: int in mini(bands.size(), mini(xs.size(), ys.size())):
			if bands[i] == 1 or bands[i] == 3 or bands[i] == 4:
				marked.append(Vector2i(xs[i], ys[i]))
		return not marked.is_empty()
	await _advance_until(world, mixed, 60, 3 * 1440)
	var points: Array[Vector2i] = [hearth]
	points.append_array(marked)
	await _frame_points(camera, points)
	await _key(KEY_M)
	await _wait(10)
	var snap: Dictionary = world.call("get_agent_snapshot")
	var bands: Dictionary = {}
	for b: int in snap.get("mood_bands", PackedByteArray()):
		bands[str(b)] = int(bands.get(str(b), 0)) + 1
	var agents: Node = main.get_node_or_null("ColonyView/AgentLayer")
	_m5_note(notes, "m5_03_mood_overlay.png", world, {"bands": bands, "shown": agents != null and bool(agents.call("mood_shown")),
		"banner": banner.call("banner_texts")})
	await _save_m1("m5_03_mood_overlay.png", world)
	await _key(KEY_M)


func _m5_why(_main: Node, world: Node, camera: Camera2D, chronicle: Node, hearth: Vector2i, notes: Dictionary) -> void:
	var page: Dictionary = world.call("get_chronicle_since", 0, 512)
	var total: int = int(page.get("total", 0))
	var rows: Array[Dictionary] = []
	var start: int = 0
	while start < total:
		var part: Dictionary = world.call("get_chronicle_since", start, 512)
		rows.append_array(ChronicleText.rows(part))
		var next: int = int(part.get("next", total))
		if next <= start:
			break
		start = next
	var pick: Dictionary = {}
	for row: Dictionary in rows:
		if int(row.get("kind", -1)) == 19 and int(row.get("cause", 0)) >= 1 and int(row.get("link", -1)) >= 0:
			pick = row
	if pick.is_empty():
		push_error("capture: m5_04 no quarrel with a cause in this save")
		return
	await _near(camera, hearth, 2.0)
	await _key(KEY_C)
	await _wait(10)
	var body: RichTextLabel = chronicle.get("_body") as RichTextLabel
	body.meta_clicked.emit("why:%d" % int(pick["index"]))
	await _wait(10)
	var why: Node = chronicle.get_parent().get_node_or_null("WhyPanel")
	_m5_note(notes, "m5_04_quarrel_why.png", world, {"entry": int(pick["index"]), "entry_tick": int(pick["tick"]), "cause": int(pick["cause"]),
		"chain": str(why.call("plain_text")) if why != null else "", "links": Array((world.call("get_chronicle_why", int(pick["index"])) as Dictionary).get("links", PackedInt32Array()))})
	await _save_m1("m5_04_quarrel_why.png", world)
	if why != null:
		why.call("close")
	await _key(KEY_C)


func _m5_cold(world: Node, camera: Camera2D, banner: Node, hearth: Vector2i, notes: Dictionary) -> void:
	var cold: Callable = func() -> bool: return ((world.call("get_story") as Dictionary).get("kinds", PackedInt32Array()) as PackedInt32Array).has(0)
	if not await _advance_until(world, cold, 60, 4 * 1440):
		push_error("capture: m5_05 no cold snap within 4 days of the save")
		return
	# Its second day when the snap lasts that long ("한파 2일째"), in the evening when the fire burns.
	var second: Callable = func() -> bool:
		var story: Dictionary = world.call("get_story")
		var kinds: PackedInt32Array = story.get("kinds", PackedInt32Array())
		var days: PackedInt64Array = story.get("days_of", PackedInt64Array())
		var i: int = kinds.find(0)
		return i < 0 or (i < days.size() and days[i] >= 2 and _minute_of_day(world) >= 17 * 60)
	await _advance_until(world, second, 30, 2 * 1440)
	if not bool(cold.call()):
		push_error("capture: m5_05 the cold snap ended before its second evening")
	await _near(camera, hearth, 2.0)
	await _wait(10)
	var story: Dictionary = world.call("get_story")
	_m5_note(notes, "m5_05_cold_snap_banner.png", world, {"banner": banner.call("banner_texts"), "days_of": Array(story.get("days_of", PackedInt64Array())),
		"firewood": int(story.get("firewood", -1)), "firewood_day_need": int(story.get("firewood_day_need", -1))})
	await _save_m1("m5_05_cold_snap_banner.png", world)


func _m5_story_tab(main: Node, world: Node, camera: Camera2D, inspector: Node, notes: Dictionary) -> void:
	var snap: Dictionary = world.call("get_agent_snapshot")
	var ids: PackedInt64Array = snap.get("agent_ids", PackedInt64Array())
	var best: int = -1
	var best_score: int = -1
	for id: int in ids:
		var read: Dictionary = world.call("get_person_chronicle", id)
		var kinds: PackedInt32Array = read.get("kind", PackedInt32Array())
		var links: PackedInt32Array = read.get("link", PackedInt32Array())
		var linked: int = 0
		for l: int in links:
			if l >= 0:
				linked += 1
		# Lines that fit the tab first (8–14), then the most with a "→ 왜?".
		var score: int = linked * 4 + mini(kinds.size(), 14) - maxi(0, kinds.size() - 14)
		if score > best_score:
			best = id
			best_score = score
	if best < 0 or not await _click_person(main, world, camera, best, 2.0):
		push_error("capture: m5_06 could not select a person")
		return
	var tab: Button = (inspector as Control).find_child("Tab_2", true, false) as Button
	if tab != null:
		await _click(tab.get_global_rect().get_center())
	await _wait(15)
	_m5_note(notes, "m5_06_story_tab.png", world, {"agent": best, "tab": int(inspector.call("current_tab")), "text": str(inspector.call("story_text")).left(600)})
	await _save_m1("m5_06_story_tab.png", world)
	inspector.call("select_tab", 0)
	inspector.call("close")


func _m5_hunt(main: Node, world: Node, camera: Camera2D, hearth: Vector2i, notes: Dictionary) -> void:
	# A new deer hunt whose meat lies in the open (a stack under a crown is hidden by the canopy, as logs are), within
	# three days; else the first new hunt; else the latest hunt before the save.
	var trees: Node = main.get_node_or_null("ColonyView/TreeLayer")
	var cursor: Array[int] = [int((world.call("get_chronicle_since", 0, 1) as Dictionary).get("total", 0))]
	var hunted: Array[Dictionary] = []
	var picked: Array[Dictionary] = []
	var found: Callable = func() -> bool:
		var page: Dictionary = world.call("get_chronicle_since", cursor[0], 512)
		cursor[0] = int(page.get("next", cursor[0]))
		for row: Dictionary in ChronicleText.rows(page):
			if int(row.get("kind", -1)) != 31 or int(row.get("cause", -1)) != 1:
				continue
			hunted.append(row)
			var meat: Vector2i = _meat_near(world, _person_at(main, world, int(row.get("subject", -1))).get("tile", hearth))
			if meat.x >= 0 and (trees == null or not bool(trees.call("under_canopy", meat))):
				picked.append(row)
			return true
		return false
	var fresh: bool = await _advance_until(world, found, 30, 3 * 1440)
	if not picked.is_empty():
		hunted.clear()
		hunted.append(picked[0])
	if hunted.is_empty():
		# No new hunt (the live world has no scripted orders after the save): the latest hunt before it.
		var total: int = cursor[0]
		var start: int = 0
		while start < total:
			var part: Dictionary = world.call("get_chronicle_since", start, 512)
			for row: Dictionary in ChronicleText.rows(part):
				if int(row.get("kind", -1)) == 31 and int(row.get("cause", -1)) == 1:
					hunted.clear()
					hunted.append(row)
			var next: int = int(part.get("next", total))
			if next <= start:
				break
			start = next
	if hunted.is_empty():
		push_error("capture: m5_07 no deer hunted in this world")
		return
	var hunter: int = int(hunted[0].get("subject", -1))
	var near: Vector2i = _person_at(main, world, hunter).get("tile", hearth)
	var meat: Vector2i = _meat_near(world, near)
	var points: Array[Vector2i] = [near]
	if meat.x >= 0:
		points.append(meat)
	var span: Vector2i = _span(points)
	var zoom: float = 3.0 if span.x * TILE * 3.0 < 1700.0 and span.y * TILE * 3.0 < 860.0 else 2.0
	await _near(camera, _centre_of(points), zoom)
	await _wait(10)
	_m5_note(notes, "m5_07_hunted_deer.png", world, {"hunter": hunter, "hunter_tile": [near.x, near.y], "meat_tile": [meat.x, meat.y], "entry_tick": int(hunted[0].get("tick", -1)),
		"meat": int(hunted[0].get("value", 0)), "zoom": zoom, "fresh_hunt": fresh, "in_the_open": not picked.is_empty(),
		"meat_under_canopy": trees != null and meat.x >= 0 and bool(trees.call("under_canopy", meat))})
	await _save_m1("m5_07_hunted_deer.png", world)
	if meat.x >= 0 and (trees == null or not bool(trees.call("under_canopy", meat))):
		return
	# The kill's meat lies under a crown (hidden by the canopy, as logs are): m5_07b the meat stack in the open nearest
	# the camp (a hauled carcass) within a day, at 3× with the hover line.
	var found_open: Array[Vector2i] = []
	var in_open: Callable = func() -> bool:
		var overlay: Dictionary = world.call("get_colony_overlay")
		var ixs: PackedInt32Array = overlay.get("item_xs", PackedInt32Array())
		var iys: PackedInt32Array = overlay.get("item_ys", PackedInt32Array())
		var iks: PackedInt32Array = overlay.get("item_kinds", PackedInt32Array())
		found_open.clear()
		for i: int in mini(ixs.size(), mini(iys.size(), iks.size())):
			var t: Vector2i = Vector2i(ixs[i], iys[i])
			if iks[i] == 11 and (trees == null or not bool(trees.call("under_canopy", t))):
				if found_open.is_empty() or _cheb(t, hearth) < _cheb(found_open[0], hearth):
					found_open.clear()
					found_open.append(t)
		return not found_open.is_empty()
	if not await _advance_until(world, in_open, 15, 1440):
		push_error("capture: m5_07b no meat stack in the open within a day of the hunt")
		return
	var open: Vector2i = found_open[0]
	await _near(camera, open, 3.0)
	var at: Vector2 = _screen_of(open)
	await _motion(at)
	Input.warp_mouse(at)
	await _motion(at)
	await _wait(10)
	_m5_note(notes, "m5_07b_carcass_stack.png", world, {"meat_tile": [open.x, open.y], "zoom": 3.0})
	await _save_m1("m5_07b_carcass_stack.png", world)
	Input.warp_mouse(Vector2(1919, 1079))
	await _motion(Vector2(1919, 1079))


## The meat stack (item kind 11) nearest `near`, or (-1, -1).
func _meat_near(world: Node, near: Vector2i) -> Vector2i:
	var overlay: Dictionary = world.call("get_colony_overlay")
	var ixs: PackedInt32Array = overlay.get("item_xs", PackedInt32Array())
	var iys: PackedInt32Array = overlay.get("item_ys", PackedInt32Array())
	var iks: PackedInt32Array = overlay.get("item_kinds", PackedInt32Array())
	var meat: Vector2i = Vector2i(-1, -1)
	for i: int in mini(ixs.size(), mini(iys.size(), iks.size())):
		var t: Vector2i = Vector2i(ixs[i], iys[i])
		if iks[i] == 11 and _cheb(t, near) <= 4 and (meat.x < 0 or _cheb(t, near) < _cheb(meat, near)):
			meat = t
	return meat


func _m5_wrap(main: Node, world: Node, camera: Camera2D, inspector: Node, notes: Dictionary) -> void:
	var wearer: Array[int] = []
	var wearing: Callable = func() -> bool:
		var snap: Dictionary = world.call("get_agent_snapshot")
		var tops: PackedByteArray = snap.get("tops", PackedByteArray())
		var ids: PackedInt64Array = snap.get("agent_ids", PackedInt64Array())
		for i: int in mini(tops.size(), ids.size()):
			if tops[i] == 1:
				wearer.append(ids[i])
				return true
		return false
	if not await _advance_until(world, wearing, 60, 6 * 1440):
		push_error("capture: m5_08 nobody wears a hide wrap within 6 days of the save")
		return
	if not await _click_person(main, world, camera, wearer[0], 2.0):
		push_error("capture: m5_08 could not select the wearer")
		return
	await _wait(10)
	var label: Label = inspector.get("_top_label") as Label
	var scroll: ScrollContainer = inspector.get("_scroll") as ScrollContainer
	if label != null and scroll != null:
		scroll.scroll_vertical = maxi(0, int(label.position.y) - 200)
		await _wait(10)
	var wearers: int = 0
	for t: int in (world.call("get_agent_snapshot") as Dictionary).get("tops", PackedByteArray()):
		if t == 1:
			wearers += 1
	_m5_note(notes, "m5_08_hide_wrap_inspector.png", world, {"agent": wearer[0], "lines": inspector.call("m5_lines"), "wearers": wearers})
	await _save_m1("m5_08_hide_wrap_inspector.png", world)
	inspector.call("close")



# --- --m55-set (M5.5, goal §9) -------------------------------------------------------------------------------

## The shot's note: the tick, the display day and `extra`.
func _m55_note(notes: Dictionary, shot: String, world: Node, extra: Dictionary) -> void:
	var note: Dictionary = extra.duplicate()
	note["tick"] = _now(world)
	note["clock_minute"] = _minute_of_day(world)
	note["display_day"] = floori(float(_now(world) + int((world.call("get_world_info") as Dictionary).get("clock_offset_ticks", 0))) / 1440.0) + 1
	notes[shot] = note


func _m55_shots(main: Node, world: Node, camera: Camera2D) -> void:
	var reads: Node = main.get_node_or_null("SimReads")
	var migrants: Node = main.get_node_or_null("UI/MigrantPanel")
	var trade: Node = main.get_node_or_null("UI/TradePanel")
	var neighbours: Node = main.get_node_or_null("UI/NeighboursPanel")
	var tech: Node = main.get_node_or_null("UI/TechPanel")
	var chronicle: Node = main.get_node_or_null("UI/ChroniclePanel")
	var inspector: Node = main.get_node_or_null("UI/AgentInspectorPanel")
	var visitors: Node = main.get_node_or_null("ColonyView/VisitorLayer")
	if reads == null or migrants == null or trade == null or neighbours == null or tech == null or chronicle == null or inspector == null or visitors == null:
		push_error("capture: --m55-set needs Main/SimReads, the M5.5 panels, the chronicle, the inspector and ColonyView/VisitorLayer")
		quit(1)
		return
	camera.set_process(false)
	Input.warp_mouse(Vector2(1919, 1079))
	var notes: Dictionary = {}
	var hearth: Vector2i = _hearth(world)
	_m55_stock(world, reads, hearth, notes)
	var traded: bool = false
	var answered: bool = false
	var spent: int = 0
	while spent < 30 * 1440 and not (traded and answered):
		world.call("advance_ticks", 240)
		spent += 240
		await _wait(3)
		if not traded:
			for caravan: Dictionary in NeighbourText.caravans(world.call("get_neighbours")):
				if int(caravan["state"]) == NeighbourText.WAITING and not traded:
					traded = true
					await _m55_trade(world, camera, visitors, trade, caravan, notes)
		if not answered and (migrants as Control).visible:
			answered = true
			await _m55_migrants(world, camera, migrants, hearth, notes)
	if not traded:
		push_error("capture: m55_02 no caravan within 30 days")
	if not answered:
		push_error("capture: m55_01 no migrant wave within 30 days")
	await _m55_neighbours(world, camera, neighbours, hearth, notes)
	await _m55_tech(world, tech, notes)
	await _m55_flee(main, world, camera, inspector, notes)
	await _m55_eureka(world, camera, chronicle, hearth, notes)
	_record["notes"] = notes
	var file: FileAccess = FileAccess.open(_out.path_join("m55_set.json"), FileAccess.WRITE)
	if file != null:
		file.store_string(JSON.stringify(_record, "  ", true))
		file.close()
	print("CAPTURE_M55_SET_DONE ", JSON.stringify({"notes": notes.keys()}))
	quit(0)


## A 3×3 stockpile near the hearth (no water, mountain or building) and chop marks on the ten nearest trees.
func _m55_stock(world: Node, reads: Node, hearth: Vector2i, notes: Dictionary) -> void:
	var terrain: Dictionary = world.call("get_terrain_snapshot")
	var blocked: Dictionary = {}
	var b: Dictionary = world.call("get_building_snapshot")
	var bxs: PackedInt32Array = b.get("xs", PackedInt32Array())
	var bys: PackedInt32Array = b.get("ys", PackedInt32Array())
	for i: int in mini(bxs.size(), bys.size()):
		blocked[Vector2i(bxs[i], bys[i])] = true
	var start: Vector2i = hearth + Vector2i(3, 0)
	var corner: Vector2i = Vector2i(-1, -1)
	for ring: int in 30:
		for y: int in range(start.y - ring, start.y + ring + 1):
			for x: int in range(start.x - ring, start.x + ring + 1):
				if corner.x >= 0 or x < 0 or y < 0 or _cheb(Vector2i(x, y), start) != ring:
					continue
				var ok: bool = true
				for dy: int in 3:
					for dx: int in 3:
						ok = ok and _zoneable(Vector2i(x + dx, y + dy), terrain, blocked)
				if ok:
					corner = Vector2i(x, y)
	var zone: Variant = reads.call("mutate_raw", "cmd_add_stockpile", [corner.x, corner.y, corner.x + 2, corner.y + 2])
	var trees: Dictionary = world.call("get_trees")
	var txs: PackedInt32Array = trees.get("xs", PackedInt32Array())
	var tys: PackedInt32Array = trees.get("ys", PackedInt32Array())
	var ranked: Array[Vector3i] = []
	for i: int in mini(txs.size(), tys.size()):
		ranked.append(Vector3i(_cheb(Vector2i(txs[i], tys[i]), hearth), tys[i], txs[i]))
	ranked.sort()
	var cxs: PackedInt32Array = PackedInt32Array()
	var cys: PackedInt32Array = PackedInt32Array()
	for k: int in mini(10, ranked.size()):
		cxs.append(ranked[k].z)
		cys.append(ranked[k].y)
	var chop: Variant = reads.call("mutate_raw", "cmd_designate_chop_tiles", [cxs, cys])
	notes["orders"] = {"stockpile": [corner.x, corner.y, corner.x + 2, corner.y + 2], "stockpile_result": zone, "chop_result": chop}


func _m55_trade(world: Node, camera: Camera2D, visitors: Node, trade: Node, caravan: Dictionary, notes: Dictionary) -> void:
	var id: int = int(caravan["id"])
	await _near(camera, caravan["tile"], 2.0)
	var rect: Rect2 = visitors.call("drawn_rect", "caravan", id)
	await _click(root.get_canvas_transform() * rect.get_center())
	await _wait(5)
	if not (trade as Control).visible:
		push_error("capture: m55_02 the click on the caravan did not open the trade dialog")
		trade.call("open_caravan", id)
		await _wait(5)
	var goods: Array[Dictionary] = trade.call("goods_rows")
	var stock: Array[Dictionary] = trade.call("stock_rows")
	var read: Dictionary = world.call("get_caravan", id)
	var dialog: Control = trade as Control
	if not goods.is_empty() and not stock.is_empty():
		var pick: Dictionary = goods[0]
		for g: Dictionary in goods:
			if (bool(g["unknown"]) and not bool(pick["unknown"])) or (bool(g["unknown"]) == bool(pick["unknown"]) and int(g["value"]) < int(pick["value"])):
				pick = g
		(dialog.find_child("Take_%d" % int(pick["index"]), true, false).find_child("Plus", true, false) as Button).pressed.emit()
		var need: int = int(pick["value"]) * int(read.get("sell_permille", 1200))
		var given: int = 0
		for i: int in stock.size():
			var plus: Button = dialog.find_child("Give_%d" % i, true, false).find_child("Plus", true, false) as Button
			for _n: int in int(stock[i]["count"]):
				if given * int(read.get("buy_permille", 800)) >= need:
					break
				plus.pressed.emit()
				given += int(stock[i]["value"])
	await _wait(10)
	_m55_note(notes, "m55_02_trade_dialog.png", world, {"caravan": id, "texts": trade.call("texts"), "balance": trade.call("balance")})
	await _save("m55_02_trade_dialog.png")
	var button: Button = dialog.find_child("Trade", true, false) as Button
	if button != null and not button.disabled:
		await _click(button.get_global_rect().get_center())
		await _wait(5)
		notes["m55_02_trade_dialog.png"]["result"] = trade.call("texts")["status"]
	trade.call("close")
	await _wait(3)


func _m55_migrants(world: Node, camera: Camera2D, migrants: Node, hearth: Vector2i, notes: Dictionary) -> void:
	var id: int = int(migrants.call("shown_wave"))
	await _near(camera, hearth, 2.0)
	migrants.call("open_wave", id)
	await _wait(10)
	_m55_note(notes, "m55_01_migrant_dialog.png", world, {"wave": id, "texts": migrants.call("texts"),
		"agents": Array((world.call("get_wave", id) as Dictionary).get("agents", PackedInt64Array()))})
	await _save("m55_01_migrant_dialog.png")
	var accept: Button = (migrants as Control).find_child("Accept", true, false) as Button
	if accept != null:
		await _click(accept.get_global_rect().get_center())
		await _wait(5)
	notes["m55_01_migrant_dialog.png"]["accepted_people"] = ((world.call("get_agent_snapshot") as Dictionary).get("ids", PackedInt64Array()) as PackedInt64Array).size()


func _m55_neighbours(world: Node, camera: Camera2D, neighbours: Node, hearth: Vector2i, notes: Dictionary) -> void:
	await _near(camera, hearth, 2.0)
	await _key(KEY_N)
	await _wait(10)
	_m55_note(notes, "m55_03_neighbours_panel.png", world, {"texts": neighbours.call("texts")})
	await _save("m55_03_neighbours_panel.png")
	await _key(KEY_N)


func _m55_tech(world: Node, tech: Node, notes: Dictionary) -> void:
	var nb: Dictionary = world.call("get_neighbours")
	var node: int = -1
	var clues: PackedInt32Array = nb.get("clue_nodes", PackedInt32Array())
	for row: Dictionary in NeighbourText.neighbours(nb):
		for n: int in NeighbourText.rumoured_nodes(int(row["rumoured"])):
			if node < 0 or (clues.has(n) and not clues.has(node)):
				node = n
	await _key(KEY_T)
	await _wait(5)
	if node >= 0:
		tech.call("select_node", node)
	await _wait(10)
	_m55_note(notes, "m55_06_tech_rumour.png", world, {"node": node, "lines": tech.call("neighbour_lines"), "clue_nodes": Array(clues)})
	await _save("m55_06_tech_rumour.png")
	await _key(KEY_T)


func _m55_flee(main: Node, world: Node, camera: Camera2D, inspector: Node, notes: Dictionary) -> void:
	var wolves: Callable = func() -> bool: return int((world.call("get_story") as Dictionary).get("wolves", 0)) > 0
	if not await _advance_until(world, wolves, 240, 40 * 1440):
		push_error("capture: m55_04 no wolves within 40 days")
		return
	var fleeing: Array[int] = []
	var someone: Callable = func() -> bool:
		var snap: Dictionary = world.call("get_agent_snapshot")
		var flags: PackedByteArray = snap.get("fleeing", PackedByteArray())
		var ids: PackedInt64Array = snap.get("agent_ids", PackedInt64Array())
		var states: PackedByteArray = snap.get("states", PackedByteArray())
		var targets: PackedByteArray = snap.get("target_kinds", PackedByteArray())
		for i: int in mini(flags.size(), ids.size()):
			var asleep: bool = i < states.size() and i < targets.size() and states[i] == 3 and targets[i] == 3
			if flags[i] != 0 and not asleep:
				fleeing.append(ids[i])
				return true
		return false
	if not await _advance_until(world, someone, 30, 2 * 1440):
		push_error("capture: m55_04 nobody fled within 2 days of the first wolves")
		return
	if not await _click_person(main, world, camera, fleeing[0], 3.0):
		push_error("capture: m55_04 could not select the fleeing person")
	await _wait(10)
	var flee_label: Label = inspector.get("_flee_label") as Label
	var scroll: ScrollContainer = inspector.get("_scroll") as ScrollContainer
	if flee_label != null and scroll != null:
		scroll.scroll_vertical = maxi(0, int(flee_label.position.y) - 200)
		await _wait(10)
	_m55_note(notes, "m55_04_fleeing.png", world, {"agent": fleeing[0], "lines": inspector.call("m5_lines"), "wolves": _wolves(world).size()})
	await _save("m55_04_fleeing.png")
	inspector.call("close")


func _m55_eureka(world: Node, camera: Camera2D, chronicle: Node, hearth: Vector2i, notes: Dictionary) -> void:
	var found: Array[Dictionary] = []
	var eureka: Callable = func() -> bool:
		for row: Dictionary in ChronicleText.rows(world.call("get_chronicle_since", 0, 4096)):
			if int(row["kind"]) == 44:
				found.append(row)
				return true
		return false
	var seen: bool = await _advance_until(world, eureka, 1440, maxi(0, 60 * 1440 - _now(world)))
	await _near(camera, hearth, 2.0)
	if not (chronicle as Control).visible:
		await _key(KEY_C)
	chronicle.call("set_filter", 4 if seen else 0)
	await _wait(10)
	_m55_note(notes, "m55_05_eureka_chronicle.png", world, {"eureka": seen, "entry": found[0] if seen else {}, "feed": str(chronicle.call("plain_text")).substr(0, 600)})
	if not seen:
		push_error("capture: m55_05 no Eureka entry by day 60; the shot shows the whole feed")
	await _save("m55_05_eureka_chronicle.png")
	await _key(KEY_C)
