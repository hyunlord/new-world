# Behavioral checks for the alert registry, every condition, cause selection and recurrence.
# Bridge methods: none; production evaluator receives explicit facts through its read boundary.
# Run with Godot --headless --path . --script res://tools/v8/mux_alerts_test.gd.
extends SceneTree

const Evaluator = preload("res://scripts/ui/mux/alert_evaluator.gd")
const Adapter = preload("res://scripts/ui/mux/ui_facts_adapter.gd")
const AlertPanel = preload("res://scripts/ui/mux/alerts_panel.gd")
const COLLECTIONS: Array[String] = ["settlements", "people", "storehouses", "unreachable_orders", "knowledge_waits", "techniques", "animals", "migrants", "traders", "stacks", "households"]

var _checks: int = 0
var _failures: int = 0
var _reads: int = 0
var _last_target: Dictionary = {}
var _activations: int = 0


func _init() -> void:
	call_deferred("_run")


func _check(label: String, passed: bool) -> void:
	_checks += 1
	if not passed:
		_failures += 1
	print("MUX_ALERTS_%s %s" % ["PASS" if passed else "FAIL", label])


static func _facts() -> Dictionary:
	var facts: Dictionary = {"schema_version": 1, "tick": 1}
	for collection: String in COLLECTIONS:
		facts[collection] = []
	return facts


static func _person(changes: Dictionary = {}) -> Dictionary:
	var person: Dictionary = {"id": 8, "x": 10, "y": 12, "adult": true, "working": true, "sleeping": false, "roofed": true, "drafted": false, "sick": false, "breakdown": false, "hp": 100}
	person.merge(changes, true)
	return person


static func _row(changes: Dictionary) -> Dictionary:
	var row: Dictionary = {"id": 8, "x": 10, "y": 12}
	row.merge(changes, true)
	return row


static func _cases() -> Array[Dictionary]:
	return [
		{"id": "food_low", "table": "settlements", "row": _row({"food": 5.9, "daily_food_need": 2.0, "firewood": 20, "daily_firewood_need": 2, "winter_days": 4, "stockpile_cells": 20}), "negative": {"food": 6.0}, "kind": "tile"},
		{"id": "firewood_low", "table": "settlements", "row": _row({"food": 6, "daily_food_need": 2, "firewood": 7, "daily_firewood_need": 2, "winter_days": 4, "stockpile_cells": 20}), "negative": {"firewood": 8}, "kind": "tile"},
		{"id": "sleeping_outside", "table": "people", "row": _person({"sleeping": true, "roofed": false}), "negative": {"roofed": true}, "kind": "person"},
		{"id": "stockpile_missing", "table": "settlements", "row": _row({"food": 6, "daily_food_need": 2, "firewood": 8, "daily_firewood_need": 2, "winter_days": 4, "stockpile_cells": 0}), "negative": {"stockpile_cells": 1}, "kind": "tile"},
		{"id": "storehouse_full", "table": "storehouses", "row": _row({"role": 2, "used": 8, "capacity": 8}), "negative": {"used": 7}, "kind": "room"},
		{"id": "idle_adults", "table": "people", "row": _person({"working": false}), "negative": {"drafted": true}, "kind": "person"},
		{"id": "orders_unreachable", "table": "unreachable_orders", "row": _row({}), "negative": {}, "kind": "tile"},
		{"id": "knowledge_waiting", "table": "knowledge_waits", "row": _row({"node_id": 13}), "negative": {}, "kind": "technique"},
		{"id": "technique_at_risk", "table": "techniques", "row": _row({"practitioners": 1}), "negative": {"practitioners": 2}, "kind": "technique"},
		{"id": "wolves_approaching", "table": "animals", "row": _row({"kind": 0, "distance_to_camp": 12}), "negative": {"distance_to_camp": 13}, "kind": "animal"},
		{"id": "migrants_waiting", "table": "migrants", "row": _row({"waiting": true}), "negative": {"waiting": false}, "kind": "migrant"},
		{"id": "trader_present", "table": "traders", "row": _row({"present": true}), "negative": {"present": false}, "kind": "trader"},
		{"id": "person_sick", "table": "people", "row": _person({"sick": true}), "negative": {"sick": false}, "kind": "person"},
		{"id": "person_breakdown", "table": "people", "row": _person({"breakdown": true}), "negative": {"breakdown": false}, "kind": "person"},
		{"id": "person_injured", "table": "people", "row": _person({"hp": 49}), "negative": {"hp": 50}, "kind": "person"},
		{"id": "meat_spoiling", "table": "stacks", "row": _row({"kind": 11, "in_stockpile": false, "spoil_ticks_left": 1440, "count": 2}), "negative": {"in_stockpile": true}, "kind": "tile"},
		{"id": "newborn_without_bed", "table": "households", "row": _row({"newborns": 1, "beds": 0}), "negative": {"beds": 1}, "kind": "person"},
	]


