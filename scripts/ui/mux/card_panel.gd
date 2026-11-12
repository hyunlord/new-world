# card_panel.gd (M-LOOP §2.2): decision cards raised by the simulation.
#   Cards are data-driven from assets/ui/cards.json (schema 1). Triggers use the alert
#   evaluator (alerts.json), chronicle entries, or custom bridge checks. At most 3 cards
#   open; the rest fold with a count. Card state is presentation only — not saved, not hashed.
#   After a load, cards rebuild from the current alert/event state.
# Bridge methods: via SimReads get_requests, get_chronicle_since, get_settlement_diagnosis,
#   get_ui_facts, get_time_snapshot, get_world_info, get_card_harvest_target,
#   get_command_log, get_neighbours, get_work_priorities, get_story;
#   cmd_* for card answers; signal world_reset.
extends PanelContainer

const KoreanText = preload("res://scripts/ui/mux/korean_text.gd")
const ChronicleText = preload("res://scripts/ui/chronicle_text.gd")
const KnowledgeText = preload("res://scripts/ui/knowledge_text.gd")
const Content = preload("res://scripts/ui/content_context.gd")
const Names = preload("res://scripts/ui/person_names.gd")
const THEME = preload("res://assets/ui/worldsim_theme.tres")
const CARDS_PATH: String = "res://assets/ui/cards.json"
const MAX_OPEN: int = 3
const TICKS_PER_HOUR: int = 60
## Consecutive clear polls before closing an alert-triggered card (hysteresis).
const DEFAULT_CLEAR_POLLS: int = 3

signal card_answered(card_id: String, card_title: String, option_text: String)
signal state_changed
signal build_requested
signal roles_requested
signal card_expired(card_id: String, card_title: String)

var _world: Node = null
var _locale: Node = null
var _alerts_panel: Node = null
var _registry: Dictionary = {}
var _card_defs: Array[Dictionary] = []
## Active cards: {id -> {def, trigger_data, opened_tick, expires_tick, answered, clear_count}}.
var _active: Dictionary = {}
## Cooldown: {card_id -> tick_when_answered}. Prevents re-trigger for 1 in-game day (1440 ticks).
var _cooldown: Dictionary = {}
## Bridge results of card answers actually sent since the last reset.
var _command_results: Array[Dictionary] = []
## Skipped commands: stale targets that were not sent, counted by kind.
var _skipped_results: Array[Dictionary] = []
var _last_tick: int = -1
var _clear_polls_needed: int = DEFAULT_CLEAR_POLLS
var _column: VBoxContainer = null
var _scroll: ScrollContainer = null
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
	if _locale != null:
		_locale.connect("language_changed", _rebuild_ui.unbind(1))
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
	_clear_polls_needed = int(_registry.get("clear_polls", DEFAULT_CLEAR_POLLS))
	_card_defs.clear()
	for card: Variant in _registry.get("cards", []):
		if card is Dictionary and bool(card.get("enabled", true)) and not bool(card.get("test_only", false)):
			_card_defs.append(card)
	_card_defs.sort_custom(func(a: Dictionary, b: Dictionary) -> bool:
		return int(a.get("priority", 99)) < int(b.get("priority", 99)))


func _build_ui() -> void:
	_scroll = ScrollContainer.new()
	_scroll.name = "CardScroll"
	_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	add_child(_scroll)
	_column = VBoxContainer.new()
	_column.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_column.add_theme_constant_override("separation", 8)
	_scroll.add_child(_column)
	_title = Label.new()
	_title.theme_type_variation = &"Heading"
	_column.add_child(_title)
	_card_list = VBoxContainer.new()
	_card_list.add_theme_constant_override("separation", 12)
	_column.add_child(_card_list)
	_fold_label = Label.new()
	_fold_label.add_theme_color_override("font_color", Color("a0a0a0"))
	_column.add_child(_fold_label)


## Full card content height, independent of the bounded scroll viewport.
func content_height() -> float:
	return _column.get_combined_minimum_size().y + get_theme_stylebox("panel").get_minimum_size().y


func _process(_delta: float) -> void:
	if _world == null:
		visible = false
		return
	var time: Dictionary = SimReads.read(_world, "get_time_snapshot")
	var tick: int = int(time.get("tick", -1))
	if tick < 0 or tick == _last_tick:
		return
	@warning_ignore("integer_division")
	@warning_ignore("integer_division")
	var current_hour: int = tick / TICKS_PER_HOUR
	@warning_ignore("integer_division")
	@warning_ignore("integer_division")
	var last_hour: int = _last_tick / TICKS_PER_HOUR if _last_tick >= 0 else -1
	_last_tick = tick
	if current_hour == last_hour:
		return
	_poll_triggers()
	_check_expiry()
	_rebuild_ui()


## Resolve the sibling alerts panel (lazy, since add_child order may vary).
func _get_alerts_panel() -> Node:
	if _alerts_panel == null and get_parent() != null:
		_alerts_panel = get_parent().get_node_or_null("Alerts")
	return _alerts_panel


## Whether the named alert is currently active in the evaluator.
func _alert_active(alert_id: String) -> bool:
	var panel: Node = _get_alerts_panel()
	if panel == null or not "evaluator" in panel:
		return false
	var evaluator: RefCounted = panel.get("evaluator")
	if evaluator == null:
		return false
	var matches: Dictionary = evaluator.get("_matches")
	var causes: Variant = matches.get(alert_id, [])
	return causes is Array and (causes as Array).size() > 0


## The causes array for the named alert (empty if inactive).
func _alert_causes(alert_id: String) -> Array:
	var panel: Node = _get_alerts_panel()
	if panel == null or not "evaluator" in panel:
		return []
	var evaluator: RefCounted = panel.get("evaluator")
	if evaluator == null:
		return []
	var matches: Dictionary = evaluator.get("_matches")
	var causes: Variant = matches.get(alert_id, [])
	return causes if causes is Array else []


func _poll_triggers() -> void:
	var tick: int = _last_tick
	if tick < 0:
		return
	var previous_ids: Array = _active.keys()
	# Expire stale cooldowns (older than 1 day = 1440 ticks).
	var expired_cooldowns: PackedStringArray = PackedStringArray()
	for cid: String in _cooldown:
		if tick - int(_cooldown[cid]) >= 1440:
			expired_cooldowns.append(cid)
	for cid: String in expired_cooldowns:
		_cooldown.erase(cid)
	# Close cards whose trigger has been clear for N consecutive polls (hysteresis).
	_close_cleared_triggers()
	# Check each card definition for new triggers.
	for card_def: Dictionary in _card_defs:
		var card_id: String = str(card_def["id"])
		if _active.has(card_id) or _cooldown.has(card_id):
			continue
		if _check_trigger(card_def):
			_active[card_id] = {
				"def": card_def,
				"opened_tick": tick,
				"expires_tick": tick + int(card_def.get("expires_hours", 24)) * TICKS_PER_HOUR,
				"answered": false,
				"clear_count": 0,
				"trigger_data": _collect_trigger_data(card_def),
			}
	if previous_ids != _active.keys():
		state_changed.emit()


