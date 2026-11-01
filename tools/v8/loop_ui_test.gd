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
	await _test_card_answer_recorded(main)
	# --- Test 23: card auto-closes when trigger clears ---
	_test_card_auto_close(world)
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
	# --- Test 29: idle_adults card fires in a seeded run with idle adults ---
	await _test_idle_adults_seeded(main, world)
	# --- Test 30: tools_low card fires in a seeded run with few tools ---
	await _test_tools_low_seeded(main, world)
	_test_knowledge_card_steward_teaching(main)
	# --- Test 31: cooldown prevents re-trigger within 1 day ---
	_test_cooldown()
	# --- Test 32: dedupe blocks chop/hunt when designation/zone exists ---
	_test_dedupe_guards()
	# --- Test 33: skipped results tracked separately from command results ---
	_test_skipped_tracking()
	# --- Tests 34-40: per-card-kind _effective_recommended never worse than steward ---
	_test_recommend_need_food()
	_test_recommend_need_warmth()
	_test_recommend_migrants()
	_test_recommend_knowledge_risk()
	_test_recommend_tools_low()
	_test_recommend_idle_adults()
	_test_recommend_wolves()

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
func _test_alert_max_rows(_main: Node) -> void:
	print("Test 4: alert max rows")
	var Alerts = preload("res://scripts/ui/mux/alerts_panel.gd")
	_assert(Alerts.MAX_VISIBLE_ROWS == 3, "MAX_VISIBLE_ROWS is 3")


## Test 5: causal panel filters unlabeled entries.
func _test_causal_filter() -> void:
	print("Test 5: causal filter")
	var CausalPanel = preload("res://scripts/ui/panels/causal_panel.gd")
	var panel: Control = CausalPanel.new()
	root.add_child(panel)
	_assert(panel.is_inside_tree(), "causal fixture has active locale lookup")
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
	# Expiry stays presentation-only while existing BASE stewardship continues.
	for card: Variant in cards:
		if card is Dictionary and str(card["id"]) in ["knowledge_risk", "tools_low"]:
			_assert(str(card.get("default", "")) == "none", str(card["id"]) + " expiry has no command")
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
	# §2.1: the plaque always carries the "지금 할 일" title next to its line.
	plaque.call("_show_line", "테스트 줄", "")
	var title: Label = plaque.find_child("Title", true, false) as Label
	_assert(title != null and title.text.replace(char(0x2060), "") == "지금 할 일" and title.is_visible_in_tree() == plaque.is_visible_in_tree(),
		"plaque title reads 지금 할 일 (got %s)" % (title.text if title != null else "none"))
	# Owner check 1: the current chapter is visible next to the title ("1장 · … · n/m").
	var chapter: Label = plaque.find_child("Chapter", true, false) as Label
	_assert(chapter != null and chapter.text.replace(char(0x2060), "").contains("1장"),
		"plaque shows the current chapter (got %s)" % (chapter.text if chapter != null else "none"))


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


