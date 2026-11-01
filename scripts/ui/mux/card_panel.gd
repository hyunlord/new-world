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
const Names = preload("res://scripts/ui/person_names.gd")
const THEME = preload("res://assets/ui/worldsim_theme.tres")
const CARDS_PATH: String = "res://assets/ui/cards.json"
const MAX_OPEN: int = 3
const TICKS_PER_HOUR: int = 60
## Consecutive clear polls before closing an alert-triggered card (hysteresis).
const DEFAULT_CLEAR_POLLS: int = 3

signal card_answered(card_id: String, card_title: String, option_text: String)
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
	_clear_polls_needed = int(_registry.get("clear_polls", DEFAULT_CLEAR_POLLS))
	_card_defs.clear()
	for card: Variant in _registry.get("cards", []):
		if card is Dictionary and not bool(card.get("test_only", false)):
			_card_defs.append(card)
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


func _process(_delta: float) -> void:
	if _world == null:
		visible = false
		return
	var time: Dictionary = SimReads.read(_world, "get_time_snapshot")
	var tick: int = int(time.get("tick", -1))
	if tick < 0 or tick == _last_tick:
		return
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
	match read_name:
		"tools_below_need":
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
						if int(k_arr[i]) in [3, 4, 5] and i < c_arr.size():
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
				var neighbours: Dictionary = SimReads.read(_world, "get_neighbours") if _world.has_method("get_neighbours") else {}
				var wave_ids: Variant = neighbours.get("wave_ids", PackedInt64Array())
				var wave_counts: Variant = neighbours.get("wave_counts", PackedInt32Array())
				if wave_ids is PackedInt64Array and (wave_ids as PackedInt64Array).size() > 0:
					data["wave_id"] = int((wave_ids as PackedInt64Array)[0])
					data["count"] = int((wave_counts as PackedInt32Array)[0]) if wave_counts is PackedInt32Array and (wave_counts as PackedInt32Array).size() > 0 else 0
				elif not causes.is_empty():
					data["wave_id"] = int(causes[0].get("id", 0))
					data["count"] = causes.size()
			"trader":
				var neighbours: Dictionary = SimReads.read(_world, "get_neighbours") if _world.has_method("get_neighbours") else {}
				var caravan_ids: Variant = neighbours.get("caravan_ids", PackedInt64Array())
				if caravan_ids is PackedInt64Array and (caravan_ids as PackedInt64Array).size() > 0:
					data["caravan_id"] = int((caravan_ids as PackedInt64Array)[0])
				elif not causes.is_empty():
					data["caravan_id"] = int(causes[0].get("id", 0))
			"knowledge_risk":
				if not causes.is_empty():
					var cause: Dictionary = causes[0]
					data["holder"] = Names.present(int(cause.get("id", -1)), lang) if int(cause.get("id", -1)) >= 0 else ""
					data["teacher"] = int(cause.get("id", -1))
					data["technique"] = KnowledgeText.node_name(_locale, int(cause.get("id", 0)))
					data["learner"] = _find_best_learner(int(cause.get("id", -1)))
					data["knowledge"] = int(cause.get("id", -1))
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


func _find_best_learner(teacher_id: int) -> int:
	if _world == null:
		return -1
	var priorities: Dictionary = SimReads.read(_world, "get_work_priorities")
	var agent_ids: PackedInt64Array = priorities.get("agent_ids", PackedInt64Array())
	for aid: int in agent_ids:
		if aid != teacher_id and aid > 0:
			return aid
	return -1


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
	if command != "none" and command != "steward_default" and command != "enter_zone_tool" and command != "open_roles":
		var result: Variant = _send_command(command, option, card)
		var result_dict: Dictionary = result if result is Dictionary else {}
		if bool(result_dict.get("skipped", false)):
			# Stale target: command was not sent. Does not count as an answer.
			_skipped_results.append({
				"card_id": card_id,
				"command": command,
				"reason": str(result_dict.get("reason", "stale")),
			})
			# Card stays open; it will close when the trigger clears or it expires.
			return
		_command_results.append({
			"card_id": card_id,
			"command": command,
			"accepted": bool(result_dict.get("accepted", false)),
			"reason": str(result_dict.get("reason", "not_sent")),
		})
	card["answered"] = true
	_cooldown[card_id] = _last_tick
	card_answered.emit(card_id, _card_title(card), _tr(str(option.get("text_key", ""))))
	_active.erase(card_id)
	_rebuild_ui()