func _check_trigger(card_def: Dictionary) -> bool:
	var trigger: Dictionary = card_def.get("trigger", {})
	var kind: String = str(trigger.get("kind", ""))
	match kind:
		"alert":
			return _alert_active(str(trigger.get("alert_id", "")))
		"chronicle":
			var chronicle_kind_val: int = int(trigger.get("chronicle_kind", -1))
			if chronicle_kind_val >= 0 and _world.has_method("get_chronicle_since"):
				var probe: Dictionary = _world.get_chronicle_since(0, 1)
				var total: int = int(probe.get("total", 0))
				if total > 0:
					var start_idx: int = maxi(0, total - 200)
					var page: Dictionary = _world.get_chronicle_since(start_idx, 200)
					var min_tick: int = maxi(0, _last_tick - 1440)
					for row: Dictionary in ChronicleText.rows(page):
						if int(row.get("kind", -1)) == chronicle_kind_val and int(row.get("tick", 0)) >= min_tick:
							return true
		"diagnosis":
			var check_id: String = str(trigger.get("check_id", ""))
			var diag: Dictionary = SimReads.read(_world, "get_settlement_diagnosis")
			var check_ids: Variant = diag.get("check_ids", PackedStringArray())
			if check_ids is PackedStringArray:
				for cid: String in check_ids as PackedStringArray:
					if cid == check_id:
						return true
		"bridge":
			return _check_bridge_trigger(str(trigger.get("read", "")))
	return false


## Custom bridge-based trigger checks for cards without a matching alert.
## Data in get_ui_facts uses parallel-array format (not Array of Dicts).
func _check_bridge_trigger(read_name: String) -> bool:
	if _world == null:
		return false
	if read_name in ["sowing_window", "harvest_ready"]:
		return not preload("res://scripts/ui/field_facts.gd").target(_world, read_name).is_empty()
	match read_name:
		"first_livestock_kid":
			var stock: Dictionary = SimReads.read(_world, "get_livestock")
			return int(stock.get("first_kid_id", -1)) >= 0 and stock.get("ids", PackedInt64Array()).has(int(stock.get("first_kid_id", -1)))
		"tin_shortage":
			return bool(SimReads.read(_world, "get_ui_facts").get("tin_shortage", false))
		"cloth_short":
			return bool(SimReads.read(_world, "get_settlement_diagnosis").get("cloth_short", false))
		"tools_below_need":
			var tool_codes: PackedInt64Array = PackedInt64Array()
			for definition: Dictionary in _card_defs:
				if str((definition.get("trigger", {}) as Dictionary).get("read", "")) != read_name:
					continue
				var items: Variant = definition.get("tool_items", [])
				if items is Array:
					for item: Variant in items:
						if item is String:
							var code: int = Content.code("items", item)
							if code >= 0:
								tool_codes.append(code)
			if tool_codes.is_empty():
				return false
			var facts: Dictionary = SimReads.read(_world, "get_ui_facts")
			# Stacks: {kinds: PackedInt64Array, counts: PackedInt64Array, ...}.
			var stacks: Variant = facts.get("stacks", {})
			var tool_count: int = 0
			if stacks is Dictionary:
				var kinds: Variant = (stacks as Dictionary).get("kinds", PackedInt64Array())
				var counts: Variant = (stacks as Dictionary).get("counts", PackedInt64Array())
				if kinds is PackedInt64Array and counts is PackedInt64Array:
					var k_arr: PackedInt64Array = kinds as PackedInt64Array
					var c_arr: PackedInt64Array = counts as PackedInt64Array
					for i: int in k_arr.size():
						if tool_codes.has(int(k_arr[i])) and i < c_arr.size():
							tool_count += int(c_arr[i])
			# People: {ids: PackedInt64Array, children: PackedByteArray, ...}.
			var people: Variant = facts.get("people", {})
			var adults: int = 0
			if people is Dictionary:
				var ids: Variant = (people as Dictionary).get("ids", PackedInt64Array())
				var children: Variant = (people as Dictionary).get("children", PackedByteArray())
				if ids is PackedInt64Array and children is PackedByteArray:
					var id_arr: PackedInt64Array = ids as PackedInt64Array
					var ch_arr: PackedByteArray = children as PackedByteArray
					for i: int in id_arr.size():
						if i < ch_arr.size() and int(ch_arr[i]) == 0:
							adults += 1
			return adults > 0 and tool_count < adults
	return false


## Close active cards whose trigger has been clear for N consecutive polls.
## Uses hysteresis to avoid flapping (the request/alert may toggle).
func _close_cleared_triggers() -> void:
	var to_close: PackedStringArray = PackedStringArray()
	for card_id: String in _active:
		var card: Dictionary = _active[card_id]
		if bool(card.get("answered", false)):
			continue
		var def: Dictionary = card.get("def", {})
		var still_active: bool = _check_trigger(def)
		if still_active:
			card["clear_count"] = 0
		else:
			card["clear_count"] = int(card.get("clear_count", 0)) + 1
			if int(card["clear_count"]) >= _clear_polls_needed:
				to_close.append(card_id)
	for card_id: String in to_close:
		# Don't mark handled — allow the card to re-trigger if the condition returns.
		_active.erase(card_id)