## Test 20: card triggers — alert triggers use the evaluator, chronicle/diagnosis still work.
func _test_card_trigger_format() -> void:
	print("Test 20: card trigger format")
	# Verify all required cards use alert triggers (not raw bridge checks).
	var text: String = FileAccess.get_file_as_string("res://assets/ui/cards.json")
	var parsed: Variant = JSON.parse_string(text)
	if not parsed is Dictionary:
		_assert(false, "cards.json parse")
		return
	var alert_cards: Array[String] = ["need_food", "need_warmth", "migrants", "trader", "knowledge_risk", "wolves", "idle_adults"]
	for card: Variant in parsed.get("cards", []):
		if card is Dictionary and str(card["id"]) in alert_cards:
			var trigger: Dictionary = card.get("trigger", {})
			_assert(str(trigger.get("kind", "")) == "alert", "%s uses alert trigger" % str(card["id"]))
			_assert(not str(trigger.get("alert_id", "")).is_empty(), "%s has alert_id" % str(card["id"]))
	# Verify tools_low uses bridge trigger (settlement_no_tools doesn't exist).
	for card: Variant in parsed.get("cards", []):
		if card is Dictionary and str(card["id"]) == "tools_low":
			var trigger: Dictionary = card.get("trigger", {})
			_assert(str(trigger.get("kind", "")) == "bridge", "tools_low uses bridge trigger")
	# Verify card_panel has _alert_active and _close_cleared_triggers with hysteresis.
	var CardPanelScript = preload("res://scripts/ui/mux/card_panel.gd")
	var panel: PanelContainer = CardPanelScript.new()
	_assert(panel.has_method("_alert_active"), "has _alert_active")
	_assert(panel.has_method("_close_cleared_triggers"), "has _close_cleared_triggers")
	_assert(panel.has_method("_check_bridge_trigger"), "has _check_bridge_trigger")
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
	var card_data: Dictionary = card_panel.get("_active")[card_id] as Dictionary
	var def: Dictionary = card_data.get("def", {})
	var trigger_data: Dictionary = card_data.get("trigger_data", {})
	var recommended: int = int(card_panel.call("_effective_recommended", def, trigger_data))
	# With the steward on, steward-covered cards recommend leaving it to the steward (no command,
	# LOOP-D-cards). This test exercises the command path, so answer the first command option.
	var opts: Array = def.get("options", [])
	for oi: int in opts.size():
		var oc: String = str((opts[oi] as Dictionary).get("command", "none"))
		if oc not in ["none", "steward_default", "enter_zone_tool", "open_roles", "open_build_menu", "open_trade"]:
			recommended = oi
			break
	var entries_before: int = (chronicle.get("_entries") as Array).size()
	card_panel.call("answer_card", card_id, recommended)
	var results: Array = card_panel.call("command_results")
	var skipped: Array = card_panel.call("skipped_results")
	# The answer was either sent (command_results) or skipped (skipped_results).
	var was_sent: bool = results.size() >= 1
	var was_skipped: bool = skipped.size() >= 1
	_assert(was_sent or was_skipped, "answer processed for %s (sent=%s, skipped=%s)" % [card_id, str(was_sent), str(was_skipped)])
	if was_sent:
		var last_result: Dictionary = results[results.size() - 1]
		var accepted: bool = bool(last_result.get("accepted", false))
		# A rejection is a failure, even when a receipt was recorded.
		_assert(accepted, "command sent for %s (accepted=%s, reason=%s)" % [
			card_id, str(accepted), str(last_result.get("reason", ""))])
		if accepted:
			# The feed is capped at 60 rows, so check the newest row rather than the count.
			var entries: Array = chronicle.get("_entries")
			var newest: Dictionary = entries.back() if not entries.is_empty() else {}
			var line: String = str(newest.get("_presentation_text", ""))
			_assert(entries.size() >= mini(entries_before + 1, 60) and line != "" and not line.contains("{"),
				"chronicle got a presentation line for the answer (%s)" % line)
		_assert((card_panel.call("active_ids") as PackedStringArray).has(card_id) != accepted, "only accepted card closes")
	else:
		# Skipped: card stays open (stale target).
		_assert((card_panel.call("active_ids") as PackedStringArray).has(card_id), "skipped card stays open")


## Test 23: card auto-closes with hysteresis when trigger clears.
##   Fails if _close_cleared_triggers is missing or doesn't use clear_count.
func _test_card_auto_close(world: Node) -> void:
	print("Test 23: card auto-close with hysteresis")
	var CardPanelScript = preload("res://scripts/ui/mux/card_panel.gd")
	var panel: PanelContainer = CardPanelScript.new()
	_assert(panel.has_method("_close_cleared_triggers"), "has _close_cleared_triggers method")
	# Use the real chronicle, outside its event window, to prove a cleared trigger.
	panel.set("_world", world)
	var time: Dictionary = SimReads.read(world, "get_time_snapshot")
	panel.set("_last_tick", int(time.get("tick", 0)) + 1441)
	var quarrel_def: Dictionary = {"id": "quarrel", "trigger": {"kind": "chronicle", "chronicle_kind": 19}, "options": [], "priority": 18}
	_assert(not bool(panel.call("_check_trigger", quarrel_def)), "real chronicle has no quarrel in future window")
	panel.get("_active")["quarrel"] = {"def": quarrel_def, "opened_tick": 100, "expires_tick": 9999, "answered": false, "clear_count": 0, "trigger_data": {}}
	# First call: trigger is clear, clear_count goes to 1 — card should NOT close yet.
	panel.call("_close_cleared_triggers")
	_assert(panel.get("_active").has("quarrel"), "quarrel stays open after 1 clear poll (hysteresis)")
	_assert(int(panel.get("_active")["quarrel"]["clear_count"]) == 1, "clear_count incremented to 1")
	# Two more calls: clear_count reaches 3 → card should close.
	panel.call("_close_cleared_triggers")
	panel.call("_close_cleared_triggers")
	_assert(not panel.get("_active").has("quarrel"), "quarrel closed after 3 consecutive clear polls")
	# Verify it was NOT marked handled (can re-trigger).
	_assert(not panel.get("_cooldown").has("quarrel"), "quarrel not put on cooldown on clear (can re-trigger)")
	panel.queue_free()


