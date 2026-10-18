extends SceneTree
const Seats = preload("res://scripts/ui/camp_seats.gd")
const Renderer = preload("res://scripts/ui/agent_renderer.gd")
var failures: int = 0
var _kernel_bridge: Node
var _kernel_adapter: GDScript

func check(ok: bool, message: String) -> void:
	if not ok:
		push_error(message)
		failures += 1

func _initialize() -> void:
	_kernel_bridge = ClassDB.instantiate("WorldSimNode") as Node
	if not ResourceLoader.exists("res://scripts/ui/art_declump_p12.gd") or _kernel_bridge == null or not _kernel_bridge.has_method("art_declump"):
		check(false, "Declump suite requires actual P12 kernel bridge and adapter")
		if _kernel_bridge != null:
			_kernel_bridge.free()
		quit(1)
		return
	_kernel_adapter = load("res://scripts/ui/art_declump_p12.gd") as GDScript
	var cells := PackedByteArray()
	cells.resize(64 * 64)
	var terrain: Dictionary = {"width": 64, "height": 64, "terrain": cells}
	var feet := Vector2(456, 42) + Vector2(30, 30) * 16.0
	var rows: Array[Dictionary] = [
		{"id": 2, "visible": true, "moving": false},
		{"id": 3, "visible": true, "moving": false}]
	var targets: Dictionary = {2: feet, 3: feet + Vector2(0, 16)}
	var rects: Dictionary = {2: Rect2(-8, -29, 16, 30), 3: Rect2(-8, -29, 16, 30)}
	var offsets: Dictionary = _declump(rows, {}, targets, terrain, {}, rects)
	check(offsets[2] == Vector2(6, 0), "equal x even ID tries right first and takes first shift below threshold")
	check(offsets[3] == Vector2.ZERO, "frontmost villager remains still")
	check(_coverage(targets, rects, offsets) < 0.30, "vertically adjacent same-x villagers end below 0.30")
	var reversed: Array[Dictionary] = rows.duplicate(true)
	reversed.reverse()
	check(_declump(reversed, {}, targets, terrain, {}, rects) == offsets, "same input and reordered rows produce identical offsets")
	rows[0]["moving"] = true
	check(not _declump(rows, {}, targets, terrain, {}, rects).has(2), "moving villagers never shift")
	rows[0]["moving"] = false
	rows[0]["activity"] = "walk"
	rows[0]["tile"] = Vector2i(30, 30)
	check(not _declump(rows, {}, targets, terrain, {}, rects).has(2), "paused walking animation never shifts")
	check(not Seats.spread_offsets([rows[0]], {}).has(2), "paused walking animation never spreads")
	rows[0].erase("activity")
	check(_declump(rows, {2: feet}, targets, terrain, {}, rects)[2] == Vector2(6, 0), "seated villagers now resolve foreground overlap")
	check(_declump(rows, {3: targets[3]}, targets, terrain, {}, rects)[2] == Vector2(6, 0), "seated front villagers still occlude")
	rows[1]["moving"] = true
	check(_declump(rows, {}, targets, terrain, {}, rects)[2] == Vector2(6, 0), "moving front villagers still occlude")
	rows[1]["moving"] = false
	rows[1]["visible"] = false
	check(_declump(rows, {}, targets, terrain, {}, rects)[2] == Vector2.ZERO, "hidden villagers do not occlude")
	rows[1]["visible"] = true
	# At a tile's right edge every positive shift enters the water tile.
	targets[2] = feet + Vector2(7, 0)
	targets[3] = feet + Vector2(7, 16)
	cells[30 * 64 + 31] = 4
	terrain["terrain"] = cells
	var wet: Dictionary = _declump(rows, {}, targets, terrain, {}, rects)
	check(wet[2] == Vector2(-6, 0), "water anywhere on map forces the other side")
	check(_coverage(targets, rects, wet) < 0.30, "water avoidance still reaches coverage threshold")
	cells[30 * 64 + 31] = 0
	terrain["terrain"] = cells
	check(_declump(rows, {}, targets, terrain, {Vector2i(31, 30): true}, rects)[2] == Vector2(-6, 0), "obstructed feet tiles force the other side")
	check(_declump(rows, {}, targets, terrain, {Vector2i(30, 30): true, Vector2i(31, 30): true}, rects)[2] == Vector2.ZERO, "all shifts blocked returns zero")
	# Wide overlapping frames cannot reach threshold within eight pixels.
	rects[2] = Rect2(-24, -29, 48, 30)
	rects[3] = Rect2(-24, -29, 48, 30)
	check(_declump(rows, {}, targets, terrain, {}, rects)[2] == Vector2(8, 0), "unreachable threshold chooses allowed minimum coverage")
	targets[2] = feet + Vector2(-1, 0)
	targets[3] = feet + Vector2(0, 16)
	check((_declump(rows, {}, targets, terrain, {}, rects)[2] as Vector2).x < 0, "first side points away from nearest front agent")
	_test_order_and_final_targets(terrain, feet)
	_test_renderer(terrain, feet)
	_test_d24(terrain, "d24_regression.json")
	_test_d24(terrain, "d24_merged_regression.json")
	print("DECLUMP %s: coverage, water, obstacles, exclusions, deterministic order, final front targets" % ("PASS" if failures == 0 else "FAIL"))
	_kernel_bridge.free()
	quit(1 if failures else 0)

