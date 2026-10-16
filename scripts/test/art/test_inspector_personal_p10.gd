extends SceneTree

var _failures: int = 0
var _checks: int = 0

class FakeWorld extends Node:
	var agent_ids := PackedInt64Array([11, 22, 33, 44, 55, 66, 77, 88, 99, 110])
	var entity_ids := PackedInt64Array([101, 202, 303, 404, 505, 606, 707, 808, 909, 1010])
	var temperament: Dictionary = {"found": true, "ns": 125, "ha": 500, "rd": 875, "p": 1000}
	var family: Dictionary = {"found": true, "female": true, "partner": 22, "mother": 33, "father": 44, "children": PackedInt64Array([55, 66, 77, 88, 99, 110])}
	var temperament_calls: int = 0
	var family_calls: int = 0
	var last_temperament_bits: int = -1
	var last_family_bits: int = -1
	var last_detail_bits: int = -1
	var state_tag: int = 0
	var target_kind: int = 0
	func get_agent_snapshot() -> Dictionary:
		return {"agent_ids": agent_ids, "ids": entity_ids}
	func get_agent_temperament(bits: int) -> Dictionary:
		temperament_calls += 1
		last_temperament_bits = bits
		return temperament
	func get_agent_family(bits: int) -> Dictionary:
		family_calls += 1
		last_family_bits = bits
		return family
	func get_agent_detail(bits: int) -> Dictionary:
		last_detail_bits = bits
		var index: int = entity_ids.find(bits)
		if index < 0:
			return {"found": false}
		return {"found": true, "agent_id": agent_ids[index], "x": 7, "y": 8, "state_tag": state_tag, "hunger": 30.0, "thirst": 40.0, "sleep": 50.0, "target_kind": target_kind}

func _initialize() -> void:
	_run.call_deferred()

func _check(condition: bool, message: String) -> void:
	_checks += 1
	if not condition:
		_failures += 1
		push_error("INSPECTOR_PERSONAL_P10: " + message)