func _collect_trigger_data(card_def: Dictionary) -> Dictionary:
	var data: Dictionary = {}
	var trigger: Dictionary = card_def.get("trigger", {})
	var kind: String = str(trigger.get("kind", ""))
	var lang: String = _current_lang()
	var card_id: String = str(card_def.get("id", ""))
	if card_id == "tin_shortage":
		data["caravan_id"] = int(SimReads.read(_world, "get_ui_facts").get("tin_caravan", -1))
		return data
	if card_id == "first_kid":
		data["livestock_id"] = int(SimReads.read(_world, "get_livestock").get("first_kid_id", -1))
		return data
	if card_id in ["sowing_window", "harvest_ready"]:
		data = preload("res://scripts/ui/field_facts.gd").target(_world, card_id)
		return data
	if kind == "alert":
		var alert_id: String = str(trigger.get("alert_id", ""))
		var causes: Array = _alert_causes(alert_id)
		match card_id:
			"need_food":
				var requests: Dictionary = SimReads.read(_world, "get_requests")
				var rule_ids: PackedStringArray = requests.get("rule_ids", PackedStringArray())
				var voices: PackedInt64Array = requests.get("voices", PackedInt64Array())
				var text_keys: PackedStringArray = requests.get("text_keys", PackedStringArray())
				for i: int in rule_ids.size():
					if rule_ids[i] == "need_food":
						var vid: int = int(voices[i]) if i < voices.size() else -1
						data["voice"] = Names.present(vid, lang) if vid >= 0 else ""
						data["cause"] = _tr(str(text_keys[i])) if i < text_keys.size() else ""
						break
				if not data.has("voice"):
					data["voice"] = ""
					data["cause"] = _tr("LOOP_ALERT_FOOD_LOW")
			"need_warmth":
				data["voice"] = ""
				data["cause"] = _tr("LOOP_ALERT_FIREWOOD_LOW")
			"migrants":
				var neighbours: Dictionary = SimReads.read(_world, "get_neighbours")
				var wave: int = _visitor_id(neighbours, "wave")
				var ids: PackedInt32Array = neighbours.get("wave_ids", PackedInt32Array())
				var sizes: PackedInt32Array = neighbours.get("wave_sizes", PackedInt32Array())
				var index: int = ids.find(wave)
				if wave >= 0:
					data["wave_id"] = wave
					data["count"] = sizes[index] if index >= 0 and index < sizes.size() else 0
			"trader":
				var neighbours: Dictionary = SimReads.read(_world, "get_neighbours")
				var caravan: int = _visitor_id(neighbours, "caravan")
				if caravan >= 0:
					data["caravan_id"] = caravan
			"knowledge_risk":
				if not causes.is_empty():
					var cause: Dictionary = causes[0]
					var knowledge_id: int = int(cause.get("id", -1))
					var node: Dictionary = _world.call("get_tech_node", knowledge_id)
					var holders: PackedInt64Array = node.get("holder_ids", PackedInt64Array())
					var teacher: int = holders[0] if not holders.is_empty() else -1
					data["holder"] = Names.present(teacher, lang) if teacher >= 0 else ""
					data["teacher"] = teacher
					data["technique"] = KnowledgeText.node_name(_locale, knowledge_id)
					data["learner"] = _find_best_learner(teacher, knowledge_id)
					data["knowledge"] = knowledge_id
					# Preserve an actual existing teaching order; new teaching requires a player choice.
					var orders: Dictionary = SimReads.read(_world, "get_teach_orders") if _world.has_method("get_teach_orders") else {}
					var o_ids: Variant = orders.get("ids", PackedInt64Array())
					var o_know: Variant = orders.get("knowledge", PackedInt64Array())
					var o_learn: Variant = orders.get("learners", PackedInt64Array())
					if o_ids is PackedInt64Array and o_know is PackedInt64Array and o_learn is PackedInt64Array:
						for oi: int in (o_ids as PackedInt64Array).size():
							if int((o_know as PackedInt64Array)[oi]) == int(data["knowledge"]):
								data["order_id"] = int((o_ids as PackedInt64Array)[oi])
								data["ordered_learner_id"] = int((o_learn as PackedInt64Array)[oi])
								data["ordered_learner"] = Names.present(int((o_learn as PackedInt64Array)[oi]), lang)
								break
			"wolves":
				data["count"] = causes.size()
			"idle_adults":
				data["count"] = causes.size()
				data["hours"] = 2
	elif kind == "chronicle":
		var chronicle_kind_val: int = int(trigger.get("chronicle_kind", -1))
		if chronicle_kind_val >= 0 and _world != null and _world.has_method("get_chronicle_since"):
			var probe: Dictionary = _world.get_chronicle_since(0, 1)
			var total: int = int(probe.get("total", 0))
			if total > 0:
				var start_idx: int = maxi(0, total - 200)
				var page: Dictionary = _world.get_chronicle_since(start_idx, 200)
				var min_tick: int = maxi(0, _last_tick - 1440)
				for row: Dictionary in ChronicleText.rows(page):
					if int(row.get("kind", -1)) == chronicle_kind_val and int(row.get("tick", 0)) >= min_tick:
						_extract_chronicle_data(card_def, row, data, lang)
						break
	return data


func _extract_chronicle_data(card_def: Dictionary, row: Dictionary, data: Dictionary, lang: String) -> void:
	var card_id: String = str(card_def.get("id", ""))
	match card_id:
		"quarrel":
			var subject: int = int(row.get("subject", -1))
			var other: int = int(row.get("other", -1))
			data["person1"] = Names.present(subject, lang) if subject >= 0 else ""
			data["person2"] = Names.present(other, lang) if other >= 0 else ""
			data["agent_a"] = subject
			data["agent_b"] = other
		"pressure":
			var cause: int = int(row.get("cause", 0))
			var pnames: Array[String] = ["LOOP_PRESSURE_COLD", "LOOP_PRESSURE_DROUGHT", "LOOP_PRESSURE_WOLVES"]
			data["pressure_name"] = _tr(pnames[cause]) if cause >= 0 and cause < pnames.size() else _tr("UI_UNKNOWN")


func _find_best_learner(teacher_id: int, knowledge_id: int) -> int:
	if _world == null or not _world.has_method("get_tech_node"):
		return -1
	var node: Dictionary = _world.call("get_tech_node", knowledge_id)
	var orders: Dictionary = SimReads.read(_world, "get_teach_orders")
	var learners: PackedInt64Array = node.get("learner_ids", PackedInt64Array())
	for learner: int in learners:
		if learner != teacher_id and _learner_available(learner, knowledge_id, node, orders):
			return learner
	return -1


static func _learner_available(learner: int, knowledge: int, node: Dictionary, orders: Dictionary) -> bool:
	var eligible: PackedInt64Array = node.get("learner_ids", PackedInt64Array())
	if not eligible.has(learner):
		return false
	var learners: PackedInt64Array = orders.get("learners", PackedInt64Array())
	var nodes: PackedInt64Array = orders.get("knowledge", PackedInt64Array())
	for i: int in mini(learners.size(), nodes.size()):
		if learners[i] == learner and nodes[i] == knowledge:
			return false
	return true


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
		card_expired.emit(card_id, _card_title(card))
	_cooldown[card_id] = _last_tick
	_active.erase(card_id)
	state_changed.emit()


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
	if command == "open_trade":
		var caravan: int = int(card.get("trigger_data", {}).get("caravan_id", -1))
		var reason: String = ""
		if caravan < 0 or _visitor_id(SimReads.read(_world, "get_neighbours"), "caravan", caravan) != caravan:
			reason = "no_caravan"
		elif not get_parent().has_method("open_card_trade") or not bool(get_parent().call("open_card_trade", caravan, card_id)):
			reason = "trade_panel_unavailable"
		if not reason.is_empty():
			_skipped_results.append({"card_id": card_id, "command": command, "reason": reason})
			card["failure_reason"] = reason
			_rebuild_ui()
			return
	elif command != "none" and command != "steward_default" and command != "enter_zone_tool" and command != "open_roles" and command != "open_build_menu":
		var log_start: int = -1
		if _world != null and _world.has_method("get_command_log"):
			var tail: Dictionary = _world.call("get_command_log", 9223372036854775807)
			log_start = int(tail.get("total", -1))
		var result: Variant = _send_command(command, option, card)
		var result_dict: Dictionary = result if result is Dictionary else {}
		var attempts: int = _record_command_attempts(card_id, command, log_start, result_dict)
		if bool(result_dict.get("skipped", false)):
			# Only pre-send failures count as skipped. Exhausted command retries
			# remain rejected receipts even though the card stays available.
			if attempts == 0:
				_skipped_results.append({
					"card_id": card_id,
					"command": command,
					"reason": str(result_dict.get("reason", "stale")),
				})
			card["failure_reason"] = str(result_dict.get("reason", "stale"))
			_rebuild_ui()
			return
		if not bool(result_dict.get("accepted", false)):
			card["failure_reason"] = str(result_dict.get("reason", "rejected"))
			_rebuild_ui()
			return
	if command == "open_roles":
		roles_requested.emit()
	elif command == "open_build_menu":
		build_requested.emit()
	card["answered"] = true
	_cooldown[card_id] = _last_tick
	card_answered.emit(card_id, _card_title(card), _tr(str(option.get("text_key", ""))))
	_active.erase(card_id)
	state_changed.emit()
	_rebuild_ui()


