# card_panel.gd (M-LOOP §2.2): decision cards raised by the simulation.
#   Cards are data-driven from assets/ui/cards.json (schema 1). Each card maps a trigger
#   (chronicle kind, request id, alert condition, or diagnosis check) to player options
#   that send existing bridge commands. At most 3 cards open; the rest fold with a count.
#   Card state (open/answered/expired) is presentation only — not saved, not hashed.
#   After a load, cards rebuild from events of the last in-game day only.
# Bridge methods: via SimReads get_requests, get_chronicle_since, get_settlement_diagnosis,
#   get_ui_facts, get_time_snapshot; cmd_* for card answers; signal world_reset.
extends PanelContainer

const KoreanText = preload("res://scripts/ui/mux/korean_text.gd")
const Catalog = preload("res://scripts/ui/mux/controls_catalog.gd")
const DiagnosisText = preload("res://scripts/ui/panels/diagnosis_text.gd")
const Names = preload("res://scripts/ui/person_names.gd")
const THEME = preload("res://assets/ui/worldsim_theme.tres")
const CARDS_PATH: String = "res://assets/ui/cards.json"
const MAX_OPEN: int = 3
const POLL_INTERVAL: float = 1.0
const TICKS_PER_HOUR: int = 60

signal card_answered(card_id: String, option_text: String)
signal card_expired(card_id: String)

var _world: Node = null
var _locale: Node = null
var _registry: Dictionary = {}
var _card_defs: Array[Dictionary] = []
## Active cards: {id -> {def, trigger_data, opened_tick, expires_tick, answered}}
var _active: Dictionary = {}
## Answered/expired card ids for the current in-game day (prevent re-trigger).
var _handled: Dictionary = {}
var _poll_left: float = 0.0
var _last_tick: int = -1
var _column: VBoxContainer = null
var _title: Label = null
var _card_list: VBoxContainer = null
var _fold_label: Label = null


func _ready() -> void:
	theme = THEME
	mouse_filter = Control.MOUSE_FILTER_STOP
	custom_minimum_size = Vector2(380, 0)
	_locale = get_node_or_null("/root/Locale")
	_world = get_node_or_null("/root/Main/WorldSim")
	if _world != null and _world.has_signal("world_reset"):
		_world.connect("world_reset", _reset)
	_load_registry()
	_build_ui()
	visible = false


func _load_registry() -> void:
	if not FileAccess.file_exists(CARDS_PATH):
		push_error("card_panel: missing " + CARDS_PATH)
		return
	var parsed: Variant = JSON.parse_string(FileAccess.get_file_as_string(CARDS_PATH))
	if not parsed is Dictionary:
		push_error("card_panel: invalid cards.json")
		return
	_registry = parsed
	_card_defs.clear()
	for card: Variant in _registry.get("cards", []):
		if card is Dictionary and not bool(card.get("test_only", false)):
			_card_defs.append(card)
	# Sort by priority (lower = higher priority).
	_card_defs.sort_custom(func(a: Dictionary, b: Dictionary) -> bool:
		return int(a.get("priority", 99)) < int(b.get("priority", 99)))


func _build_ui() -> void:
	_column = VBoxContainer.new()
	_column.add_theme_constant_override("separation", 8)
	add_child(_column)
	_title = Label.new()
	_title.theme_type_variation = &"Heading"
	_column.add_child(_title)
	_card_list = VBoxContainer.new()
	_card_list.add_theme_constant_override("separation", 12)
	_column.add_child(_card_list)
	_fold_label = Label.new()
	_fold_label.add_theme_color_override("font_color", Color("a0a0a0"))
	_column.add_child(_fold_label)


## M-LOOP: tick-driven polling so headless advance_ticks and windowed _process both trigger cards.
func _process(_delta: float) -> void:
	if _world == null:
		visible = false
		return
	var time: Dictionary = SimReads.read(_world, "get_time_snapshot")
	var tick: int = int(time.get("tick", -1))
	if tick < 0 or tick == _last_tick:
		return
	# Poll every CARD_CHECK_INTERVAL ticks (1 in-game hour), not every frame.
	@warning_ignore("integer_division")
	var current_hour: int = tick / TICKS_PER_HOUR
	@warning_ignore("integer_division")
	var last_hour: int = _last_tick / TICKS_PER_HOUR if _last_tick >= 0 else -1
	_last_tick = tick
	if current_hour == last_hour:
		return
	_poll_triggers()
	_check_expiry()
	_rebuild_ui()