func _has(evaluator: Evaluator, id: String) -> bool:
	for alert: Dictionary in evaluator.active_alerts():
		if alert["id"] == id:
			return true
	return false


func _read() -> Dictionary:
	_reads += 1
	return _facts()


func _run() -> void:
	var registry: Dictionary = JSON.parse_string(FileAccess.get_file_as_string("res://assets/ui/alerts.json"))
	var evaluator: Evaluator = Evaluator.new()
	_check("valid registry", evaluator.configure(registry))
	_check("17 required definitions", registry["alerts"].size() == 17 and _cases().size() == 17)
	for test_case: Dictionary in _cases():
		var facts: Dictionary = _facts()
		facts[test_case["table"]] = [test_case["row"]]
		_check(test_case["id"] + " evaluates", evaluator.evaluate(facts))
		_check(test_case["id"] + " positive", _has(evaluator, test_case["id"]))
		var target: Dictionary = evaluator.next_target(test_case["id"])
		_check(test_case["id"] + " jump kind", target.get("kind") == test_case["kind"])
		_check(test_case["id"] + " jump coordinates", target.get("x") == 10 and target.get("y") == 12)
		_check(test_case["id"] + " jump selects id", target.get("id") == test_case["row"].get("node_id", 8))
		if test_case["negative"].is_empty():
			facts[test_case["table"]] = []
		else:
			facts[test_case["table"]][0].merge(test_case["negative"], true)
		_check(test_case["id"] + " negative evaluates", evaluator.evaluate(facts))
		_check(test_case["id"] + " negative", not _has(evaluator, test_case["id"]))
	_test_occurrences(evaluator)
	_test_schema(registry)
	_test_extension(registry)
	_test_panel()
	_test_raw_adapter(registry)
	_test_household_counts(registry)
	_test_unreachable_locations(registry)
	evaluator.reset()
	_check("first poll", evaluator.poll(0.0, _read))
	_check("no refresh before half second", not evaluator.poll(0.49, _read) and _reads == 1)
	_check("refresh at half second", evaluator.poll(0.02, _read) and _reads == 2)
	_check("long frame performs only one refresh", evaluator.poll(8.0, _read) and _reads == 3)
	print("MUX_ALERTS_DONE checks=%d failures=%d" % [_checks, _failures])
	quit(0 if _failures == 0 else 1)


func _test_occurrences(evaluator: Evaluator) -> void:
	var facts: Dictionary = _facts()
	facts["people"] = [_person({"id": 9, "hp": 40}), _person({"id": 2, "hp": 30})]
	_check("multiple causes valid", evaluator.evaluate(facts))
	_check("cycle uses stable id order", evaluator.next_target("person_injured").get("id") == 2)
	_check("cycle second", evaluator.next_target("person_injured").get("id") == 9)
	_check("cycle wraps", evaluator.next_target("person_injured").get("id") == 2)
	evaluator.dismiss("person_injured")
	_check("dismiss removes row", not _has(evaluator, "person_injured"))
	evaluator.evaluate(facts)
	_check("dismiss lasts while condition holds", not _has(evaluator, "person_injured"))
	var incomplete: Dictionary = facts.duplicate(true)
	incomplete.erase("people")
	_check("missing facts are not a resolved condition", not evaluator.evaluate(incomplete))
	evaluator.evaluate(facts)
	_check("invalid snapshot cannot reset dismissal", not _has(evaluator, "person_injured"))
	evaluator.evaluate(_facts())
	evaluator.evaluate(facts)
	_check("recurrence reappears", _has(evaluator, "person_injured"))
	_check("recurrence resets cycle", evaluator.next_target("person_injured").get("id") == 2)
	evaluator.reset()
	_check("world reset clears alerts", evaluator.active_alerts().is_empty())


