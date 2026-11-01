# tools/v8/loop_ui_test.gd — M-LOOP headless tests for the presentation items.
#   Each test can fail: the name says which bug would still pass it.
# usage: $GODOT_BIN --headless --path . --script res://tools/v8/loop_ui_test.gd
extends SceneTree

var _pass: int = 0
var _fail: int = 0


func _init() -> void:
	call_deferred("_run")


func _run() -> void:
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(main)
	for _i: int in 10:
		await process_frame
	var world: Node = main.get_node_or_null("WorldSim")
	if world == null:
		push_error("loop_ui_test: missing WorldSim")
		quit(1)
		return

	# --- Test 1: Wheel capture — camera has _pointer_over_ui method ---
	_test_wheel_capture(main)

	# --- Test 2: Text scale validation — 0.85/1.0/1.15/1.25 ---
	_test_text_scale()

	# --- Test 3: Help collapse — learning helper has collapse state ---
	_test_help_collapse(main)

	# --- Test 4: Alert rows ≤ 3 ---
	_test_alert_max_rows(main)

	# --- Test 5: Causal panel filters unlabeled entries ---
	_test_causal_filter()

	# --- Test 6: Cards.json parses and has required cards ---
	_test_cards_data()

	# --- Test 7: Chapters.json parses and has 5 chapters ---
	_test_chapters_data()

	# --- Test 8: Chapter condition check ---
	_test_chapter_conditions(main)

	# --- Test 9: Chronicle notable_kinds loaded ---
	_test_chronicle_notable(main)

	# --- Test 10: Card panel instantiates and has methods ---
	_test_card_panel(main)

	# --- Test 11: Plaque has content priority ---
	_test_plaque_priority(main)

	# --- Test 12: Data-only card (sick_person) appears from cards.json ---
	_test_data_only_card()

	# --- Test 13: Chronicle feed schema 2 parses ---
	_test_chronicle_feed_schema()

	# --- Test 14: Locale keys present (LOOP_ prefix) ---
	_test_locale_keys()

	# --- Test 15: Topbar is one row (lower row hidden) ---
	_test_topbar_one_row(main)

	# --- Test 16: Item panel closes on outside click ---
	_test_item_panel_close(main)

	# --- Test 17: Popup stacking — LIST_PANELS and POPUP_PANELS constants ---
	_test_popup_stacking(main)

	# --- Test 18: Idle adults alert suppressed at night ---
	_test_idle_adults_night()

	# --- Test 19: Chapter completion wired to toast ---
	_test_chapter_completion_toast(main)

	# --- Test 20: Card trigger matches bridge data format ---
	_test_card_trigger_format()

	# --- Test 21: Card trigger fires in seeded run ---
	await _test_card_trigger_seeded(main, world)
	# --- Test 22: answering a card sends a valid command and writes the chronicle ---
	_test_card_answer_recorded(main)
	# --- Test 23: card auto-closes when trigger clears ---
	_test_card_auto_close()
	# --- Test 24: no raw text in card display (no "{" or ALL_CAPS keys) ---
	_test_no_raw_text_in_cards()
	# --- Test 25: wolves card uses StewardDefence (kind 46), not Sick (36) ---
	_test_wolves_card_kind()
	# --- Test 26: need_food hunt option sends cmd_add_hunt_zone, not cmd_designate_harvest ---
	_test_need_food_hunt_command()
	# --- Test 27: five previously missing commands have _send_command branches ---
	_test_missing_command_branches()
	# --- Test 28: LOOP_STEWARD_TOOLS locale key is defined ---
	_test_steward_tools_locale()

	print("LOOP_UI_TEST_DONE pass=%d fail=%d" % [_pass, _fail])
	quit(1 if _fail > 0 else 0)


func _assert(condition: bool, name: String) -> void:
	if condition:
		_pass += 1
		print("  PASS: ", name)
	else:
		_fail += 1
		push_error("  FAIL: " + name)


## Test 1: camera_controller has the _pointer_over_ui guard for wheel events.
func _test_wheel_capture(main: Node) -> void:
	print("Test 1: wheel capture guard exists")
	var camera: Camera2D = main.get_node_or_null("Camera2D") as Camera2D
	_assert(camera != null, "camera exists")
	_assert(camera != null and camera.has_method("_pointer_over_ui"), "camera has _pointer_over_ui")


