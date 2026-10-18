extends SceneTree
const Text = preload("res://scripts/ui/chronicle_text.gd")
const PanelScript = preload("res://scripts/ui/panels/chronicle_panel.gd")
var checks: int = 0
var failures: int = 0
class FakeWorld extends Node:
	var tick: int = 1
	var entries: Array[Dictionary] = []
	var starts: Array[int] = []
	func get_time_snapshot() -> Dictionary:
		return {"tick": tick}
	func get_world_info() -> Dictionary:
		return {"ticks_per_day": 1440, "clock_offset_ticks": 480}
	func get_chronicle_since(start: int, maximum: int) -> Dictionary:
		starts.append(start)
		var end: int = mini(start + maximum, entries.size())
		var page: Dictionary = {"next": end, "total": entries.size()}
		for column: String in Text.COLUMNS:
			page[column] = PackedInt64Array()
		for index: int in range(start, end):
			for column: String in Text.COLUMNS:
				page[column].append(entries[index].get(column, -1))
		return page

func _initialize() -> void:
	_run.call_deferred()

func check(value: bool, message: String) -> void:
	checks += 1
	if not value:
		failures += 1
		push_error(message)

func row(kind: int, cause: int = 0, amount: int = 10) -> Dictionary:
	return {"tick": 1440, "kind": kind, "subject": 25, "other": 26, "third": -1, "settlement": 2, "settlement2": 1, "cause": cause, "value": amount}

func _run() -> void:
	var locale: Node = root.get_node("Locale")
	var original: String = str(locale.get("_current_lang"))
	var info: Dictionary = {"ticks_per_day": 1440, "clock_offset_ticks": 480}
	for language: String in ["en", "ko"]:
		locale.call("set_language", language)
		for cause: int in 4:
			var purpose: String = Text.loc(locale, "CHRON_PORTER_AID" if cause < 2 else "CHRON_PORTER_EXCHANGE")
			var load_name: String = Text.loc(locale, "CHRON_LOAD_FOOD" if cause % 2 == 0 else "CHRON_LOAD_WOOD")
			check(Text.sentence(row(17, cause), 25, locale, info) == Text.loc(locale, "CHRON_PORTER_DEPARTED_FMT") % [Text.village(2, locale), Text.village(1, locale), load_name, 10, purpose], "departure roles/cause " + language)
			check(Text.sentence(row(18, cause), 25, locale, info) == Text.loc(locale, "CHRON_PORTER_DELIVERED_FMT") % [load_name, 10, Text.village(1, locale), Text.village(2, locale), purpose], "delivery receiver/sender " + language)
			check(Text.sentence(row(18, cause + 10, 0), 25, locale, info) == Text.loc(locale, "CHRON_PORTER_FAILED_FMT") % [Text.village(2, locale), purpose], "failed receiver/cause " + language)
			for failed: bool in [false, true]:
				var group: Dictionary = row(18, cause + (10 if failed else 0), 0 if failed else 19)
				group["porter_count"] = 2
				var expected_group: String
				if failed:
					expected_group = Text.loc(locale, "CHRON_PORTER_GROUP_FAILED_FMT") % [2, Text.village(1, locale), Text.village(2, locale), purpose]
				else:
					expected_group = Text.loc(locale, "CHRON_PORTER_GROUP_FMT") % [2, load_name, 19, Text.village(1, locale), Text.village(2, locale), purpose]
				var actual_group: String = Text.line(group, -1, locale, info, true)
				check(actual_group.ends_with(expected_group) and not actual_group.contains("person:"), "all grouped causes and prefixes " + language)
			for kind: int in [17, 18]:
				check(Text.sentence(row(kind, cause), 26, locale, info).is_empty(), "non-subject porter omitted")
		check(Text.sentence(row(19), 25, locale, info) == Text.loc(locale, "CHRON_QUARREL_FMT") % [Text.person(26, locale), Text.village(1, locale)], "quarrel subject sees other village")
		check(Text.sentence(row(19), 26, locale, info) == Text.loc(locale, "CHRON_QUARREL_FMT") % [Text.person(25, locale), Text.village(2, locale)], "quarrel other sees subject village")
		check(Text.sentence(row(19), 99, locale, info).is_empty(), "unrelated quarrel omitted")
		for kind: int in [20, 21]:
			var village_row: Dictionary = row(kind)
			village_row["subject"] = -1
			village_row["other"] = -1
			var key: String = "CHRON_VILLAGES_ESTRANGED_FMT" if kind == 20 else "CHRON_VILLAGES_RECONCILED_FMT"
			check(Text.sentence(village_row, -1, locale, info) == Text.loc(locale, key) % [Text.village(2, locale), Text.village(1, locale)], "village sentence " + language)
			check(not Text.line(village_row, -1, locale, info, true).contains("person:"), "village line has no person prefix")
		check(Text.sentence(row(22), -1, locale, info).is_empty(), "unknown sentence skipped")
		_test_panel(locale)
	locale.call("set_language", original)
	print("RELATIONS_CHRONICLE_P12 checks=", checks, " failures=", failures)
	quit(0 if failures == 0 else 1)