func _test_schema(registry: Dictionary) -> void:
	var evaluator: Evaluator = Evaluator.new()
	var invalid: Dictionary = registry.duplicate(true)
	invalid["schema_version"] = 99
	_check("unknown schema rejected", not evaluator.configure(invalid))
	invalid = registry.duplicate(true)
	invalid["unknown"] = true
	_check("unknown root field rejected", not evaluator.configure(invalid))
	invalid = registry.duplicate(true)
	invalid["alerts"].append(invalid["alerts"][0].duplicate(true))
	_check("duplicate id rejected", not evaluator.configure(invalid))
	invalid = registry.duplicate(true)
	invalid["alerts"][0]["condition_id"] = "missing"
	_check("unknown condition rejected", not evaluator.configure(invalid))
	invalid = registry.duplicate(true)
	invalid["conditions"]["food_low"]["predicates"][0]["op"] = "eval"
	_check("unknown operator rejected", not evaluator.configure(invalid))
	invalid = registry.duplicate(true)
	invalid["fallback_severity"] = "missing"
	_check("missing fallback rejected", not evaluator.configure(invalid))


func _test_extension(registry: Dictionary) -> void:
	var extended: Dictionary = registry.duplicate(true)
	extended["conditions"]["new_data_condition"] = {"collection": "people", "predicates": [{"field": "hp", "op": "gt", "value": 90}]}
	extended["alerts"].append({"id": "new_data_alert", "condition_id": "new_data_condition", "severity": "new_category", "text_key": "TITLE", "explanation_key": "WHY", "fix_key": "FIX", "jump_target": "unknown_target"})
	var evaluator: Evaluator = Evaluator.new()
	_check("data-only extension loads", evaluator.configure(extended))
	var facts: Dictionary = _facts()
	facts["people"] = [_person()]
	_check("data-only extension evaluates", evaluator.evaluate(facts))
	_check("data-only alert appears", _has(evaluator, "new_data_alert"))
	_check("unknown target uses fallback", evaluator.next_target("new_data_alert").get("kind") == "tile")
	var rows: Array[Dictionary] = evaluator.active_alerts()
	_check("unknown severity uses fallback", rows.size() == 1 and rows[0]["rank"] == registry["severities"]["notice"]["rank"])


func _on_jump(target: Dictionary) -> void:
	_last_target = target


func _on_activation(_id: String) -> void:
	_activations += 1


func _test_panel() -> void:
	var panel: AlertPanel = AlertPanel.new()
	root.add_child(panel)
	panel.set_process(false)
	panel.jump_requested.connect(_on_jump)
	panel.alert_activated.connect(_on_activation)
	var facts: Dictionary = _facts()
	facts["people"] = [_person({"hp": 49})]
	_check("panel accepts facts", panel.refresh_from_facts(facts))
	var button: Button = panel.find_child("person_injured", true, false) as Button
	_check("panel has alert button", button != null)
	if button != null:
		_check("alert explains cause and fix", button.tooltip_text.count("\n") == 2)
		button.pressed.emit()
		_check("panel emits cause selection", _last_target.get("kind") == "person" and _last_target.get("id") == 8)
		_check("panel informs learning helper", _activations == 1)
	panel.dismiss("person_injured")
	_check("panel dismiss updates row", panel.find_child("person_injured", true, false) == null)
	panel.free()


static func _raw_empty() -> Dictionary:
	var raw: Dictionary = {"schema_version": 1, "enabled": true, "tick": 1, "day": 4, "ticks_per_day": 1440, "clock_offset_ticks": 5760, "season": 3, "season_day": 1, "days_per_season": 4, "meat_spoil_days": 3, "story": {"firewood": 0}, "neighbours": {}}
	for table: String in Adapter.TABLES:
		raw[table] = {}
		for column: String in Adapter.TABLES[table]:
			raw[table][column] = []
	for prefix: String in ["wave", "caravan"]:
		for suffix: String in ["ids", "states", "x", "y"]:
			raw["neighbours"][prefix + "_" + suffix] = []
	return raw


static func _append_raw(raw: Dictionary, table: String, values: Dictionary) -> void:
	for column: String in Adapter.TABLES[table]:
		raw[table][column].append(values.get(column, 0))