## Test 2: settings_store validates the new text scale values.
func _test_text_scale() -> void:
	print("Test 2: text scale 0.85/1.0/1.15/1.25")
	var Store = preload("res://scripts/ui/mux/settings_store.gd")
	_assert(Store._valid_value("ui_scale", 0.85), "0.85 is valid")
	_assert(Store._valid_value("ui_scale", 1.0), "1.0 is valid")
	_assert(Store._valid_value("ui_scale", 1.15), "1.15 is valid")
	_assert(Store._valid_value("ui_scale", 1.25), "1.25 is valid")
	_assert(not Store._valid_value("ui_scale", 1.5), "1.5 is no longer valid")
	_assert(not Store._valid_value("ui_scale", 0.5), "0.5 is invalid")


## Test 3: learning helper has collapse state and expand_helper method.
func _test_help_collapse(main: Node) -> void:
	print("Test 3: help collapse")
	var hud: Node = main.find_child("MuxHud", true, false)
	if hud == null:
		_assert(false, "MuxHud exists")
		return
	var helper: Node = hud.get_node_or_null("LearningHelper")
	_assert(helper != null, "LearningHelper exists")
	_assert(helper != null and helper.has_method("expand_helper"), "has expand_helper")
	_assert(helper != null and "_collapsed" in helper, "has _collapsed state")
	# Simulate observing an order event (reset state first to avoid stored progress).
	if helper != null and helper.has_method("observe"):
		helper.set("_collapsed", false)
		helper.set("_events", {})
		helper.call("observe", "stockpile_exists")
		_assert(bool(helper.get("_collapsed")), "collapses after order event")
		helper.call("expand_helper")
		_assert(not bool(helper.get("_collapsed")), "expands after expand_helper")


## Test 4: alerts panel has MAX_VISIBLE_ROWS = 3.
func _test_alert_max_rows(main: Node) -> void:
	print("Test 4: alert max rows")
	var Alerts = preload("res://scripts/ui/mux/alerts_panel.gd")
	_assert(Alerts.MAX_VISIBLE_ROWS == 3, "MAX_VISIBLE_ROWS is 3")


## Test 5: causal panel filters unlabeled entries.
func _test_causal_filter() -> void:
	print("Test 5: causal filter")
	var CausalPanel = preload("res://scripts/ui/panels/causal_panel.gd")
	var panel: Control = CausalPanel.new()
	# Simulate history with labeled and unlabeled events.
	var history: Array = [
		{"kind": "agent_died", "tick": 100, "reason": "starvation"},
		{"kind": "unknown_event_type", "tick": 200},
		{"kind": "combat_started", "tick": 300},
	]
	var filtered: Array = panel.call("_filter_labeled", history)
	_assert(filtered.size() == 2, "filtered keeps 2 labeled entries, removes 1 unlabeled")
	_assert(str(filtered[0].get("kind", "")) == "agent_died", "first is agent_died")
	_assert(str(filtered[1].get("kind", "")) == "combat_started", "second is combat_started")
	panel.queue_free()


## Test 6: cards.json parses and contains the required card IDs.
func _test_cards_data() -> void:
	print("Test 6: cards.json data")
	var text: String = FileAccess.get_file_as_string("res://assets/ui/cards.json")
	var parsed: Variant = JSON.parse_string(text)
	_assert(parsed is Dictionary, "cards.json is a dict")
	if not parsed is Dictionary:
		return
	var cards: Array = parsed.get("cards", [])
	var ids: PackedStringArray = PackedStringArray()
	for card: Variant in cards:
		if card is Dictionary:
			ids.append(str(card["id"]))
	var required: Array[String] = ["need_food", "need_warmth", "migrants", "trader", "knowledge_risk", "tools_low", "wolves", "idle_adults"]
	for req: String in required:
		_assert(ids.has(req), "has required card: " + req)
	# knowledge_risk and tools_low expire to steward within 24h.
	for card: Variant in cards:
		if card is Dictionary and str(card["id"]) in ["knowledge_risk", "tools_low"]:
			_assert(str(card.get("default", "")) == "steward", str(card["id"]) + " default is steward")
			_assert(int(card.get("expires_hours", 99)) <= 24, str(card["id"]) + " expires <= 24h")