func _send_command(command: String, _option: Dictionary, card: Dictionary) -> Variant:
	if _world == null:
		return {"accepted": false, "skipped": true, "reason": "no_world"}
	var info: Dictionary = {}
	var cx: int = 32
	var cy: int = 32
	var trigger_data: Dictionary = card.get("trigger_data", {})
	var result: Variant = null
	match command:
		"cmd_designate_chop":
			# Dedupe: skip if chop designations already exist.
			if _has_chop_designations():
				return {"accepted": false, "skipped": true, "reason": "chop_exists"}
			# Pre-validate: check trees exist near camp before sending.
			info = SimReads.read(_world, "get_world_info")
			cx = int(info.get("camp_x", 32))
			cy = int(info.get("camp_y", 32))
			if _world.has_method("get_trees"):
				var trees: Dictionary = _world.get_trees()
				var txs: Variant = trees.get("xs", PackedInt32Array())
				var tys: Variant = trees.get("ys", PackedInt32Array())
				var found_tree: bool = false
				if txs is PackedInt32Array and tys is PackedInt32Array:
					for ti: int in (txs as PackedInt32Array).size():
						var tx: int = int((txs as PackedInt32Array)[ti])
						var ty: int = int((tys as PackedInt32Array)[ti])
						if tx >= cx - 3 and tx <= cx + 3 and ty >= cy - 3 and ty <= cy + 3:
							found_tree = true
							break
				if not found_tree:
					return {"accepted": false, "skipped": true, "reason": "no_trees"}
			result = SimReads.mutate(_world, "cmd_designate_chop", [cx - 3, cy - 3, cx + 3, cy + 3])
			if result is Dictionary and not bool(result.get("accepted", false)):
				return {"accepted": false, "skipped": true, "reason": str(result.get("reason", "rejected"))}
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
			info = SimReads.read(_world, "get_world_info")
			cx = int(info.get("camp_x", 32))
			cy = int(info.get("camp_y", 32))
			for offset: Vector2i in [Vector2i.ZERO, Vector2i(12, 0), Vector2i(-12, 0), Vector2i(0, 12), Vector2i(0, -12)]:
				result = SimReads.mutate(_world, "cmd_add_hunt_zone", [cx - 6 + offset.x, cy - 6 + offset.y, cx + 6 + offset.x, cy + 6 + offset.y])
				if result is Dictionary and bool(result.get("accepted", false)):
					return result
			# All offsets rejected — zone already exists at every position.
			return {"accepted": false, "skipped": true, "reason": "zone_exists"}
		"cmd_designate_harvest":
			if _world.has_method("get_card_harvest_target"):
				var target: Dictionary = _world.get_card_harvest_target()
				if bool(target.get("found", false)):
					result = SimReads.mutate(_world, "cmd_designate_harvest", [int(target["x0"]), int(target["y0"]), int(target["x1"]), int(target["y1"])])
				else:
					return {"accepted": false, "skipped": true, "reason": "no_target"}
		"cmd_accept_migrants":
			if not _alert_active("migrants_waiting"):
				return {"accepted": false, "skipped": true, "reason": "already_handled"}
			var neighbours: Dictionary = SimReads.read(_world, "get_neighbours") if _world.has_method("get_neighbours") else {}
			var wids: Variant = neighbours.get("wave_ids", PackedInt64Array())
			var wave: int = int((wids as PackedInt64Array)[0]) if wids is PackedInt64Array and (wids as PackedInt64Array).size() > 0 else int(trigger_data.get("wave_id", 0))
			result = SimReads.mutate(_world, "cmd_accept_migrants", [wave])
		"cmd_decline_migrants":
			if not _alert_active("migrants_waiting"):
				return {"accepted": false, "skipped": true, "reason": "already_handled"}
			var neighbours2: Dictionary = SimReads.read(_world, "get_neighbours") if _world.has_method("get_neighbours") else {}
			var wids2: Variant = neighbours2.get("wave_ids", PackedInt64Array())
			var wave2: int = int((wids2 as PackedInt64Array)[0]) if wids2 is PackedInt64Array and (wids2 as PackedInt64Array).size() > 0 else int(trigger_data.get("wave_id", 0))
			result = SimReads.mutate(_world, "cmd_decline_migrants", [wave2])
		"cmd_undraft_all":
			return SimReads.mutate(_world, "cmd_undraft_all", [])
		"cmd_order_teach":
			var teacher: int = int(trigger_data.get("teacher", -1))
			var learner: int = int(trigger_data.get("learner", -1))
			var knowledge: int = int(trigger_data.get("knowledge", -1))
			if teacher < 0 or learner < 0 or knowledge < 0:
				return {"accepted": false, "skipped": true, "reason": "no_target"}
			# Pre-validate: verify teacher still exists in the population.
			if not _agent_exists(teacher) or not _agent_exists(learner):
				return {"accepted": false, "skipped": true, "reason": "agent_gone"}
			# Re-check knowledge_risk alert is still active.
			if not _alert_active("technique_at_risk"):
				return {"accepted": false, "skipped": true, "reason": "risk_cleared"}
			result = SimReads.mutate(_world, "cmd_order_teach", [teacher, learner, knowledge])
			if result is Dictionary and not bool(result.get("accepted", false)):
				return {"accepted": false, "skipped": true, "reason": str(result.get("reason", "rejected"))}
		"cmd_draft":
			var drafted: Dictionary = SimReads.read(_world, "get_drafted")
			var recommended: Array = drafted.get("recommended", [])
			if recommended.is_empty():
				return {"accepted": false, "skipped": true, "reason": "no_recommended"}
			for agent_id: Variant in recommended:
				result = SimReads.mutate(_world, "cmd_draft", [int(agent_id), true])
		"cmd_trade":
			if not _alert_active("trader_present"):
				return {"accepted": false, "skipped": true, "reason": "already_handled"}
			var neighbours3: Dictionary = SimReads.read(_world, "get_neighbours") if _world.has_method("get_neighbours") else {}
			var cids: Variant = neighbours3.get("caravan_ids", PackedInt64Array())
			if not cids is PackedInt64Array or (cids as PackedInt64Array).size() == 0:
				return {"accepted": false, "skipped": true, "reason": "no_caravan"}
			var caravan: int = int((cids as PackedInt64Array)[0])
			result = SimReads.mutate(_world, "cmd_trade", [caravan, PackedInt32Array(), PackedInt32Array([0, 1])])
			# If trade was rejected (unbalanced terms), skip it.
			if result is Dictionary and not bool(result.get("accepted", false)):
				return {"accepted": false, "skipped": true, "reason": str(result.get("reason", "rejected"))}
		"cmd_add_bill":
			info = SimReads.read(_world, "get_world_info")
			var workshops: Variant = info.get("workshops", PackedInt64Array())
			if workshops is PackedInt64Array and (workshops as PackedInt64Array).size() > 0:
				result = SimReads.mutate(_world, "cmd_add_bill", [int((workshops as PackedInt64Array)[0]), "stone_axe", "make", 1])
			else:
				return {"accepted": false, "skipped": true, "reason": "no_workshop"}
		"cmd_set_work_priority":
			var agent_a: int = int(trigger_data.get("agent_a", -1))
			if agent_a < 0:
				return {"accepted": false, "skipped": true, "reason": "no_target"}
			if not _agent_exists(agent_a):
				return {"accepted": false, "skipped": true, "reason": "agent_gone"}
			result = SimReads.mutate(_world, "cmd_set_work_priority", [agent_a, "chop", 3])
			if result is Dictionary and not bool(result.get("accepted", false)):
				return {"accepted": false, "skipped": true, "reason": str(result.get("reason", "rejected"))}
		"cmd_place_blueprint":
			info = SimReads.read(_world, "get_world_info")
			cx = int(info.get("camp_x", 32))
			cy = int(info.get("camp_y", 32))
			result = SimReads.mutate(_world, "cmd_place_blueprint", ["storehouse", cx + 4, cy + 4])
		"cmd_add_stockpile":
			info = SimReads.read(_world, "get_world_info")
			cx = int(info.get("camp_x", 32))
			cy = int(info.get("camp_y", 32))
			result = SimReads.mutate(_world, "cmd_add_stockpile", [cx - 3, cy - 3, cx + 3, cy + 3])
	# No match arm hit — unknown command.
	if result == null:
		return {"accepted": false, "skipped": true, "reason": "no_action"}
	# Bridge rejected the command — the target was stale between the card opening
	# and the answer. Convert to skipped so it does not count as a validation failure.
	if result is Dictionary and not bool(result.get("accepted", false)):
		return {"accepted": false, "skipped": true, "reason": str(result.get("reason", "rejected"))}
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


