# Stage -1 passive UI identity and explicit role actions; run against the candidate bridge.
extends SceneTree

const Adapter = preload("res://scripts/ui/mux/ui_facts_adapter.gd")
const Priority = preload("res://scripts/ui/panels/priority_panel.gd")
const CardsRun = preload("res://tools/v8/loop_cards_run.gd")

class AttemptWorld:
	extends Node
	var attempts: PackedByteArray = PackedByteArray()
	var available_stock: int = 100
	var test_item: int = 3

	func get_ui_facts() -> Dictionary:
		return {"stacks": {"kinds": PackedInt64Array([test_item]), "counts": PackedInt64Array([1])}, "people": {"ids": PackedInt64Array([0]), "children": PackedByteArray([0])}}

	func get_colony_overlay() -> Dictionary:
		return {"designation_kinds": PackedInt32Array([0])}

	func get_neighbours() -> Dictionary:
		return {"wave_ids": PackedInt32Array([7]), "wave_sizes": PackedInt32Array([2]), "wave_states": PackedInt32Array([1]), "caravan_ids": PackedInt32Array([9]), "caravan_states": PackedInt32Array([1])}

	func get_caravan(_id: int) -> Dictionary:
		return {"found": true, "goods_value": PackedInt32Array([10]), "goods_count": PackedInt32Array([1]), "buy_permille": 800, "sell_permille": 1200, "stock_x": PackedInt32Array([12]), "stock_y": PackedInt32Array([13]), "stock_count": PackedInt32Array([100]), "stock_available_count": PackedInt32Array([available_stock]), "stock_value": PackedInt32Array([2])}

	func record_attempt(accepted: bool) -> void:
		attempts.append(1 if accepted else 0)

	func get_command_log(start: int) -> Dictionary:
		var kinds: PackedStringArray = PackedStringArray()
		for _i: int in range(start, attempts.size()):
			kinds.append("fixture_attempt")
		return {"total": attempts.size(), "accepted": attempts.slice(start), "kinds": kinds}


class RetryingCards:
	extends "res://scripts/ui/mux/card_panel.gd"

	func _alert_causes(_alert_id: String) -> Array:
		return []

	func _send_command(_command: String, _option: Dictionary, _card: Dictionary) -> Variant:
		_world.call("record_attempt", false)
		_world.call("record_attempt", false)
		return {"accepted": false, "skipped": true, "reason": "fixture_retry_exhausted"}

class RejectedCards:
	extends "res://scripts/ui/mux/card_panel.gd"

	func _send_command(_command: String, _option: Dictionary, _card: Dictionary) -> Variant:
		return {"accepted": false, "reason": "fixture_rejected"}

	func _rebuild_ui() -> void:
		pass

var _failed: int = 0


func _init() -> void:
	call_deferred("_run")


func _check(ok: bool, label: String) -> void:
	print("%s: %s" % ["PASS" if ok else "FAIL", label])
	if not ok:
		_failed += 1


func _run() -> void:
	if "--cards-acceptance-tests" in OS.get_cmdline_user_args():
		_test_cards_acceptance()
		print("CARDS_ACCEPTANCE_DONE failed=%d" % _failed)
		quit(0 if _failed == 0 else 1)
		return
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(main)
	for _i: int in 10:
		await process_frame
	var world: Node = main.get_node("WorldSim")
	world.call("set_sim_speed", 0.0)
	SimReads.mutate(world, "cmd_set_steward", [true, 0])
	var panel: Node = main.find_child("PriorityPanel", true, false)
	var cards: Node = main.find_child("CardPanel", true, false)
	if "--scoped-regressions" in OS.get_cmdline_user_args():
		_test_stale_teach_cancel(cards, world)
		_test_ambiguous_role(panel, world)
		quit(0 if _failed == 0 else 1)
		return
	_test_stale_teach_cancel(cards, world)
	var before: Dictionary = world.call("get_command_log", 0)
	var priorities: Dictionary = SimReads.read(world, "get_work_priorities").duplicate(true)
	panel.call("open_roles")
	for _i: int in 5:
		panel.call("_refresh")
		cards.call("_poll_triggers")
		cards.call("_check_expiry")
	_test_plaque_format(main, cards)
	_test_same_tick_plaque(main, world, cards)
	_test_rejected_card()
	_test_admission_contracts()
	_test_hunting_tool_catalog()
	_test_tool_count_data()
	_test_observers(main, world)
	var definitions: Array = cards.get("_card_defs")
	for definition: Dictionary in definitions:
		_check(str(definition.get("default", "")) == "none", "card expiry is none: " + str(definition["id"]))
		cards.set("_active", {str(definition["id"]): {"def": definition, "trigger_data": {}}})
		cards.call("_expire_card", str(definition["id"]))
	panel.set("visible", false)
	for definition: Dictionary in definitions:
		if definition["id"] == "idle_adults":
			cards.set("_active", {"idle_adults": {"def": definition, "trigger_data": {}}})
			cards.call("answer_card", "idle_adults", 0)
	_check(bool(panel.get("visible")), "idle-adults card actually opens roles")
	_check(before == world.call("get_command_log", 0), "open, poll, expiry and founder display emit zero commands")
	_check(priorities == SimReads.read(world, "get_work_priorities"), "founder priorities unchanged")
	var data: Dictionary = Priority.table(priorities)
	var works: PackedStringArray = data["works"]
	var roles: Array[Dictionary] = panel.get("_roles")
	_check(roles.size() == 4, "four data-driven presets loaded")
	var role_ids: PackedStringArray = []
	for role: Dictionary in roles:
		role_ids.append(str(role["id"]))
	for role_id: String in ["hunter_gatherer", "wood_build", "craft", "care_haul"]:
		_check(role_ids.has(role_id), "required role loaded: " + role_id)
	var extra: Dictionary = {"id": "extra_from_data", "levels": {"haul": 1}}
	var added: Array = roles.duplicate(true)
	added.append(extra)
	var parsed: Array[Dictionary] = Priority.parse_roles({"schema": 1, "roles": added}, works)
	_check(parsed.size() == roles.size() + 1, "data-only additional role parses")
	_check(str(panel.call("_role_label", extra)).contains("extra_from_data"), "unknown role label has visible fallback")
	for invalid: Variant in [null, {}, {"schema": 1, "roles": [{"id": "bad", "levels": {"missing": 1}}]}, {"schema": 1, "roles": [{"id": "bad", "levels": {"haul": 1.5}}]}, {"schema": 1, "roles": [extra, extra]}]:
		_check(Priority.parse_roles(invalid, works).is_empty(), "malformed role registry is unavailable")
	var rows: Array[Dictionary] = data["rows"]
	_check(not rows.is_empty(), "founder adults exist")
	if not rows.is_empty():
		panel.set("_roles", parsed)
		panel.set("_key", 0)
		panel.call("_refresh")
		var choice: OptionButton = panel.find_child("Role_%d" % int(rows[0]["agent"]), true, false) as OptionButton
		_check(choice != null and choice.item_count == parsed.size() + 1, "data-only role is rendered as a real choice")
		if choice != null:
			_check(choice.get_item_text(choice.item_count - 1).contains("extra_from_data"), "rendered additional role has fallback label")
			choice.item_selected.emit(choice.item_count - 1)
		_check(world.call("get_command_log", 0) != before, "explicit choice reaches command log")
		var after: Dictionary = Priority.table(SimReads.read(world, "get_work_priorities"))
		for r: int in rows.size():
			for w: int in works.size():
				var expected: int = 1 if r == 0 and works[w] == "haul" else int(rows[r]["levels"][w])
				_check(int(after["rows"][r]["levels"][w]) == expected, "only chosen person's named work changes")
		_check((panel.call("apply_role", int(rows[0]["agent"]), "extra_from_data") as Array).is_empty(), "repeat selection sends no redundant command")
	_test_ambiguous_role(panel, world)
	_test_cards_acceptance()
	_test_attempt_accounting()
	_test_tool_recipe(world)
	_test_knowledge_identity(cards, world)
	_test_observer_future(main, world)
	_test_card_route_contracts(main, world, cards)
	_test_typed_visitors_and_trade()
	if "--all-card-fixtures" in OS.get_cmdline_user_args():
		await _all_card_fixtures(main, world, cards)
	else:
		print("NOT_RUN: all-kind live bridge fixtures (pass --all-card-fixtures)")
	quit(0 if _failed == 0 else 1)