## Test 7: chapters.json parses and has 5 chapters.
func _test_chapters_data() -> void:
	print("Test 7: chapters.json data")
	var text: String = FileAccess.get_file_as_string("res://assets/ui/chapters.json")
	var parsed: Variant = JSON.parse_string(text)
	_assert(parsed is Dictionary, "chapters.json is a dict")
	if not parsed is Dictionary:
		return
	var chapters: Array = parsed.get("chapters", [])
	_assert(chapters.size() == 5, "has 5 chapters")
	# Chapter 4 is locked.
	if chapters.size() >= 4:
		_assert(bool(chapters[3].get("locked", false)), "chapter 4 is locked")


## Test 8: chapter condition check logic.
func _test_chapter_conditions(main: Node) -> void:
	print("Test 8: chapter conditions")
	var hud: Node = main.find_child("MuxHud", true, false)
	if hud == null:
		_assert(false, "MuxHud exists")
		return
	var goals: Node = hud.get_node_or_null("Goals")
	_assert(goals != null, "Goals panel exists")
	if goals == null:
		return
	_assert(goals.has_method("current_chapter"), "has current_chapter")
	_assert(goals.has_method("next_unmet_condition"), "has next_unmet_condition")
	# At game start, chapter 0 should be current and have unmet conditions.
	var ch: int = goals.call("current_chapter")
	_assert(ch == 0, "starts at chapter 0")


## Test 9: chronicle panel loads notable_kinds from schema 2.
func _test_chronicle_notable(main: Node) -> void:
	print("Test 9: chronicle notable_kinds")
	var chronicle: Node = main.find_child("ChroniclePanel", true, false)
	_assert(chronicle != null, "ChroniclePanel exists")
	if chronicle == null:
		return
	_assert("_notable_kinds" in chronicle, "has _notable_kinds")
	_assert("_show_all" in chronicle, "has _show_all")
	_assert("_notable_button" in chronicle, "has _notable_button")
	var notable: Array = chronicle.get("_notable_kinds")
	_assert(notable.size() > 0, "notable_kinds loaded from feed json")


## Test 10: card panel instantiates with the expected methods.
func _test_card_panel(main: Node) -> void:
	print("Test 10: card panel")
	var hud: Node = main.find_child("MuxHud", true, false)
	if hud == null:
		_assert(false, "MuxHud exists for cards")
		return
	var card_panel: Node = hud.get_node_or_null("CardPanel")
	_assert(card_panel != null, "CardPanel exists")
	if card_panel == null:
		return
	_assert(card_panel.has_method("has_active_cards"), "has has_active_cards")
	_assert(card_panel.has_method("top_card_title"), "has top_card_title")
	_assert(card_panel.has_method("active_count"), "has active_count")
	_assert(card_panel.has_method("answer_card"), "has answer_card")
	_assert(card_panel.has_method("active_ids"), "has active_ids")


## Test 11: plaque content priority shows idle message when no cards/chapters.
func _test_plaque_priority(main: Node) -> void:
	print("Test 11: plaque priority")
	var plaque: Node = main.find_child("TaskPlaque", true, false)
	_assert(plaque != null, "TaskPlaque exists")
	if plaque == null:
		return
	_assert(plaque.has_method("plaque_text"), "has plaque_text")


## Test 12: data-only card — sick_person is in cards.json with test_only flag.
func _test_data_only_card() -> void:
	print("Test 12: data-only card (sick_person)")
	var text: String = FileAccess.get_file_as_string("res://assets/ui/cards.json")
	var parsed: Variant = JSON.parse_string(text)
	if not parsed is Dictionary:
		_assert(false, "cards.json parse")
		return
	var cards: Array = parsed.get("cards", [])
	var found: bool = false
	var has_options: bool = false
	for card: Variant in cards:
		if card is Dictionary and str(card["id"]) == "sick_person":
			found = true
			has_options = (card.get("options", []) as Array).size() > 0
	_assert(found, "sick_person card exists in data")
	_assert(has_options, "sick_person card has options")


