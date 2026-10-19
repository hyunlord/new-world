extends SceneTree
const Activity = preload("res://scripts/ui/agent_activity.gd")
var failures: int = 0
func check(ok: bool, message: String) -> void:
	if not ok:
		push_error(message)
		failures += 1
func _initialize() -> void:
	call_deferred("run")
func run() -> void:
	var elapsed: float = Activity.idle_elapsed("idle", Vector2.ZERO, 0.0, 0.99)
	check(Activity.render_facing("idle", Vector2.ZERO, Vector2.ZERO, "N", elapsed) == "N", "before one second preserves facing")
	elapsed = Activity.idle_elapsed("idle", Vector2.ZERO, elapsed, 0.01)
	check(Activity.render_facing("idle", Vector2.ZERO, Vector2.ZERO, "N", elapsed) == "S", "one render second faces viewer")
	check(Activity.render_facing("idle", Vector2.LEFT, Vector2.ZERO, "S", elapsed) == "W", "movement immediately restores facing")
	check(Activity.idle_elapsed("idle", Vector2.LEFT, elapsed, 0.1) == 0.0, "movement resets timer")
	check(Activity.idle_elapsed("talk", Vector2.ZERO, elapsed, 1.0) == 0.0, "nonidle resets timer")
	check(Activity.render_facing("talk", Vector2.ZERO, Vector2.UP, "S", 50.0) == "N", "talk keeps partner facing")
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	var world: Node = main.get_node("WorldSim")
	world.call("set_sim_speed", 0.0)
	world.process_mode = Node.PROCESS_MODE_DISABLED
	root.add_child(main)
	await process_frame
	var renderer: Node = main.get_node("AgentRenderer")
	renderer.set_process(false)
	renderer.call("refresh_snapshot")
	var before: Dictionary = SimReads.read(world, "get_agent_snapshot").duplicate(true)
	var time_before: Dictionary = SimReads.read(world, "get_time_snapshot").duplicate(true)
	var sprites: Array[Sprite2D] = renderer.call("get_active_agent_sprites")
	for sprite: Sprite2D in sprites:
		sprite.set_meta("facing", "N")
	renderer.set("_idle_seconds", {})
	renderer.call("_process", 1.0)
	var context: Dictionary = renderer.call("_activity_context")
	var stages: PackedByteArray = before.get("life_stages", PackedByteArray())
	var ids: PackedInt64Array = before["agent_ids"]
	var idle_count: int = 0
	for sprite: Sprite2D in sprites:
		if str(sprite.get_meta("activity")) == "idle":
			idle_count += 1
			var agent_id: int = int(sprite.get_meta("art_id"))
			var index: int = ids.find(agent_id)
			var stage: int = int(stages[index]) if index >= 0 and index < stages.size() else 0
			var tile: Dictionary = renderer.call("_tile_context", Vector2i(int(sprite.get_meta("tile_x")), int(sprite.get_meta("tile_y"))), context)
			var expected: String = Activity.camp_pose("idle", stage, agent_id, int(time_before.get("day_index", 0)), float(time_before.get("time_of_day", 12.0)), bool(tile["near_camp"]))
			check(str(sprite.get_meta("pose")) == expected, "actual renderer selects camp pose while activity remains idle")
			check(str(sprite.get_meta("facing")) == "S", "actual idle sprite faces S with frozen simulation")
	check(idle_count > 0, "actual snapshot includes idle agents")
	check(SimReads.resolve(world).peek_uncounted("get_agent_snapshot") == before, "render-time facing does not mutate simulation")
	check(SimReads.resolve(world).peek_uncounted("get_time_snapshot") == time_before, "render timer independent of simulation clock")
	renderer.call("_retire_missing", {})
	check((renderer.get("_idle_seconds") as Dictionary).is_empty(), "retired timers removed")
	renderer.set("_idle_seconds", {-999: 5.0})
	renderer.call("_on_world_reset")
	check(not (renderer.get("_idle_seconds") as Dictionary).has(-999), "world reset clears old timers")
	main.free()
	print("IDLE FACING %s: boundary, movement, talk, frozen renderer invariance" % ("PASS" if failures == 0 else "FAIL"))
	quit(1 if failures else 0)