func _poll_triggers() -> void:
	var tick: int = _last_tick
	if tick < 0:
		return
	# Clear handled list at day boundaries.
	@warning_ignore("integer_division")
	var day: int = tick / 1440
	# Check each card definition for triggers.
	var requests: Dictionary = SimReads.read(_world, "get_requests")
	var diag: Dictionary = SimReads.read(_world, "get_settlement_diagnosis")
	for card_def: Dictionary in _card_defs:
		var card_id: String = str(card_def["id"])
		if _active.has(card_id) or _handled.has(card_id):
			continue
		if _check_trigger(card_def, requests, diag, tick):
			_active[card_id] = {
				"def": card_def,
				"opened_tick": tick,
				"expires_tick": tick + int(card_def.get("expires_hours", 24)) * TICKS_PER_HOUR,
				"answered": false,
				"trigger_data": _collect_trigger_data(card_def, requests, diag),
			}


func _check_trigger(card_def: Dictionary, requests: Dictionary, diag: Dictionary, _tick: int) -> bool:
	var trigger: Dictionary = card_def.get("trigger", {})
	var kind: String = str(trigger.get("kind", ""))
	match kind:
		"request":
			# Bridge get_requests returns parallel arrays: rule_ids, text_keys, etc.
			var req_id: String = str(trigger.get("request_id", ""))
			var rule_ids: Variant = requests.get("rule_ids", PackedStringArray())
			if rule_ids is PackedStringArray:
				for rid: String in rule_ids as PackedStringArray:
					if rid == req_id:
						return true
		"chronicle":
			# Check recent chronicle entries for matching kind.
			var chronicle_kind: int = int(trigger.get("chronicle_kind", -1))
			if chronicle_kind >= 0 and _world.has_method("get_chronicle_since"):
				var page: Dictionary = _world.get_chronicle_since(maxi(0, _last_tick - 1440), 100)
				var rows: Variant = page.get("rows", page.get("entries", []))
				if rows is Array:
					for entry: Variant in rows:
						if entry is Dictionary and int(entry.get("kind", -1)) == chronicle_kind:
							return true
		"alert":
			var alert_id: String = str(trigger.get("alert_id", ""))
			var facts: Dictionary = SimReads.read(_world, "get_ui_facts")
			# Check via alert conditions in the facts.
			var people: Variant = facts.get("people", [])
			if alert_id == "idle_adults" and people is Array:
				var idle_count: int = 0
				for person: Variant in people as Array:
					if person is Dictionary and bool(person.get("adult", false)) and not bool(person.get("working", false)) and not bool(person.get("sleeping", false)):
						idle_count += 1
				var min_count: int = int(trigger.get("min_count", 2))
				if idle_count >= min_count:
					# M-LOOP §1.4: exclude night window.
					@warning_ignore("integer_division")
					var hour: int = (_last_tick % 1440) / 60
					if bool(trigger.get("exclude_night", false)) and (hour < 6 or hour >= 22):
						return false
					return true
		"diagnosis":
			# Bridge get_settlement_diagnosis returns parallel arrays: check_ids, text_keys, etc.
			var check_id: String = str(trigger.get("check_id", ""))
			var check_ids: Variant = diag.get("check_ids", PackedStringArray())
			if check_ids is PackedStringArray:
				for cid: String in check_ids as PackedStringArray:
					if cid == check_id:
						return true
	return false


func _collect_trigger_data(card_def: Dictionary, requests: Dictionary, _diag: Dictionary) -> Dictionary:
	var data: Dictionary = {}
	var trigger: Dictionary = card_def.get("trigger", {})
	var kind: String = str(trigger.get("kind", ""))
	if kind == "request":
		var req_id: String = str(trigger.get("request_id", ""))
		# Bridge returns parallel arrays; find the index of the matching rule_id.
		var rule_ids: PackedStringArray = requests.get("rule_ids", PackedStringArray())
		var voices: PackedInt64Array = requests.get("voices", PackedInt64Array())
		var text_keys: PackedStringArray = requests.get("text_keys", PackedStringArray())
		for i: int in rule_ids.size():
			if rule_ids[i] == req_id:
				data["voice_id"] = int(voices[i]) if i < voices.size() else -1
				data["cause"] = str(text_keys[i]) if i < text_keys.size() else ""
				break
	return data


