extends SceneTree
const Candidate = preload("res://scripts/ui/agent_pose_moments.gd")
const Reference = preload("res://scripts/test/art/agent_pose_moments_reference_p10.gd")
var failures: int = 0

func check(ok: bool, message: String) -> void:
	if not ok:
		push_error(message)
		failures += 1

func _initialize() -> void:
	var candidate := Candidate.new()
	var legacy := Candidate.new()
	var reference := Reference.new()
	var output: Dictionary = {"unrelated": true}
	var hearths: Array = [Vector2i(10, 10)]
	var woods := PackedInt32Array([0, 0, 0, 0])
	var hungers := PackedFloat32Array([90, 90, 90, 90])
	for frame: int in range(240):
		for index: int in range(4):
			var agent_id: int = index + 10
			var tile := Vector2i(10 + index, 10)
			var stationary: bool = frame % 13 != 0
			var deadwood: bool = frame % 3 != 0
			if frame % 7 == 0:
				woods[index] += 1
			if frame % 11 == 0:
				hungers[index] -= 20.0
			if frame % 17 == 0:
				tile.x += 9
			var snapshot: Dictionary = {"carried_woods": woods, "hungers": hungers}
			if frame % 19 == 0:
				snapshot.erase("carried_woods")
			if frame % 23 == 0:
				snapshot["hungers"] = PackedInt32Array([1, 2, 3, 4])
			var read_index: int = index
			if frame % 29 == 0:
				read_index = -1
			if frame % 31 == 0:
				read_index = 40
			var delta: float = [-0.5, 0.0, 1.0 / 60.0, 0.4, 1.0, 3.1][frame % 6]
			var expected: Dictionary = reference.update(agent_id, read_index, snapshot, tile, stationary, deadwood, hearths, delta)
			candidate.update_into(agent_id, read_index, snapshot, tile, stationary, deadwood, hearths, delta, output)
			check(output == expected, "reused output matches original frame=%d id=%d" % [frame, agent_id])
			var fresh: Dictionary = legacy.update(agent_id, read_index, snapshot, tile, stationary, deadwood, hearths, delta)
			check(fresh == expected, "legacy API matches original")
			check(candidate._previous == reference._previous, "previous state exact")
			check(candidate._pickup == reference._pickup, "pickup state and timer exact")
			check(candidate._meal == reference._meal, "meal state and timer exact")
			check(not is_same(fresh, output), "legacy result distinct from reusable output")
			fresh["caller_mutation"] = true
			output["stale_key"] = true
		if frame % 37 == 0:
			candidate.retire(11)
			legacy.retire(11)
			reference.retire(11)
		if frame == 120:
			candidate.clear()
			legacy.clear()
			reference.clear()
	var empty: Dictionary = {}
	var one: Dictionary = legacy.update(99, 0, empty, Vector2i.ZERO, true, false, [], 0.0)
	var two: Dictionary = legacy.update(99, 0, empty, Vector2i.ZERO, true, false, [], 0.0)
	check(not is_same(one, two), "two legacy calls return distinct dictionaries")
	one["pickup"] = true
	check(not bool(two["pickup"]), "legacy caller mutation remains isolated")
	print("POSE REUSE P10 %s: 960 differential updates" % ("PASS" if failures == 0 else "FAIL"))
	quit(1 if failures else 0)
