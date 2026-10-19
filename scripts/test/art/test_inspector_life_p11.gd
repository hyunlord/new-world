extends SceneTree
const PanelScript = preload("res://scripts/ui/panels/agent_inspector_panel.gd")
const Text = preload("res://scripts/ui/chronicle_text.gd")
var failures: int = 0
var checks: int = 0
class FakeWorld extends Node:
	var tick: int = 100
	var calls: Dictionary = {}
	var founder: bool = true
	var partner_id: int = 2
	func counted(method: String, id: int) -> void:
		var key: String = "%s:%d:%d" % [method, id, tick]
		calls[key] = int(calls.get(key, 0)) + 1
	func get_time_snapshot() -> Dictionary:
		return {"tick": tick}
	func get_world_info() -> Dictionary:
		return {"ticks_per_day": 100, "clock_offset_ticks": 25}
	func get_agent_snapshot() -> Dictionary:
		return {"ids": PackedInt64Array([101, 202]), "agent_ids": PackedInt64Array([1, 2])}
	func get_agent_detail(bits: int) -> Dictionary:
		return {"found": bits in [101, 202], "agent_id": 1 if bits == 101 else 2, "x": 2, "y": 3, "hunger": 30.0, "thirst": 40.0, "sleep": 50.0, "state_tag": 0, "target_kind": 0}
	func get_agent_temperament(bits: int) -> Dictionary:
		counted("temperament", bits)
		return {"found": true, "ns": 100, "ha": 200, "rd": 300, "p": 400}
	func get_agent_family(bits: int) -> Dictionary:
		counted("family", bits)
		return {"found": true, "female": true, "partner": partner_id, "mother": 3, "father": -1, "children": PackedInt64Array()}
	func get_person(id: int) -> Dictionary:
		counted("person", id)
		return {"found": id in [1, 2, 3], "alive": id in [1, 2], "female": 1, "born_tick": -10 if founder else 0, "died_tick": 80, "death_reason": 3, "mother": 2, "father": -1, "ns": 100, "ha": 200, "rd": 300, "p": 400}
	func get_person_chronicle(id: int) -> Dictionary:
		counted("chronicle", id)
		var result: Dictionary = {}
		for column: String in Text.COLUMNS:
			result[column] = PackedInt64Array()
		for index: int in 15:
			var row: Dictionary = {"tick": index, "kind": 3, "subject": id, "other": -1, "third": -1, "settlement": -1, "settlement2": -1, "cause": 0, "value": -1}
			if id == 1 and index == 14:
				row["kind"] = 10
				row["other"] = 3
			for column: String in Text.COLUMNS:
				result[column].append(row[column])
		return result
func _initialize() -> void:
	_run.call_deferred()
func check(value: bool, message: String) -> void:
	checks += 1
	if not value:
		failures += 1
		push_error(message)
func _run() -> void:
	var panel: Control = PanelScript.new()
	root.add_child(panel)
	panel.set_process(false)
	var world := FakeWorld.new()
	panel.set("_world_sim", world)
	var locale: Node = root.get_node("Locale")
	var original: String = str(locale.get("_current_lang"))
	for language: String in ["en", "ko"]:
		locale.call("set_language", language)
		panel.call("show_person", 1)
		for _frame: int in 120:
			panel.call("_process", 1.0 / 60.0)
		for key: String in world.calls:
			check(int(world.calls[key]) <= 1, "one direct read per person/tick: " + key)
		var buttons: Array = panel.get("_family_buttons")
		check(not (buttons[1] as Button).disabled and (buttons[1] as Button).text.contains(Text.loc(locale, "UI_DECEASED")), "dead parent named and linked")
		check((panel.get("_life_more") as Button).visible, "last12 expansion offered")
		check((panel.get("_life_text") as RichTextLabel).text.split("\n\n").size() == 12, "last12 only")
		panel.call("_expand_life")
		check((panel.get("_life_text") as RichTextLabel).text.split("\n\n").size() == 15, "earlier lines expand")
		(buttons[1] as Button).pressed.emit()
		check(bool(panel.get("_record_mode")) and int(panel.get("_selected_person")) == 3, "dead link enters record")
		check(not (panel.get("_hunger_bar") as Control).visible and not (panel.get("_portrait") as Node2D).is_visible_in_tree(), "record hides live bars and portrait")
		check((panel.get("_record_summary") as Label).text.begins_with(Text.loc(locale, "INSPECTOR_RECORD_FOUNDER_LIFE_FMT") % 2), "founder life")
		check((panel.get("_record_summary") as Label).text.contains(Text.loc(locale, "CHRON_DEATH_OLD_AGE")), "record cause")
		for _frame: int in 120:
			panel.call("_process", 1.0 / 60.0)
		for key: String in world.calls:
			check(int(world.calls[key]) <= 1, "record raw cache: " + key)
		world.founder = false
		world.tick += 1
		panel.call("_process", 0.0)
		check((panel.get("_record_summary") as Label).text.begins_with(Text.loc(locale, "INSPECTOR_RECORD_LIFE_FMT") % [1, 2]), "non-founder record")
		panel.call("_follow_link", "person:2")
		check(not bool(panel.get("_record_mode")) and int(panel.call("get_selected_entity_bits")) == 202, "life link selects living")
		panel.call("show_person", 3)
		var event := InputEventKey.new()
		event.keycode = KEY_ESCAPE
		event.pressed = true
		panel.call("_unhandled_input", event)
		check(not panel.visible and not bool(panel.get("_record_mode")), "escape exits record")
		world.founder = true
		world.tick += 1
	world.partner_id = -1
	world.tick += 1
	panel.call("show_person", 1)
	var spouse: Button = (panel.get("_family_buttons") as Array)[0] as Button
	check(not spouse.disabled and int(spouse.get_meta("family_agent_id", -1)) == 3 and spouse.text.contains(Text.loc(locale, "UI_DECEASED")), "simulation-cleared spouse recovered from Widowed")
	spouse.pressed.emit()
	check(bool(panel.get("_record_mode")) and int(panel.get("_selected_person")) == 3, "deceased spouse opens record")
	# Reset can reuse exactly the same tick and agent ids: never reuse old data.
	panel.call("show_person", 1)
	var before_reset: int = int(world.calls.get("family:101:%d" % world.tick, 0))
	panel.call("_world_reset")
	panel.call("show_person", 1)
	check(int(world.calls.get("family:101:%d" % world.tick, 0)) == before_reset + 1, "same-tick world reset invalidates reads")
	check(panel.visible and int(panel.get("_selected_person")) == 1, "same-tick reset can select again")
	locale.call("set_language", original)
	panel.free()
	world.free()
	print("INSPECTOR_LIFE_P11 checks=", checks, " failures=", failures)
	quit(0 if failures == 0 else 1)
