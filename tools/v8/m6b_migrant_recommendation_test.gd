# Recommendation admission only. No simulation mutation, forecast or survival claim.
extends SceneTree

const Cards = preload("res://scripts/ui/mux/card_panel.gd")

class FactsWorld:
	extends Node
	var wave: Dictionary = {"found": true, "id": 7, "state": 1, "agents": PackedInt64Array([20, 21]), "food_need_per_day": 2.0, "reachable_roofed_beds": 4}
	var facts: Dictionary = {
		"enabled": true,
		"people": {"ids": PackedInt64Array([0, 1]), "bed_xs": PackedInt64Array([10, 11]), "bed_ys": PackedInt64Array([10, 10])},
		"settlements": {"ids": PackedInt64Array([0]), "food": PackedInt64Array([20]), "food_need": PackedFloat64Array([2.0]), "wood": PackedInt64Array([20]), "wood_target": PackedFloat64Array([19.5])},
	}
	var rooms: Dictionary = {"ids": PackedInt32Array([0]), "beds": PackedInt32Array([4]), "doors": PackedInt32Array([1]), "roofed": PackedByteArray([1])}

	func get_wave(_id: int) -> Dictionary:
		return wave

	func get_ui_facts() -> Dictionary:
		return facts

	func get_rooms() -> Dictionary:
		return rooms

var _failures: int = 0
var _passes: int = 0
var _definition: Dictionary = {}

func _init() -> void:
	call_deferred("_run")

func _check(ok: bool, label: String) -> void:
	print("%s: %s" % ["PASS" if ok else "FAIL", label])
	if ok:
		_passes += 1
	else:
		_failures += 1

func _recommend(world: FactsWorld, label: String, accept: bool, reason: String = "", trigger: Dictionary = {"wave_id": 7, "count": 2}) -> void:
	var panel: PanelContainer = Cards.new()
	panel.set("_world", world)
	var data: Dictionary = trigger.duplicate(true)
	var option: int = int(panel.call("_effective_recommended", _definition, data))
	var options: Array = _definition["options"]
	var command: String = str(options[option].get("command", "")) if option >= 0 and option < options.size() else ""
	_check(command == ("cmd_accept_migrants" if accept else "cmd_decline_migrants"), label)
	if not reason.is_empty():
		_check(str(data.get("support_reason", "")) == reason, label + " reports shortage")
	panel.free()
	world.free()

