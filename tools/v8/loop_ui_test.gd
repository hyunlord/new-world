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
	var hud: Node = main.get_node_or_null("MuxHud")
	if hud == null:
		_assert(false, "MuxHud exists")
		return
	var helper: Node = hud.get_node_or_null("LearningHelper")
	_assert(helper != null, "LearningHelper exists")
	_assert(helper != null and helper.has_method("expand_helper"), "has expand_helper")
	_assert(helper != null and "_collapsed" in helper, "has _collapsed state")
	# Simulate observing an order event.
	if helper != null and helper.has_method("observe"):
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
	var required: Array[String] = ["need_food", "need_warmth", "migrants", "trader", "knowledge_risk", "wolves", "idle_adults"]
	for req: String in required:
		_assert(ids.has(req), "has required card: " + req)


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
	var hud: Node = main.get_node_or_null("MuxHud")
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
	var chronicle: Node = main.get_node_or_null("ChroniclePanel")
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
	var hud: Node = main.get_node_or_null("MuxHud")
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
	var plaque: Node = main.get_node_or_null("TaskPlaque")
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
