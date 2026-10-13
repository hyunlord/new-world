extends SceneTree

const Topbar = preload("res://scripts/ui/panels/hud_topbar.gd")
const News = preload("res://scripts/ui/panels/hud_status_panel.gd")
const DeathViz = preload("res://scripts/ui/death_viz_renderer.gd")

class SnapshotWorld extends Node:
	signal world_reset
	var tallies: Dictionary = {"deaths_starvation": 2, "deaths_dehydration": 3, "deaths_combat": 4}
	func get_tallies() -> Dictionary: return tallies
	func get_agent_snapshot() -> Dictionary: return {}
	func get_settlement_snapshot() -> Dictionary: return {}
	func get_construction_snapshot() -> Dictionary: return {}
	func get_building_snapshot() -> Dictionary: return {}
	func get_resource_snapshot() -> Dictionary: return {}
	func get_time_snapshot() -> Dictionary: return {}
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
	assert(topbar._resource_labels[3].text.ends_with(" 9"))
	world.tallies["deaths_old_age"] = 5
	topbar._refresh()
	assert(topbar._resource_labels[3].text.ends_with(" 14"))
	world.tallies["deaths_exposure"] = 6
	topbar._refresh()
	assert(topbar._resource_labels[3].text.ends_with(" 20"))
	assert(topbar.get_death_total() == 20)
	world.tallies.erase("deaths_exposure")
	world.tallies.erase("deaths_old_age")
	topbar._refresh()
	assert(topbar._resource_labels[3].text.ends_with(" 9"))
	var news: Control = News.new()
	main.add_child(news)
	news.set_process(false)
	var snapshot: Dictionary = {"xs": PackedInt32Array([1, 1, 1, 1]), "ys": PackedInt32Array([2, 2, 2, 2]), "reasons": PackedInt32Array([0, 1, 2, 3]), "ticks": PackedInt64Array([10, 10, 10, 10]), "current_tick": 10}
	news._ingest_deaths(snapshot)
	assert(news._entries.size() == 4)
	for index: int in 4:
		assert(news._entries[index]["key"] == news.DEATH_NEWS_KEYS[index])
	news._ingest_deaths(snapshot)
	assert(news._entries.size() == 4 and news._seen_deaths.size() == 4)
	news._push("HUD_FEED_AGENT_BORN", [1])
	assert(news._entries.size() == 4)
	news._entries.clear()
	news._ingest_deaths(snapshot)
	assert(news._entries.is_empty())
	news._push("HUD_FEED_AGENT_BORN", [1])
	news._frame_tick = 500
	news._prune_old()
	assert(news._entries.is_empty())
	news._ingest_deaths(snapshot)
	assert(news._entries.is_empty())
	var malformed: Dictionary = snapshot.duplicate(true)
	malformed["xs"] = PackedInt32Array([1])
	news._ingest_deaths(malformed)
	assert(news._entries.is_empty())
	snapshot["current_tick"] = 130
	news._ingest_deaths(snapshot)
	assert(news._seen_deaths.is_empty() and news._entries.is_empty())
	snapshot["current_tick"] = 10
	news._ingest_deaths(snapshot)
	assert(news._entries.size() == 4)
	news._on_world_reset()
	assert(news._entries.is_empty() and news._seen_deaths.is_empty())
	news._ingest_deaths(snapshot)
	assert(news._entries.size() == 4)
	news._refresh()
	var viz: Node2D = DeathViz.new()
	assert(viz._reason_color(3) == Color(0.70, 0.70, 0.70))
	assert(viz._reason_color(0) == Color(0.45, 0.30, 0.15))
	viz.free()
	main.free()
	print("HUD OLD AGE PASS: optional tally, four causes, tuple dedup beyond feed expiry, retention, rollback, reset, malformed arrays, grey marker")
	quit(0)