func _run() -> void:
	var path: String = OS.get_environment("P10_INSPECTOR_DRAFT")
	if path.is_empty():
		path = "res://scripts/ui/panels/agent_inspector_panel.gd"
	var inspector: Script = load(path) as Script
	var panel: Control = inspector.new()
	root.add_child(panel)
	panel.set_process(false)
	var world := FakeWorld.new()
	panel.set("_world_sim", world)
	var locale: Node = root.get_node("Locale")
	var original_language: String = str(locale.get("_current_lang"))
	for language: String in ["en", "ko"]:
		locale.call("set_language", language)
		panel.call("display_agent", world.get_agent_detail(101))
		_check(world.get_agent_detail(101).size() == 9, "detail fixture retains nine-key bridge schema")
		_check(world.last_temperament_bits == 101 and world.last_family_bits == 101, "component calls use entity bits, never agent IDs")
		var bars: Array = panel.get("_temperament_bars")
		_check(bars.size() == 4 and (panel.get("_temperament_box") as Control).visible, "four temperament bars visible")
		for index: int in 4:
			_check((bars[index] as ProgressBar).max_value == 1000.0 and (bars[index] as ProgressBar).value == [125, 500, 875, 1000][index], "permille scale and exact value")
		var labels: Array = panel.get("_temperament_labels")
		var keys: Array[String] = ["UI_TEMPERAMENT_NS", "UI_TEMPERAMENT_HA", "UI_TEMPERAMENT_RD", "UI_TEMPERAMENT_P"]
		for index: int in 4:
			_check((labels[index] as Label).text == locale.call("ltr", keys[index]) and (labels[index] as Label).text != keys[index], "translated temperament labels")
		_check((panel.get("_family_sex") as Label).text == locale.call("ltr", "UI_FEMALE"), "localized female field")
		_check((panel.get("_family_children") as Label).text == str(locale.call("ltr", "UI_FAMILY_CHILDREN_FMT")) % 6, "all living children counted")
		var buttons: Array = panel.get("_family_buttons")
		_check(buttons.size() == 8, "three relations and five child buttons")
		for index: int in 8:
			_check(not (buttons[index] as Button).disabled and (buttons[index] as Button).visible, "all populated family links clickable")
		_check(int((buttons[7] as Button).get_meta("family_agent_id")) == 99, "first five sorted children only")
		(buttons[0] as Button).pressed.emit()
		_check(int(panel.call("get_selected_entity_bits")) == 202 and world.last_detail_bits == 202, "partner button selects mapped entity")
		panel.call("display_agent", world.get_agent_detail(101))
		(buttons[3] as Button).pressed.emit()
		_check(int(panel.call("get_selected_entity_bits")) == 505, "child click selects mapped entity")
		panel.call("display_agent", world.get_agent_detail(101))
		world.family["female"] = false
		panel.call("display_agent", world.get_agent_detail(101))
		_check((panel.get("_family_sex") as Label).text == locale.call("ltr", "UI_MALE"), "localized male field")
		world.family["female"] = true
	panel.call("display_agent", world.get_agent_detail(101))
	world.temperament["found"] = false
	world.family["found"] = false
	panel.call("display_agent", world.get_agent_detail(101))
	_check(not (panel.get("_temperament_box") as Control).visible, "missing temperament hides stale bars")
	_check(not (panel.get("_family_box") as Control).visible, "missing family hides stale buttons")
	world.temperament["found"] = true
	world.family["found"] = true
	world.family["partner"] = 999
	world.family["mother"] = -1
	panel.call("display_agent", world.get_agent_detail(101))
	var buttons: Array = panel.get("_family_buttons")
	for index: int in 2:
		_check((buttons[index] as Button).disabled and (buttons[index] as Button).text.ends_with(str(locale.call("ltr", "PANEL_FAMILY_NONE"))), "dead and absent links show None")
	world.family["partner"] = 22
	panel.call("display_agent", world.get_agent_detail(101))
	world.agent_ids.remove_at(1)
	world.entity_ids.remove_at(1)
	(buttons[0] as Button).pressed.emit()
	_check(int(panel.call("get_selected_entity_bits")) == 101 and (buttons[0] as Button).disabled, "death between paint and click is safely re-resolved")
	world.family["children"] = PackedInt64Array([55, 999])
	panel.call("display_agent", world.get_agent_detail(101))
	_check((panel.get("_family_children") as Label).text == str(locale.call("ltr", "UI_FAMILY_CHILDREN_FMT")) % 1, "dead children excluded from count")
	_check((buttons[3] as Button).visible and not (buttons[4] as Button).visible, "previous child buttons hidden after shrink")
	panel.call("close")
	var calls: int = world.family_calls + world.temperament_calls
	panel.call("_process", 1.0 / 60.0)
	panel.call("_refresh_personal_sections")
	_check(world.family_calls + world.temperament_calls == calls, "closed panel does not call component getters")
	world.family["children"] = PackedInt64Array([55, 66, 77, 88, 99, 110])
	panel.call("display_agent", world.get_agent_detail(101))
	await process_frame
	await process_frame
	var scroll: ScrollContainer = panel.get("_scroll") as ScrollContainer
	_check(panel.size.y == 620.0 and scroll.size.y <= 620.0, "scroll stays within 620px panel")
	_check(scroll.get_v_scroll_bar().max_value > scroll.get_v_scroll_bar().page, "full family content overflows through scroll")
	scroll.scroll_vertical = 100000
	await process_frame
	_check(scroll.scroll_vertical > 0, "overflow content can be scrolled")
	locale.call("set_language", original_language)
	panel.free()
	world.free()
	print("INSPECTOR_PERSONAL_P10 checks=", _checks, " failures=", _failures)
	quit(0 if _failures == 0 else 1)