func _test_observers(main: Node, world: Node) -> void:
	var before: Dictionary = world.call("get_command_log", 0)
	var goals: Node = main.find_child("Goals", true, false)
	var plaque: Node = main.find_child("TaskPlaque", true, false)
	var chronicle: Node = main.find_child("ChroniclePanel", true, false)
	_check(goals != null and plaque != null and chronicle != null, "chapter/plaque/chronicle observer nodes exist")
	if goals != null:
		var facts: Dictionary = {}
		for chapter: Dictionary in goals.get("chapters"):
			for condition: Dictionary in chapter.get("conditions", []):
				facts[str(condition["fact"])] = int(condition.get("threshold", 1))
		goals.call("update_facts", facts)
		_check(int(goals.call("current_chapter")) > 0, "chapter completion callback exercised")
	if plaque != null:
		plaque.call("_refresh")
	if chronicle != null:
		chronicle.call("add_presentation_entry", "M6B observer regression")
		chronicle.call("_refresh")
	_check(world.call("get_command_log", 0) == before, "chapter completion, plaque and chronicle emit zero commands")


func _test_knowledge_identity(cards: Node, world: Node) -> void:
	var alerts: Node = cards.call("_get_alerts_panel")
	var evaluator: RefCounted = alerts.get("evaluator")
	evaluator.call("evaluate", Adapter.adapt(world.call("get_ui_facts")))
	var tech: Dictionary = world.call("get_tech")
	var ids: PackedInt64Array = tech.get("ids", PackedInt64Array())
	var names: PackedInt64Array = tech.get("first_names", PackedInt64Array())
	var practitioners: PackedInt64Array = tech.get("practitioners", PackedInt64Array())
	var probes: int = 0
	for i: int in ids.size():
		if i >= names.size() or i >= practitioners.size() or practitioners[i] != 1:
			continue
		probes += 1
		_check(bool(cards.call("_teacher_holds_knowledge", names[i], ids[i])), "actual holder matches distinct technique id")
		_check(not bool(cards.call("_teacher_holds_knowledge", names[i], 99999)), "unrelated knowledge is not held")
	_check(probes > 0, "knowledge identity fixture exercised an at-risk technique")