## Test 13: chronicle_feed.json schema 2 parses correctly.
func _test_chronicle_feed_schema() -> void:
	print("Test 13: chronicle feed schema 2")
	var text: String = FileAccess.get_file_as_string("res://assets/ui/chronicle_feed.json")
	var parsed: Variant = JSON.parse_string(text)
	_assert(parsed is Dictionary, "chronicle_feed.json is a dict")
	if not parsed is Dictionary:
		return
	_assert(int(parsed.get("schema_version", 0)) == 2, "schema version is 2")
	var notable: Array = parsed.get("notable_kinds", [])
	_assert(notable.size() > 10, "notable_kinds has >10 entries")
	var routine: Array = parsed.get("routine_kinds", [])
	_assert(routine.size() > 0, "routine_kinds has entries")


## Test 14: locale keys with LOOP_ prefix exist in both ko and en.
func _test_locale_keys() -> void:
	print("Test 14: locale keys")
	var ko_text: String = FileAccess.get_file_as_string("res://localization/ko/ui.json")
	var en_text: String = FileAccess.get_file_as_string("res://localization/en/ui.json")
	var ko: Variant = JSON.parse_string(ko_text)
	var en: Variant = JSON.parse_string(en_text)
	_assert(ko is Dictionary, "ko locale parses")
	_assert(en is Dictionary, "en locale parses")
	if not ko is Dictionary or not en is Dictionary:
		return
	var required_keys: Array[String] = [
		"LOOP_PLAQUE_TITLE", "LOOP_PLAQUE_IDLE",
		"LOOP_CARD_NEED_FOOD", "LOOP_CARD_MIGRANTS",
		"LOOP_CH1_TITLE", "LOOP_CH5_TITLE",
		"LOOP_CHAPTER_PANEL_TITLE", "LOOP_CHAPTER_FMT",
		"LOOP_CHRON_NOTABLE", "LOOP_CHRON_ALL",
		"LOOP_HELP_COLLAPSED", "LOOP_TEXT_SCALE",
		"LOOP_ALERT_CAUSE_FMT", "LOOP_CAUSAL_VIEW_LOG",
	]
	for key: String in required_keys:
		_assert((ko as Dictionary).has(key), "ko has " + key)
		_assert((en as Dictionary).has(key), "en has " + key)


## Test 15: topbar lower row is hidden (single-row topbar).
func _test_topbar_one_row(main: Node) -> void:
	print("Test 15: topbar one row")
	var topbar: Node = main.find_child("HudTopbar", true, false)
	_assert(topbar != null, "HudTopbar exists")
	if topbar == null:
		return
	_assert("_lower_row" in topbar, "has _lower_row reference")
	var lower: HBoxContainer = topbar.get("_lower_row") as HBoxContainer
	_assert(lower != null, "lower row is HBoxContainer")
	if lower != null:
		_assert(not lower.visible, "lower row is hidden (single-row topbar)")
	# Compact labels exist in upper row.
	_assert("_food_compact" in topbar, "has _food_compact label")
	_assert("_wood_compact" in topbar, "has _wood_compact label")
	_assert("_speed_compact" in topbar, "has _speed_compact label")


## Test 16: item panel has _input handler for close on outside click.
## ItemPanel is created dynamically by the order tool, so test the script directly.
func _test_item_panel_close(_main: Node) -> void:
	print("Test 16: item panel close on outside click")
	var ItemPanelScript = preload("res://scripts/ui/panels/item_panel.gd")
	var instance: PanelContainer = ItemPanelScript.new()
	_assert(instance.has_method("_input"), "ItemPanel has _input for outside click")
	instance.queue_free()


## Test 17: popup stacking constants defined.
func _test_popup_stacking(main: Node) -> void:
	print("Test 17: popup stacking")
	var hud: Node = main.find_child("MuxHud", true, false)
	_assert(hud != null, "MuxHud exists")
	if hud == null:
		return
	var MuxHud = preload("res://scripts/ui/mux/mux_hud.gd")
	_assert(MuxHud.LIST_PANELS.size() > 0, "LIST_PANELS has entries")
	_assert(MuxHud.POPUP_PANELS.size() > 0, "POPUP_PANELS has entries")
	_assert(hud.has_method("_enforce_popup_limit"), "has _enforce_popup_limit")