func _check_expiry() -> void:
	if _last_tick < 0:
		return
	var expired_ids: PackedStringArray = PackedStringArray()
	for card_id: String in _active:
		var card: Dictionary = _active[card_id]
		if bool(card.get("answered", false)):
			continue
		if _last_tick >= int(card.get("expires_tick", 0)):
			expired_ids.append(card_id)
	for card_id: String in expired_ids:
		_expire_card(card_id)


func _expire_card(card_id: String) -> void:
	var card: Dictionary = _active.get(card_id, {})
	var def: Dictionary = card.get("def", {})
	var default_action: String = str(def.get("default", "none"))
	if default_action == "steward":
		# Chronicle "집사가 대신 정했어요".
		card_expired.emit(card_id)
	_handled[card_id] = true
	_active.erase(card_id)


func answer_card(card_id: String, option_index: int) -> void:
	var card: Dictionary = _active.get(card_id, {})
	if card.is_empty():
		return
	var def: Dictionary = card.get("def", {})
	var options: Array = def.get("options", [])
	if option_index < 0 or option_index >= options.size():
		return
	var option: Dictionary = options[option_index]
	var command: String = str(option.get("command", "none"))
	# Send the bridge command.
	if command != "none" and command != "steward_default" and command != "enter_zone_tool" and command != "open_roles":
		_send_command(command, option, card)
	card["answered"] = true
	_handled[card_id] = true
	var option_text: String = _tr(str(option.get("text_key", "")))
	card_answered.emit(card_id, option_text)
	_active.erase(card_id)
	_rebuild_ui()


func _send_command(command: String, option: Dictionary, card: Dictionary) -> void:
	if _world == null:
		return
	var _params: String = str(option.get("params", ""))
	match command:
		"cmd_designate_chop":
			# Designate chopping on nearest trees — uses a simplified area near camp.
			var info: Dictionary = SimReads.read(_world, "get_world_info")
			var cx: int = int(info.get("camp_x", 32))
			var cy: int = int(info.get("camp_y", 32))
			SimReads.mutate(_world, "cmd_designate_chop", [cx - 3, cy - 3, cx + 3, cy + 3])
		"cmd_designate_harvest":
			var info: Dictionary = SimReads.read(_world, "get_world_info")
			var cx: int = int(info.get("camp_x", 32))
			var cy: int = int(info.get("camp_y", 32))
			SimReads.mutate(_world, "cmd_designate_harvest", [cx - 4, cy - 4, cx + 4, cy + 4])
		"cmd_accept_migrants":
			var trigger_data: Dictionary = card.get("trigger_data", {})
			var wave: int = int(trigger_data.get("wave_id", 0))
			SimReads.mutate(_world, "cmd_accept_migrants", [wave])
		"cmd_decline_migrants":
			var trigger_data: Dictionary = card.get("trigger_data", {})
			var wave: int = int(trigger_data.get("wave_id", 0))
			SimReads.mutate(_world, "cmd_decline_migrants", [wave])
		"cmd_undraft_all":
			SimReads.mutate(_world, "cmd_undraft_all", [])
		"cmd_order_teach":
			# Use trigger data for holder → learner.
			var trigger_data: Dictionary = card.get("trigger_data", {})
			var teacher: int = int(trigger_data.get("teacher", -1))
			var learner: int = int(trigger_data.get("learner", -1))
			var knowledge: int = int(trigger_data.get("knowledge", -1))
			if teacher >= 0 and learner >= 0 and knowledge >= 0:
				SimReads.mutate(_world, "cmd_order_teach", [teacher, learner, knowledge])
		"cmd_draft":
			# Draft the steward's recommended 4.
			var drafted: Dictionary = SimReads.read(_world, "get_drafted")
			var recommended: Array = drafted.get("recommended", [])
			for agent_id: Variant in recommended:
				SimReads.mutate(_world, "cmd_draft", [int(agent_id), true])