## Live bridge acceptance by kind. Inputs come from the world; skipped stale targets
## stay uncovered, and sent rejections fail. This does not replace the natural run.
func _all_card_fixtures(_main: Node, world: Node, cards: Node) -> void:
	var registry: Dictionary = JSON.parse_string(FileAccess.get_file_as_string("res://assets/ui/cards.json"))
	var required_kinds: Dictionary = {}
	var required_commands: Dictionary = {}
	for definition: Dictionary in registry["cards"]:
		required_kinds[str(definition["id"])] = true
		for option: Dictionary in definition["options"]:
			required_commands[str(option["command"])] = true
	var covered_kinds: Dictionary = {}
	var covered_commands: Dictionary = {}
	var evidence: Array[Dictionary] = []
	world.set_process(false)
	world.call("set_sim_speed", 0.0)
	for seed: int in range(8):
		SimReads.mutate(world, "reset_world", [seed])
		world.call("set_sim_speed", 0.0)
		for _frame: int in 2:
			await process_frame
		world.call("set_sim_speed", 0.0)
		_check(int((world.call("get_time_snapshot") as Dictionary)["tick"]) == 0, "all-route fixture starts exactly at tick zero")
		_check_camp_target(world, cards)
		SimReads.mutate(world, "cmd_set_steward", [true, 0])
		for hour: int in range(48 * 24):
			var alerts: Node = cards.call("_get_alerts_panel")
			(alerts.get("evaluator") as RefCounted).call("evaluate", Adapter.adapt(world.call("get_ui_facts")))
			for definition: Dictionary in registry["cards"]:
				var id: String = str(definition["id"])
				var options: Array = definition["options"]
				for option_index: int in options.size():
					var command: String = str(options[option_index]["command"])
					if covered_kinds.has(id) and covered_commands.has(command):
						continue
					var trigger: Dictionary = cards.call("_collect_trigger_data", definition)
					if bool(cards.call("_option_hidden", options[option_index], trigger)):
						continue
					if not _prepare_fixture_target(world, cards, command, trigger, evidence):
						_persist_fixture_failure(world, seed, hour, evidence)
						return
					var pre_state: Dictionary = {"drafted": world.call("get_drafted"), "priorities": world.call("get_work_priorities"), "tick": world.call("get_time_snapshot"), "camp": cards.call("_camp_tile")}
					var previous: int = (cards.call("command_results") as Array).size()
					cards.set("_active", {id: {"def": definition, "trigger_data": trigger}})
					cards.call("answer_card", id, option_index)
					var receipts: Array = cards.call("command_results")
					if receipts.size() > previous:
						for receipt_index: int in range(previous, receipts.size()):
							var receipt: Dictionary = receipts[receipt_index]
							var entry: Dictionary = {"seed": seed, "hour": hour, "kind": id, "receipt": receipt, "trigger": trigger, "option": options[option_index], "pre_state": pre_state}
							evidence.append(entry)
							print("ALL_KIND_RECEIPT " + JSON.stringify(entry))
							_check(bool(receipt.get("accepted", false)), "live card command accepted: " + id + "/" + command)
							if not bool(receipt.get("accepted", false)):
								_persist_fixture_failure(world, seed, hour, evidence)
								return
						covered_kinds[id] = true
						covered_commands[command] = true
					elif command in ["none", "steward_default", "open_roles", "open_build_menu", "open_trade"] and not (cards.call("active_ids") as PackedStringArray).has(id):
						covered_kinds[id] = true
						covered_commands[command] = true
						evidence.append({"seed": seed, "hour": hour, "kind": id, "local_action": command, "sent_commands": 0})
			if covered_kinds.size() == required_kinds.size() and covered_commands.size() == required_commands.size():
				break
			SimReads.mutate(world, "advance_ticks", [60])
			await process_frame
			var tick: int = int((world.call("get_time_snapshot") as Dictionary)["tick"])
			if tick != (hour + 1) * 60:
				_check(false, "fixture wall clock drift")
				_persist_fixture_failure(world, seed, hour, evidence)
				return
		print("ALL_KIND_WORLD_LOG seed=%d %s" % [seed, JSON.stringify(world.call("get_command_log", 0))])
		if covered_kinds.size() == required_kinds.size() and covered_commands.size() == required_commands.size():
			break
	for kind: String in required_kinds:
		_check(covered_kinds.has(kind), "live fixture acceptance coverage: " + kind)
	for command: String in required_commands:
		_check(covered_commands.has(command), "live selectable command coverage: " + command)
	print("ALL_KIND_COVERAGE " + JSON.stringify({"required_kinds": required_kinds.keys(), "covered_kinds": covered_kinds.keys(), "required_commands": required_commands.keys(), "covered_commands": covered_commands.keys()}))
	print("ALL_KIND_CARD_EVIDENCE " + JSON.stringify(evidence))


## Test setup is explicit and recorded; it must not be mistaken for a card receipt.
func _prepare_fixture_target(world: Node, cards: Node, command: String, trigger: Dictionary, evidence: Array[Dictionary]) -> bool:
	if command == "cmd_add_hunt_zone" and bool(cards.call("_has_hunter_blades")):
		var zones: PackedInt32Array = (world.call("get_animals") as Dictionary).get("zones", PackedInt32Array())
		while zones.size() >= 4:
			var cancelled: Dictionary = SimReads.mutate(world, "cmd_cancel_hunt", [zones[0], zones[1], zones[2], zones[3]])
			evidence.append({"fixture_setup_for": command, "action": "cancel_existing_hunt_zone", "rectangle": [zones[0], zones[1], zones[2], zones[3]], "receipt": cancelled})
			print("ALL_KIND_SETUP " + JSON.stringify(evidence.back()))
			_check(bool(cancelled.get("accepted", false)), "fixture existing hunt-zone cancellation accepted")
			if not bool(cancelled.get("accepted", false)):
				return false
			zones = (world.call("get_animals") as Dictionary).get("zones", PackedInt32Array())
	if command not in ["cmd_undraft_all", "cmd_set_work_priority"]:
		return true
	var priorities: Dictionary = world.call("get_work_priorities")
	var adults: PackedInt64Array = priorities.get("agent_ids", PackedInt64Array())
	if adults.is_empty():
		return true
	var result: Dictionary = {}
	if command == "cmd_undraft_all":
		var drafted: Dictionary = world.call("get_drafted")
		if (drafted.get("ids", PackedInt64Array()) as PackedInt64Array).is_empty():
			result = SimReads.mutate(world, "cmd_draft", [adults[0], true])
	else:
		trigger["agent_a"] = adults[0]
		var works: PackedStringArray = priorities.get("work_types", PackedStringArray())
		var levels: PackedByteArray = priorities.get("levels", PackedByteArray())
		var column: int = works.find("chop")
		if column >= 0 and column < levels.size() and levels[column] == 3:
			result = SimReads.mutate(world, "cmd_set_work_priority", [adults[0], "chop", 2])
	if not result.is_empty():
		evidence.append({"fixture_setup_for": command, "receipt": result})
		print("ALL_KIND_SETUP " + JSON.stringify(evidence.back()))
		_check(bool(result.get("accepted", false)), "fixture precondition accepted: " + command)
		return bool(result.get("accepted", false))
	return true



