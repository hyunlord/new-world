extends SceneTree
const Activity = preload("res://scripts/ui/agent_activity.gd")
const Hash = preload("res://scripts/ui/art_hash.gd")
const Renderer = preload("res://scripts/ui/agent_renderer.gd")
var failures: int = 0
func check(ok: bool, message: String) -> void:
	if not ok:
		push_error(message)
		failures += 1
func _initialize() -> void:
	call_deferred("run")
func run() -> void:
	var adult_poses: Dictionary = {}
	var child_poses: Dictionary = {}
	for day: int in [0, 1, 40]:
		for agent_id: int in range(100):
			var active: bool = posmod(Hash.mix(agent_id + day), 3) != 0
			for hour: float in [6.0, 12.0, 22.99]:
				var adult: String = Activity.camp_pose("idle", 0, agent_id, day, hour, true)
				var child: String = Activity.camp_pose("idle", 1, agent_id, day, hour, true)
				adult_poses[adult] = true
				child_poses[child] = true
				check(adult == ("sit" if active else "idle"), "adult day hash pose")
				check(child == ("hop" if active else "idle"), "child day hash pose")
				check(Activity.camp_pose("idle", 2, agent_id, day, hour, true) == "sit", "elders always sit at camp")
			for stage: int in [0, 1, 2]:
				for hour: float in [0.0, 5.99, 23.0, 23.99]:
					check(Activity.camp_pose("idle", stage, agent_id, day, hour, true) == "idle", "night excludes idle camp poses")
				check(Activity.camp_pose("idle", stage, agent_id, day, 12.0, false) == "idle", "outside camp unchanged")
				for activity: String in ["talk", "walk", "carry", "build", "asleep"]:
					check(Activity.camp_pose(activity, stage, agent_id, day, 12.0, true) == activity, "nonidle unchanged")
	check(adult_poses.has("sit") and adult_poses.has("idle"), "adult pose variety")
	check(child_poses.has("hop") and child_poses.has("idle"), "child pose variety")
	var renderer: Node = Renderer.new()
	for source: String in ["hearths", "shelters"]:
		var context: Dictionary = {"hearths": [], "shelters": {}, "sites": {}, "food": {}, "terrain": {}}
		if source == "hearths":
			context[source].append(Vector2i(10, 10))
		else:
			context[source][Vector2i(10, 10)] = true
		for offset: Vector2i in [Vector2i(3,3), Vector2i(-3,-3), Vector2i.ZERO]:
			check(bool(renderer.call("_tile_context", Vector2i(10,10)+offset, context)["near_camp"]), "Chebyshev3 included for " + source)
		check(not bool(renderer.call("_tile_context", Vector2i(14,10), context)["near_camp"]), "Chebyshev4 excluded for " + source)
		check(not bool(renderer.call("_tile_context", Vector2i(10,10), context)["wood"]), "legacy context without wood defaults false")
		context["wood"] = {Vector2i(10,10): true}
		check(bool(renderer.call("_tile_context", Vector2i(10,10), context)["wood"]), "deadwood tile included when optional context present")
		check(not bool(renderer.call("_tile_context", Vector2i(11,10), context)["wood"]), "deadwood context applies only to exact tile")
	renderer.free()
	check(Activity.resolve(0, 0, true, false, 0.0, {"near_hearth": true}, 18.0) == "idle", "semantic activity stays idle")
	print("IDLE CAMP %s: stages, day hash, hour and radius boundaries, semantic idle" % ("PASS" if failures == 0 else "FAIL"))
	quit(1 if failures else 0)
