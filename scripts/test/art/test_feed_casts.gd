extends SceneTree
const News = preload("res://scripts/ui/panels/hud_status_panel.gd")
const GodPanel = preload("res://scripts/ui/panels/god_hand_panel.gd")
const Selection = preload("res://scripts/core/settlement_selection.gd")

func _initialize() -> void:
	call_deferred("run")

func run() -> void:
	Selection.reset()
	var world := WorldSimNode.new()
	root.add_child(world)
	world.set_process(false)
	for attempt: int in 120:
		if not SimReads.read(world, "get_settlement_snapshot").get("settlement_ids", PackedInt32Array()).is_empty():
			break
		SimReads.mutate(world, "advance_ticks", [60])
	var settlement_ids: PackedInt32Array = SimReads.read(world, "get_settlement_snapshot")["settlement_ids"]
	assert(not settlement_ids.is_empty())
	var news := News.new()
	root.add_child(news)
	news.set_process(false)
	news._world_sim = world
	news._ingest_snapshots()
	var panel := GodPanel.new()
	root.add_child(panel)
	panel.set_process(false)
	panel.configure(world)
	Selection.select(settlement_ids[0])
	var button: Button = panel._buttons["gift_food"]
	for index: int in 40:
		button.emit_signal("pressed")
	var state: Dictionary = SimReads.read(world, "get_intervention_state")
	assert(state["log_ids"] == PackedInt64Array(range(8, 40)), "rolling source retains absolute identity")
	assert(state["log_settlement_ids"].size() == 32)
	var casts: Array[Dictionary] = []
	for record: Dictionary in news.get_feed_log():
		if record["key"] == "HUD_FEED_GOD_CAST_FMT":
			casts.append(record)
	assert(casts.size() == 40, "same-tick identical button casts each appear exactly once")
	assert(casts[0]["event_id"] == "cast:0" and casts[-1]["event_id"] == "cast:39")
	for record: Dictionary in casts:
		assert(record["args"][0] == Selection.number_for(settlement_ids[0]))
		assert(record["class"] == "protected" and record["settlement_id"] == settlement_ids[0])
	var food: Dictionary = news.capture_feed_metadata()["food"]
	assert(food["transfers"].size() == 40)
	for movement: Dictionary in food["movements"]:
		assert(movement["delta"] == 0 and movement["excluded_delta"] == 60, "cast never enters normal daily food")
	panel.cast_kind("cold_snap")
	var global_cast: Dictionary = news.get_feed_log().back()
	assert(global_cast["target_scope"] == "global" and global_cast["settlement_id"] == -1)
	assert(global_cast["context_settlement_id"] == settlement_ids[0], "global cast retains honest UI context")
	assert(news.get_visible_feed().size() == 4 and news._overflow_ids.size() >= 37)
	panel.free()
	news.free()
	world.free()
	print("FEED_CASTS PASS: real button, 40 identical same-tick casts, rolling IDs, exclusion, global context")
	quit()