## Same saved world, same future ticks: only one arm invokes presentation observers.
func _test_observer_future(main: Node, world: Node) -> void:
	# The engine node is registered by bootstrap, not directly instantiable.
	# Serial save/reload arms use the same engine and exact initial bytes.
	var evidence_dir: String = OS.get_environment("M6B_PRESENTATION_EVIDENCE_DIR")
	if evidence_dir.is_empty():
		var test_dir: String = OS.get_environment("WORLDSIM_TEST_SETTINGS_DIR")
		if test_dir.is_empty():
			test_dir = "/tmp"
		evidence_dir = test_dir.path_join("m6b-presentation-%d" % OS.get_process_id())
	var made: Error = DirAccess.make_dir_recursive_absolute(evidence_dir)
	_check(made == OK, "external evidence directory ready")
	if made != OK:
		return
	var stem: String = evidence_dir.path_join("observer-%d" % OS.get_process_id())
	var initial: String = stem + "-initial.wsave"
	var observed: String = stem + "-observed.wsave"
	var unchanged: String = stem + "-control.wsave"
	var saved: bool = bool(world.call("save_world", initial))
	_check(saved, "observer/control initial full state saved")
	if saved:
		SimReads.mutate(world, "advance_ticks", [120])
		var control_log: Dictionary = world.call("get_command_log", 0)
		var saved_control: bool = bool(world.call("save_world", unchanged))
		var loaded: bool = bool(SimReads.mutate(world, "load_world", [initial]))
		_check(saved_control and loaded, "control future saved and identical initial state reloaded")
		if saved_control and loaded:
			for _i: int in 10:
				_test_observers(main, world)
				var cards: Node = main.find_child("CardPanel", true, false)
				cards.call("_poll_triggers")
				cards.call("_check_expiry")
			SimReads.mutate(world, "advance_ticks", [120])
			_check(world.call("get_command_log", 0) == control_log, "observer/control future command logs identical")
			var saved_observed: bool = bool(world.call("save_world", observed))
			_check(saved_observed, "observed future full state saved")
			if saved_observed:
				_check(FileAccess.get_file_as_bytes(observed) == FileAccess.get_file_as_bytes(unchanged), "observer/control future full save bytes identical")
	print("OBSERVER_RAW_EVIDENCE " + JSON.stringify([initial, unchanged, observed]))


func _test_tool_recipe(world: Node) -> void:
	var registry: Dictionary = JSON.parse_string(FileAccess.get_file_as_string("res://assets/ui/cards.json"))
	var recipes: Dictionary = world.call("get_recipes")
	var names: PackedStringArray = recipes.get("names", PackedStringArray())
	for definition: Dictionary in registry["cards"]:
		for option: Dictionary in definition.get("options", []):
			if str(option.get("command", "")) == "cmd_add_bill":
				_check(names.has(str(option.get("recipe", ""))), "card bill names a real recipe rather than an item")



## Scripted log contract, not a substitute for live bridge card acceptance.
func _test_attempt_accounting() -> void:
	var fake_world: AttemptWorld = AttemptWorld.new()
	var fake_cards: RetryingCards = RetryingCards.new()
	fake_cards.set("_world", fake_world)
	fake_cards.set("_active", {"fixture": {"def": {"id": "fixture", "title_key": "fixture", "options": [{"command": "cmd_add_hunt_zone", "text_key": "fixture"}]}, "trigger_data": {}}})
	fake_cards.call("answer_card", "fixture", 0)
	var receipts: Array = fake_cards.call("command_results")
	_check(receipts.size() == 2, "every rejected retry retained despite final skipped result")
	for receipt: Dictionary in receipts:
		_check(not bool(receipt.get("accepted", true)), "rejected retry remains rejected")
	_check((fake_cards.call("skipped_results") as Array).is_empty(), "sent retries are not counted as presend skips")
	fake_cards.free()
	fake_world.free()



func _persist_fixture_failure(world: Node, seed: int, hour: int, evidence: Array[Dictionary]) -> void:
	var directory: String = OS.get_environment("M6B_PRESENTATION_EVIDENCE_DIR")
	if directory.is_empty():
		directory = OS.get_environment("WORLDSIM_TEST_SETTINGS_DIR")
	if directory.is_empty():
		directory = "/tmp"
	directory = directory.path_join("all-kind-%d-seed%d-hour%d" % [OS.get_process_id(), seed, hour])
	var made: Error = DirAccess.make_dir_recursive_absolute(directory)
	_check(made == OK, "fixture failure evidence directory created")
	var log: Dictionary = world.call("get_command_log", 0)
	print("ALL_KIND_FAILURE_WORLD_LOG " + JSON.stringify(log))
	print("ALL_KIND_CARD_EVIDENCE " + JSON.stringify(evidence))
	if made != OK:
		return
	var snapshot_path: String = directory.path_join("rejected-world.wsave")
	var saved: bool = bool(world.call("save_world", snapshot_path))
	var observer_state: Dictionary = {"ui_facts": world.call("get_ui_facts"), "priorities": world.call("get_work_priorities"), "drafted": world.call("get_drafted"), "time": world.call("get_time_snapshot")}
	var receipt_path: String = directory.path_join("failure.json")
	var file: FileAccess = FileAccess.open(receipt_path, FileAccess.WRITE)
	_check(file != null, "fixture failure receipt opened")
	if file != null:
		file.store_string(JSON.stringify({"seed": seed, "hour": hour, "evidence": evidence, "world_command_log": log, "observer_state": observer_state, "snapshot_saved": saved, "snapshot_path": snapshot_path if saved else "", "snapshot_failure": "" if saved else "bridge rejected save; raw observer state and command log retained"}, "\t"))
		file.flush()
		_check(file.get_error() == OK, "fixture failure receipts persisted")
		file.close()
	print("ALL_KIND_FAILURE_PATHS " + JSON.stringify({"receipt": receipt_path, "snapshot": snapshot_path if saved else "UNAVAILABLE", "snapshot_saved": saved}))