static func _raw_positive() -> Dictionary:
	var raw: Dictionary = _raw_empty()
	_append_raw(raw, "settlements", {"ids": 1, "xs": 10, "ys": 12, "food": 1, "food_need": 0.75, "winter_wood_need": 2})
	_append_raw(raw, "people", {"ids": 8, "household_ids": 8, "xs": 10, "ys": 12, "hps": 49, "sleeping": 1, "sick": 1, "breakdown": 1, "bed_xs": -1, "bed_ys": -1, "mothers": -1, "fathers": -1, "partners": 9})
	_append_raw(raw, "people", {"ids": 9, "household_ids": 8, "xs": 11, "ys": 12, "hps": 100, "idle": 1, "bed_xs": -1, "bed_ys": -1, "mothers": -1, "fathers": -1, "partners": 8})
	_append_raw(raw, "people", {"ids": 10, "household_ids": 8, "xs": 10, "ys": 12, "hps": 100, "children": 1, "infants": 1, "bed_xs": -1, "bed_ys": -1, "mothers": 8, "fathers": 9, "partners": -1})
	_append_raw(raw, "rooms", {"ids": 2, "roles": 2, "centre_xs": 10, "centre_ys": 12})
	_append_raw(raw, "stockpile_tiles", {"zones": 1, "xs": 10, "ys": 12, "can_hold": 1, "rooms": 2, "room_roles": 2})
	_append_raw(raw, "stacks", {"xs": 10, "ys": 12, "kinds": 11, "counts": 25, "capacities": 25, "zones": 1})
	_append_raw(raw, "stacks", {"xs": 11, "ys": 12, "kinds": 11, "counts": 2, "capacities": 25, "zones": -1})
	_append_raw(raw, "meat_lots", {"days": 0, "counts": 27})
	_append_raw(raw, "unreachable", {"agent_ids": 8, "xs": 10, "ys": 12, "until_ticks": 100})
	_append_raw(raw, "waits", {"kinds": 1, "a": 10, "b": 12, "nodes": 13})
	_append_raw(raw, "tech", {"ids": 13, "practitioners": 1})
	_append_raw(raw, "animals", {"ids": 1, "kinds": 0, "xs": 10, "ys": 12})
	for prefix: String in ["wave", "caravan"]:
		for suffix: String in ["ids", "states", "x", "y"]:
			raw["neighbours"][prefix + "_" + suffix] = [1]
	return _packed(raw)


static func _packed(raw: Dictionary) -> Dictionary:
	for table: String in Adapter.TABLES:
		for column: String in Adapter.TABLES[table]:
			raw[table][column] = PackedFloat64Array(raw[table][column]) if column in ["hps", "food_need", "winter_wood_need"] else PackedInt64Array(raw[table][column])
	for column: String in raw["neighbours"]:
		raw["neighbours"][column] = PackedInt32Array(raw["neighbours"][column])
	return raw


func _test_raw_adapter(registry: Dictionary) -> void:
	var evaluator: Evaluator = Evaluator.new()
	_check("raw evaluator registry", evaluator.configure(registry))
	for test_case: Dictionary in _cases():
		var raw: Dictionary = _raw_positive()
		if test_case["id"] == "stockpile_missing":
			for column: String in raw["stockpile_tiles"]:
				raw["stockpile_tiles"][column] = []
		_check(test_case["id"] + " packed positive evaluates", evaluator.evaluate(Adapter.adapt(raw)))
		_check(test_case["id"] + " packed positive", _has(evaluator, test_case["id"]))
		_check(test_case["id"] + " packed negative evaluates", evaluator.evaluate(Adapter.adapt(_raw_empty())))
		_check(test_case["id"] + " packed negative", not _has(evaluator, test_case["id"]))
	var raw: Dictionary = _raw_positive()
	var facts: Dictionary = Adapter.adapt(raw)
	_check("daily food need keeps fractions", facts["settlements"][0]["daily_food_need"] == 0.75)
	_check("FIFO includes stockpiled quantity", facts["stacks"].size() == 2 and facts["stacks"][1]["count"] == 2)
	raw["meat_lots"]["counts"] = [25]
	facts = Adapter.adapt(raw)
	_check("FIFO prevents duplicated outside risk", facts["stacks"].size() == 1 and facts["stacks"][0]["in_stockpile"])
	raw["stacks"]["reserved"][0] = 1
	facts = Adapter.adapt(raw)
	_check("reserved stacks skipped in allocation", facts["stacks"].size() == 1 and not facts["stacks"][0]["in_stockpile"])
	raw["people"]["bed_xs"][0] = 10
	raw["people"]["bed_ys"][0] = 12
	facts = Adapter.adapt(raw)
	_check("infant shares assigned parent bed", facts["households"][0]["beds"] == 1)
	raw["people"]["hps"][0] = -1
	_check("unknown health evaluates", evaluator.evaluate(Adapter.adapt(raw)))
	_check("unknown health is not injury", not _has(evaluator, "person_injured"))
	raw["people"]["ids"].append(99)
	_check("unequal packed columns rejected", Adapter.adapt(raw).is_empty())
	raw = _raw_positive()
	raw["schema_version"] = 2
	_check("raw schema version rejected", Adapter.adapt(raw).is_empty())
	raw = _raw_positive()
	raw["enabled"] = false
	_check("disabled colony evaluates empty", evaluator.evaluate(Adapter.adapt(raw)) and evaluator.active_alerts().is_empty())