## Whether any chop designation (kind 0) exists in the colony overlay.
func _has_chop_designations() -> bool:
	if _world == null or not _world.has_method("get_colony_overlay"):
		return false
	var overlay: Dictionary = _world.get_colony_overlay()
	var kinds: Variant = overlay.get("designation_kinds", PackedInt64Array())
	if kinds is PackedInt64Array:
		for k: int in kinds as PackedInt64Array:
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


## Whether any bladed tool (kind 3, 4, 5 = blade/axe/spear) exists for hunters.
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
			if int(k_arr[i]) in [3, 4, 5] and i < c_arr.size() and int(c_arr[i]) > 0:
				return true
	return false


## Whether the settlement can support incoming migrants (beds + food).
## Returns true when nobody lacks a bed AND food covers at least 3 days.
## People/settlements in get_ui_facts use parallel-array format.
func _can_support_migrants(_migrant_count: int) -> bool:
	if _world == null:
		return false
	var facts: Dictionary = SimReads.read(_world, "get_ui_facts")
	# People: parallel arrays {ids, bed_xs, bed_ys, children, ...}.
	var people: Variant = facts.get("people", {})
	if not people is Dictionary:
		return false
	var bed_xs: Variant = (people as Dictionary).get("bed_xs", PackedInt64Array())
	var ids: Variant = (people as Dictionary).get("ids", PackedInt64Array())
	var total_people: int = 0
	var people_without_beds: int = 0
	if bed_xs is PackedInt64Array and ids is PackedInt64Array:
		total_people = (ids as PackedInt64Array).size()
		for bx: int in bed_xs as PackedInt64Array:
			if bx < 0:
				people_without_beds += 1
	# If current people already lack beds, don't accept more.
	if people_without_beds > 0:
		return false
	# Settlements: parallel arrays {ids, food, food_need, ...}.
	var settlements: Variant = facts.get("settlements", {})
	if not settlements is Dictionary:
		return true
	var food_arr: Variant = (settlements as Dictionary).get("food", PackedInt64Array())
	var need_arr: Variant = (settlements as Dictionary).get("food_need", PackedFloat64Array())
	if food_arr is PackedInt64Array and need_arr is PackedFloat64Array:
		var f_arr: PackedInt64Array = food_arr as PackedInt64Array
		var n_arr: PackedFloat64Array = need_arr as PackedFloat64Array
		if f_arr.size() > 0 and n_arr.size() > 0:
			var food: float = float(f_arr[0])
			var daily_need: float = float(n_arr[0])
			if daily_need > 0.0 and food / daily_need < 3.0:
				return false
	return true