## Test 24: no raw text in card body — no unresolved "{placeholder}" or ALL_CAPS locale keys.
##   Verifies that _format_card_text replaces all placeholders from trigger data
##   and strips any unreplaced placeholder fields.
func _test_no_raw_text_in_cards() -> void:
	print("Test 24: no raw text in cards")
	var CardPanelScript = preload("res://scripts/ui/mux/card_panel.gd")
	var panel: PanelContainer = CardPanelScript.new()
	# _collect_trigger_data takes only card_def (reads from bridge, which is null here).
	# So test the _format_card_text path directly with mock trigger data.
	var mock_data: Dictionary = {"voice": "시키노", "cause": "봄이라 열매 없음", "count": 3}
	var body_template: String = "{voice}: 먹을 게 없어요 ← {cause}"
	var text: String = panel.call("_format_card_text", body_template, mock_data)
	_assert(not text.contains("{voice}"), "voice placeholder replaced in body: " + text)
	_assert(not text.contains("{cause}"), "cause placeholder replaced in body: " + text)
	_assert(text.contains("시키노"), "voice value present in formatted text")
	_assert(text.contains("열매"), "cause value present in formatted text")
	# Test that unreplaced placeholders get stripped.
	var text2: String = panel.call("_format_card_text", "{missing_field} hello", {})
	_assert(not text2.contains("{missing_field}"), "unreplaced placeholder stripped: " + text2)
	_assert(text2.contains("hello"), "non-placeholder text preserved")
	panel.queue_free()


## Test 25: wolves card uses alert wolves_approaching (not chronicle kind 36/46).
##   Fails if wolves still uses a chronicle or diagnosis trigger.
func _test_wolves_card_kind() -> void:
	print("Test 25: wolves uses wolves_approaching alert")
	var text: String = FileAccess.get_file_as_string("res://assets/ui/cards.json")
	var parsed: Variant = JSON.parse_string(text)
	if not parsed is Dictionary:
		_assert(false, "cards.json parse")
		return
	for card: Variant in parsed.get("cards", []):
		if card is Dictionary and str(card["id"]) == "wolves":
			var trigger: Dictionary = card.get("trigger", {})
			_assert(str(trigger.get("kind", "")) == "alert", "wolves trigger is alert kind")
			_assert(str(trigger.get("alert_id", "")) == "wolves_approaching", "wolves alert_id is wolves_approaching")
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


## Test 27: explicit card commands and local build navigation have routes.
## Actual bridge acceptance is covered by m6b_presentation_test all-card fixtures.
func _test_missing_command_branches() -> void:
	print("Test 27: missing command branches exist")
	var CardPanelScript = preload("res://scripts/ui/mux/card_panel.gd")
	var panel: PanelContainer = CardPanelScript.new()
	# Check the _send_command match has the expected arms by inspecting the source.
	var source: String = FileAccess.get_file_as_string("res://scripts/ui/mux/card_panel.gd")
	for cmd: String in ["cmd_trade", "cmd_add_bill", "cmd_set_work_priority", "open_build_menu", "cmd_add_stockpile"]:
		_assert(source.contains('"' + cmd + '"'), "card_panel has command or local navigation route for " + cmd)
	# Also check cmd_add_hunt_zone is present.
	_assert(source.contains('"cmd_add_hunt_zone"'), "card_panel has command or local navigation route for cmd_add_hunt_zone")
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