func _run() -> void:
	var registry: Dictionary = JSON.parse_string(FileAccess.get_file_as_string("res://assets/ui/cards.json"))
	for definition: Dictionary in registry.get("cards", []):
		if str(definition.get("id", "")) == "migrants":
			_definition = definition
	_check(not _definition.is_empty(), "real migration card loaded")
	if _definition.is_empty():
		quit(1)
		return
	if "--live" in OS.get_cmdline_user_args():
		await _run_live()
		print("MIGRANT_LIVE_DONE pass=%d fail=%d" % [_passes, _failures])
		quit(0 if _failures == 0 else 1)
		return
	var commands: Array[String] = []
	for option: Dictionary in _definition.get("options", []):
		commands.append(str(option.get("command", "")))
	_check(commands.has("cmd_accept_migrants") and commands.has("cmd_decline_migrants"), "direct accept and decline remain available")
	_recommend(FactsWorld.new(), "qualified wave at exact 5-day food boundary may be accepted", true)
	for shortfall: Vector2 in [Vector2(12, 19.5), Vector2(36, 156)]:
		var world: FactsWorld = FactsWorld.new()
		world.facts["settlements"]["wood"] = PackedInt64Array([int(shortfall.x)])
		world.facts["settlements"]["wood_target"] = PackedFloat64Array([shortfall.y])
		_recommend(world, "captured fuel shortfall %s rejected" % str(shortfall), false, "M6B_MIGRANT_WOOD")
	var incoming_food: FactsWorld = FactsWorld.new()
	incoming_food.facts["settlements"]["food"] = PackedInt64Array([19])
	_recommend(incoming_food, "current people have 5 days but incoming demand makes food insufficient", false, "M6B_MIGRANT_FOOD")
	var incoming_beds: FactsWorld = FactsWorld.new()
	incoming_beds.wave["reachable_roofed_beds"] = 2
	_recommend(incoming_beds, "assigned current beds cannot replace incoming bed capacity", false, "M6B_MIGRANT_BEDS")
	for axis: String in ["bed_xs", "bed_ys"]:
		var world: FactsWorld = FactsWorld.new()
		world.facts["people"][axis] = PackedInt64Array([-1, 10])
		_recommend(world, "current resident lacks assigned " + axis, false, "M6B_MIGRANT_BEDS")
	var disconnected: FactsWorld = FactsWorld.new()
	disconnected.wave["reachable_roofed_beds"] = 2
	_recommend(disconnected, "roofed beds behind disconnected doors cannot supply incoming capacity", false, "M6B_MIGRANT_BEDS")
	var missing_capacity: FactsWorld = FactsWorld.new()
	missing_capacity.wave.erase("reachable_roofed_beds")
	_recommend(missing_capacity, "missing reachable capacity fails closed", false, "M6B_MIGRANT_UNKNOWN")
	for capacity: Variant in [-1, 4.0, "4", null]:
		var world: FactsWorld = FactsWorld.new()
		world.wave["reachable_roofed_beds"] = capacity
		_recommend(world, "unknown or malformed reachable capacity %s" % str(capacity), false, "M6B_MIGRANT_UNKNOWN")
	for key: String in ["people", "settlements"]:
		var world: FactsWorld = FactsWorld.new()
		world.facts.erase(key)
		_recommend(world, "missing " + key + " fails closed", false, "M6B_MIGRANT_UNKNOWN")
	for key: String in ["ids", "bed_xs", "bed_ys"]:
		var world: FactsWorld = FactsWorld.new()
		world.facts["people"].erase(key)
		_recommend(world, "missing people column " + key, false, "M6B_MIGRANT_UNKNOWN")
	for key: String in ["ids", "food", "food_need", "wood", "wood_target"]:
		var world: FactsWorld = FactsWorld.new()
		world.facts["settlements"].erase(key)
		_recommend(world, "missing settlement column " + key, false, "M6B_MIGRANT_UNKNOWN")
	var malformed: FactsWorld = FactsWorld.new()
	malformed.facts["people"]["bed_xs"] = PackedInt64Array([10])
	_recommend(malformed, "truncated assigned-bed columns fail closed", false, "M6B_MIGRANT_UNKNOWN")
	var missing_demand: FactsWorld = FactsWorld.new()
	missing_demand.wave.erase("food_need_per_day")
	_recommend(missing_demand, "missing incoming demand fails closed", false, "M6B_MIGRANT_UNKNOWN")
	for demand: float in [0.0, -1.0, NAN, INF]:
		var world: FactsWorld = FactsWorld.new()
		world.wave["food_need_per_day"] = demand
		_recommend(world, "invalid incoming demand %s" % str(demand), false, "M6B_MIGRANT_UNKNOWN")
	for trigger: Dictionary in [{"wave_id": 7, "count": 0}, {"wave_id": 7, "count": 1}, {"wave_id": 8, "count": 2}, {"count": 2}]:
		_recommend(FactsWorld.new(), "captured wave/count mismatch %s" % str(trigger), false, "M6B_MIGRANT_UNKNOWN", trigger)
	var handled: FactsWorld = FactsWorld.new()
	handled.wave["state"] = 2
	_recommend(handled, "handled wave cannot be recommended", false, "M6B_MIGRANT_UNKNOWN")
	var missing_wave: FactsWorld = FactsWorld.new()
	missing_wave.wave["found"] = false
	_recommend(missing_wave, "missing wave fails closed", false, "M6B_MIGRANT_UNKNOWN")
	var multiple: FactsWorld = FactsWorld.new()
	multiple.facts["settlements"] = {"ids": PackedInt64Array([0, 1]), "food": PackedInt64Array([20, 20]), "food_need": PackedFloat64Array([2.0, 2.0]), "wood": PackedInt64Array([20, 20]), "wood_target": PackedFloat64Array([19.5, 19.5])}
	_recommend(multiple, "multiple settlements lack attributable room capacity", false, "M6B_MIGRANT_UNKNOWN")
	var disabled: FactsWorld = FactsWorld.new()
	disabled.facts["enabled"] = false
	_recommend(disabled, "disabled colony facts fail closed", false, "M6B_MIGRANT_UNKNOWN")
	print("MIGRANT_RECOMMENDATION_DONE pass=%d fail=%d" % [_passes, _failures])
	quit(0 if _failures == 0 else 1)