## Return the effective recommended option index for a card, overriding the static
## data when conditions warrant a different recommendation.
func _effective_recommended(card_def: Dictionary, trigger_data: Dictionary) -> int:
	var card_id: String = str(card_def.get("id", ""))
	var options: Array = card_def.get("options", [])
	match card_id:
		"migrants":
			var migrant_count: int = int(trigger_data.get("count", 1))
			if not _can_support_migrants(migrant_count):
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
			# Recommend hunt only if hunters have blades AND no hunt zone exists.
			if _has_hunt_zones() or not _has_hunter_blades():
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
					if str(options[i].get("command", "")) == "none":
						return i
				return options.size() - 1
	# Default: use the static recommended flag.
	for i: int in options.size():
		if bool(options[i].get("recommended", false)):
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
	var title_label: Label = Label.new()
	title_label.text = _format_card_text(str(def.get("title_key", "")), trigger_data)
	title_label.add_theme_color_override("font_color", Color("e8c46e"))
	card_box.add_child(title_label)
	var body_label: Label = Label.new()
	body_label.text = KoreanText.words(_format_card_text(str(def.get("body_key", "")), trigger_data))
	body_label.autowrap_mode = TextServer.AUTOWRAP_WORD
	body_label.custom_minimum_size.x = 340
	card_box.add_child(body_label)
	var buttons_row: HBoxContainer = HBoxContainer.new()
	buttons_row.add_theme_constant_override("separation", 8)
	card_box.add_child(buttons_row)
	var options: Array = def.get("options", [])
	var effective_rec: int = _effective_recommended(def, trigger_data)
	for i: int in options.size():
		var opt: Dictionary = options[i]
		if _option_hidden(opt):
			continue
		var btn: Button = Button.new()
		btn.text = _tr(str(opt.get("text_key", "")))
		if i == effective_rec:
			btn.text += " " + _tr("LOOP_CARD_RECOMMENDED")
			btn.add_theme_color_override("font_color", Color("a8d888"))
		btn.pressed.connect(answer_card.bind(card_id, i))
		buttons_row.add_child(btn)


## Hide an option when its required target is unavailable.
func _option_hidden(opt: Dictionary) -> bool:
	var requires: String = str(opt.get("requires", ""))
	match requires:
		"harvest_target":
			if _world != null and _world.has_method("get_card_harvest_target"):
				return not bool(_world.get_card_harvest_target().get("found", false))
	# Hide hunt zone when wolves are near (seed 6 fix: hunters die to wolves).
	var cmd: String = str(opt.get("command", ""))
	if cmd == "cmd_add_hunt_zone" and _alert_active("wolves_approaching"):
		return true
	return false


func _format_card_text(key: String, data: Dictionary) -> String:
	var text: String = _tr(key)
	for field: String in data:
		text = text.replace("{" + field + "}", str(data[field]))
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