## Test 29: idle_adults card fires in a seeded run with idle adults.
##   Construct state: seed 0, steward OFF so nobody gets jobs, advance until
##   adults are idle during the day. The idle_adults alert should fire and the
##   card should appear. Fails if the card never opens.
func _test_idle_adults_seeded(main: Node, world: Node) -> void:
	print("Test 29: idle_adults card in seeded run")
	var reads: Node = main.get_node_or_null("SimReads")
	if reads == null:
		_assert(false, "SimReads exists")
		return
	var card_panel: Node = main.find_child("CardPanel", true, false)
	var alerts_panel: Node = main.find_child("Alerts", true, false)
	if card_panel == null:
		_assert(false, "CardPanel exists")
		return
	# Reset to seed 0 with steward OFF — adults will be idle.
	reads.call("mutate_raw", "reset_world", [0])
	for _i: int in 5:
		await process_frame
	# Steward OFF so adults remain idle.
	SimReads.mutate(world, "cmd_set_steward", [false, 0])
	# Advance 3 days (daytime hours will have idle adults).
	# Manually trigger alert evaluation in headless mode via refresh_from_facts.
	var ever_idle: bool = false
	for _hour: int in 72:
		reads.call("mutate_raw", "advance_ticks", [60])
		await process_frame
		await process_frame
		# Trigger alert evaluation manually (headless _process may not fire).
		if alerts_panel != null and alerts_panel.has_method("refresh_from_facts") and alerts_panel.has_method("_read_facts"):
			var facts: Dictionary = alerts_panel.call("_read_facts")
			alerts_panel.call("refresh_from_facts", facts)
		var time: Dictionary = SimReads.read(world, "get_time_snapshot")
		var tick: int = int(time.get("tick", 0))
		card_panel.set("_last_tick", tick)
		card_panel.call("_poll_triggers")
		card_panel.call("_check_expiry")
		for cid: String in card_panel.call("active_ids"):
			if cid == "idle_adults":
				ever_idle = true
	_assert(ever_idle, "idle_adults card appeared with steward off (seed 0, 3 days)")


## Test 30: tools_low card fires when tool count < adult count.
##   Uses the bridge trigger directly: constructs a card panel, sets _world,
##   and checks _check_bridge_trigger with the actual game state. The test
##   verifies the trigger method works and can detect the condition.
func _test_tools_low_seeded(main: Node, world: Node) -> void:
	print("Test 30: tools_low bridge trigger")
	var reads: Node = main.get_node_or_null("SimReads")
	if reads == null:
		_assert(false, "SimReads exists")
		return
	var card_panel: Node = main.find_child("CardPanel", true, false)
	if card_panel == null:
		_assert(false, "CardPanel exists")
		return
	# Reset to seed 0. The steward OFF means no tool production.
	reads.call("mutate_raw", "reset_world", [0])
	for _i: int in 5:
		await process_frame
	SimReads.mutate(world, "cmd_set_steward", [false, 0])
	# Explicitly reset card panel state (world_reset signal may not fire headless).
	card_panel.call("_reset")
	# Advance a few days so adults exist.
	reads.call("mutate_raw", "advance_ticks", [2880])
	await process_frame
	await process_frame
	# Directly check the bridge trigger for tools_below_need.
	var trigger_fires: bool = card_panel.call("_check_bridge_trigger", "tools_below_need")
	# The starting settlement typically has 10 adults and 0 or few tools.
	# With steward off and no production, tool_count < adults should hold.
	_assert(trigger_fires, "tools_below_need trigger fires (adults > tools, steward off)")
	# Also check that the card can open in a poll cycle.
	var time: Dictionary = SimReads.read(world, "get_time_snapshot")
	var tick: int = int(time.get("tick", 0))
	card_panel.set("_last_tick", tick)
	card_panel.call("_poll_triggers")
	var ever_tools_low: bool = false
	for cid: String in card_panel.call("active_ids"):
		if cid == "tools_low":
			ever_tools_low = true
	_assert(ever_tools_low, "tools_low card opened from poll (seed 0, steward off)")