## Capture the synchronous answer window, including every hunt/draft attempt.
## No simulation ticks execute between these reads, so autonomous orders cannot enter it.
func _record_command_attempts(card_id: String, command: String, start: int, result: Dictionary) -> int:
	var count: int = 0
	if start >= 0 and _world != null:
		var log: Dictionary = _world.call("get_command_log", start)
		var accepted: PackedByteArray = log.get("accepted", PackedByteArray())
		var kinds: PackedStringArray = log.get("kinds", PackedStringArray())
		for i: int in accepted.size():
			_command_results.append({
				"card_id": card_id, "command": command,
				"accepted": accepted[i] != 0,
				"reason": "accepted" if accepted[i] != 0 else "command_log_rejected",
				"dispatch_reason": str(result.get("reason", "")),
				"log_index": start + i,
				"log_kind": kinds[i] if i < kinds.size() else "",
			})
			count += 1
	# Bridge argument validation can reject before a command-log entry exists.
	if count == 0 and not bool(result.get("skipped", false)):
		_command_results.append({
			"card_id": card_id, "command": command,
			"accepted": bool(result.get("accepted", false)),
			"reason": str(result.get("reason", "not_sent")),
		})
		count = 1
	return count


func _send_command(command: String, option: Dictionary, card: Dictionary) -> Variant:
	if _world == null:
		return {"accepted": false, "skipped": true, "reason": "no_world"}
	var camp: Vector2i = _camp_tile()
	var cx: int = camp.x
	var cy: int = camp.y
	var trigger_data: Dictionary = card.get("trigger_data", {})
	var result: Variant = null
	match command:
		"cmd_slaughter_livestock":
			var stock: Dictionary = SimReads.read(_world, "get_livestock")
			var target: int = int(trigger_data.get("livestock_id", -1))
			if target < 0 or int(stock.get("first_kid_id", -2)) != target or not stock.get("ids", PackedInt64Array()).has(target):
				return {"accepted": false, "skipped": true, "reason": "first_kid_gone"}
			result = SimReads.mutate(_world, "cmd_slaughter_livestock", [target])
		"cmd_set_dress_order":
			var requested: bool = bool(option.get("params", true))
			if bool(SimReads.read(_world, "get_story").get("dress_order", true)) == requested:
				return {"accepted": false, "skipped": true, "reason": "nothing"}
			result = SimReads.mutate(_world, command, [requested])
		"cmd_set_field_work":
			var tile := Vector2i(int(trigger_data.get("x", -1)), int(trigger_data.get("y", -1)))
			var current: Dictionary = preload("res://scripts/ui/field_facts.gd").at(_world, tile)
			var mode: String = str(option.get("params", "auto"))
			if current.is_empty() or not bool(current.get("active", false)) or int(current.get("crop", -1)) != int(trigger_data.get("crop", -2)) or str(current.get("mode", "")) == mode:
				return {"accepted": false, "skipped": true, "reason": "nothing"}
			if not preload("res://scripts/ui/field_facts.gd").eligible(current, str(card.get("def", {}).get("id", ""))):
				return {"accepted": false, "skipped": true, "reason": "field_phase_changed"}
			result = SimReads.mutate(_world, command, [tile.x, tile.y, mode])
		"cmd_designate_chop":
			# Dedupe: skip if chop designations already exist.
			if _has_chop_designations():
				return {"accepted": false, "skipped": true, "reason": "chop_exists"}
			if not _world.has_method("get_card_chop_target"):
				return {"accepted": false, "skipped": true, "reason": "no_chop_target_read"}
			var target: Dictionary = _world.call("get_card_chop_target")
			if not bool(target.get("found", false)):
				return {"accepted": false, "skipped": true, "reason": "no_trees"}
			result = SimReads.mutate(_world, "cmd_designate_chop", [int(target["x0"]), int(target["y0"]), int(target["x1"]), int(target["y1"])])
			if result is Dictionary and not bool(result.get("accepted", false)):
				return {"accepted": false, "skipped": false, "reason": str(result.get("reason", "rejected"))}
		"cmd_add_hunt_zone":
			# Dedupe: skip if a hunt zone already exists.
			if _has_hunt_zones():
				return {"accepted": false, "skipped": true, "reason": "zone_exists"}
			# Don't send hunters out when wolves are approaching.
			if _alert_active("wolves_approaching"):
				return {"accepted": false, "skipped": true, "reason": "wolves_near"}
			# Guard: skip if no bladed tools exist (hunters need them).
			if not _has_hunter_blades():
				return {"accepted": false, "skipped": true, "reason": "no_blades"}
			var info: Dictionary = SimReads.read(_world, "get_world_info")
			var width: int = int(info.get("width", 0))
			var height: int = int(info.get("height", 0))
			if width <= 0 or height <= 0:
				return {"accepted": false, "skipped": true, "reason": "no_map_bounds"}
			result = SimReads.mutate(_world, "cmd_add_hunt_zone", [clampi(cx - 6, 0, width - 1), clampi(cy - 6, 0, height - 1), clampi(cx + 6, 0, width - 1), clampi(cy + 6, 0, height - 1)])
		"cmd_designate_harvest":
			if _world.has_method("get_card_harvest_target"):
				var target: Dictionary = _world.get_card_harvest_target()
				if bool(target.get("found", false)):
					result = SimReads.mutate(_world, "cmd_designate_harvest", [int(target["x0"]), int(target["y0"]), int(target["x1"]), int(target["y1"])])
				else:
					return {"accepted": false, "skipped": true, "reason": "no_target"}
		"cmd_accept_migrants", "cmd_decline_migrants":
			var neighbours: Dictionary = SimReads.read(_world, "get_neighbours")
			var wave: int = _visitor_id(neighbours, "wave", int(trigger_data.get("wave_id", -1)))
			if int(trigger_data.get("wave_id", -1)) < 0 or wave < 0:
				return {"accepted": false, "skipped": true, "reason": "already_handled"}
			result = SimReads.mutate(_world, command, [wave])
		"cmd_undraft_all":
			var current: Dictionary = SimReads.read(_world, "get_drafted")
			if (current.get("ids", PackedInt64Array()) as PackedInt64Array).is_empty():
				return {"accepted": false, "skipped": true, "reason": "nobody_drafted"}
			return SimReads.mutate(_world, "cmd_undraft_all", [])
		"cmd_order_teach":
			var teacher: int = int(trigger_data.get("teacher", -1))
			var learner: int = int(trigger_data.get("learner", -1))
			var knowledge: int = int(trigger_data.get("knowledge", -1))
			if teacher < 0 or learner < 0 or knowledge < 0:
				return {"accepted": false, "skipped": true, "reason": "no_target"}
			# Pre-validate: verify teacher and learner still exist.
			if not _agent_exists(teacher) or not _agent_exists(learner):
				return {"accepted": false, "skipped": true, "reason": "agent_gone"}
			# Re-check knowledge_risk alert is still active.
			if not _alert_active("technique_at_risk"):
				return {"accepted": false, "skipped": true, "reason": "risk_cleared"}
			# Check teacher still has a knowledge wait (the technique is still at risk
			# with this specific teacher). Use get_ui_facts waits collection.
			if not _teacher_holds_knowledge(teacher, knowledge):
				return {"accepted": false, "skipped": true, "reason": "no_practitioner"}
			var node: Dictionary = _world.call("get_tech_node", knowledge)
			var orders: Dictionary = SimReads.read(_world, "get_teach_orders")
			if not _learner_available(learner, knowledge, node, orders):
				return {"accepted": false, "skipped": true, "reason": "learner_unavailable"}
			result = SimReads.mutate(_world, "cmd_order_teach", [teacher, learner, knowledge])
			if result is Dictionary and not bool(result.get("accepted", false)):
				return {"accepted": false, "skipped": false, "reason": str(result.get("reason", "rejected"))}
		"cmd_cancel_teach":
			var order_id: int = int(trigger_data.get("order_id", -1))
			var orders: Dictionary = SimReads.read(_world, "get_teach_orders")
			var live_ids: PackedInt64Array = orders.get("ids", PackedInt64Array())
			if order_id < 0 or not live_ids.has(order_id):
				return {"accepted": false, "skipped": true, "reason": "no_order"}
			result = SimReads.mutate(_world, "cmd_cancel_teach", [order_id])
			if result is Dictionary and not bool(result.get("accepted", false)):
				return {"accepted": false, "skipped": false, "reason": str(result.get("reason", "rejected"))}
		"cmd_draft":
			if int(option.get("count", 1)) != 1:
				return {"accepted": false, "skipped": true, "reason": "invalid_count"}
			var drafted: Dictionary = SimReads.read(_world, "get_drafted")
			var drafted_ids: PackedInt64Array = drafted.get("ids", PackedInt64Array())
			var people: Dictionary = SimReads.read(_world, "get_ui_facts").get("people", {})
			var ids: PackedInt64Array = people.get("ids", PackedInt64Array())
			var children: PackedByteArray = people.get("children", PackedByteArray())
			var hps: PackedFloat64Array = people.get("hps", PackedFloat64Array())
			for i: int in mini(ids.size(), mini(children.size(), hps.size())):
				if children[i] == 0 and is_finite(hps[i]) and hps[i] >= float(option.get("min_hp", 50)) and not drafted_ids.has(ids[i]):
					return SimReads.mutate(_world, "cmd_draft", [ids[i], true])
			return {"accepted": false, "skipped": true, "reason": "no_eligible_adult"}
		"cmd_trade":
			var neighbours: Dictionary = SimReads.read(_world, "get_neighbours")
			var caravan: int = _visitor_id(neighbours, "caravan", int(trigger_data.get("caravan_id", -1)))
			if int(trigger_data.get("caravan_id", -1)) < 0 or caravan < 0:
				return {"accepted": false, "skipped": true, "reason": "no_caravan"}
			# Build balanced trade terms from the caravan's actual offer.
			var trade_terms: Dictionary = _build_trade_terms(caravan)
			if not bool(trade_terms.get("valid", false)):
				return {"accepted": false, "skipped": true, "reason": "no_balanced_trade"}
			result = SimReads.mutate(_world, "cmd_trade", [caravan, trade_terms["give"], trade_terms["take"]])
		"cmd_add_bill":
			var recipe: String = str(option.get("recipe", ""))
			var recipe_data: Dictionary = Content.row("recipes", Content.code("recipes", recipe))
			var compatible: PackedInt64Array = recipe_data.get("compatible_workshops", PackedInt64Array())
			var shops: Dictionary = SimReads.read(_world, "get_workshops")
			var ids: PackedInt32Array = shops.get("ids", PackedInt32Array())
			var kinds: PackedInt64Array = shops.get("kinds", PackedInt64Array())
			var workshop: int = -1
			for i: int in mini(ids.size(), kinds.size()):
				if compatible.has(kinds[i]):
					workshop = ids[i]
					break
			if workshop < 0:
				return {"accepted": false, "skipped": true, "reason": "no_workshop"}
			result = SimReads.mutate(_world, "cmd_add_bill", [workshop, recipe, "make", 1])
		"cmd_set_work_priority":
			var agent_a: int = int(trigger_data.get("agent_a", -1))
			if agent_a < 0:
				return {"accepted": false, "skipped": true, "reason": "no_target"}
			if not _agent_is_adult(agent_a):
				return {"accepted": false, "skipped": true, "reason": "not_adult"}
			var priorities: Dictionary = SimReads.read(_world, "get_work_priorities")
			var adults: PackedInt64Array = priorities.get("agent_ids", PackedInt64Array())
			var works: PackedStringArray = priorities.get("work_types", PackedStringArray())
			var levels: PackedByteArray = priorities.get("levels", PackedByteArray())
			var person_row: int = adults.find(agent_a)
			var work_column: int = works.find("chop")
			if person_row < 0 or work_column < 0 or person_row * works.size() + work_column >= levels.size():
				return {"accepted": false, "skipped": true, "reason": "no_priority_target"}
			if levels[person_row * works.size() + work_column] == 3:
				return {"accepted": false, "skipped": true, "reason": "priority_already_set"}
			result = SimReads.mutate(_world, "cmd_set_work_priority", [agent_a, "chop", 3])
			if result is Dictionary and not bool(result.get("accepted", false)):
				return {"accepted": false, "skipped": false, "reason": str(result.get("reason", "rejected"))}
		"cmd_place_blueprint":
			result = SimReads.mutate(_world, "cmd_place_blueprint", ["storehouse", cx + 4, cy + 4])
		"cmd_add_stockpile":
			if not _world.has_method("get_card_stockpile_target"):
				return {"accepted": false, "skipped": true, "reason": "no_stockpile_target_read"}
			var target: Dictionary = _world.call("get_card_stockpile_target")
			if not bool(target.get("found", false)):
				return {"accepted": false, "skipped": true, "reason": "no_stockpile_space"}
			result = SimReads.mutate(_world, "cmd_add_stockpile", [int(target["x0"]), int(target["y0"]), int(target["x1"]), int(target["y1"])])
	# No match arm hit — unknown command.
	if result == null:
		return {"accepted": false, "skipped": true, "reason": "no_action"}
	# A command that was sent and rejected by the bridge is a validation failure, never a skip
	# (LOOP-D-cards); only pre-send checks above may skip.
	if result is Dictionary and not bool(result.get("accepted", false)):
		return {"accepted": false, "skipped": false, "reason": str(result.get("reason", "rejected"))}
	return result


