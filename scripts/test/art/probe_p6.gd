extends SceneTree

const OUTPUT := "res://docs/art/production6/baseline/probe_live_seed.json"
var failures: Array[String] = []

func _initialize() -> void:
	call_deferred("run")

func _world() -> Node:
	var world: Node = ClassDB.instantiate("WorldSimNode") as Node
	world.call("set_sim_speed", 0.0)
	world.process_mode = Node.PROCESS_MODE_DISABLED
	root.add_child(world)
	return world

func _check(condition: bool, message: String) -> void:
	if not condition:
		failures.append(message)
		push_error(message)

func _advance(world: Node, target: int) -> void:
	var clock: Dictionary = world.call("get_time_snapshot")
	var tick: int = int(clock["tick"])
	while tick < target:
		var advanced: int = int(world.call("advance_ticks", mini(60, target - tick)))
		if advanced <= tick:
			_check(false, "advance_ticks stopped before target %d" % target)
			return
		tick = advanced

func _sample(world: Node, display_day: int, hour: int) -> Dictionary:
	var info: Dictionary = world.call("get_world_info")
	var target: int = (display_day - 1) * int(info["ticks_per_day"]) + hour * int(info["ticks_per_day"]) / 24 - int(info["clock_offset_ticks"])
	_advance(world, target)
	var clock: Dictionary = world.call("get_time_snapshot")
	var agents: Dictionary = world.call("get_agent_snapshot")
	var settlements: Dictionary = world.call("get_settlement_snapshot")
	var cold_ids: Array[int] = []
	var gathering_ids: Array[int] = []
	var hauling_ids: Array[int] = []
	var cold_sleeping_ids: Array[int] = []
	for index: int in range(agents["agent_ids"].size()):
		var id: int = int(agents["agent_ids"][index])
		var state: int = int(agents["states"][index])
		var target_kind: int = int(agents["target_kinds"][index])
		if int(agents["colds"][index]) == 1:
			cold_ids.append(id)
			if state == 3 and target_kind == 3:
				cold_sleeping_ids.append(id)
		if state == 3 and target_kind == 7:
			gathering_ids.append(id)
		if state == 1 and int(agents["carried_woods"][index]) > 0:
			hauling_ids.append(id)
	_check(int(clock["tick"]) == target, "target tick matches")
	_check(clock.has("winter_severity"), "time snapshot contains winter_severity")
	var row: Dictionary = {
		"display_day": display_day, "hour": hour, "time_snapshot": clock,
		"population": agents["agent_ids"].size(), "cold_count": cold_ids.size(),
		"cold_agent_ids": cold_ids, "cold_sleeping_ids": cold_sleeping_ids,
		"wood_gatherers": gathering_ids.size(), "wood_gatherer_ids": gathering_ids,
		"wood_haulers": hauling_ids.size(), "wood_hauler_ids": hauling_ids,
		"settlement_ids": Array(settlements["settlement_ids"]),
		"fire_lit": Array(settlements["fire_lit"]), "wood_stocks": Array(settlements["wood_stocks"]),
	}
	print("P6 PROBE " + JSON.stringify(row))
	return row

func run() -> void:
	var natural: Node = _world()
	var info: Dictionary = natural.call("get_world_info")
	_check(int(info.get("seed", -1)) == 20261010, "default live seed")
	var samples: Array[Dictionary] = []
	for day: int in [15, 16]:
		samples.append(_sample(natural, day, 15))
	for day: int in [23, 24, 25, 26, 27, 28, 29, 54, 55, 56, 57, 84, 85]:
		samples.append(_sample(natural, day, 2))
	natural.free()
	var intervention: Node = _world()
	var before: Dictionary = intervention.call("get_settlement_snapshot")
	_check(before["settlement_ids"].is_empty(), "fresh world has no settlement")
	var rejected: Dictionary = intervention.call("apply_intervention", "gift_food", -1, 10)
	_check(not bool(rejected.get("ok", true)), "gift_food rejected before settlement")
	var settlement: Dictionary = before
	var tick: int = 0
	while settlement["settlement_ids"].is_empty() and tick < 14400:
		tick = int(intervention.call("advance_ticks", 60))
		settlement = intervention.call("get_settlement_snapshot")
	_check(not settlement["settlement_ids"].is_empty(), "settlement formed within ten days")
	var accepted: Dictionary = intervention.call("apply_intervention", "gift_food", -1, 10)
	_check(bool(accepted.get("ok", false)), "gift_food accepted after settlement")
	_check(int(accepted.get("applied", -1)) >= 0, "gift_food applied is nonnegative")
	var result: Dictionary = {
		"schema_version": 1, "world_info": info,
		"source_contract": "world_node.rs init and reset_world use WorldGenConfig::game_v14 and WorldRules::game_v14",
		"planner_reference": {"cold_0200_days": [28, 29, 54, 55, 56, 57, 84, 85], "lit_0200_days": [23, 24, 25, 26, 27], "wood_1500": [{"day": 15, "gatherers": 14, "haulers": 9}, {"day": 16, "gatherers": 8, "haulers": 7}]},
		"natural_samples": samples, "natural_interventions": [],
		"intervention_probe": {"scenario": "INTERVENTION SCENARIO", "separate_fresh_world": true, "before_settlement": rejected, "settlement_tick": tick, "after_settlement": accepted},
		"failures": failures, "pass": failures.is_empty(),
	}
	DirAccess.make_dir_recursive_absolute(ProjectSettings.globalize_path(OUTPUT.get_base_dir()))
	var file: FileAccess = FileAccess.open(OUTPUT, FileAccess.WRITE)
	if file == null:
		_check(false, "cannot write probe output")
	else:
		file.store_string(JSON.stringify(result, "\t") + "\n")
		file.close()
	intervention.free()
	print("P6 LIVE PROBE %s" % ("PASS" if failures.is_empty() else "FAIL"))
	quit(0 if failures.is_empty() else 1)