func _test_card_route_contracts(main: Node, world: Node, cards: Node) -> void:
	var fake_world: AttemptWorld = AttemptWorld.new()
	var probe: RetryingCards = RetryingCards.new()
	probe.set("_world", fake_world)
	_check(bool(probe.call("_has_chop_designations")), "chop guard accepts bridge PackedInt32 designation kinds")
	probe.free()
	fake_world.free()
	var definitions: Array = cards.get("_card_defs")
	for definition: Dictionary in definitions:
		if str(definition["id"]) == "wolves":
			var previous: int = (cards.call("command_results") as Array).size()
			cards.set("_active", {"wolves": {"def": definition, "trigger_data": {}}})
			cards.call("answer_card", "wolves", 0)
			var receipts: Array = cards.call("command_results")
			_check(receipts.size() == previous + 1, "explicit draft action sends one real adult command")
			if receipts.size() > previous:
				_check(bool(receipts.back().get("accepted", false)), "explicit adult draft accepted")
		if str(definition["id"]) == "storage":
			var before: Dictionary = world.call("get_command_log", 0)
			cards.set("_active", {"storage": {"def": definition, "trigger_data": {}}})
			cards.call("answer_card", "storage", 0)
			var bar: Node = main.find_child("OrderBar", true, false)
			_check(bar != null and bool(bar.call("build_menu_open")), "storage construction action opens real build menu")
			_check(world.call("get_command_log", 0) == before, "construction menu navigation sends no command")
			if bar != null:
				bar.call("close_build_menu")



func _test_typed_visitors_and_trade() -> void:
	var fake_world: AttemptWorld = AttemptWorld.new()
	var probe: RetryingCards = RetryingCards.new()
	probe.set("_world", fake_world)
	var migrant: Dictionary = probe.call("_collect_trigger_data", {"id": "migrants", "trigger": {"kind": "alert", "alert_id": "migrants_waiting"}})
	var caravan: Dictionary = probe.call("_collect_trigger_data", {"id": "trader", "trigger": {"kind": "alert", "alert_id": "trader_present"}})
	_check(int(migrant.get("wave_id", -1)) == 7, "migrant route reads PackedInt32 wave IDs")
	_check(int(caravan.get("caravan_id", -1)) == 9, "trader route reads PackedInt32 caravan IDs")
	_check(int(probe.call("_visitor_id", {"wave_ids": PackedInt32Array([7]), "wave_states": PackedInt32Array([2])}, "wave")) == -1, "answered visitor is not an actionable target")
	var terms: Dictionary = probe.call("_build_trade_terms", 9)
	_check(bool(terms.get("valid", false)), "trade can buy one item using multiple units from a stock stack")
	if bool(terms.get("valid", false)):
		var take: PackedInt32Array = terms["take"]
		var give: PackedInt32Array = terms["give"]
		_check(take == PackedInt32Array([0, 1]), "trade take is index/count pairs")
		_check(give.size() == 3 and give[2] * 2 * 800 >= 10 * 1200, "trade pays actual buy/sell weighted price")
	probe.free()
	fake_world.free()



func _check_camp_target(world: Node, cards: Node) -> void:
	var camp: Vector2i = cards.call("_camp_tile")
	var info: Dictionary = world.call("get_world_info")
	_check(camp.x >= 0 and camp.y >= 0 and camp.x < int(info["width"]) and camp.y < int(info["height"]), "card camp target lies in actual world bounds")
	var founders: Dictionary = world.call("get_agent_snapshot")
	var xs: PackedInt32Array = founders.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = founders.get("ys", PackedInt32Array())
	var nearest: int = 2147483647
	for i: int in mini(xs.size(), ys.size()):
		nearest = mini(nearest, maxi(absi(xs[i] - camp.x), absi(ys[i] - camp.y)))
	_check(nearest <= 6, "card camp target is near actual initial founders, not assumed tile32")


func _test_plaque_format(main: Node, cards: Node) -> void:
	var active: Dictionary = cards.get("_active")
	cards.set("_active", {})
	var plaque: Node = main.find_child("TaskPlaque", true, false)
	plaque.call("_refresh")
	var shown: String = plaque.call("plaque_text")
	_check(not shown.contains("{") and not shown.contains("}"), "plaque condition contains no unresolved chapter tokens")
	cards.set("_active", active)


func _test_rejected_card() -> void:
	var panel: RejectedCards = RejectedCards.new()
	panel.set("_active", {"fixture": {"def": {"id": "fixture", "title_key": "fixture", "options": [{"command": "cmd_add_bill", "text_key": "fixture"}]}, "trigger_data": {}}})
	var emitted: Array = []
	panel.card_answered.connect(func(_id: String, _title: String, _option: String) -> void: emitted.append(true))
	panel.call("answer_card", "fixture", 0)
	var active: Dictionary = panel.get("_active")
	_check(active.has("fixture"), "rejected card stays available")
	_check(str(active.get("fixture", {}).get("failure_reason", "")) == "fixture_rejected", "rejected card retains visible failure reason")
	_check((panel.get("_cooldown") as Dictionary).is_empty() and emitted.is_empty(), "rejected card has no answer event or cooldown")
	panel.free()


func _test_admission_contracts() -> void:
	var fake: AttemptWorld = AttemptWorld.new()
	var panel: RetryingCards = RetryingCards.new()
	panel.set("_world", fake)
	var neighbours: Dictionary = {"wave_ids": PackedInt32Array([7, 8]), "wave_states": PackedInt32Array([2, 1])}
	_check(int(panel.call("_visitor_id", neighbours, "wave", 7)) == -1, "departed captured visitor never redirects to another visitor")
	_check(int(panel.call("_visitor_id", neighbours, "wave", 8)) == 8, "captured waiting visitor is still actionable")
	var node: Dictionary = {"learner_ids": PackedInt64Array([0, 2])}
	var orders: Dictionary = {"learners": PackedInt64Array([2]), "knowledge": PackedInt64Array([10])}
	_check(not bool(panel.call("_learner_available", 1, 10, node, orders)), "already practising or ineligible learner excluded")
	_check(not bool(panel.call("_learner_available", 2, 10, node, orders)), "duplicate learner technique order excluded")
	_check(bool(panel.call("_learner_available", 0, 10, node, orders)), "eligible founder zero is a valid learner")
	fake.available_stock = 0
	_check(not bool((panel.call("_build_trade_terms", 9) as Dictionary).get("valid", false)), "owed or stale raw stock cannot fund trade")
	panel.free()
	fake.free()