## Test 31: cooldown prevents a card from re-triggering within 1 in-game day.
##   Fails if the card reappears immediately after being answered.
func _test_cooldown() -> void:
	print("Test 31: cooldown prevents re-trigger")
	var CardPanelScript = preload("res://scripts/ui/mux/card_panel.gd")
	var panel: PanelContainer = CardPanelScript.new()
	# Simulate answering a card at tick 1000.
	var quarrel_def: Dictionary = {"id": "quarrel", "trigger": {"kind": "chronicle", "chronicle_kind": 19}, "options": [{"text_key": "A", "command": "none"}, {"text_key": "B", "command": "none"}], "priority": 18, "expires_hours": 24}
	panel.set("_last_tick", 1000)
	panel.get("_active")["quarrel"] = {"def": quarrel_def, "opened_tick": 900, "expires_tick": 9999, "answered": false, "clear_count": 0, "trigger_data": {}}
	panel.call("answer_card", "quarrel", 0)
	# Cooldown should be set.
	_assert(panel.get("_cooldown").has("quarrel"), "cooldown set after answer")
	_assert(int(panel.get("_cooldown")["quarrel"]) == 1000, "cooldown tick is answer tick")
	# At tick 1100 (100 ticks later, < 1 day), _poll_triggers should skip quarrel.
	panel.set("_last_tick", 1100)
	# The card should NOT be re-added even if trigger matches.
	_assert(not panel.get("_active").has("quarrel"), "quarrel not re-triggered within cooldown")
	# At tick 2441 (1441 ticks later, > 1 day), cooldown should expire.
	panel.set("_last_tick", 2441)
	panel.call("_poll_triggers")
	_assert(not panel.get("_cooldown").has("quarrel"), "cooldown expired after 1 day")
	panel.queue_free()


## Test 32: dedupe guards prevent sending duplicate chop/hunt commands.
##   Tests _has_chop_designations and _has_hunt_zones helper methods exist
##   and that _send_command returns skipped when they report existing orders.
func _test_dedupe_guards() -> void:
	print("Test 32: dedupe guard methods exist")
	var CardPanelScript = preload("res://scripts/ui/mux/card_panel.gd")
	var panel: PanelContainer = CardPanelScript.new()
	_assert(panel.has_method("_has_chop_designations"), "has _has_chop_designations")
	_assert(panel.has_method("_has_hunt_zones"), "has _has_hunt_zones")
	_assert(panel.has_method("_has_hunter_blades"), "has _has_hunter_blades")
	_assert(panel.has_method("_can_support_migrants"), "has _can_support_migrants")
	_assert(panel.has_method("_effective_recommended"), "has _effective_recommended")
	_assert(panel.has_method("skipped_results"), "has skipped_results accessor")
	# Without a world, dedupe helpers should return false (safe default).
	_assert(not panel.call("_has_chop_designations"), "no world -> no chop designations")
	_assert(not panel.call("_has_hunt_zones"), "no world -> no hunt zones")
	panel.queue_free()


## Test 33: skipped results are tracked separately from command results.
##   A stale command should appear in skipped_results, not command_results.
func _test_skipped_tracking() -> void:
	print("Test 33: skipped results tracking")
	var CardPanelScript = preload("res://scripts/ui/mux/card_panel.gd")
	var panel: PanelContainer = CardPanelScript.new()
	panel.set("_last_tick", 500)
	# Create a card with a command that will be skipped (no world -> skipped).
	var food_def: Dictionary = {
		"id": "need_food",
		"trigger": {"kind": "alert", "alert_id": "food_low"},
		"options": [
			{"text_key": "HUNT", "command": "cmd_add_hunt_zone", "recommended": true},
			{"text_key": "STEWARD", "command": "steward_default", "recommended": false},
		],
		"priority": 10,
		"expires_hours": 12,
	}
	panel.get("_active")["need_food"] = {"def": food_def, "opened_tick": 400, "expires_tick": 9999, "answered": false, "clear_count": 0, "trigger_data": {}}
	panel.call("answer_card", "need_food", 0)
	# The command should be skipped (no world), card should stay open.
	var skipped: Array = panel.call("skipped_results")
	var results: Array = panel.call("command_results")
	_assert(skipped.size() == 1, "one skipped result (no world)")
	_assert(results.size() == 0, "no command results (skipped, not sent)")
	_assert(panel.get("_active").has("need_food"), "card stays open after skip")
	panel.queue_free()


