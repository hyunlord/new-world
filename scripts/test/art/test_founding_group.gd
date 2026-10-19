extends SceneTree
const News = preload("res://scripts/ui/panels/hud_status_panel.gd")
const Selection = preload("res://scripts/core/settlement_selection.gd")
const Groups = preload("res://scripts/ui/founding_group_renderer.gd")
const Loads = preload("res://scripts/ui/carry_load_layer.gd")
const Library = preload("res://scripts/ui/art_sprite_library.gd")
var failures: int = 0
class World extends Node:
	signal world_reset
	var society: Dictionary = {}
	var settlements: Dictionary = {}
	func get_society_snapshot() -> Dictionary: return society
	func get_settlement_snapshot() -> Dictionary: return settlements
	func get_agent_snapshot() -> Dictionary: return {}
	func get_construction_snapshot() -> Dictionary: return {}
	func get_tallies() -> Dictionary: return {}
	func get_time_snapshot() -> Dictionary: return {"tick": society.get("current_tick", 0)}

func check(ok: bool, message: String) -> void:
	if not ok:
		failures += 1
		push_error(message)
func _initialize() -> void:
	call_deferred("run")
func run() -> void:
	Selection.reset()
	var world := World.new()
	var news := News.new()
	root.add_child(news)
	news.set_process(false)
	news._world_sim = world
	world.society = {"current_tick": 1, "settlement_ids": PackedInt32Array([7]), "parent_ids": PackedInt32Array([-1]), "founded_ticks": PackedInt64Array([0])}
	world.settlements = {"ids": PackedInt64Array([7]), "settlement_ids": PackedInt32Array([7]), "food_stocks": PackedInt32Array([5])}
	news._ingest_snapshots()
	check(news.get_feed_log().size() == 1 and news.get_feed_log()[0]["args"] == [1], "original formed uses UI number")
	world.society.merge({"current_tick": 100, "fission_ids": PackedInt32Array([9]), "fission_parents": PackedInt32Array([7]), "fission_sizes": PackedInt32Array([6]), "fission_depart_ticks": PackedInt64Array([96]), "fission_founded_ticks": PackedInt64Array([-1]), "fission_cancelled_ticks": PackedInt64Array([-1]), "fission_daughters": PackedInt32Array([-1]), "founder_agent_ids": PackedInt64Array([101, 102]), "founder_fission_ids": PackedInt32Array([9, 9]), "founder_leader": PackedByteArray([1, 0]), "founder_site_xs": PackedInt32Array([30, 30]), "founder_site_ys": PackedInt32Array([31, 31]), "home_agent_ids": PackedInt64Array([101, 102]), "home_settlement_ids": PackedInt32Array([7, 7])}, true)
	news._ingest_snapshots()
	check(news.get_feed_log()[-1]["key"] == "HUD_FEED_FISSION_DEPARTED" and news.get_feed_log()[-1]["args"] == [6, 1] and news.get_feed_log()[-1]["tick"] == 96, "departure original tick and exact group size")
	var groups := Groups.new()
	root.add_child(groups)
	groups.set_process(false)
	groups.refresh_snapshots(world.society, {"agent_ids": PackedInt64Array([101, 102]), "xs": PackedInt32Array([10, 11]), "ys": PackedInt32Array([11, 12])})
	check(groups._groups.size() == 1 and groups._groups[0]["leader_id"] == 101 and groups._groups[0]["site_tile"] == [30, 31], "one trail and pennant for leader and site")
	check(groups._groups[0]["marker_color"] == Selection.color_for(7), "parent pennant colour")
	world.society["current_tick"] = 110
	news._ingest_snapshots()
	check(news.get_feed_log().size() == 2, "walking does not repeat departure")
	world.society.merge({"current_tick": 140, "settlement_ids": PackedInt32Array([7, 42]), "parent_ids": PackedInt32Array([-1, 7]), "founded_ticks": PackedInt64Array([0, 137]), "fission_founded_ticks": PackedInt64Array([137]), "fission_daughters": PackedInt32Array([42]), "founder_agent_ids": PackedInt64Array()}, true)
	world.settlements = {"ids": PackedInt64Array([7, 42]), "settlement_ids": PackedInt32Array([7, 42]), "food_stocks": PackedInt32Array([5, 0])}
	news._ingest_snapshots()
	news._ingest_snapshots()
	check(news.get_feed_log().size() == 3 and news.get_feed_log()[-1]["key"] == "HUD_FEED_SETTLEMENT_FOUNDED" and news.get_feed_log()[-1]["args"] == [2] and news.get_feed_log()[-1]["tick"] == 137, "daughter once from source tick, no generic formed")
	groups.refresh_snapshots(world.society, {})
	check(groups._groups.is_empty(), "founding removes trail and marker")
	world.society.merge({"current_tick": 200, "fission_ids": PackedInt32Array([9, 10]), "fission_parents": PackedInt32Array([7, 42]), "fission_sizes": PackedInt32Array([6, 8]), "fission_depart_ticks": PackedInt64Array([96, 180]), "fission_founded_ticks": PackedInt64Array([137, -1]), "fission_cancelled_ticks": PackedInt64Array([-1, 195]), "fission_daughters": PackedInt32Array([42, -1])}, true)
	news._ingest_snapshots()
	check(news.get_feed_log().size() == 5 and news.get_feed_log()[3]["args"] == [8, 2] and news.get_feed_log()[-1]["key"] == "HUD_FEED_FISSION_CANCELLED" and news.get_feed_log()[-1]["tick"] == 195, "skipped refresh still recovers depart then cancel from log")
	world.society["current_tick"] = 208
	world.society["settlement_ids"] = PackedInt32Array([42])
	world.society["parent_ids"] = PackedInt32Array([7])
	world.society["founded_ticks"] = PackedInt64Array([137])
	world.settlements = {"ids": PackedInt64Array([42]), "settlement_ids": PackedInt32Array([42]), "food_stocks": PackedInt32Array([0])}
	news._ingest_snapshots()
	news._ingest_snapshots()
	check(news.get_feed_log().size() == 6 and news.get_feed_log()[-1]["key"] == "HUD_FEED_SETTLEMENT_ABANDONED" and news.get_feed_log()[-1]["args"] == [1] and news.get_feed_log()[-1]["tick_source"] == "first_observed_absent", "abandonment once, stable number, observed tick honestly marked")
	world.settlements["food_stocks"] = PackedInt32Array([5])
	news._ingest_snapshots()
	check(news.get_feed_log()[-1]["args"] == [2, 5], "stockpile reports village number not raw42")
	var sprite := Sprite2D.new()
	root.add_child(sprite)
	Library.apply(sprite, "V_a2_01")
	Loads.apply_founder(sprite, true, "walk", 0.0, 0)
	var bundle: Sprite2D = sprite.get_node("FounderBundle")
	var anchor: Vector2 = bundle.position
	check(bundle.visible and bundle.scale == Vector2(0.5, 0.5), "accepted wood bundle reused")
	check(anchor.y < sprite.offset.y * 0.5, "upper back above torso midpoint")
	sprite.flip_h = true
	Loads.apply_founder(sprite, true, "walk", 0.0, 0)
	check(bundle.position == Vector2(-anchor.x, anchor.y) and bundle.flip_h, "bundle anchor and facing mirror")
	for activity: String in ["eat", "asleep"]:
		Loads.apply_founder(sprite, true, activity, 0.0, 0)
		check(not bundle.visible, "eat/sleep suppress bundle")
	Loads.apply_founder(sprite, true, "walk", 1.0, 0)
	check(not bundle.visible, "real food hides bundle")
	Loads.apply_founder(sprite, true, "walk", 0.0, 1)
	check(not bundle.visible, "real wood hides bundle")
	Loads.apply_founder(sprite, true, "idle", 0.0, 0, "eat")
	check(not bundle.visible, "meal pose hides bundle")
	Loads.apply_founder(sprite, false, "walk", 0.0, 0)
	check(not bundle.visible, "completed founding hides bundle")
	world.society = {}
	world.settlements = {}
	news._on_world_reset()
	check(news.get_feed_log().is_empty() and news._seen_society_events.is_empty(), "reset clears event records")
	groups._reset()
	check(groups.capture_metadata()["founding_groups"].is_empty(), "reset clears actual draw metadata")
	sprite.free()
	groups.free()
	news.free()
	world.free()
	print("FOUNDING_GROUP failures=", failures)
	quit(1 if failures else 0)
