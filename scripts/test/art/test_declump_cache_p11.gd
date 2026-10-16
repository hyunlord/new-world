extends SceneTree
const Seats = preload("res://scripts/ui/camp_seats.gd")
const Renderer = preload("res://scripts/ui/agent_renderer.gd")
var failures: int = 0
var cache = Seats.new()
var rows: Array[Dictionary] = [{"id": 2, "visible": true, "moving": false}, {"id": 3, "visible": true, "moving": false}]
var targets: Dictionary = {2: Vector2(936, 522), 3: Vector2(936, 538)}
var rects: Dictionary = {2: Rect2(-8, -29, 16, 30), 3: Rect2(-8, -29, 16, 30)}
var terrain: Dictionary = {"width": 64, "height": 64}
var blocked: Dictionary = {}

func check(ok: bool, label: String) -> void:
	if not ok:
		push_error(label)
		failures += 1

func compare(label: String, hit: bool = false, spatial: bool = true) -> Dictionary:
	var hits: int = cache.declump_cache_hits
	var actual: Dictionary = cache.declump_cached(rows, {}, targets, terrain, blocked, rects, spatial)
	check(actual == Seats.declump(rows, {}, targets, terrain, blocked, rects, spatial), label + " reference equality")
	check(cache.declump_cache_hits == hits + int(hit), label + " cache transition")
	return actual

func _initialize() -> void:
	if not cache.has_method("declump_cached") or not cache.has_method("set_declump_cache_enabled"):
		push_error("Missing P11 cache API")
		quit(1)
		return
	var cells := PackedByteArray()
	cells.resize(4096)
	terrain["terrain"] = cells
	compare("initial miss")
	rows = rows.duplicate(true)
	targets = targets.duplicate(true)
	rects = rects.duplicate(true)
	terrain = terrain.duplicate(true)
	var output: Dictionary = compare("fresh equal containers", true)
	output[2] = Vector2(999, 999)
	compare("output alias protection", true)
	for field: String in ["visible", "moving", "activity_place_known"]:
		rows[0][field] = not bool(rows[0].get(field, false))
		compare(field)
		rows[0][field] = not bool(rows[0][field])
		compare(field + " reversed")
	rows[0]["pose"] = "walk"
	compare("pose fallback")
	rows[0]["activity"] = "idle"
	compare("activity overrides pose")
	rows[0].erase("activity")
	compare("activity removed")
	rows[0].erase("pose")
	compare("pose removed")
	var saved_target: Vector2 = targets[2]
	targets.erase(2)
	compare("visible target absent")
	targets[2] = saved_target
	compare("visible target restored")
	var saved_rect: Rect2 = rects[2]
	rects.erase(2)
	compare("visible rect absent")
	rects[2] = saved_rect
	compare("visible rect restored")
	rows.reverse()
	compare("row order")
	rows.append(rows[0].duplicate())
	compare("duplicate row")
	rows.pop_back()
	compare("duplicate removed")
	for offset: Vector2 in [Vector2(1, 0), Vector2(0, 1)]:
		targets[2] += offset
		compare("target component")
		targets[2] -= offset
		compare("target reversed")
	targets[99] = Vector2.ZERO
	compare("unused target")
	targets.erase(99)
	compare("target removed")
	rects[99] = Rect2(-100, -100, 200, 200)
	compare("non-row rect changes cell size")
	rects.erase(99)
	compare("non-row rect removed")
	rects[2] = Rect2(-9, -30, 17, 31)
	compare("footprint origin and size")
	rects[2] = Rect2(-8, -29, 16, 30)
	compare("footprint reversed")
	blocked[Vector2i(30, 30)] = true
	compare("same blocked dictionary mutated")
	blocked.clear()
	compare("blocked reversed")
	terrain["terrain"][1950] = 4
	compare("same packed cells mutated")
	terrain["terrain"][1950] = 0
	compare("cells reversed")
	for field: String in ["width", "height"]:
		terrain[field] = 63
		compare(field)
		terrain[field] = 64
		compare(field + " reversed")
	compare("brute force mode", false, false)
	compare("brute force hit", true, false)
	compare("hashed mode")
	cache.set_declump_cache_enabled(false)
	compare("disabled")
	compare("disabled again")
	cache.set_declump_cache_enabled(true)
	compare("reenabled miss")
	compare("reenabled hit", true)
	cache.clear()
	compare("reset miss")
	rows.clear()
	compare("empty output miss")
	compare("empty output hit", true)
	_test_renderer()
	print("DECLUMP_CACHE_P11 %s" % ("PASS" if failures == 0 else "FAIL"))
	quit(1 if failures else 0)

func _test_renderer() -> void:
	var renderer = Renderer.new()
	check(renderer.set_perf_candidate_p11("declump-input-cache", true), "P11 switch accepted")
	check(not renderer.set_perf_candidate_p11("allocation-cache", false), "P11 rejects other controls")
	var old_control: Dictionary = renderer.get_perf_candidate_p10()
	var actor := Sprite2D.new()
	actor.set_meta("frame_size", Vector2i(16, 30))
	actor.set_meta("anchor", Vector2(8, 29))
	renderer._sprites[2] = actor
	var actor_rows: Array[Dictionary] = [{"id": 2, "ipos": Vector2(936, 522), "tile": Vector2i(30, 30), "visible": true, "moving": false}]
	var context: Dictionary = {"hearths": [], "seat_blocked": {}, "terrain": terrain}
	renderer._drawn_positions[2] = Vector2(900, 522)
	renderer._apply_drawn_positions(actor_rows, context, 0.01)
	var before: Vector2 = renderer._drawn_positions[2]
	actor.set_meta("drawn_target", Vector2.ZERO)
	actor.set_meta("walker_lateral_offset", Vector2(999, 0))
	renderer._apply_drawn_positions(actor_rows, context, 0.02)
	check(renderer._camp_seats.declump_cache_hits == 1, "renderer cache hit")
	check(renderer._drawn_positions[2] != before, "easing runs on cache hit")
	check(actor.get_meta("drawn_target") == Vector2(936, 522), "metadata refreshed on hit")
	check(actor.get_meta("walker_lateral_offset") == Vector2.ZERO, "walker tail runs on hit")
	renderer.set_perf_candidate_p11("declump-input-cache", false)
	check(renderer.get_perf_candidate_p10() == old_control, "P10 control unchanged")
	renderer._sprites.erase(2)
	actor.free()
	renderer._on_world_reset()
	check(not renderer._camp_seats.declump_cache_enabled(), "reset preserves switch")
	renderer.free()
