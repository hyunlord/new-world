extends SceneTree
const News = preload("res://scripts/ui/panels/hud_status_panel.gd")
const Selection = preload("res://scripts/core/settlement_selection.gd")
const Food = preload("res://scripts/ui/panels/feed_food_ledger.gd")
var failures: int = 0

class World extends Node:
	signal world_reset
	var clock: Dictionary = {"tick": 0, "day_index": 0, "time_of_day": 19.0}
	var food: int = 1000
	var society: Dictionary = {"settlement_ids": PackedInt32Array([7]), "parent_ids": PackedInt32Array([-1]), "founded_ticks": PackedInt64Array([0])}
	var casts: Dictionary = {}
	func get_time_snapshot() -> Dictionary: return clock
	func get_settlement_snapshot() -> Dictionary: return {"ids": PackedInt64Array([7]), "settlement_ids": PackedInt32Array([7]), "food_stocks": PackedInt32Array([food])}
	func get_society_snapshot() -> Dictionary: return society
	func get_intervention_state() -> Dictionary: return casts
	func get_agent_snapshot() -> Dictionary: return {}
	func get_construction_snapshot() -> Dictionary: return {}
	func get_tallies() -> Dictionary: return {}

func check(ok: bool, description: String) -> void:
	if not ok:
		failures += 1
		push_error(description)

func _initialize() -> void:
	call_deferred("run")

func run() -> void:
	Selection.reset()
	var world := World.new()
	var news := News.new()
	root.add_child(news)
	news.set_process(false)
	news._world_sim = world
	news._clock_provider = func() -> Dictionary: return world.clock
	news._ingest_snapshots()
	for index: int in 50:
		world.clock["tick"] = index + 1
		world.food += 4 if index % 2 == 0 else -3
		news._ingest_snapshots()
	var routine: int = 0
	for record: Dictionary in news.get_feed_log():
		if record["class"] == "routine":
			routine += 1
			check(not bool(record["visible"]), "routine remains hidden")
	check(routine == 50, "all fifty threshold movements preserved")
	world.clock["tick"] = 51
	world.society.merge({"current_tick": 51, "fission_ids": PackedInt32Array([9]), "fission_parents": PackedInt32Array([7]), "fission_sizes": PackedInt32Array([6]), "fission_depart_ticks": PackedInt64Array([51]), "fission_founded_ticks": PackedInt64Array([-1]), "fission_cancelled_ticks": PackedInt64Array([-1]), "fission_daughters": PackedInt32Array([-1]), "fission_food_taken": PackedInt64Array([20])}, true)
	world.food -= 20
	news._ingest_snapshots()
	world.clock["tick"] = 52
	world.casts = {"log_ids": PackedInt64Array([0]), "log_settlement_ids": PackedInt32Array([7]), "log_ticks": PackedInt64Array([52]), "log_kinds": PackedStringArray(["gift_food"]), "log_applied": PackedInt64Array([60])}
	world.food += 60
	news._ingest_snapshots()
	world.clock["time_of_day"] = 20.0
	world.clock["tick"] = 53
	news._ingest_snapshots()
	news._ingest_snapshots()
	var summaries: Array = news.capture_feed_metadata()["food"]["summaries"]
	check(summaries.size() == 1, "one summary at boundary despite repeated observation")
	check(summaries[0]["stored"] == 100 and summaries[0]["eaten"] == 75 and summaries[0]["net"] == 25, "summary excludes departure and gift")
	check(summaries[0]["coverage_complete"], "per-tick audit establishes complete observed coverage")
	for record: Dictionary in news.get_visible_feed():
		check(record["class"] != "routine", "visible lines never routine")
	for ignored: int in 500:
		news._prune_old()
	check(has_visible(news, "depart:9") and has_visible(news, "cast:0"), "paused clock cannot consume protection")
	world.clock["tick"] = 290
	for index: int in 8:
		news._push("HUD_FEED_AGENT_BORN", [20 + index])
	check(has_visible(news, "depart:9") and has_visible(news, "cast:0"), "both protected through 239 ticks despite ordinary pressure")
	world.clock["tick"] = 292
	news._push("HUD_FEED_AGENT_BORN", [99])
	check(not has_visible(news, "depart:9"), "ordinary news can replace departure only after protection")
	for index: int in 6:
		news._push("HUD_FEED_FISSION_CANCELLED", [], 292, {"event_id": "burst:%d" % index})
	check(news.get_visible_feed().size() == 4 and news._overflow_ids.has("burst:0") and news._overflow_ids.has("burst:1"), "oldest protected overflow counted")
	var before: int = news.get_feed_log().size()
	news._ingest_snapshots()
	check(news.get_feed_log().size() == before, "source cast and departure deduplicated")
	check(news.capture_feed_metadata()["overflow_count"] == news._overflow_ids.size(), "counter exposes represented event IDs")
	world.clock["tick"] = 800
	news._prune_old()
	check(news.get_visible_feed().is_empty(), "expired entries removed")
	check(news._overflow_ids.has("depart:9") and news._overflow_ids.has("cast:0") and news._overflow_ids.has("burst:0"), "expired protected history remains represented by counter")
	for record: Dictionary in news.get_feed_log():
		check(not bool(record["visible"]), "expired log visibility never stale")
	world.clock["tick"] = 900
	news._ingest_snapshots()
	world.clock["tick"] = 5
	news._ingest_snapshots()
	check(news.get_feed_log().is_empty() and news._food.movements.is_empty(), "injected rewind resets to suppressed baseline")
	check_food_edges()
	news._on_world_reset()
	check(news.get_feed_log().is_empty() and news.get_visible_feed().is_empty() and news._overflow_ids.is_empty(), "reset clears counter and suppresses source baseline events")
	news.free()
	world.free()
	print("FEED_PRIORITY failures=", failures)
	quit(1 if failures else 0)