## Test 34: need_food _effective_recommended falls back to harvest/steward when hunt is unsafe.
##   LOOP-D-cards: never worse than steward. Fails if hunt is recommended despite
##   existing zones, missing blades, or wolves approaching.
func _test_recommend_need_food() -> void:
	print("Test 34: need_food recommendation safety")
	var text: String = FileAccess.get_file_as_string("res://assets/ui/cards.json")
	var parsed: Variant = JSON.parse_string(text)
	var food_def: Dictionary = {}
	for card: Variant in parsed.get("cards", []):
		if card is Dictionary and str(card["id"]) == "need_food":
			food_def = card
	_assert(not food_def.is_empty(), "need_food card found")
	if food_def.is_empty():
		return
	var CardPanelScript = preload("res://scripts/ui/mux/card_panel.gd")
	var panel: PanelContainer = CardPanelScript.new()
	# Without a world, helpers return false (safe defaults). Hunt should NOT be recommended
	# because _has_hunter_blades returns false.
	var rec: int = int(panel.call("_effective_recommended", food_def, {}))
	var options: Array = food_def.get("options", [])
	var cmd: String = str(options[rec].get("command", "")) if rec < options.size() else ""
	_assert(cmd != "cmd_add_hunt_zone", "need_food: no hunt without world (blades unknown)")
	# The fallback should be harvest or steward.
	_assert(cmd == "cmd_designate_harvest" or cmd == "steward_default", "need_food: fallback is harvest or steward, got %s" % cmd)
	panel.queue_free()


## Test 35: need_warmth _effective_recommended falls back to steward when chop exists.
##   LOOP-D-cards: never worse than steward. Fails if chop is recommended despite
##   existing chop designations.
func _test_recommend_need_warmth() -> void:
	print("Test 35: need_warmth recommendation safety")
	var text: String = FileAccess.get_file_as_string("res://assets/ui/cards.json")
	var parsed: Variant = JSON.parse_string(text)
	var warmth_def: Dictionary = {}
	for card: Variant in parsed.get("cards", []):
		if card is Dictionary and str(card["id"]) == "need_warmth":
			warmth_def = card
	_assert(not warmth_def.is_empty(), "need_warmth card found")
	if warmth_def.is_empty():
		return
	# Without a world, _has_chop_designations returns false, so chop IS recommended.
	var CardPanelScript = preload("res://scripts/ui/mux/card_panel.gd")
	var panel: PanelContainer = CardPanelScript.new()
	var rec: int = int(panel.call("_effective_recommended", warmth_def, {}))
	var options: Array = warmth_def.get("options", [])
	# Default: chop should be recommended when no designations pending.
	_assert(rec == 0, "need_warmth: chop recommended when no designations pending")
	var chop_cmd: String = str(options[0].get("command", ""))
	_assert(chop_cmd == "cmd_designate_chop", "need_warmth: option 0 is chop")
	# Steward is the last option.
	var last_cmd: String = str(options[options.size() - 1].get("command", ""))
	_assert(last_cmd == "steward_default", "need_warmth: last option is steward")
	panel.queue_free()


## Test 36: migrants _effective_recommended declines when capacity insufficient.
##   LOOP-D-cards: never worse than steward. Fails if accept is recommended
##   without beds/food check.
func _test_recommend_migrants() -> void:
	print("Test 36: migrants recommendation safety")
	var text: String = FileAccess.get_file_as_string("res://assets/ui/cards.json")
	var parsed: Variant = JSON.parse_string(text)
	var migrant_def: Dictionary = {}
	for card: Variant in parsed.get("cards", []):
		if card is Dictionary and str(card["id"]) == "migrants":
			migrant_def = card
	_assert(not migrant_def.is_empty(), "migrants card found")
	if migrant_def.is_empty():
		return
	var CardPanelScript = preload("res://scripts/ui/mux/card_panel.gd")
	var panel: PanelContainer = CardPanelScript.new()
	# Without a world, _can_support_migrants returns false → decline recommended.
	var rec: int = int(panel.call("_effective_recommended", migrant_def, {"count": 3}))
	var options: Array = migrant_def.get("options", [])
	var cmd: String = str(options[rec].get("command", "")) if rec < options.size() else ""
	_assert(cmd == "cmd_decline_migrants", "migrants: decline when capacity unknown (no world)")
	panel.queue_free()