func _test_same_tick_plaque(main: Node, world: Node, cards: Node) -> void:
	var plaque: Node = main.find_child("TaskPlaque", true, false)
	var active: Dictionary = cards.get("_active")
	var cooldown: Dictionary = cards.get("_cooldown")
	var tick: int = int(world.call("get_time_snapshot").get("tick", -1))
	var commands: Dictionary = world.call("get_command_log", 0)
	cards.set("_active", {})
	cards.set("_cooldown", {})
	plaque.call("_refresh")
	cards.call("_poll_triggers")
	_check(bool(cards.call("has_active_cards")), "same-tick fixture opens a real triggered card")
	_check(str(plaque.call("plaque_text")).replace("\u2060", "") == str(cards.call("top_card_title")).replace("\u2060", ""), "actual card poll updates plaque without advancing tick")
	var ids: PackedStringArray = cards.call("active_ids")
	for id: String in ids:
		cards.call("_expire_card", id)
		if bool(cards.call("has_active_cards")):
			_check(str(plaque.call("plaque_text")).replace("\u2060", "") == str(cards.call("top_card_title")).replace("\u2060", ""), "same-tick expiry advances plaque to next card")
	_check(not str(plaque.call("plaque_text")).is_empty(), "last-card expiry restores guidance")
	_check(int(world.call("get_time_snapshot").get("tick", -1)) == tick and world.call("get_command_log", 0) == commands, "plaque card-state updates advance no ticks and send no commands")
	cards.set("_active", active)
	cards.set("_cooldown", cooldown)
	plaque.call("_refresh")


func _test_hunting_tool_catalog() -> void:
	var fake: AttemptWorld = AttemptWorld.new()
	var panel: RetryingCards = RetryingCards.new()
	panel.set("_world", fake)
	var content = preload("res://scripts/ui/content_context.gd")
	for item: String in ["flint", "stone_axe", "flint_blade", "spear"]:
		fake.test_item = content.code("items", item)
		_check(fake.test_item >= 0, "hunt fixture catalog item exists: " + item)
		_check(bool(panel.call("_has_hunter_blades")) == (item in ["flint_blade", "spear"]), "hunter guard uses authored capability: " + item)
	panel.free()
	fake.free()


func _test_tool_count_data() -> void:
	var fake: AttemptWorld = AttemptWorld.new()
	var panel: RetryingCards = RetryingCards.new()
	panel.set("_world", fake)
	var registry: Dictionary = JSON.parse_string(FileAccess.get_file_as_string("res://assets/ui/cards.json"))
	var definitions: Array[Dictionary] = []
	for definition: Dictionary in registry["cards"]:
		definitions.append(definition)
	panel.set("_card_defs", definitions)
	var content = preload("res://scripts/ui/content_context.gd")
	for item: String in ["flint", "stone_axe", "flint_blade", "spear"]:
		fake.test_item = content.code("items", item)
		_check(bool(panel.call("_check_bridge_trigger", "tools_below_need")) == (item not in ["stone_axe", "flint_blade"]), "tools card counts only its authored axes/blades: " + item)
	for definition: Dictionary in definitions:
		if str(definition["id"]) == "tools_low":
			definition["tool_items"] = ["spear"]
	fake.test_item = content.code("items", "spear")
	_check(not bool(panel.call("_check_bridge_trigger", "tools_below_need")), "data-only tool list can select another catalog item")
	panel.free()
	fake.free()


func _test_stale_teach_cancel(cards: Node, world: Node) -> void:
	var orders: Dictionary = world.call("get_teach_orders")
	var ids: PackedInt64Array = orders.get("ids", PackedInt64Array())
	var stale_id: int = 1
	while ids.has(stale_id):
		stale_id += 1
	var active: Dictionary = cards.get("_active")
	var previous: int = (cards.call("command_results") as Array).size()
	var skipped_before: int = (cards.call("skipped_results") as Array).size()
	var before: Dictionary = world.call("get_command_log", 0)
	cards.set("_active", {"stale_teach": {"def": {"id": "stale_teach", "title_key": "fixture", "options": [{"command": "cmd_cancel_teach", "text_key": "fixture"}]}, "trigger_data": {"order_id": stale_id}}})
	cards.call("answer_card", "stale_teach", 0)
	_check(world.call("get_command_log", 0) == before, "non-live teaching ID sends no native command")
	_check((cards.call("command_results") as Array).size() == previous, "stale teaching cancellation produces no rejected sent receipt")
	var skipped: Array = cards.call("skipped_results")
	_check(skipped.size() == skipped_before + 1 and str(skipped.back().get("reason", "")) == "no_order", "stale teaching cancellation is an explicit pre-send skip")
	cards.set("_active", active)


func _test_ambiguous_role(panel: Node, world: Node) -> void:
	panel.call("open_roles")
	var registry: Dictionary = JSON.parse_string(FileAccess.get_file_as_string("res://assets/ui/roles.json"))
	var data: Dictionary = Priority.table(SimReads.read(world, "get_work_priorities"))
	var roles_before: Array[Dictionary] = panel.get("_roles")
	panel.set("_roles", Priority.parse_roles(registry, data["works"]))
	var rows: Array[Dictionary] = data["rows"]
	_check(not rows.is_empty(), "mixed role fixture has an adult")
	if not rows.is_empty():
		var agent: int = int(rows[0]["agent"])
		for role_id: String in ["hunter_gatherer", "craft"]:
			var receipts: Array = panel.call("apply_role", agent, role_id)
			for receipt: Dictionary in receipts:
				_check(bool(receipt.get("accepted", false)), "explicit role fixture command accepted: " + role_id)
		var after: Dictionary = Priority.table(SimReads.read(world, "get_work_priorities"))
		var works: PackedStringArray = after["works"]
		var first: Dictionary = after["rows"][0]
		_check(int(first["levels"][works.find("hunt")]) == 1 and int(first["levels"][works.find("craft")]) == 1, "hunter then craft preserves both named priorities")
		for w: int in works.size():
			if works[w] not in ["hunt", "split", "haul", "craft"]:
				_check(first["levels"][w] == rows[0]["levels"][w], "role sequence preserves unspecified cell: " + works[w])
		var before_refresh: Dictionary = world.call("get_command_log", 0)
		panel.set("_key", 0)
		panel.call("_refresh")
		var choice: OptionButton = panel.find_child("Role_%d" % agent, true, false) as OptionButton
		_check(choice != null and choice.selected == 0, "multiple matching hunter/craft presets show neutral custom choice")
		_check(world.call("get_command_log", 0) == before_refresh, "ambiguous role display emits no automatic commands")
	panel.set("_roles", roles_before)