func has_visible(news: Control, identity: String) -> bool:
	for row: Dictionary in news.get_visible_feed():
		if row["event_id"] == identity:
			return true
	return false

func check_food_edges() -> void:
	var food = Food.new()
	food.prepare_clock({"tick": 0, "day_index": 0, "time_of_day": 19.0})
	food.observe(7, 100, 0)
	food.observe(42, 100, 0)
	food.begin()
	food.exclude("cast:0", 7, 60, 1, 1, "gift_food")
	food.exclude("cast:1", 42, -20, 1, 1, "spoil_food")
	food.exclude("cast:2", 7, 60, 1, 1, "gift_food")
	food.observe(7, 222, 1)
	food.observe(42, 79, 1)
	check(food.buckets[7]["stored"] == 2 and food.buckets[42]["eaten"] == 1, "same-tick repeated casts preserve ordinary movement by village")
	food.begin()
	food.exclude("food:depart:2", 7, -20, 2, 2, "fission_depart")
	food.observe(7, 201, 2)
	food.begin()
	food.exclude("food:cancel:2", 7, 20, 3, 3, "fission_cancel")
	food.observe(7, 223, 3)
	check(food.buckets[7]["stored"] == 4 and food.buckets[7]["eaten"] == 1, "departure and cancellation do not erase adjacent normal changes")
	food.exclude("food:found:3", 99, 20, 3, 3, "fission_found")
	food.observe(99, 20, 3)
	check(not food.buckets.has(99), "new daughter baseline not false eating")
	food.begin()
	food.exclude("cast:old", 7, 60, 0, 4, "gift_food")
	food.observe(7, 223, 4)
	check(food.buckets[7]["stored"] == 4, "older source history never adjusts fresh baseline")
	food.reset()
	food.prepare_clock({"tick": 0, "day_index": 0, "time_of_day": 19.0})
	food.observe(7, 100, 0)
	for tick: int in range(1, 11):
		food.prepare_clock({"tick": tick, "day_index": 0, "time_of_day": 20.0 if tick == 10 else 19.0})
		food.begin()
		food.observe(7, 100 - tick, tick)
	var rows: Array = food.finish({"tick": 10, "day_index": 0, "time_of_day": 20.0})
	check(rows.size() == 1 and rows[0]["eaten"] == 10 and rows[0]["net"] == -10, "subthreshold changes included at exact threshold")
	food.prepare_clock({"tick": 50, "day_index": 1, "time_of_day": 21.0})
	food.observe(7, 80, 50)
	rows = food.finish({"tick": 50, "day_index": 1, "time_of_day": 21.0})
	check(rows.size() == 1 and not rows[0]["coverage_complete"] and not rows[0]["boundary_exact"], "skipped boundary reported honestly")