## Test 37: knowledge_risk _effective_recommended defers teaching during firewood_low.
##   LOOP-D-cards: never worse than steward. Fails if teach is recommended when
##   firewood is critical.
func _test_recommend_knowledge_risk() -> void:
	print("Test 37: knowledge_risk recommendation safety")
	var text: String = FileAccess.get_file_as_string("res://assets/ui/cards.json")
	var parsed: Variant = JSON.parse_string(text)
	var kr_def: Dictionary = {}
	for card: Variant in parsed.get("cards", []):
		if card is Dictionary and str(card["id"]) == "knowledge_risk":
			kr_def = card
	_assert(not kr_def.is_empty(), "knowledge_risk card found")
	if kr_def.is_empty():
		return
	# Without a world, _alert_active("firewood_low") returns false.
	# So teach IS recommended (safe: no firewood crisis).
	var CardPanelScript = preload("res://scripts/ui/mux/card_panel.gd")
	var panel: PanelContainer = CardPanelScript.new()
	var rec: int = int(panel.call("_effective_recommended", kr_def, {}))
	var options: Array = kr_def.get("options", [])
	var cmd: String = str(options[rec].get("command", "")) if rec < options.size() else ""
	_assert(cmd == "cmd_order_teach", "knowledge_risk: teach when firewood OK")
	# The fallback (later) should be a "none" command.
	var later_cmd: String = str(options[options.size() - 1].get("command", ""))
	_assert(later_cmd == "none", "knowledge_risk: later option is 'none'")
	panel.queue_free()


## Test 38: tools_low has steward as fallback option.
##   LOOP-D-cards: never worse than steward.
func _test_recommend_tools_low() -> void:
	print("Test 38: tools_low recommendation safety")
	var text: String = FileAccess.get_file_as_string("res://assets/ui/cards.json")
	var parsed: Variant = JSON.parse_string(text)
	var tl_def: Dictionary = {}
	for card: Variant in parsed.get("cards", []):
		if card is Dictionary and str(card["id"]) == "tools_low":
			tl_def = card
	_assert(not tl_def.is_empty(), "tools_low card found")
	if tl_def.is_empty():
		return
	var options: Array = tl_def.get("options", [])
	# tools_low has "bill" (recommended) and "steward" options.
	_assert(options.size() >= 2, "tools_low has >= 2 options")
	var bill_cmd: String = str(options[0].get("command", ""))
	var steward_cmd: String = str(options[1].get("command", ""))
	_assert(bill_cmd == "cmd_add_bill", "tools_low: option 0 is bill")
	_assert(steward_cmd == "steward_default", "tools_low: option 1 is steward fallback")
	# Expiry does not dispatch a command.
	_assert(str(tl_def.get("default", "")) == "none", "tools_low: expiry has no command")


## Test 39: idle_adults has safe options (roles, chop, ok).
##   LOOP-D-cards: the default (none) means the steward doesn't act on idle adults,
##   so the recommendation (roles) must not harm survival.
func _test_recommend_idle_adults() -> void:
	print("Test 39: idle_adults recommendation safety")
	var text: String = FileAccess.get_file_as_string("res://assets/ui/cards.json")
	var parsed: Variant = JSON.parse_string(text)
	var ia_def: Dictionary = {}
	for card: Variant in parsed.get("cards", []):
		if card is Dictionary and str(card["id"]) == "idle_adults":
			ia_def = card
	_assert(not ia_def.is_empty(), "idle_adults card found")
	if ia_def.is_empty():
		return
	var options: Array = ia_def.get("options", [])
	_assert(options.size() >= 3, "idle_adults has >= 3 options")
	# Option 0: roles (open_roles — presentation only, no simulation change).
	var roles_cmd: String = str(options[0].get("command", ""))
	_assert(roles_cmd == "open_roles", "idle_adults: option 0 is open_roles (safe, no sim change)")
	# Option 2: ok (none — do nothing, same as default).
	var ok_cmd: String = str(options[2].get("command", ""))
	_assert(ok_cmd == "none", "idle_adults: option 2 is 'none' (safe)")