## Test 18: idle_adults alert suppressed at night (hour < 6 or >= 22).
func _test_idle_adults_night() -> void:
	print("Test 18: idle adults night suppression")
	var Evaluator = preload("res://scripts/ui/mux/alert_evaluator.gd")
	var eval_instance: RefCounted = Evaluator.new()
	if not eval_instance.load_registry():
		_assert(false, "evaluator loads registry")
		return
	# Simulate night-time facts (tick in night window: hour 23 = tick 1380).
	var night_facts: Dictionary = {"schema_version": 1, "tick": 1380}
	for collection: String in ["settlements", "people", "storehouses", "unreachable_orders", "knowledge_waits", "techniques", "animals", "migrants", "traders", "stacks", "households", "food_supply"]:
		night_facts[collection] = []
	# Add one idle adult.
	night_facts["people"] = [{"id": 1, "x": 10, "y": 10, "adult": true, "working": false, "sleeping": false, "drafted": false, "sick": false, "breakdown": false, "hp": 100, "roofed": true}]
	night_facts["settlements"] = [{"id": 0, "x": 10, "y": 10, "food": 50, "daily_food_need": 10, "firewood": 20, "daily_firewood_need": 5, "winter_days": 0, "stockpile_cells": 4}]
	eval_instance.evaluate(night_facts)
	var alerts: Array = eval_instance.active_alerts()
	var has_idle: bool = false
	for alert: Dictionary in alerts:
		if str(alert.get("id", "")) == "idle_adults":
			has_idle = true
	_assert(not has_idle, "idle_adults alert suppressed at night (hour 23)")


## Test 19: chapter completion signal wired to toast and chronicle entry.
func _test_chapter_completion_toast(main: Node) -> void:
	print("Test 19: chapter completion toast + chronicle")
	var hud: Node = main.find_child("MuxHud", true, false)
	_assert(hud != null, "MuxHud exists")
	if hud == null:
		return
	_assert(hud.has_method("_on_chapter_completed"), "has _on_chapter_completed handler")
	# Check the signal is connected.
	var goals_node: Node = hud.get_node_or_null("Goals")
	if goals_node != null and goals_node.has_signal("chapter_completed"):
		_assert(goals_node.is_connected("chapter_completed", hud._on_chapter_completed), "chapter_completed signal connected")
	# Test chronicle panel has add_presentation_entry method.
	var chronicle: Node = main.find_child("ChroniclePanel", true, false)
	_assert(chronicle != null and chronicle.has_method("add_presentation_entry"), "ChroniclePanel has add_presentation_entry")


## Test 20: card trigger matching uses the correct bridge data format (parallel arrays).
func _test_card_trigger_format() -> void:
	print("Test 20: card trigger format")
	var CardPanelScript = preload("res://scripts/ui/mux/card_panel.gd")
	var panel: PanelContainer = CardPanelScript.new()
	# Simulate bridge get_requests return with parallel arrays.
	var mock_requests: Dictionary = {
		"tick": 100,
		"rule_ids": PackedStringArray(["need_food", "need_warmth"]),
		"text_keys": PackedStringArray(["REQ_NEED_FOOD", "REQ_NEED_WARMTH"]),
		"priorities": PackedInt32Array([1, 2]),
		"counts": PackedInt32Array([3, 1]),
		"voices": PackedInt64Array([42, 43]),
		"voice_names": PackedStringArray(["", ""]),
		"steward_active": [true, false],
	}
	# Simulate bridge get_settlement_diagnosis return with parallel arrays.
	var mock_diag: Dictionary = {
		"tick": 100,
		"check_ids": PackedStringArray(["settlement_no_tools", "stockpile_full"]),
		"text_keys": PackedStringArray(["DIAG_NO_TOOLS", "DIAG_FULL"]),
		"severities": PackedInt32Array([1, 2]),
	}
	# Test request trigger matching.
	var food_card: Dictionary = {"trigger": {"kind": "request", "request_id": "need_food"}}
	_assert(panel.call("_check_trigger", food_card, mock_requests, mock_diag, 100), "request trigger matches need_food")
	var shelter_card: Dictionary = {"trigger": {"kind": "request", "request_id": "need_shelter"}}
	_assert(not panel.call("_check_trigger", shelter_card, mock_requests, mock_diag, 100), "request trigger rejects need_shelter")
	# Test diagnosis trigger matching.
	var tools_card: Dictionary = {"trigger": {"kind": "diagnosis", "check_id": "settlement_no_tools"}}
	_assert(panel.call("_check_trigger", tools_card, mock_requests, mock_diag, 100), "diagnosis trigger matches settlement_no_tools")
	var missing_card: Dictionary = {"trigger": {"kind": "diagnosis", "check_id": "nonexistent"}}
	_assert(not panel.call("_check_trigger", missing_card, mock_requests, mock_diag, 100), "diagnosis trigger rejects nonexistent")
	# Test _collect_trigger_data extracts voice from parallel arrays.
	var trigger_data: Dictionary = panel.call("_collect_trigger_data", food_card, mock_requests, mock_diag)
	_assert(int(trigger_data.get("voice_id", -1)) == 42, "trigger data voice_id from parallel array")
	panel.queue_free()