## Each negative starts from an independently valid pair, so an earlier failure
## cannot conceal a missing gate in the next assertion.
func _card_acceptance_pair(seed: int = 0) -> Array[Dictionary]:
	var receipt: Dictionary = {"accepted": true, "log_index": 4, "log_kind": "draft"}
	var native: Dictionary = {"total": 5, "kinds": PackedStringArray(["draft"]), "ticks": PackedInt64Array([60]), "accepted": PackedByteArray([1])}
	var attempt: Dictionary = {
		"card": "wolves", "tick": 60, "opened_tick": 0, "within_deadline": true,
		"synchronous_clock_valid": true, "native_log_start": 4,
		"native_command_delta": native, "command_results": [receipt],
		"matching": CardsRun.match_native_receipts([receipt], native, 4, 60),
		"chronicle_delta": {"complete": true, "rows": [], "total": 0},
		"consumed": true, "native_source": "UNAVAILABLE", "chronicle_causal_link": "UNVERIFIED",
	}
	var pair: Array[Dictionary] = [
		{"seed": seed, "mode": "answer", "deaths": 0, "validation_failures": 0,
		"cards_kinds_seen": ["wolves"], "cards_answered": 1,
		"answer_log": [{"card": "wolves", "tick": 60, "attempt_index": 0}],
		"command_results": [receipt], "answer_attempts": [attempt]},
		{"seed": seed, "mode": "no_answer", "deaths": 0, "validation_failures": 0,
		"cards_answered": 0, "answer_log": [], "command_results": [], "answer_attempts": []},
	]
	for row: Dictionary in pair:
		row.merge({"initial_tick": 0, "final_tick": 69120, "requested_ticks": 69120, "clock_valid": true})
	return pair


