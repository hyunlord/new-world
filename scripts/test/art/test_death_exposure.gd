extends SceneTree

const Topbar = preload("res://scripts/ui/panels/hud_topbar.gd")
const News = preload("res://scripts/ui/panels/hud_status_panel.gd")
const DeathViz = preload("res://scripts/ui/death_viz_renderer.gd")
const KEYS: Array[String] = ["HUD_FEED_DEATH_STARVATION", "HUD_FEED_DEATH_DEHYDRATION", "HUD_FEED_DEATH_COMBAT", "HUD_FEED_DEATH_OLD_AGE", "HUD_FEED_DEATH_EXPOSURE"]

class SnapshotWorld extends Node:
	signal world_reset
	var clock: Dictionary = {"tick": 40389, "time_of_day": 9.149999999999999}
	func get_tallies() -> Dictionary: return {"deaths_starvation": 1, "deaths_dehydration": 2, "deaths_combat": 3, "deaths_old_age": 4, "deaths_exposure": 5, "deaths_unrelated": 999}
	func get_agent_snapshot() -> Dictionary: return {}
	func get_settlement_snapshot() -> Dictionary: return {}
	func get_construction_snapshot() -> Dictionary: return {}
	func get_building_snapshot() -> Dictionary: return {}
	func get_resource_snapshot() -> Dictionary: return {}
	func get_time_snapshot() -> Dictionary: return clock
	func get_recent_deaths() -> Dictionary: return {}

func _initialize() -> void:
	call_deferred("_run")

func _run() -> void:
	var main: Node = Node.new()
	main.name = "Main"
	var world: SnapshotWorld = SnapshotWorld.new()
	world.name = "WorldSim"
	main.add_child(world)
	root.add_child(main)
	var topbar: Control = Topbar.new()
	main.add_child(topbar)
	topbar.set_process(false)
	assert(topbar._resource_labels[3].text.ends_with(" 15"), "all five causes must count")
	assert(topbar.get_death_total() == 15)
	assert(topbar._clock_label.text.contains("09:09"), "floating point minutes must round")
	world.clock["time_of_day"] = 23.98333333333333
	topbar._refresh()
	assert(topbar._clock_label.text.contains("23:59"))
	world.clock["time_of_day"] = 0.0
	topbar._refresh()
	assert(topbar._clock_label.text.contains("00:00"))
	var news: Control = News.new()
	main.add_child(news)
	news.set_process(false)
	var snapshot: Dictionary = {"xs": PackedInt32Array([1, 1, 1, 1, 1]), "ys": PackedInt32Array([2, 2, 2, 2, 2]), "reasons": PackedInt32Array([0, 1, 2, 3, 4]), "ticks": PackedInt64Array([10, 10, 10, 10, 10]), "current_tick": 10}
	news._ingest_deaths(snapshot)
	assert(news.get_feed_log().size() == 5)
	assert(news.get_visible_feed().size() == 4)
	for index: int in 5:
		assert(news.get_feed_log()[index]["key"] == KEYS[index])
		assert(news.get_feed_log()[index]["tick"] == 10)
		assert(news.get_feed_log()[index]["args"] == [])
	news._ingest_deaths(snapshot)
	assert(news.get_feed_log().size() == 5)
	var duplicate: Dictionary = {"xs": PackedInt32Array([3, 3]), "ys": PackedInt32Array([4, 4]), "reasons": PackedInt32Array([4, 4]), "ticks": PackedInt64Array([11, 11]), "current_tick": 11}
	news._ingest_deaths(duplicate)
	assert(news.get_feed_log().size() == 7, "same tile, tick and cause can be two people")
	assert(news.get_feed_log()[5]["death_id"] != news.get_feed_log()[6]["death_id"])
	news._ingest_deaths(duplicate)
	assert(news.get_feed_log().size() == 7)
	world.clock["tick"] = 40389 + 500
	news._prune_old()
	assert(news.get_visible_feed().is_empty() and news.get_feed_log().size() == 7)
	world.clock["tick"] = 40389
	news._push("HUD_FEED_AGENT_BORN", [9])
	assert(news.get_feed_log()[-1]["tick"] == 40389)
	var copied_log: Array = news.get_feed_log()
	copied_log[0]["args"].append("mutated")
	assert(news.get_feed_log()[0]["args"].is_empty())
	var viz: Node2D = DeathViz.new()
	var colors: Array[Color] = [Color(0.45, 0.30, 0.15), Color(0.20, 0.45, 0.95), Color(0.95, 0.15, 0.10), Color(0.70, 0.70, 0.70), Color(0.70, 0.84, 0.97)]
	for index: int in 5:
		assert(viz._reason_color(index) == colors[index])
		if index != 2:
			assert(viz._reason_color(index) != colors[2])
	for reason: int in [-1, 5, 99]:
		assert(viz._reason_color(reason) == Color(0.55, 0.55, 0.55))
		assert(viz._reason_color(reason) == Color(0.55, 0.55, 0.55))
	assert(viz._unknown_reasons_logged.size() == 3)
	news._on_world_reset()
	assert(news.get_feed_log().is_empty() and news.get_visible_feed().is_empty())
	assert(news._seen_deaths.is_empty())
	viz.free()
	main.free()
	print("DEATH EXPOSURE PASS: five totals/keys/colors, four visible, durable exact ticks, duplicate-person rows, repeat dedup, grey unknowns, reset, minute rounding")
	quit(0)