## Whether an agent (person) with the given id is alive in the settlement.
func _agent_exists(agent_id: int) -> bool:
	if _world == null or agent_id < 0:
		return false
	var facts: Dictionary = SimReads.read(_world, "get_ui_facts")
	var people: Variant = facts.get("people", {})
	if people is Dictionary:
		var ids: Variant = (people as Dictionary).get("ids", PackedInt64Array())
		if ids is PackedInt64Array:
			for aid: int in ids as PackedInt64Array:
				if aid == agent_id:
					return true
	return false


## Revalidate both identities: alert causes name techniques, not their practitioners.
func _teacher_holds_knowledge(teacher_id: int, knowledge_id: int) -> bool:
	if _world == null or knowledge_id < 0 or teacher_id < 0:
		return false
	var causes: Array = _alert_causes("technique_at_risk")
	for cause: Variant in causes:
		if cause is Dictionary and int(cause.get("id", -1)) == knowledge_id:
			var node: Dictionary = _world.call("get_tech_node", knowledge_id)
			var holders: PackedInt64Array = node.get("holder_ids", PackedInt64Array())
			return holders.has(teacher_id)
	return false


## Whether an agent is an adult (not a child) and alive.
func _agent_is_adult(agent_id: int) -> bool:
	if _world == null or agent_id < 0:
		return false
	var facts: Dictionary = SimReads.read(_world, "get_ui_facts")
	var people: Variant = facts.get("people", {})
	if not people is Dictionary:
		return false
	var ids: Variant = (people as Dictionary).get("ids", PackedInt64Array())
	var children: Variant = (people as Dictionary).get("children", PackedByteArray())
	if ids is PackedInt64Array and children is PackedByteArray:
		var id_arr: PackedInt64Array = ids as PackedInt64Array
		var ch_arr: PackedByteArray = children as PackedByteArray
		for i: int in id_arr.size():
			if int(id_arr[i]) == agent_id:
				return i < ch_arr.size() and int(ch_arr[i]) == 0
	return false