func _test_cards_acceptance() -> void:
	var good: Array[Dictionary] = _card_acceptance_pair()
	_check(CardsRun.results_pass(good, [0], 0), "equal survival with complete synchronous evidence passes; causal join stays unverified")
	var navigation: Array[Dictionary] = _card_acceptance_pair()
	var local_attempt: Dictionary = navigation[0]["answer_attempts"][0].duplicate(true)
	var empty_native: Dictionary = {"total": 5, "kinds": PackedStringArray(), "ticks": PackedInt64Array(), "accepted": PackedByteArray()}
	local_attempt.merge({"card": "trader", "command": "open_trade", "native_log_start": 5,
		"native_command_delta": empty_native, "command_results": [],
		"matching": CardsRun.match_native_receipts([], empty_native, 5, 60)}, true)
	navigation[0]["answer_attempts"].append(local_attempt)
	navigation[0]["answer_log"].append({"card": "trader", "tick": 60, "attempt_index": 1})
	navigation[0]["cards_answered"] = 2
	_check(CardsRun.results_pass(navigation, [0], 0), "presentation navigation needs no invented native receipt or chronicle entry")
	var stale: Array[Dictionary] = _card_acceptance_pair()
	var first_stale: Dictionary = local_attempt.duplicate(true)
	first_stale.merge({"consumed": false, "classification": "not_sent"}, true)
	stale[0]["answer_attempts"].append(first_stale)
	var retry: Dictionary = first_stale.duplicate(true)
	retry.merge({"tick": 120, "within_deadline": false}, true)
	stale[0]["answer_attempts"].append(retry)
	_check(CardsRun.results_pass(stale, [0], 0), "late not-sent retry allowed after timely first attempt for same opening")
	var late_first: Array[Dictionary] = _card_acceptance_pair()
	late_first[0]["answer_attempts"].append(retry.duplicate(true))
	_check(not CardsRun.results_pass(late_first, [0], 0), "late first attempt rejected even when not sent")
	var new_opening: Array[Dictionary] = stale.duplicate(true)
	new_opening[0]["answer_attempts"][2]["opened_tick"] = 1
	_check(not CardsRun.results_pass(new_opening, [0], 0), "old timely opening cannot excuse new late opening of same card")
	var consumed_late: Array[Dictionary] = stale.duplicate(true)
	consumed_late[0]["answer_attempts"][2]["consumed"] = true
	consumed_late[0]["answer_log"].append({"card": "trader", "tick": 120, "attempt_index": 2})
	consumed_late[0]["cards_answered"] = 2
	_check(not CardsRun.results_pass(consumed_late, [0], 0), "late consumption rejected even after timely first attempt")
	var better: Array[Dictionary] = _card_acceptance_pair()
	better[1]["deaths"] = 1
	_check(CardsRun.results_pass(better, [0], 0), "strictly better survival passes")
	var tolerated: Array[Dictionary] = _card_acceptance_pair()
	tolerated[0]["deaths"] = 1
	_check(CardsRun.results_pass(tolerated, [0], 0), "D-S1-1 one extra death in one seed passes")
	var worse: Array[Dictionary] = _card_acceptance_pair()
	worse[0]["deaths"] = 2
	_check(not CardsRun.results_pass(worse, [0], 0), "D-S1-1 two extra deaths in one seed fail")
	var offset: Array[Dictionary] = _card_acceptance_pair()
	offset.append_array(_card_acceptance_pair(1))
	offset[0]["deaths"] = 2
	offset[3]["deaths"] = 2
	_check(not CardsRun.results_pass(offset, [0, 1], 0), "D-S1-1 per-seed excess fails even with equal aggregate deaths")
	var distributed: Array[Dictionary] = _card_acceptance_pair()
	distributed.append_array(_card_acceptance_pair(1))
	distributed[0]["deaths"] = 1
	_check(CardsRun.results_pass(distributed, [0, 1], 0), "D-S1-1 single extra death across multiple seeds passes")
	distributed[2]["deaths"] = 1
	_check(not CardsRun.results_pass(distributed, [0, 1], 0), "D-S1-1 two seeds each plus one fail aggregate limit")
	for invalid_deaths: Variant in [-1, "0"]:
		var invalid: Array[Dictionary] = _card_acceptance_pair()
		invalid[0]["deaths"] = invalid_deaths
		_check(not CardsRun.results_pass(invalid, [0], 0), "malformed death count rejected: %s" % str(invalid_deaths))
	var no_cards: Array[Dictionary] = _card_acceptance_pair()
	no_cards[0].merge({"cards_kinds_seen": [], "cards_answered": 0, "answer_log": [], "command_results": [], "answer_attempts": []}, true)
	_check(not CardsRun.results_pass(no_cards, [0], 0), "zero-card synthetic run rejected")
	var drift: Array[Dictionary] = _card_acceptance_pair()
	drift[0]["final_tick"] = 69121
	_check(not CardsRun.results_pass(drift, [0], 0), "natural-card clock drift rejects run")
	_check(not CardsRun.results_pass(_card_acceptance_pair(), [0, 1], 0), "missing seed rejects card run")
	_check(not CardsRun.results_pass(_card_acceptance_pair(), [0], 1), "synthetic validation failure rejects card run")
	var missing: Array[Dictionary] = _card_acceptance_pair()
	missing.pop_back()
	_check(not CardsRun.results_pass(missing, [0], 0), "missing arm rejects otherwise passing card run")
	for field: String in ["within_deadline", "synchronous_clock_valid"]:
		var invalid: Array[Dictionary] = _card_acceptance_pair()
		invalid[0]["answer_attempts"][0][field] = false
		_check(not CardsRun.results_pass(invalid, [0], 0), "attempt evidence rejects false " + field)
	var late: Array[Dictionary] = _card_acceptance_pair()
	late[0]["answer_attempts"][0]["opened_tick"] = -1
	_check(not CardsRun.results_pass(late, [0], 0), "missing opening tick rejected despite claimed timely answer")
	var overdue: Array[Dictionary] = _card_acceptance_pair()
	overdue[0]["answer_attempts"][0]["tick"] = 61
	overdue[0]["answer_attempts"][0]["native_command_delta"]["ticks"] = PackedInt64Array([61])
	overdue[0]["answer_log"][0]["tick"] = 61
	_check(not CardsRun.results_pass(overdue, [0], 0), "61-tick answer rejected despite claimed within-deadline flag")
	var mismatch: Array[Dictionary] = _card_acceptance_pair()
	mismatch[0]["answer_attempts"][0]["matching"]["complete"] = false
	_check(not CardsRun.results_pass(mismatch, [0], 0), "recorded incomplete native matching rejected")
	var forged: Array[Dictionary] = _card_acceptance_pair()
	forged[0]["answer_attempts"][0]["native_command_delta"]["kinds"] = PackedStringArray(["wrong_kind"])
	_check(not CardsRun.results_pass(forged, [0], 0), "native kind mismatch rejected despite claimed complete matching")
	var chronicle: Array[Dictionary] = _card_acceptance_pair()
	chronicle[0]["answer_attempts"][0]["chronicle_delta"]["complete"] = false
	_check(not CardsRun.results_pass(chronicle, [0], 0), "incomplete chronicle read rejected")
	var absent: Array[Dictionary] = _card_acceptance_pair()
	absent[0]["answer_attempts"] = []
	_check(not CardsRun.results_pass(absent, [0], 0), "answer without attempt evidence rejected")
	var unlinked: Array[Dictionary] = _card_acceptance_pair()
	unlinked[0]["answer_log"][0]["attempt_index"] = 1
	_check(not CardsRun.results_pass(unlinked, [0], 0), "answer referring to missing attempt rejected")
	var omitted: Array[Dictionary] = _card_acceptance_pair()
	omitted[0]["answer_attempts"][0]["command_results"] = []
	_check(not CardsRun.results_pass(omitted, [0], 0), "native receipt omitted from attempt rejected")
	var rejected: Array[Dictionary] = _card_acceptance_pair()
	rejected[0]["command_results"][0]["accepted"] = false
	_check(not CardsRun.results_pass(rejected, [0], 0), "rejected receipt fails even with zero validation counter")
	_test_cards_output_write()


func _test_cards_output_write() -> void:
	var base: String = OS.get_environment("M6B_PRESENTATION_EVIDENCE_DIR")
	if base.is_empty():
		base = OS.get_environment("WORLDSIM_TEST_SETTINGS_DIR")
	_check(base.is_absolute_path(), "card output tests use explicit external evidence directory")
	if not base.is_absolute_path():
		return
	var directory: String = base.path_join("cards-acceptance-%d" % OS.get_process_id())
	_check(DirAccess.make_dir_recursive_absolute(directory) == OK, "card output test directory created")
	var path: String = directory.path_join("valid.json")
	var payload: Dictionary = {"fixture": "persisted", "accepted": true}
	_check(CardsRun.write_result_file(path, payload), "card result write succeeds on valid path")
	_check(JSON.parse_string(FileAccess.get_file_as_string(path)) == payload, "written card result round-trips exactly")
	_check(not CardsRun.write_result_file(directory, payload), "card result write rejects directory target")
	_check(not CardsRun.write_result_file(path.path_join("child.json"), payload), "card result write rejects parent that is a file")