func _coverage(targets: Dictionary, rects: Dictionary, offsets: Dictionary) -> float:
	var back: Rect2 = rects[2]
	var front: Rect2 = rects[3]
	back.position += Vector2(targets[2]) + Vector2(offsets.get(2, Vector2.ZERO))
	front.position += Vector2(targets[3]) + Vector2(offsets.get(3, Vector2.ZERO))
	return back.intersection(front).get_area() / back.get_area()

func _test_order_and_final_targets(terrain: Dictionary, feet: Vector2) -> void:
	var rows: Array[Dictionary] = [
		{"id": 1, "visible": true, "moving": false},
		{"id": 2, "visible": true, "moving": false},
		{"id": 3, "visible": true, "moving": false}]
	var targets: Dictionary = {1: feet, 2: feet, 3: feet}
	var rects: Dictionary = {1: Rect2(-4, -29, 8, 30), 2: Rect2(-4, -29, 8, 30), 3: Rect2(-4, -29, 8, 30)}
	var offsets: Dictionary = _declump(rows, {}, targets, terrain, {}, rects)
	check(offsets[3] == Vector2.ZERO, "equal-y higher ID is frontmost")
	check(offsets[2] == Vector2(6, 0), "middle ID moves right")
	check(offsets[1] == Vector2(-6, 0), "rear odd ID chooses left against already-shifted fronts")

func _test_renderer(terrain: Dictionary, feet: Vector2) -> void:
	var renderer = Renderer.new()
	renderer.world_sim = _kernel_bridge
	check(renderer.set_perf_candidate_p12("declump-kernel", true), "renderer accepts actual kernel candidate")
	var rows: Array[Dictionary] = []
	for agent_id: int in [2, 3]:
		var sprite := Sprite2D.new()
		sprite.set_meta("frame_size", Vector2i(16, 30))
		sprite.set_meta("anchor", Vector2(8, 29))
		renderer._sprites[agent_id] = sprite
		var tile := Vector2i(30, 30 + agent_id - 2)
		rows.append({"id": agent_id, "ipos": feet + Vector2(0, (agent_id - 2) * 16), "tile": tile, "visible": true, "moving": false})
	var context: Dictionary = {"hearths": [], "seat_blocked": {}, "terrain": terrain}
	renderer._apply_drawn_positions(rows, context, 1.15)
	var back: Sprite2D = renderer._sprites[2]
	check(back.position == feet + Vector2(6, 0), "renderer applies declump using manifest metadata without a GPU texture")
	check(back.get_meta("declump_offset") == Vector2(6, 0), "capture metadata exposes applied declump target")
	check(back.get_meta("drawn_target") == feet + Vector2(6, 0), "capture metadata exposes final target")
	check(rows[0]["ipos"] == feet, "declump never modifies simulation position")
	for sprite: Sprite2D in renderer._sprites.values():
		sprite.free()
	renderer.free()

func _test_d24(terrain: Dictionary, fixture_name: String) -> void:
	var fixture: Array = JSON.parse_string(FileAccess.get_file_as_string("res://docs/art/production6/crowd/" + fixture_name))
	var rows: Array[Dictionary] = []
	var targets: Dictionary = {}
	var rects: Dictionary = {}
	var seats: Dictionary = {}
	for record: Dictionary in fixture:
		var agent_id: int = int(record["id"])
		rows.append({"id": agent_id, "visible": true, "moving": bool(record["moving"]), "activity": record["activity"]})
		targets[agent_id] = Vector2(record["target"][0], record["target"][1])
		var rect: Array = record["rect"]
		rects[agent_id] = Rect2(rect[0], rect[1], rect[2], rect[3])
		if record["seated"]:
			seats[agent_id] = targets[agent_id]
	var offsets: Dictionary = _declump(rows, seats, targets, terrain, {}, rects)
	for row: Dictionary in rows:
		if row["activity"] == "walk":
			check(not offsets.has(row["id"]), "d24 walking poses remain untouched even when moving flag false")
	var moved_seats: int = 0
	for agent_id: int in seats:
		if offsets.get(agent_id, Vector2.ZERO) != Vector2.ZERO:
			moved_seats += 1
	check(moved_seats > 0, "d24 previously skipped seated actors now shift")
	print(fixture_name, " moved_seats=", moved_seats, " offsets=", offsets)
	for offset: Vector2 in offsets.values():
		check(absf(offset.x) <= 8.0 and offset.y == 0.0, "d24 shifts stay within horizontal8px")
	var reversed: Array[Dictionary] = rows.duplicate(true)
	reversed.reverse()
	check(_declump(reversed, seats, targets, terrain, {}, rects) == offsets, "d24 front-to-back result deterministic")

func _declump(rows: Array[Dictionary], seated: Dictionary, targets: Dictionary, terrain: Dictionary, blocked: Dictionary, rects: Dictionary) -> Dictionary:
	var candidate: Dictionary = {}
	for spatial: bool in [false, true]:
		var reference: Dictionary = Seats.declump(rows, seated, targets, terrain, blocked, rects, spatial)
		candidate = _kernel_adapter.declump(_kernel_bridge, rows, targets, terrain, blocked, rects, spatial)
		check(_kernel_adapter.bit_equal(reference, candidate), "declump kernel preserves ordered ids and exact f32 offsets spatial=%s" % spatial)
	return candidate