func _test_panel(locale: Node) -> void:
	var world := FakeWorld.new()
	for index: int in 511:
		world.entries.append(row(3))
	world.entries.append(row(18, 0, 10))
	var second: Dictionary = row(18, 0, 7)
	second["subject"] = 77
	world.entries.append(second)
	world.entries.append(row(17))
	world.entries.append(row(18, 0, 4))
	world.entries.append(row(22))
	world.entries.append(row(18, 0, 3))
	for key: String in ["tick", "settlement", "settlement2", "cause"]:
		var changed: Dictionary = row(18, 0, 2)
		changed[key] = int(changed[key]) + 10
		world.entries.append(changed)
	world.entries.append(row(18, 13, 0))
	world.entries.append(row(18, 13, 0))
	world.entries.append(row(19))
	world.entries.append(row(20))
	world.entries.append(row(21))
	var panel: Control = PanelScript.new()
	root.add_child(panel)
	panel.set_process(false)
	panel.set("_world", world)
	panel.call("_process", 0.0)
	check(int(panel.get("_next")) == 512, "first page cursor preserved")
	world.tick += 1
	panel.call("_process", 0.0)
	var entries: Array = panel.get("_entries")
	check(entries.size() == 11, "raw adjacency groups across page, hidden rows separate, all group keys separate")
	if entries.size() == 11:
		check(int(entries[0].get("porter_count", 0)) == 2 and int(entries[0]["value"]) == 17, "cross-page count and amount")
		check(int(entries[1].get("porter_count", 0)) == 1 and int(entries[2].get("porter_count", 0)) == 1, "17 and unknown break adjacency")
		check(int(entries[7].get("porter_count", 0)) == 2, "failed trips grouped")
		var info: Dictionary = world.get_world_info()
		var delivered: String = Text.line(entries[0], -1, locale, info, true)
		var expected: String = Text.loc(locale, "CHRON_PORTER_GROUP_FMT") % [2, Text.loc(locale, "CHRON_LOAD_FOOD"), 17, Text.village(1, locale), Text.village(2, locale), Text.loc(locale, "CHRON_PORTER_AID")]
		check(delivered.ends_with(expected) and not delivered.contains("person:"), "group text order and no person prefix")
		var failed: String = Text.line(entries[7], -1, locale, info, true)
		check(failed.ends_with(Text.loc(locale, "CHRON_PORTER_GROUP_FAILED_FMT") % [2, Text.village(1, locale), Text.village(2, locale), Text.loc(locale, "CHRON_PORTER_EXCHANGE")]), "failed group cause13 purpose/order")
	check(panel.get("_unknown_kinds") == 1, "unknown raw entries counted once")
	panel.call("_refresh")
	panel.call("_refresh")
	check(panel.get("_unknown_kinds") == 1 and world.starts == [0, 512], "refresh neither recounts nor repolls")
	panel.call("_reset")
	check(panel.get("_unknown_kinds") == 0 and (panel.get("_entries") as Array).is_empty(), "reset clears grouping and debug counter")
	panel.free()
	world.free()
