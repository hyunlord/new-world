# ACTION1 bridge contract in the shipped scene, with ordinary commands and a paused clock.
# Bridge methods via SimReads: get_visible_action_snapshot, get_visible_action_cues, get_agent_snapshot,
#   get_trees, get_building_snapshot, reset_world, cmd_set_steward, advance_ticks, cmd_draft,
#   cmd_draft_move, cmd_add_stockpile, cmd_designate_chop_tiles, save_world, load_world.
extends SceneTree

const SNAPSHOT_COLUMNS: Dictionary = {
	"agent_ids": TYPE_PACKED_INT64_ARRAY, "actions": TYPE_PACKED_BYTE_ARRAY, "phases": TYPE_PACKED_BYTE_ARRAY,
	"goal_xs": TYPE_PACKED_INT32_ARRAY, "goal_ys": TYPE_PACKED_INT32_ARRAY,
	"target_kinds": TYPE_PACKED_INT32_ARRAY, "target_xs": TYPE_PACKED_INT32_ARRAY,
	"target_ys": TYPE_PACKED_INT32_ARRAY, "target_ids": TYPE_PACKED_INT64_ARRAY,
	"partner_ids": TYPE_PACKED_INT64_ARRAY, "held_items": TYPE_PACKED_INT64_ARRAY,
	"held_materials": TYPE_PACKED_INT64_ARRAY, "held_quantities": TYPE_PACKED_INT32_ARRAY,
	"tool_items": TYPE_PACKED_INT64_ARRAY, "tool_materials": TYPE_PACKED_INT64_ARRAY,
	"tool_qualities": TYPE_PACKED_INT32_ARRAY, "recipes": TYPE_PACKED_INT64_ARRAY,
	"started_at_ticks": TYPE_PACKED_INT64_ARRAY, "until_ticks": TYPE_PACKED_INT64_ARRAY,
	"visual_keys": TYPE_PACKED_STRING_ARRAY, "fallback_keys": TYPE_PACKED_STRING_ARRAY,
}
const CUE_COLUMNS: Dictionary = {
	"agent_ids": TYPE_PACKED_INT64_ARRAY, "ticks": TYPE_PACKED_INT64_ARRAY,
	"sequences": TYPE_PACKED_INT64_ARRAY, "kinds": TYPE_PACKED_BYTE_ARRAY,
	"target_kinds": TYPE_PACKED_INT32_ARRAY, "target_xs": TYPE_PACKED_INT32_ARRAY,
	"target_ys": TYPE_PACKED_INT32_ARRAY, "target_ids": TYPE_PACKED_INT64_ARRAY,
	"items": TYPE_PACKED_INT64_ARRAY, "materials": TYPE_PACKED_INT64_ARRAY,
	"quantities": TYPE_PACKED_INT32_ARRAY, "visual_keys": TYPE_PACKED_STRING_ARRAY,
	"fallback_keys": TYPE_PACKED_STRING_ARRAY,
}

var _checks: int = 0
var _failures: int = 0


func _init() -> void:
	call_deferred("_run")


func _check(name: String, ok: bool) -> bool:
	_checks += 1
	if not ok:
		_failures += 1
	print("VISIBLE_ACTION %s %s" % ["PASS" if ok else "FAIL", name])
	return ok


func _finish() -> void:
	print("VISIBLE_ACTION_DONE failures=%d checks=%d" % [_failures, _checks])
	quit(1 if _failures > 0 else 0)


func _columns_match(snapshot: Dictionary, spec: Dictionary, size: int) -> bool:
	for key: String in spec:
		var column: Variant = snapshot.get(key)
		if typeof(column) != int(spec[key]) or column.size() != size:
			return false
	return true


func _snapshot(world: Node) -> Dictionary:
	return SimReads.read(world, "get_visible_action_snapshot")


func _order(world: Node, command: String, args: Array) -> bool:
	var result: Dictionary = SimReads.mutate(world, command, args)
	return bool(result.get("accepted", false))