## Build balanced trade terms from the caravan's actual goods and colony stock.
## Returns {valid: bool, give: PackedInt32Array, take: PackedInt32Array}.
## give = flat (x, y, count) triples; take = (goods index, count) pairs.
func _build_trade_terms(caravan_id: int) -> Dictionary:
	if _world == null or not _world.has_method("get_caravan"):
		return {"valid": false}
	var cv: Dictionary = _world.get_caravan(caravan_id)
	if not bool(cv.get("found", false)):
		return {"valid": false}
	var goods_value: Variant = cv.get("goods_value", PackedInt32Array())
	var goods_count: Variant = cv.get("goods_count", PackedInt32Array())
	if not goods_value is PackedInt32Array or (goods_value as PackedInt32Array).size() == 0:
		return {"valid": false}
	# Pick the first good with count > 0.
	var take_idx: int = -1
	var take_value: int = 0
	var gv: PackedInt32Array = goods_value as PackedInt32Array
	var gc: PackedInt32Array = goods_count as PackedInt32Array
	for i: int in gv.size():
		if i < gc.size() and int(gc[i]) > 0:
			take_idx = i
			take_value = int(gv[i])
			break
	if take_idx < 0 or take_value <= 0:
		return {"valid": false}
	# Find colony stock to give that covers the value.
	var buy_permille: int = int(cv.get("buy_permille", 800))
	var sell_permille: int = int(cv.get("sell_permille", 1200))
	if buy_permille <= 0 or sell_permille <= 0:
		return {"valid": false}
	@warning_ignore("integer_division")
	var needed_value: int = maxi(1, (take_value * sell_permille + buy_permille - 1) / buy_permille)
	var stock_x: Variant = cv.get("stock_x", PackedInt32Array())
	var stock_y: Variant = cv.get("stock_y", PackedInt32Array())
	var stock_count: Variant = cv.get("stock_available_count", PackedInt32Array())
	var stock_value: Variant = cv.get("stock_value", PackedInt32Array())
	if not stock_x is PackedInt32Array or (stock_x as PackedInt32Array).size() == 0:
		return {"valid": false}
	var sx: PackedInt32Array = stock_x as PackedInt32Array
	var sy: PackedInt32Array = stock_y as PackedInt32Array
	var sc: PackedInt32Array = stock_count as PackedInt32Array
	var sv: PackedInt32Array = stock_value as PackedInt32Array
	# Accumulate stock items until we have enough value.
	var give: PackedInt32Array = PackedInt32Array()
	var accumulated: int = 0
	for i: int in sx.size():
		if i >= sy.size() or i >= sc.size() or i >= sv.size():
			break
		if int(sc[i]) <= 0 or int(sv[i]) <= 0:
			continue
		@warning_ignore("integer_division")
		var count: int = mini(int(sc[i]), (needed_value - accumulated + int(sv[i]) - 1) / int(sv[i]))
		give.append(int(sx[i]))
		give.append(int(sy[i]))
		give.append(count)
		accumulated += int(sv[i]) * count
		if accumulated >= needed_value:
			break
	if accumulated < needed_value:
		return {"valid": false}
	var take: PackedInt32Array = PackedInt32Array([take_idx, 1])
	return {"valid": true, "give": give, "take": take}


## Whether any chop designation (kind 0) exists in the colony overlay.
func _has_chop_designations() -> bool:
	if _world == null or not _world.has_method("get_colony_overlay"):
		return false
	var overlay: Dictionary = _world.get_colony_overlay()
	var kinds: Variant = overlay.get("designation_kinds", PackedInt32Array())
	if kinds is PackedInt32Array:
		for k: int in kinds as PackedInt32Array:
			if k == 0:
				return true
	return false


## Whether any hunt zone exists (from the wildlife subsystem).
func _has_hunt_zones() -> bool:
	if _world == null or not _world.has_method("get_animals"):
		return false
	var animals: Dictionary = _world.get_animals()
	var zones: Variant = animals.get("zones", PackedInt32Array())
	return zones is PackedInt32Array and (zones as PackedInt32Array).size() > 0


## Whether an authored hunting weapon or cutter exists for hunters.
## Stacks in get_ui_facts use parallel-array format: {kinds: PackedInt64Array, counts: PackedInt64Array, ...}.
func _has_hunter_blades() -> bool:
	if _world == null:
		return false
	var facts: Dictionary = SimReads.read(_world, "get_ui_facts")
	var stacks: Variant = facts.get("stacks", {})
	if not stacks is Dictionary:
		return false
	var kinds: Variant = (stacks as Dictionary).get("kinds", PackedInt64Array())
	var counts: Variant = (stacks as Dictionary).get("counts", PackedInt64Array())
	if kinds is PackedInt64Array and counts is PackedInt64Array:
		var k_arr: PackedInt64Array = kinds as PackedInt64Array
		var c_arr: PackedInt64Array = counts as PackedInt64Array
		for i: int in k_arr.size():
			if bool(Content.row("items", int(k_arr[i])).get("hunting_tool", false)) and i < c_arr.size() and int(c_arr[i]) > 0:
				return true
	return false


## Recommendation only: current assigned beds, completed roofed capacity, existing
## fuel target and five days at the initial food rate. This is not a forecast of
## cold exposure, spoilage or future food production; explicit acceptance remains available.
func _can_support_migrants(migrant_count: int, wave_id: int = -1) -> bool:
	return _migrant_shortage(migrant_count, wave_id).is_empty()