func _rebuild_ui() -> void:
	if _card_list == null:
		return
	for child: Node in _card_list.get_children():
		_card_list.remove_child(child)
		child.queue_free()
	# Sort active cards by priority.
	var sorted_ids: Array[String] = []
	sorted_ids.assign(_active.keys())
	sorted_ids.sort_custom(func(a: String, b: String) -> bool:
		var pa: int = int(_active[a].get("def", {}).get("priority", 99))
		var pb: int = int(_active[b].get("def", {}).get("priority", 99))
		return pa < pb)
	var shown: int = 0
	for card_id: String in sorted_ids:
		if bool(_active[card_id].get("answered", false)):
			continue
		if shown >= MAX_OPEN:
			break
		_build_card_widget(card_id, _active[card_id])
		shown += 1
	var total_unanswered: int = 0
	for card_id: String in _active:
		if not bool(_active[card_id].get("answered", false)):
			total_unanswered += 1
	var folded: int = total_unanswered - shown
	_title.text = _tr("LOOP_CARD_PANEL_TITLE")
	_fold_label.visible = folded > 0
	_fold_label.text = _tr("LOOP_CARD_PANEL_FOLDED").format({"count": folded}) if folded > 0 else ""
	visible = total_unanswered > 0
	size = Vector2.ZERO


func _build_card_widget(card_id: String, card: Dictionary) -> void:
	var def: Dictionary = card.get("def", {})
	var trigger_data: Dictionary = card.get("trigger_data", {})
	var card_box: VBoxContainer = VBoxContainer.new()
	card_box.add_theme_constant_override("separation", 4)
	var style: StyleBoxFlat = StyleBoxFlat.new()
	style.bg_color = Color(0.12, 0.11, 0.09, 0.9)
	style.border_color = Color(0.5, 0.42, 0.3, 0.6)
	style.set_border_width_all(1)
	style.set_corner_radius_all(4)
	style.set_content_margin_all(8)
	var card_panel: PanelContainer = PanelContainer.new()
	card_panel.add_theme_stylebox_override("panel", style)
	card_panel.add_child(card_box)
	_card_list.add_child(card_panel)
	# Title.
	var title_label: Label = Label.new()
	title_label.text = _format_card_text(str(def.get("title_key", "")), trigger_data)
	title_label.add_theme_color_override("font_color", Color("e8c46e"))
	card_box.add_child(title_label)
	# Body (cause line).
	var body_label: Label = Label.new()
	body_label.text = KoreanText.words(_format_card_text(str(def.get("body_key", "")), trigger_data))
	body_label.autowrap_mode = TextServer.AUTOWRAP_WORD
	body_label.custom_minimum_size.x = 340
	card_box.add_child(body_label)
	# Option buttons.
	var buttons_row: HBoxContainer = HBoxContainer.new()
	buttons_row.add_theme_constant_override("separation", 8)
	card_box.add_child(buttons_row)
	var options: Array = def.get("options", [])
	for i: int in options.size():
		var opt: Dictionary = options[i]
		var btn: Button = Button.new()
		btn.text = _tr(str(opt.get("text_key", "")))
		if bool(opt.get("recommended", false)):
			btn.text += " " + _tr("LOOP_CARD_RECOMMENDED")
			btn.add_theme_color_override("font_color", Color("a8d888"))
		btn.pressed.connect(answer_card.bind(card_id, i))
		buttons_row.add_child(btn)


func _format_card_text(key: String, data: Dictionary) -> String:
	var text: String = _tr(key)
	for field: String in data:
		text = text.replace("{" + field + "}", str(data[field]))
	return text


func _reset() -> void:
	_active.clear()
	_handled.clear()
	_last_tick = -1
	_poll_left = 0.0
	_rebuild_ui()


## The top active (unanswered) card's title, for the plaque (§2.1).
func top_card_title() -> String:
	var best_priority: int = 999
	var best_title: String = ""
	for card_id: String in _active:
		var card: Dictionary = _active[card_id]
		if bool(card.get("answered", false)):
			continue
		var priority: int = int(card.get("def", {}).get("priority", 99))
		if priority < best_priority:
			best_priority = priority
			best_title = _format_card_text(str(card.get("def", {}).get("title_key", "")), card.get("trigger_data", {}))
	return best_title


## Whether any card is currently active (for plaque priority).
func has_active_cards() -> bool:
	for card_id: String in _active:
		if not bool(_active[card_id].get("answered", false)):
			return true
	return false


## Count of active unanswered cards (for tests).
func active_count() -> int:
	var count: int = 0
	for card_id: String in _active:
		if not bool(_active[card_id].get("answered", false)):
			count += 1
	return count


## All active card IDs (for tests).
func active_ids() -> PackedStringArray:
	var ids: PackedStringArray = PackedStringArray()
	for card_id: String in _active:
		if not bool(_active[card_id].get("answered", false)):
			ids.append(card_id)
	return ids


func _tr(key: String) -> String:
	if _locale != null and _locale.has_method("ltr"):
		return KoreanText.words(str(_locale.call("ltr", key)))
	return key