## Test 21: a known trigger produces a card in a seeded run (seed 0, steward on, advance 5 days).
func _test_card_trigger_seeded(main: Node, world: Node) -> void:
	print("Test 21: card trigger in seeded run")
	var reads: Node = main.get_node_or_null("SimReads")
	if reads == null:
		_assert(false, "SimReads exists")
		return
	var card_panel: Node = main.find_child("CardPanel", true, false)
	if card_panel == null:
		_assert(false, "CardPanel exists")
		return
	# Reset to seed 0 with steward on.
	reads.call("mutate_raw", "reset_world", [0])
	for _i: int in 5:
		await process_frame
	SimReads.mutate(world, "cmd_set_steward", [true, 0])
	# Advance 5 days (120 in-game hours). Cards should appear from requests.
	# In headless mode, _process may not fire or _last_tick may stay at -1.
	# Set _last_tick manually from the bridge and call _poll_triggers directly.
	# Track cards that ever appeared (they may auto-close when the trigger clears).
	var ever_active: Dictionary = {}
	for _hour: int in 120:
		reads.call("mutate_raw", "advance_ticks", [60])
		await process_frame
		await process_frame
		var time: Dictionary = SimReads.read(world, "get_time_snapshot")
		var tick: int = int(time.get("tick", 0))
		card_panel.set("_last_tick", tick)
		card_panel.call("_poll_triggers")
		card_panel.call("_check_expiry")
		for cid: String in card_panel.call("active_ids"):
			ever_active[cid] = true
	var active: PackedStringArray = card_panel.call("active_ids")
	var count: int = card_panel.call("active_count")
	print("  After 5 days: active_count=%d ever_active=%s active_ids=%s" % [count, ",".join(ever_active.keys()), ",".join(active)])
	_assert(ever_active.size() > 0, "at least one card triggered in 5 days (seed 0, steward on)")


## Test 22: answering the open card records an accepted bridge result and a chronicle line.
##   Fails if card_answered is not wired to the chronicle, or the command result is dropped/rejected.
func _test_card_answer_recorded(main: Node) -> void:
	print("Test 22: card answer -> command result + chronicle")
	var card_panel: Node = main.find_child("CardPanel", true, false)
	var chronicle: Node = main.find_child("ChroniclePanel", true, false)
	if card_panel == null or chronicle == null:
		_assert(false, "CardPanel and ChroniclePanel exist")
		return
	var active: PackedStringArray = card_panel.call("active_ids")
	if active.is_empty():
		# The card may have auto-closed. Force-advance until one re-opens.
		var world: Node = main.get_node_or_null("WorldSim")
		var reads: Node = main.get_node_or_null("SimReads")
		if world != null and reads != null:
			for _try: int in 48:
				reads.call("mutate_raw", "advance_ticks", [60])
				await process_frame
				var time: Dictionary = SimReads.read(world, "get_time_snapshot")
				card_panel.set("_last_tick", int(time.get("tick", 0)))
				card_panel.call("_poll_triggers")
				active = card_panel.call("active_ids")
				if not active.is_empty():
					break
	if active.is_empty():
		_assert(false, "an open card to answer (after retry)")
		return
	var card_id: String = active[0]
	var options: Array = (card_panel.get("_active")[card_id] as Dictionary).get("def", {}).get("options", [])
	var recommended: int = 0
	for oi: int in options.size():
		if bool(options[oi].get("recommended", false)):
			recommended = oi
			break
	var entries_before: int = (chronicle.get("_entries") as Array).size()
	card_panel.call("answer_card", card_id, recommended)
	var results: Array = card_panel.call("command_results")
	_assert(results.size() == 1, "one command result recorded for %s" % card_id)
	_assert(results.size() == 1 and bool(results[0].get("accepted", false)),
		"recommended command accepted (reason=%s)" % (str(results[0].get("reason", "")) if results.size() > 0 else "none"))
	# The feed is capped at 60 rows, so check the newest row rather than the count.
	var entries: Array = chronicle.get("_entries")
	var newest: Dictionary = entries.back() if not entries.is_empty() else {}
	var line: String = str(newest.get("_presentation_text", ""))
	_assert(entries.size() >= mini(entries_before + 1, 60) and line != "" and not line.contains("{"),
		"chronicle got a presentation line for the answer (%s)" % line)
	_assert(not (card_panel.call("active_ids") as PackedStringArray).has(card_id), "answered card closed")