func _migrant_shortage(migrant_count: int, wave_id: int) -> String:
	var unknown: String = "M6B_MIGRANT_UNKNOWN"
	if _world == null or migrant_count <= 0 or wave_id < 0 or not _world.has_method("get_wave") or not _world.has_method("get_ui_facts"):
		return unknown
	var wave: Dictionary = _world.call("get_wave", wave_id)
	if not bool(wave.get("found", false)) or int(wave.get("id", -1)) != wave_id or int(wave.get("state", -1)) != 1:
		return unknown
	if not wave.get("agents") is PackedInt64Array or (wave["agents"] as PackedInt64Array).size() != migrant_count:
		return unknown
	if not wave.get("food_need_per_day") is float and not wave.get("food_need_per_day") is int:
		return unknown
	var incoming_need: float = float(wave["food_need_per_day"])
	if not is_finite(incoming_need) or incoming_need <= 0.0:
		return unknown
	var facts: Dictionary = SimReads.read(_world, "get_ui_facts")
	if not bool(facts.get("enabled", false)) or not facts.get("people") is Dictionary or not facts.get("settlements") is Dictionary:
		return unknown
	var people: Dictionary = facts["people"]
	for key: String in ["ids", "bed_xs", "bed_ys"]:
		if not people.get(key) is PackedInt64Array:
			return unknown
	var residents: int = people["ids"].size()
	if residents <= 0 or people["bed_xs"].size() != residents or people["bed_ys"].size() != residents:
		return unknown
	for i: int in residents:
		if int(people["bed_xs"][i]) < 0 or int(people["bed_ys"][i]) < 0:
			return "M6B_MIGRANT_BEDS"
	if not wave.get("reachable_roofed_beds") is int:
		return unknown
	var bed_capacity: int = int(wave["reachable_roofed_beds"])
	if bed_capacity < 0:
		return unknown
	if bed_capacity < residents + migrant_count:
		return "M6B_MIGRANT_BEDS"
	var settlements: Dictionary = facts["settlements"]
	for key: String in ["ids", "food", "wood"]:
		if not settlements.get(key) is PackedInt64Array:
			return unknown
	for key: String in ["food_need", "wood_target"]:
		if not settlements.get(key) is PackedFloat64Array:
			return unknown
	var count: int = settlements["ids"].size()
	if count != 1:
		return unknown
	for key: String in ["food", "wood", "food_need", "wood_target"]:
		if settlements[key].size() != count:
			return unknown
	# The native capacity observation is defined only for one settlement.
	if int(settlements["ids"][0]) < 0:
		return unknown
	var destination: int = 0
	var wood: int = int(settlements["wood"][destination])
	var target: float = float(settlements["wood_target"][destination])
	var food: int = int(settlements["food"][destination])
	var current_need: float = float(settlements["food_need"][destination])
	if wood < 0 or food < 0 or not is_finite(target) or target < 0.0 or not is_finite(current_need) or current_need <= 0.0:
		return unknown
	if float(wood) < target:
		return "M6B_MIGRANT_WOOD"
	if float(food) < 5.0 * (current_need + incoming_need):
		return "M6B_MIGRANT_FOOD"
	return ""


## Return the effective recommended option index for a card, overriding the static
## data when conditions warrant a different recommendation.
## Cards whose need the steward already covers when it is on (LOOP-D-cards).
const STEWARD_COVERED_CARDS: Array[String] = ["need_food", "need_warmth", "tools_low", "idle_adults", "wolves"]


func _steward_on() -> bool:
	if _world == null or not _world.has_method("get_steward"):
		return false
	return bool(SimReads.read(_world, "get_steward").get("enabled", false))


## A no-command option that leaves existing stewardship unchanged.
func _steward_option(options: Array, trigger_data: Dictionary = {}) -> int:
	for i: int in options.size():
		var cmd: String = str(options[i].get("command", ""))
		if (cmd == "steward_default" or cmd == "none") and not _option_hidden(options[i], trigger_data):
			return i
	return options.size() - 1


func _effective_recommended(card_def: Dictionary, trigger_data: Dictionary) -> int:
	var card_id: String = str(card_def.get("id", ""))
	var options: Array = card_def.get("options", [])
	# BASE rules cover wood (2), tools and firewood bills (4), food (5/8),
	# and defence (7). Teaching requires an explicit card choice in Stage 1.
	# Idle work can also remain unchanged;
	# opening roles is available but never assigns priorities automatically.
	if _steward_on() and card_id in STEWARD_COVERED_CARDS:
		return _steward_option(options, trigger_data)
	match card_id:
		"cloth_short":
			return 2 if bool(SimReads.read(_world, "get_story").get("dress_order", true)) else 0
		"migrants":
			var migrant_count: int = int(trigger_data.get("count", 0))
			var shortage: String = _migrant_shortage(migrant_count, int(trigger_data.get("wave_id", -1)))
			trigger_data["support_reason"] = shortage
			if not shortage.is_empty():
				# Recommend decline (option 1) instead of accept (option 0).
				for i: int in options.size():
					if str(options[i].get("command", "")) == "cmd_decline_migrants":
						return i
				return 1
		"need_warmth":
			# Recommend chop only if no chop designation pending.
			if _has_chop_designations():
				# Recommend steward (last option).
				return options.size() - 1
		"need_food":
			# LOOP-D-cards: if the steward already covers the need, recommend steward.
			# Check: harvest target unavailable means steward already designated plants
			# or no plants exist — either way, the player's gather adds nothing.
			var harvest_available: bool = false
			if _world != null and _world.has_method("get_card_harvest_target"):
				harvest_available = bool(_world.get_card_harvest_target().get("found", false))
			if not harvest_available:
				# Steward covers food or nothing to gather — recommend steward.
				return options.size() - 1
			# Recommend hunt only if: blades exist, no hunt zone exists, and no
			# wolves on the map (hunting near wolves causes combat deaths).
			var hunt_unsafe: bool = _has_hunt_zones() or not _has_hunter_blades() or _alert_active("wolves_approaching")
			if hunt_unsafe:
				# Fall back to harvest if available, else steward.
				for i: int in options.size():
					var cmd: String = str(options[i].get("command", ""))
					if cmd == "cmd_designate_harvest":
						return i
				return options.size() - 1
		"knowledge_risk":
			# Don't recommend teaching when firewood is critically low — teaching
			# pulls both teacher and learner away from chopping.
			if _alert_active("firewood_low"):
				for i: int in options.size():
					if str(options[i].get("command", "")) == "none" and not _option_hidden(options[i], trigger_data):
						return i
				return options.size() - 1
	# Default: the first visible option with the static recommended flag (data can mark one per state).
	for i: int in options.size():
		if bool(options[i].get("recommended", false)) and not _option_hidden(options[i], trigger_data):
			return i
	return 0