func _test_household_counts(registry: Dictionary) -> void:
	var evaluator: Evaluator = Evaluator.new()
	evaluator.configure(registry)
	var raw: Dictionary = _raw_positive()
	_append_raw(raw, "people", {"ids": 11, "household_ids": 8, "xs": 13, "ys": 14, "hps": 100, "children": 1, "infants": 1, "bed_xs": -1, "bed_ys": -1})
	var facts: Dictionary = Adapter.adapt(raw)
	_check("two infants form one authoritative household cause", facts["households"].size() == 1 and facts["households"][0]["newborns"] == 2)
	evaluator.evaluate(facts)
	var target: Dictionary = evaluator.next_target("newborn_without_bed")
	_check("household jump uses real lowest infant", target.get("id") == 10 and target.get("x") == 10)
	_check("household cycle does not repeat siblings", evaluator.next_target("newborn_without_bed") == target)
	_append_raw(raw, "people", {"ids": 12, "household_ids": 8, "hps": 100, "children": 1, "bed_xs": 20, "bed_ys": 21})
	evaluator.evaluate(Adapter.adapt(raw))
	_check("assigned sibling bed clears household alert", not _has(evaluator, "newborn_without_bed"))
	raw["people"]["children"][4] = 0
	evaluator.evaluate(Adapter.adapt(raw))
	_check("assigned household partner bed clears alert", not _has(evaluator, "newborn_without_bed"))
	raw["people"]["household_ids"][4] = 12
	evaluator.evaluate(Adapter.adapt(raw))
	_check("another household bed cannot clear alert", _has(evaluator, "newborn_without_bed"))
	raw["people"]["household_ids"][3] = 11
	facts = Adapter.adapt(raw)
	_check("distinct infant households remain distinct", facts["households"].size() == 2)
	evaluator.evaluate(facts)
	_check("distinct households cycle to both infants", evaluator.next_target("newborn_without_bed").get("id") != evaluator.next_target("newborn_without_bed").get("id"))
	raw["people"].erase("household_ids")
	_check("missing authoritative household column rejected", Adapter.adapt(raw).is_empty())


func _test_unreachable_locations(registry: Dictionary) -> void:
	var evaluator: Evaluator = Evaluator.new()
	evaluator.configure(registry)
	var raw: Dictionary = _raw_positive()
	_append_raw(raw, "unreachable", {"agent_ids": 9, "xs": 10, "ys": 12, "until_ticks": 100})
	var facts: Dictionary = Adapter.adapt(raw)
	_check("two worker failures count one work location", facts["unreachable_orders"].size() == 1)
	evaluator.evaluate(facts)
	var first: Dictionary = evaluator.next_target("orders_unreachable")
	_check("duplicate workers cycle one location", evaluator.next_target("orders_unreachable") == first)
	raw["unreachable"]["agent_ids"][0] = 99
	raw["unreachable"]["until_ticks"][1] = 0
	facts = Adapter.adapt(raw)
	_check("location identity survives worker change", facts["unreachable_orders"][0]["id"] == first["id"])
	_append_raw(raw, "unreachable", {"agent_ids": 9, "xs": 11, "ys": 12, "until_ticks": 100})
	evaluator.evaluate(Adapter.adapt(raw))
	var second: Dictionary = evaluator.next_target("orders_unreachable")
	var third: Dictionary = evaluator.next_target("orders_unreachable")
	_check("distinct locations cycle distinct real tiles", second["x"] != third["x"] and second["y"] == 12 and third["y"] == 12)
	_check("distinct work locations count two", Adapter.adapt(raw)["unreachable_orders"].size() == 2)
	raw["unreachable"]["until_ticks"][0] = 0
	raw["unreachable"]["until_ticks"][2] = 0
	evaluator.evaluate(Adapter.adapt(raw))
	_check("expired locations clear alert", not _has(evaluator, "orders_unreachable"))