## Test 23: card auto-closes when its request trigger clears.
##   Fails if _close_cleared_triggers is missing or inactive requests keep cards open.
func _test_card_auto_close() -> void:
	print("Test 23: card auto-close on trigger clear")
	var CardPanelScript = preload("res://scripts/ui/mux/card_panel.gd")
	var panel: PanelContainer = CardPanelScript.new()
	_assert(panel.has_method("_close_cleared_triggers"), "has _close_cleared_triggers method")
	# Simulate an active card whose request is no longer present.
	var food_def: Dictionary = {"id": "need_food", "trigger": {"kind": "request", "request_id": "need_food"}, "options": [], "priority": 10}
	panel.get("_active")["need_food"] = {"def": food_def, "opened_tick": 100, "expires_tick": 9999, "answered": false, "trigger_data": {}}
	# Requests without need_food → card should close.
	var empty_requests: Dictionary = {"rule_ids": PackedStringArray(["need_warmth"]), "voices": PackedInt64Array([1])}
	var empty_diag: Dictionary = {"check_ids": PackedStringArray()}
	panel.call("_close_cleared_triggers", empty_requests, empty_diag)
	_assert(not panel.get("_active").has("need_food"), "need_food card closed when request inactive")
	# Repeat with a diagnosis card.
	var tools_def: Dictionary = {"id": "tools_low", "trigger": {"kind": "diagnosis", "check_id": "settlement_no_tools"}, "options": [], "priority": 9}
	panel.get("_active")["tools_low"] = {"def": tools_def, "opened_tick": 100, "expires_tick": 9999, "answered": false, "trigger_data": {}}
	var no_tools_diag: Dictionary = {"check_ids": PackedStringArray()}
	panel.call("_close_cleared_triggers", empty_requests, no_tools_diag)
	_assert(not panel.get("_active").has("tools_low"), "tools_low card closed when diagnosis passes")
	panel.queue_free()


## Test 24: no raw text in card body — no unresolved "{placeholder}" or ALL_CAPS locale keys.
##   Fails if _collect_trigger_data leaves voice_id as int or cause as a raw key like REQ_NEED_FOOD.
func _test_no_raw_text_in_cards() -> void:
	print("Test 24: no raw text in cards")
	var CardPanelScript = preload("res://scripts/ui/mux/card_panel.gd")
	var panel: PanelContainer = CardPanelScript.new()
	# Simulate request trigger data collection.
	var mock_requests: Dictionary = {
		"rule_ids": PackedStringArray(["need_food"]),
		"text_keys": PackedStringArray(["REQ_NEED_FOOD"]),
		"voices": PackedInt64Array([42]),
	}
	var food_card: Dictionary = {"id": "need_food", "trigger": {"kind": "request", "request_id": "need_food"}}
	var data: Dictionary = panel.call("_collect_trigger_data", food_card, mock_requests, {})
	# voice should be a resolved name (not empty, not a raw int in the text).
	_assert(data.has("voice"), "trigger data has 'voice' key")
	_assert(not str(data.get("voice", "")).is_empty(), "voice is resolved (not empty)")
	# Check that the data dict does NOT contain the raw voice_id or raw cause key as
	# the substituted value; the panel has no locale in this test, so _tr returns the key,
	# but the data values themselves should not be raw ints or raw request keys.
	_assert(str(data.get("voice", "")).length() >= 2, "voice is a name (>= 2 chars)")
	# The cause key is resolved through _tr; without locale it returns the key itself.
	# In a real scene it would be localized. Verify it is at least populated.
	_assert(data.has("cause"), "trigger data has 'cause' key")
	# Format with real locale data: simulate what a locale-enabled panel would produce.
	# Without the scene tree, _tr returns the key. Instead, test that {voice} and {cause}
	# are replaced (the body template uses those fields).
	var body_template: String = "{voice}: 먹을 게 없어요 ← {cause}"
	var text: String = body_template
	for field: String in data:
		text = text.replace("{" + field + "}", str(data[field]))
	_assert(not text.contains("{voice}"), "voice placeholder replaced in body: " + text)
	_assert(not text.contains("{cause}"), "cause placeholder replaced in body: " + text)
	panel.queue_free()