func _rebuild_ui() -> void:
	if _card_list == null:
		return
	for child: Node in _card_list.get_children():
		_card_list.remove_child(child)
		child.queue_free()
	var sorted_ids: Array[String] = []
	sorted_ids.assign(_active.keys())
	sorted_ids.sort_custom(func(a: String, b: String) -> bool:
		return int(_active[a].get("def", {}).get("priority", 99)) < int(_active[b].get("def", {}).get("priority", 99)))
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
	var effective_rec: int = _effective_recommended(def, trigger_data)
	var title_label: Label = Label.new()
	title_label.text = _format_card_text(str(def.get("title_key", "")), trigger_data)
	title_label.add_theme_color_override("font_color", Color("e8c46e"))
	card_box.add_child(title_label)
	var body_label: Label = Label.new()
	var body_key: String = str(def.get("body_ordered_key", "")) if trigger_data.has("order_id") and def.has("body_ordered_key") else str(def.get("body_key", ""))
	var body: String = _format_card_text(body_key, trigger_data)
	if str(def.get("id", "")) == "migrants" and not str(trigger_data.get("support_reason", "")).is_empty():
		body += "\n" + _tr(str(trigger_data["support_reason"]))
	body_label.text = KoreanText.words(body)
	body_label.autowrap_mode = TextServer.AUTOWRAP_WORD
	body_label.custom_minimum_size.x = 340
	card_box.add_child(body_label)
	if card.has("failure_reason"):
		var failure: Label = Label.new()
		failure.name = "FailureReason"
		failure.text = _tr("M6B_CARD_NOT_APPLIED").format({"reason": str(card["failure_reason"])})
		failure.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		card_box.add_child(failure)
	var buttons_row: HFlowContainer = HFlowContainer.new()
	buttons_row.add_theme_constant_override("separation", 8)
	card_box.add_child(buttons_row)
	var options: Array = def.get("options", [])
	for i: int in options.size():
		var opt: Dictionary = options[i]
		if _option_hidden(opt, trigger_data):
			continue
		var btn: Button = Button.new()
		btn.text = _tr(str(opt.get("text_key", "")))
		if i == effective_rec:
			btn.text += " " + _tr("LOOP_CARD_RECOMMENDED")
			btn.add_theme_color_override("font_color", Color("a8d888"))
		btn.pressed.connect(answer_card.bind(card_id, i))
		buttons_row.add_child(btn)


## Hide an option when its required target is unavailable.
func _option_hidden(opt: Dictionary, trigger_data: Dictionary = {}) -> bool:
	var requires: String = str(opt.get("requires", ""))
	match requires:
		"teach_order":
			return not trigger_data.has("order_id")
		"no_teach_order":
			return trigger_data.has("order_id")
		"other_learner":
			return not trigger_data.has("order_id") or int(trigger_data.get("learner", -1)) < 0 \
				or int(trigger_data.get("learner", -1)) == int(trigger_data.get("ordered_learner_id", -1))
		"harvest_target":
			if _world != null and _world.has_method("get_card_harvest_target"):
				return not bool(_world.get_card_harvest_target().get("found", false))
	# Hide hunt zone when wolves are near (seed 6 fix: hunters die to wolves).
	var cmd: String = str(opt.get("command", ""))
	if cmd == "cmd_set_dress_order":
		return bool(SimReads.read(_world, "get_story").get("dress_order", true)) == bool(opt.get("params", true))
	if cmd == "cmd_add_hunt_zone" and _alert_active("wolves_approaching"):
		return true
	return false


func _format_card_text(key: String, data: Dictionary) -> String:
	var text: String = _tr(key)
	var values: Dictionary = data.duplicate()
	var voice: String = str(data.get("voice", "")).strip_edges()
	values["voice_prefix"] = _tr("M6B_CARD_VOICE_PREFIX").format({"voice": voice}) if not voice.is_empty() else ""
	for field: String in values:
		text = text.replace("{" + field + "}", str(values[field]))
	while "{" in text:
		var start: int = text.find("{")
		var end: int = text.find("}", start)
		if end < 0:
			break
		var inner: String = text.substr(start + 1, end - start - 1)
		var is_placeholder: bool = not inner.is_empty()
		for ch: String in inner:
			if ch != "_" and not (ch >= "a" and ch <= "z") and not (ch >= "0" and ch <= "9"):
				is_placeholder = false
				break
		if is_placeholder:
			text = text.left(start) + text.substr(end + 1)
		else:
			break
	return text


func _reset() -> void:
	_active.clear()
	_cooldown.clear()
	_command_results.clear()
	_skipped_results.clear()
	_last_tick = -1
	_rebuild_ui()
	state_changed.emit()


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


func has_active_cards() -> bool:
	for card_id: String in _active:
		if not bool(_active[card_id].get("answered", false)):
			return true
	return false


func active_count() -> int:
	var count: int = 0
	for card_id: String in _active:
		if not bool(_active[card_id].get("answered", false)):
			count += 1
	return count


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


func _current_lang() -> String:
	if _locale != null and "_current_lang" in _locale:
		return str(_locale.get("_current_lang"))
	return "ko"


func command_results() -> Array[Dictionary]:
	return _command_results


func skipped_results() -> Array[Dictionary]:
	return _skipped_results


func _card_title(card: Dictionary) -> String:
	return _format_card_text(str(card.get("def", {}).get("title_key", "")), card.get("trigger_data", {}))



## Read actual camp placement; world_info intentionally contains no camp_x/y.
func _camp_tile() -> Vector2i:
	var facts: Dictionary = SimReads.read(_world, "get_ui_facts")
	var settlements: Dictionary = facts.get("settlements", {})
	var sx: PackedInt64Array = settlements.get("xs", PackedInt64Array())
	var sy: PackedInt64Array = settlements.get("ys", PackedInt64Array())
	if not sx.is_empty() and not sy.is_empty():
		return Vector2i(sx[0], sy[0])
	var buildings: Dictionary = SimReads.read(_world, "get_building_snapshot")
	var kinds: PackedByteArray = buildings.get("kinds", PackedByteArray())
	var bx: PackedInt32Array = buildings.get("xs", PackedInt32Array())
	var by: PackedInt32Array = buildings.get("ys", PackedInt32Array())
	for i: int in mini(kinds.size(), mini(bx.size(), by.size())):
		if kinds[i] == 0:
			return Vector2i(bx[i], by[i])
	var agents: Dictionary = SimReads.read(_world, "get_agent_snapshot")
	var xs: PackedInt32Array = agents.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = agents.get("ys", PackedInt32Array())
	if not xs.is_empty() and xs.size() == ys.size():
		var low: Vector2i = Vector2i(xs[0], ys[0])
		var high: Vector2i = low
		for i: int in xs.size():
			low = Vector2i(mini(low.x, xs[i]), mini(low.y, ys[i]))
			high = Vector2i(maxi(high.x, xs[i]), maxi(high.y, ys[i]))
		return Vector2i((Vector2(low) + Vector2(high)) * 0.5)
	var info: Dictionary = SimReads.read(_world, "get_world_info")
	return Vector2i(Vector2(int(info.get("width", 0)), int(info.get("height", 0))) * 0.5)



## The neighbour table retains departed/answered rows; only state 1 is actionable.
static func _visitor_id(neighbours: Dictionary, prefix: String, captured_id: int = -1) -> int:
	var ids: PackedInt32Array = neighbours.get(prefix + "_ids", PackedInt32Array())
	var states: PackedInt32Array = neighbours.get(prefix + "_states", PackedInt32Array())
	for i: int in mini(ids.size(), states.size()):
		if states[i] == 1 and (captured_id < 0 or ids[i] == captured_id):
			return ids[i]
	return -1