func _run_live() -> void:
	var base: String = OS.get_environment("M6B_MIGRANT_EVIDENCE_DIR")
	_check(base.is_absolute_path(), "live evidence uses explicit external path")
	if not base.is_absolute_path():
		return
	var directory: String = base.path_join("run-%d" % OS.get_process_id())
	_check(DirAccess.make_dir_recursive_absolute(directory) == OK, "live evidence directory created")
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(main)
	var world: Node = main.get_node_or_null("WorldSim")
	_check(world != null, "live bootstrap creates WorldSim")
	if world == null:
		return
	world.set_process(false)
	world.call("set_sim_speed", 0.0)
	for _i: int in 5:
		await process_frame
	_check(bool(SimReads.mutate(world, "reset_world", [0])), "live seed0 reset succeeds")
	world.set_process(false)
	world.call("set_sim_speed", 0.0)
	var initial_tick: int = int((world.call("get_time_snapshot") as Dictionary).get("tick", -1))
	_check(initial_tick == 0, "live world starts at exact tick0")
	var neighbours: Dictionary = {}
	var wave_id: int = -1
	var wave_count: int = 0
	var ticks: int = 0
	while ticks <= 30 * 1440:
		neighbours = world.call("get_neighbours")
		var ids: PackedInt32Array = neighbours.get("wave_ids", PackedInt32Array())
		var states: PackedInt32Array = neighbours.get("wave_states", PackedInt32Array())
		var sizes: PackedInt32Array = neighbours.get("wave_sizes", PackedInt32Array())
		if ids.size() != states.size() or ids.size() != sizes.size():
			_check(false, "live neighbour columns aligned")
			break
		for i: int in ids.size():
			if states[i] == 1:
				wave_id = ids[i]
				wave_count = sizes[i]
				break
		if wave_id >= 0 or ticks == 30 * 1440:
			break
		SimReads.mutate(world, "advance_ticks", [60])
		ticks += 60
		await process_frame
		if ticks % 1440 == 0:
			print("MIGRANT_LIVE_PROGRESS tick=%d" % ticks)
	_check(wave_id >= 0, "real waiting wave observed by day30")
	if wave_id < 0:
		_write_live(directory, {"found": false, "tick": ticks, "neighbours": neighbours})
		return
	var tick_before: int = int((world.call("get_time_snapshot") as Dictionary).get("tick", -1))
	_check(tick_before == ticks, "live explicit advances have no clock drift")
	var commands_before: Dictionary = (world.call("get_command_log", 0) as Dictionary).duplicate(true)
	var wave: Dictionary = world.call("get_wave", wave_id)
	var facts: Dictionary = world.call("get_ui_facts")
	var rooms: Dictionary = world.call("get_rooms")
	_check(bool(wave.get("found", false)) and int(wave.get("id", -1)) == wave_id and int(wave.get("state", -1)) == 1, "native wave identity and waiting state match")
	var agents: Variant = wave.get("agents")
	_check(agents is PackedInt64Array and (agents as PackedInt64Array).size() == wave_count and wave_count > 0, "native wave agents marshal with actual positive count")
	var demand: Variant = wave.get("food_need_per_day")
	var numeric: bool = demand is float or demand is int
	_check(numeric and is_finite(float(demand)) and float(demand) > 0.0, "native incoming initial food demand is finite and positive")
	var cards: Node = main.find_child("CardPanel", true, false)
	_check(cards != null, "live CardPanel exists")
	if cards == null:
		_write_live(directory, {"wave": wave, "facts": facts, "rooms": rooms, "tick": tick_before})
		return
	var data: Dictionary = {"wave_id": wave_id, "count": wave_count}
	var option: int = int(cards.call("_effective_recommended", _definition, data))
	var options: Array = _definition.get("options", [])
	var command: String = str(options[option].get("command", "")) if option >= 0 and option < options.size() else ""
	var reason: String = str(data.get("support_reason", ""))
	_check(command == "cmd_decline_migrants", "real first-wave shortfall recommends decline")
	_check(reason in ["M6B_MIGRANT_BEDS", "M6B_MIGRANT_WOOD", "M6B_MIGRANT_FOOD"], "live reason names measured shortage rather than unknown native data")
	cards.set("_active", {"migrants": {"def": _definition, "trigger_data": data, "opened_tick": tick_before, "expires_tick": tick_before + 1440}})
	cards.call("_rebuild_ui")
	await process_frame
	var localized: String = str(cards.call("_tr", reason)).replace(char(0x2060), "")
	var visible_lines: Array[String] = []
	for child: Node in cards.find_children("*", "Label", true, false):
		var label: Label = child as Label
		if label.is_visible_in_tree():
			visible_lines.append(label.text.replace(char(0x2060), ""))
	_check(not localized.is_empty() and localized != reason and "\n".join(visible_lines).contains(localized), "visible live card body contains localized shortage reason")
	var tick_after: int = int((world.call("get_time_snapshot") as Dictionary).get("tick", -1))
	var commands_after: Dictionary = world.call("get_command_log", 0)
	_check(tick_before == tick_after, "recommendation and rendering do not advance simulation")
	_check(commands_before == commands_after, "recommendation and rendering send no commands")
	_write_live(directory, {"seed": 0, "wave": wave, "neighbours": neighbours, "facts": facts, "rooms": rooms,
		"tick_before": tick_before, "tick_after": tick_after, "commands_before": commands_before, "commands_after": commands_after,
		"trigger": data, "recommended_command": command, "visible_labels": visible_lines, "localized_reason": localized,
		"passes_before_write": _passes, "failures_before_write": _failures,
		"limits": "Initial-rate admission only; no future survival or rendered pixel-layout claim."})


func _write_live(directory: String, evidence: Dictionary) -> void:
	var path: String = directory.path_join("evidence.json")
	var file: FileAccess = FileAccess.open(path, FileAccess.WRITE)
	_check(file != null, "live evidence opens")
	if file == null:
		return
	file.store_string(JSON.stringify(evidence, "\t"))
	file.flush()
	_check(file.get_error() == OK, "live evidence persists")
	file.close()
	print("MIGRANT_LIVE_EVIDENCE " + path)