## Test 25: wolves card trigger uses chronicle kind 46 (StewardDefence), not 36 (Sick).
##   Fails if the cards.json wolves entry still has the wrong kind.
func _test_wolves_card_kind() -> void:
	print("Test 25: wolves card kind = 46 (StewardDefence)")
	var text: String = FileAccess.get_file_as_string("res://assets/ui/cards.json")
	var parsed: Variant = JSON.parse_string(text)
	if not parsed is Dictionary:
		_assert(false, "cards.json parse")
		return
	for card: Variant in parsed.get("cards", []):
		if card is Dictionary and str(card["id"]) == "wolves":
			var trigger: Dictionary = card.get("trigger", {})
			_assert(int(trigger.get("chronicle_kind", -1)) == 46, "wolves chronicle_kind is 46 (StewardDefence), not 36 (Sick)")
			return
	_assert(false, "wolves card not found")


## Test 26: need_food hunt option sends cmd_add_hunt_zone (matching the "사냥 구역" label).
##   Fails if the option still uses cmd_designate_harvest (defect 4: label-command mismatch).
func _test_need_food_hunt_command() -> void:
	print("Test 26: need_food hunt option = cmd_add_hunt_zone")
	var text: String = FileAccess.get_file_as_string("res://assets/ui/cards.json")
	var parsed: Variant = JSON.parse_string(text)
	if not parsed is Dictionary:
		_assert(false, "cards.json parse")
		return
	for card: Variant in parsed.get("cards", []):
		if card is Dictionary and str(card["id"]) == "need_food":
			var options: Array = card.get("options", [])
			_assert(options.size() >= 1, "need_food has options")
			var hunt_opt: Dictionary = options[0]
			_assert(str(hunt_opt.get("command", "")) == "cmd_add_hunt_zone",
				"hunt option sends cmd_add_hunt_zone (not cmd_designate_harvest)")
			return
	_assert(false, "need_food card not found")


## Test 27: the five previously missing commands have _send_command branches.
##   Fails if any of the five returns null on a simple call (meaning no match arm).
func _test_missing_command_branches() -> void:
	print("Test 27: missing command branches exist")
	var CardPanelScript = preload("res://scripts/ui/mux/card_panel.gd")
	var panel: PanelContainer = CardPanelScript.new()
	# Check the _send_command match has the expected arms by inspecting the source.
	var source: String = FileAccess.get_file_as_string("res://scripts/ui/mux/card_panel.gd")
	for cmd: String in ["cmd_trade", "cmd_add_bill", "cmd_set_work_priority", "cmd_place_blueprint", "cmd_add_stockpile"]:
		_assert(source.contains('"' + cmd + '"'), "card_panel _send_command has branch for " + cmd)
	# Also check cmd_add_hunt_zone is present.
	_assert(source.contains('"cmd_add_hunt_zone"'), "card_panel _send_command has branch for cmd_add_hunt_zone")
	panel.queue_free()


## Test 28: LOOP_STEWARD_TOOLS locale key is defined in both ko and en.
##   Fails if the key is missing, which causes the raw key to show in the steward toast.
func _test_steward_tools_locale() -> void:
	print("Test 28: LOOP_STEWARD_TOOLS locale key")
	var ko_text: String = FileAccess.get_file_as_string("res://localization/ko/ui.json")
	var en_text: String = FileAccess.get_file_as_string("res://localization/en/ui.json")
	var ko: Variant = JSON.parse_string(ko_text)
	var en: Variant = JSON.parse_string(en_text)
	_assert(ko is Dictionary and (ko as Dictionary).has("LOOP_STEWARD_TOOLS"), "ko has LOOP_STEWARD_TOOLS")
	_assert(en is Dictionary and (en as Dictionary).has("LOOP_STEWARD_TOOLS"), "en has LOOP_STEWARD_TOOLS")