func _run() -> void:
	var directory: String = OS.get_environment("WORLDSIM_TEST_SETTINGS_DIR")
	if not _check("isolated save directory", directory.is_absolute_path() and DirAccess.dir_exists_absolute(directory)
			and not directory.simplify_path().begins_with(ProjectSettings.globalize_path("res://"))):
		_finish()
		return
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(main)
	var world: Node = main.get_node("WorldSim")
	world.process_mode = Node.PROCESS_MODE_DISABLED
	_check("reset accepted", bool(SimReads.mutate(world, "reset_world", [100])))
	_check("steward disabled", _order(world, "cmd_set_steward", [false, 0]))
	SimReads.mutate(world, "advance_ticks", [360])
	var people: Dictionary = SimReads.read(world, "get_agent_snapshot")
	var people_ids: PackedInt64Array = people["agent_ids"]
	var ids: PackedInt64Array = people_ids.duplicate()
	ids.sort()
	var before: Dictionary = _snapshot(world)
	if not _check("one aligned action row per live agent", ids.size() > 1
			and _columns_match(before, SNAPSHOT_COLUMNS, ids.size()) and before["agent_ids"] == ids):
		_finish()
		return
	var strictly_sorted: bool = true
	for i: int in range(1, ids.size()):
		strictly_sorted = strictly_sorted and ids[i - 1] < ids[i]
	_check("strict AgentId order", strictly_sorted)
	var reads: SimReads = main.get_node("SimReads")
	_check("paused snapshot cached", _snapshot(world) == before
			and int(reads.frame_call_counts().get("get_visible_action_snapshot", 0)) == 1)

	# A paused movement order must replace a row without an intervening simulation tick.
	var worker: int = ids[0]
	var physical_row: int = people_ids.find(worker)
	var goal: Vector2i = Vector2i(people["xs"][physical_row] + 2, people["ys"][physical_row])
	_check("draft accepted", _order(world, "cmd_draft", [worker, true]))
	_check("move accepted", _order(world, "cmd_draft_move", [worker, goal.x, goal.y]))
	var moved: Dictionary = _snapshot(world)
	_check("paused command revision", before["tick"] == moved["tick"] and moved["revision"] > before["revision"])
	_check("paused order changes the selected agent row", int(moved["actions"][0]) == 18
			and int(moved["phases"][0]) == 1 and int(moved["goal_xs"][0]) == goal.x
			and int(moved["goal_ys"][0]) == goal.y and moved["fallback_keys"][0] == "walk"
			and int(moved["target_kinds"][0]) == 0 and int(moved["target_xs"][0]) == goal.x
			and int(moved["target_ys"][0]) == goal.y and int(moved["target_ids"][0]) == -1)
	_check("undraft accepted", _order(world, "cmd_draft", [worker, false]))

	var save_path: String = directory.path_join("action1-before-transfer.wsave")
	if not _check("save before transfer", bool(SimReads.mutate(world, "save_world", [save_path]))):
		_finish()
		return
	var frozen: Dictionary = _snapshot(world)
	var cues: Dictionary = _produce_cues(world)
	_check_cues(world, cues)
	var before_load: Dictionary = _snapshot(world)
	_check("load accepted", bool(SimReads.mutate(world, "load_world", [save_path])))
	var restored: Dictionary = _snapshot(world)
	_check("load invalidates same-frame snapshot", restored["tick"] == frozen["tick"]
			and restored["revision"] > before_load["revision"])
	_check("load discards cues", _empty_cues(world))
	var restored_fields: bool = true
	for key: String in SNAPSHOT_COLUMNS:
		restored_fields = restored_fields and restored[key] == frozen[key]
	_check("load restores all projected rows", restored_fields)

	# Replay the same harvest to check an independently populated reset boundary.
	cues = _produce_cues(world)
	_check("transfer witnessed again before reset", not cues["ticks"].is_empty())
	var before_reset: Dictionary = _snapshot(world)
	_check("second reset accepted", bool(SimReads.mutate(world, "reset_world", [100])))
	var reset: Dictionary = _snapshot(world)
	_check("reset invalidates same-frame snapshot", int(reset["tick"]) == 0 and reset["revision"] > before_reset["revision"])
	_check("reset discards cues", _empty_cues(world))
	_finish()


func _produce_cues(world: Node) -> Dictionary:
	var buildings: Dictionary = SimReads.read(world, "get_building_snapshot")
	var camp: Vector2i = Vector2i(buildings["xs"][0], buildings["ys"][0])
	_check("stockpile accepted", _order(world, "cmd_add_stockpile", [camp.x + 3, camp.y, camp.x + 6, camp.y + 3]))
	var trees: Dictionary = SimReads.read(world, "get_trees")
	var tree_xs: PackedInt32Array = trees["xs"]
	var tree_ys: PackedInt32Array = trees["ys"]
	var ranked: Array[Vector3i] = []
	for i: int in tree_xs.size():
		var distance: int = maxi(absi(tree_xs[i] - camp.x), absi(tree_ys[i] - camp.y))
		ranked.append(Vector3i(distance, tree_ys[i], tree_xs[i]))
	ranked.sort()
	var xs: PackedInt32Array = []
	var ys: PackedInt32Array = []
	for i: int in mini(15, ranked.size()):
		xs.append(ranked[i].z)
		ys.append(ranked[i].y)
	_check("chop accepted", _order(world, "cmd_designate_chop_tiles", [xs, ys]))
	var cues: Dictionary = SimReads.visible_action_cues(world, -1)
	for _block: int in 16:
		SimReads.mutate(world, "advance_ticks", [120])
		cues = SimReads.visible_action_cues(world, -1)
		if not cues["ticks"].is_empty():
			break
	return cues


func _check_cues(world: Node, cues: Dictionary) -> void:
	var ticks: PackedInt64Array = cues["ticks"]
	if not _check("physical action cues have packed equal-length columns",
			not ticks.is_empty() and ticks.size() <= 256 and _columns_match(cues, CUE_COLUMNS, ticks.size())):
		return
	var valid: bool = true
	var previous_tick: int = -1
	var previous_sequence: int = -1
	for i: int in ticks.size():
		var sequence: int = cues["sequences"][i]
		valid = valid and int(cues["quantities"][i]) > 0 and int(cues["items"][i]) >= 0
		valid = valid and int(cues["target_kinds"][i]) >= 0 and not str(cues["visual_keys"][i]).is_empty()
		valid = valid and (ticks[i] > previous_tick or (ticks[i] == previous_tick and sequence == previous_sequence + 1))
		previous_tick = ticks[i]
		previous_sequence = sequence
	_check("cues report physical payloads in commit order", valid)
	_check("cursor is exclusive", SimReads.visible_action_cues(world, ticks[-1])["ticks"].is_empty())
	_check("repeated reads return the same cue keys", SimReads.visible_action_cues(world, -1) == cues)


func _empty_cues(world: Node) -> bool:
	var cues: Dictionary = SimReads.visible_action_cues(world, -1)
	return _columns_match(cues, CUE_COLUMNS, 0) and int(cues["evicted_through_tick"]) == -1