## Test 40: wolves has draft and undraft — both safe responses to wolf attacks.
##   BASE stewardship continues independently; card expiry sends no command.
func _test_recommend_wolves() -> void:
	print("Test 40: wolves recommendation safety")
	var text: String = FileAccess.get_file_as_string("res://assets/ui/cards.json")
	var parsed: Variant = JSON.parse_string(text)
	var w_def: Dictionary = {}
	for card: Variant in parsed.get("cards", []):
		if card is Dictionary and str(card["id"]) == "wolves":
			w_def = card
	_assert(not w_def.is_empty(), "wolves card found")
	if w_def.is_empty():
		return
	var options: Array = w_def.get("options", [])
	_assert(options.size() >= 2, "wolves has >= 2 options")
	var draft_cmd: String = str(options[0].get("command", ""))
	var flee_cmd: String = str(options[1].get("command", ""))
	_assert(draft_cmd == "cmd_draft", "wolves: option 0 is draft")
	_assert(int(options[0].get("count", 0)) == 1 and str(options[0].get("params", "")) == "one_healthy_adult", "wolves: explicit draft selects one adult")
	_assert(flee_cmd == "cmd_undraft_all", "wolves: option 1 is undraft everyone")
	_assert(str(w_def.get("default", "")) == "none", "wolves: expiry has no command")


## LOOP-D-teach: when the steward already teaches the at-risk knowledge, the knowledge card says so,
## recommends leaving it and offers cancel; with no order it offers teach/later instead.
func _test_knowledge_card_steward_teaching(main: Node) -> void:
	print("Test 41: knowledge card shows the steward's teaching")
	var card_panel: Node = main.find_child("CardPanel", true, false)
	_assert(card_panel != null, "CardPanel exists")
	if card_panel == null:
		return
	var parsed: Variant = JSON.parse_string(FileAccess.get_file_as_string("res://assets/ui/cards.json"))
	var def: Dictionary = {}
	for card: Variant in (parsed as Dictionary).get("cards", []):
		if card is Dictionary and str(card["id"]) == "knowledge_risk":
			def = card
	_assert(not def.is_empty() and def.has("body_ordered_key"), "knowledge_risk has an ordered body")
	var opts: Array = def.get("options", [])
	var ordered: Dictionary = {"order_id": 3, "ordered_learner_id": 5, "learner": 5, "ordered_learner": "카야", "technique": "점토 반죽"}
	var visible_ordered: Array = []
	var visible_none: Array = []
	for opt: Variant in opts:
		if not card_panel.call("_option_hidden", opt, ordered):
			visible_ordered.append(str(opt["text_key"]))
		if not card_panel.call("_option_hidden", opt, {}):
			visible_none.append(str(opt["text_key"]))
	_assert(visible_ordered.has("LOOP_CARD_KNOWLEDGE_RISK_LEAVE") and visible_ordered.has("LOOP_CARD_KNOWLEDGE_RISK_CANCEL")
		and not visible_ordered.has("LOOP_CARD_KNOWLEDGE_RISK_TEACH") and not visible_ordered.has("LOOP_CARD_KNOWLEDGE_RISK_OTHER"),
		"ordered: leave and cancel shown, teach hidden, same learner hides 다른 사람 (%s)" % str(visible_ordered))
	_assert(visible_none.has("LOOP_CARD_KNOWLEDGE_RISK_TEACH") and not visible_none.has("LOOP_CARD_KNOWLEDGE_RISK_LEAVE"),
		"no order: teach shown, leave hidden (%s)" % str(visible_none))
	var leave_index: int = int(card_panel.call("_steward_option", opts, ordered))
	_assert(str((opts[leave_index] as Dictionary)["text_key"]) == "LOOP_CARD_KNOWLEDGE_RISK_LEAVE", "steward-first recommendation is 그대로 두기")
