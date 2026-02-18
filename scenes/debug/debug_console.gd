extends CanvasLayer

# NO class_name — headless compatibility

const TraitSystem = preload("res://scripts/systems/trait_system.gd")
const MAX_HISTORY: int = 50
const MAX_OUTPUT_LINES: int = 200
const TICKS_PER_YEAR: int = 4380  # 365 * 12

var _command_history: Array = []
var _history_index: int = -1
var _commands: Dictionary = {}
var _output_lines: int = 0

# Injected by main.gd BEFORE _ready
var _entity_manager = null
var _stress_system = null
var _mental_break_system = null
var _trauma_scar_system = null
var _trait_violation_system = null
var _sim_engine = null

# UI nodes (built programmatically)
var _output: RichTextLabel = null
var _input_field: LineEdit = null
var _panel: PanelContainer = null


func _ready() -> void:
	if not OS.is_debug_build():
		queue_free()
		return
	layer = 100
	visible = false
	_build_ui()
	_register_all_commands()


func _build_ui() -> void:
	_panel = PanelContainer.new()
	_panel.anchor_left = 0.0
	_panel.anchor_top = 0.6
	_panel.anchor_right = 1.0
	_panel.anchor_bottom = 1.0
	_panel.offset_left = 0.0
	_panel.offset_top = 0.0
	_panel.offset_right = 0.0
	_panel.offset_bottom = 0.0

	var style: StyleBoxFlat = StyleBoxFlat.new()
	style.bg_color = Color(0, 0, 0, 0.8)
	_panel.add_theme_stylebox_override("panel", style)
	add_child(_panel)

	var root: VBoxContainer = VBoxContainer.new()
	root.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	root.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_panel.add_child(root)

	_output = RichTextLabel.new()
	_output.bbcode_enabled = true
	_output.scroll_following = true
	_output.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_output.size_flags_vertical = Control.SIZE_EXPAND_FILL
	root.add_child(_output)

	var input_row: HBoxContainer = HBoxContainer.new()
	input_row.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	root.add_child(input_row)

	var prompt: Label = Label.new()
	prompt.text = ">"
	input_row.add_child(prompt)

	_input_field = LineEdit.new()
	_input_field.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_input_field.text_submitted.connect(_on_input_submitted)
	input_row.add_child(_input_field)


func _input(event: InputEvent) -> void:
	if not (event is InputEventKey):
		return
	var key_event: InputEventKey = event
	if not key_event.pressed or key_event.echo:
		return

	if key_event.keycode == KEY_F12:
		visible = not visible
		if visible and _input_field != null:
			_input_field.grab_focus()
		get_viewport().set_input_as_handled()
		return

	if not visible:
		return

	if key_event.keycode == KEY_UP:
		if _command_history.is_empty():
			return
		if _history_index < _command_history.size() - 1:
			_history_index += 1
		var idx_up: int = _command_history.size() - 1 - _history_index
		_input_field.text = str(_command_history[idx_up])
		_input_field.caret_column = _input_field.text.length()
		get_viewport().set_input_as_handled()
		return

	if key_event.keycode == KEY_DOWN:
		if _command_history.is_empty():
			return
		if _history_index > 0:
			_history_index -= 1
			var idx_down: int = _command_history.size() - 1 - _history_index
			_input_field.text = str(_command_history[idx_down])
		else:
			_history_index = -1
			_input_field.text = ""
		_input_field.caret_column = _input_field.text.length()
		get_viewport().set_input_as_handled()
		return

	if key_event.keycode == KEY_TAB:
		var current: String = _input_field.text.strip_edges()
		if current.is_empty():
			return
		var keys: Array = _commands.keys()
		keys.sort()
		for i in range(keys.size()):
			var cmd_name: String = str(keys[i])
			if cmd_name.begins_with(current):
				_input_field.text = cmd_name + " "
				_input_field.caret_column = _input_field.text.length()
				break
		get_viewport().set_input_as_handled()


func _on_input_submitted(raw: String) -> void:
	var trimmed: String = raw.strip_edges()
	if not trimmed.is_empty():
		_command_history.append(trimmed)
		if _command_history.size() > MAX_HISTORY:
			_command_history.remove_at(0)
	_history_index = -1
	if _input_field != null:
		_input_field.clear()
	print_output("> " + raw, Color(0.7, 0.7, 0.7))
	execute(trimmed)


func execute(raw: String) -> void:
	var parts: Array = raw.strip_edges().split(" ", false)
	if parts.is_empty():
		return
	var cmd: String = str(parts[0]).to_lower()
	var args: Dictionary = _parse_args(raw)
	if not _commands.has(cmd):
		print_output("Unknown command: " + cmd + ". Type 'help'", Color.RED)
		return
	var callable: Callable = _commands[cmd]
	callable.call(args)


func _parse_args(raw: String) -> Dictionary:
	var result: Dictionary = {}
	var parts: Array = raw.strip_edges().split(" ", false)
	var positional: Array = []
	for i in range(1, parts.size()):
		var part: String = str(parts[i])
		if part.contains(":"):
			var kv: Array = part.split(":", true, 1)
			var key: String = str(kv[0])
			var value: String = ""
			if kv.size() > 1:
				value = str(kv[1])
			result[key] = value
		elif not part.is_empty():
			result[part] = ""
			positional.append(part)
	if not positional.is_empty():
		result["_pos"] = positional
	return result


func print_output(text: String, color: Color = Color.WHITE) -> void:
	if _output == null:
		return
	if _output_lines > MAX_OUTPUT_LINES:
		_output.clear()
		_output_lines = 0
	_output.append_text("[color=#%s]%s[/color]\n" % [color.to_html(false), text])
	_output_lines += 1
	_output.scroll_to_line(_output.get_line_count())


func _register_all_commands() -> void:
	_commands = {
		"stress": Callable(self, "_cmd_stress"),
		"event": Callable(self, "_cmd_event"),
		"violation": Callable(self, "_cmd_violation"),
		"scar": Callable(self, "_cmd_scar"),
		"emotion": Callable(self, "_cmd_emotion"),
		"trait": Callable(self, "_cmd_trait"),
		"entity": Callable(self, "_cmd_entity"),
		"time": Callable(self, "_cmd_time"),
		"log": Callable(self, "_cmd_log"),
		"help": Callable(self, "_cmd_help"),
	}


func _get_entity(id_str: String):
	if _entity_manager == null:
		print_output("entity_manager not connected", Color.RED)
		return null
	var id_int: int = id_str.to_int()
	var entity = _entity_manager.get_entity(id_int)
	if entity == null:
		print_output("Entity not found: " + id_str, Color.RED)
		return null
	return entity


func _cmd_stress(args: Dictionary) -> void:
	var entity_id = str(args.get("entity", ""))
	if entity_id.is_empty():
		print_output("Usage: stress entity:<id> set:<v>|add:<v>|break|reset", Color.RED)
		return
	var entity = _get_entity(entity_id)
	if entity == null:
		return
	if entity.emotion_data == null:
		print_output("Entity has no emotion_data: " + entity.entity_name, Color.RED)
		return

	if args.has("set"):
		var set_value: float = float(str(args.get("set", "0")))
		entity.emotion_data.stress = set_value
		print_output("Set %s stress -> %.1f" % [entity.entity_name, entity.emotion_data.stress])
		return

	if args.has("add"):
		var add_value: float = float(str(args.get("add", "0")))
		entity.emotion_data.stress = clampf(entity.emotion_data.stress + add_value, 0.0, 9999.0)
		print_output("%s stress +%.1f = %.1f" % [entity.entity_name, add_value, entity.emotion_data.stress])
		return

	if args.has("break"):
		if _mental_break_system == null or _sim_engine == null:
			print_output("mental_break_system or sim_engine not connected", Color.RED)
			return
		_mental_break_system.force_break(entity, _sim_engine.current_tick)
		print_output("Forced break on %s" % entity.entity_name)
		return

	if args.has("reset"):
		entity.emotion_data.stress = 0.0
		print_output("Reset %s stress to 0" % entity.entity_name)
		return

	print_output("Usage: stress entity:<id> set:<v>|add:<v>|break|reset", Color.RED)


func _cmd_event(args: Dictionary) -> void:
	if _stress_system == null:
		print_output("stress_system not connected", Color.RED)
		return

	var entity = _get_entity(str(args.get("entity", "")))
	if entity == null:
		return

	var event_id: String = str(args.get("id", ""))
	if event_id.is_empty():
		print_output("Usage: event entity:<id> id:<event_id> [context:key=val]", Color.RED)
		return

	var context: Dictionary = {}
	var context_raw: String = str(args.get("context", ""))
	if not context_raw.is_empty():
		var pairs: Array = context_raw.split(",", false)
		for i in range(pairs.size()):
			var pair: String = str(pairs[i]).strip_edges()
			if not pair.contains("="):
				continue
			var kv: Array = pair.split("=", true, 1)
			var ckey: String = str(kv[0])
			var cval: String = ""
			if kv.size() > 1:
				cval = str(kv[1])
			var lower: String = cval.to_lower()
			if lower == "true":
				context[ckey] = true
			elif lower == "false":
				context[ckey] = false
			elif cval.is_valid_int():
				context[ckey] = int(cval)
			elif cval.is_valid_float():
				context[ckey] = float(cval)
			else:
				context[ckey] = cval

	_stress_system.inject_event(entity, event_id, context)
	print_output("Injected event %s -> %s" % [event_id, entity.entity_name])


func _cmd_violation(args: Dictionary) -> void:
	if _trait_violation_system == null:
		print_output("trait_violation_system not connected", Color.RED)
		return
	if _sim_engine == null:
		print_output("sim_engine not connected", Color.RED)
		return

	var entity = _get_entity(str(args.get("entity", "")))
	if entity == null:
		return

	TraitSystem.update_trait_strengths(entity)  # populate trait_strengths before violation check

	var action_id: String = str(args.get("action", ""))
	if action_id.is_empty():
		print_output("Usage: violation entity:<id> action:<id> [witness:<rel>] [victim:<rel>] [count:<n>]", Color.RED)
		return

	var context: Dictionary = {"tick": _sim_engine.current_tick}
	if args.has("witness"):
		context["witness_relationship"] = str(args.get("witness", "none"))
	if args.has("victim"):
		context["victim_relationship"] = str(args.get("victim", "stranger"))
	if args.has("forced"):
		context["forced_by_authority"] = true

	var count: int = int(str(args.get("count", "1")))
	if count < 1:
		count = 1

	for i in range(count):
		_trait_violation_system.on_action_performed(entity, action_id, context)

	print_output("Forced violation %s x %d on %s" % [action_id, count, entity.entity_name])


func _cmd_scar(args: Dictionary) -> void:
	var entity = _get_entity(str(args.get("entity", "")))
	if entity == null:
		return

	if args.has("add"):
		if _trauma_scar_system == null or _sim_engine == null:
			print_output("trauma_scar_system or sim_engine not connected", Color.RED)
			return
		var scar_id: String = str(args.get("add", ""))
		if scar_id.is_empty():
			print_output("Usage: scar entity:<id> add:<id>", Color.RED)
			return
		_trauma_scar_system.try_acquire_scar(entity, scar_id, 1.0, _sim_engine.current_tick)
		print_output("Added scar %s to %s" % [scar_id, entity.entity_name])
		return

	if args.has("remove"):
		var remove_id: String = str(args.get("remove", ""))
		var removed: int = 0
		for i in range(entity.trauma_scars.size() - 1, -1, -1):
			var scar_entry = entity.trauma_scars[i]
			if str(scar_entry.get("scar_id", "")) == remove_id:
				entity.trauma_scars.remove_at(i)
				removed += 1
		print_output("Removed scar %s from %s (%d)" % [remove_id, entity.entity_name, removed])
		return

	if args.has("list"):
		if entity.trauma_scars.is_empty():
			print_output("%s has no trauma scars" % entity.entity_name)
			return
		for i in range(entity.trauma_scars.size()):
			var entry = entity.trauma_scars[i]
			print_output("%s stacks:%d" % [str(entry.get("scar_id", "?")), int(entry.get("stacks", 1))])
		return

	if args.has("clear"):
		entity.trauma_scars.clear()
		print_output("Cleared all scars from %s" % entity.entity_name)
		return

	print_output("Usage: scar entity:<id> add:<id>|remove:<id>|list|clear", Color.RED)


func _cmd_emotion(args: Dictionary) -> void:
	var entity = _get_entity(str(args.get("entity", "")))
	if entity == null:
		return
	var ed = entity.emotion_data
	if ed == null:
		print_output("Entity has no emotion_data: " + entity.entity_name, Color.RED)
		return

	if args.has("set"):
		var set_spec: String = str(args.get("set", ""))
		var sep: int = set_spec.rfind(":")
		if sep <= 0:
			print_output("Usage: emotion entity:<id> set:<name>:<v>", Color.RED)
			return
		var emotion_name: String = set_spec.substr(0, sep)
		var value: float = clampf(float(set_spec.substr(sep + 1)), 0.0, 10.0)

		if ed.fast.has(emotion_name):
			ed.fast[emotion_name] = value
			print_output("Set %s fast:%s = %.2f" % [entity.entity_name, emotion_name, value])
			return
		if ed.slow.has(emotion_name):
			ed.slow[emotion_name] = value
			print_output("Set %s slow:%s = %.2f" % [entity.entity_name, emotion_name, value])
			return
		print_output("Unknown emotion: " + emotion_name, Color.RED)
		return

	if args.has("reset"):
		for key in ed.fast.keys():
			ed.fast[key] = 0.0
		for key in ed.slow.keys():
			ed.slow[key] = 0.0
		print_output("Reset all emotions for %s" % entity.entity_name)
		return

	print_output("Usage: emotion entity:<id> set:<name>:<v>|reset", Color.RED)


func _cmd_trait(args: Dictionary) -> void:
	var entity = _get_entity(str(args.get("entity", "")))
	if entity == null:
		return

	if args.has("list"):
		if entity.personality == null:
			print_output("No personality data", Color.RED)
			return
		TraitSystem.evaluate_traits(entity)
		if entity.active_traits.is_empty():
			print_output("No active traits for %s" % entity.entity_name)
			return
		for i in range(entity.active_traits.size()):
			var t = entity.active_traits[i]
			var tid: String = str(t.get("id", "?"))
			print_output("trait: %s" % tid)
		return

	if args.has("facet"):
		if entity.personality == null:
			print_output("No personality data", Color.RED)
			return
		var facet_spec: String = str(args.get("facet", ""))
		var sep: int = facet_spec.rfind(":")
		if sep <= 0:
			print_output("Usage: trait entity:<id> facet:<name>:<v>", Color.RED)
			return
		var facet_name: String = facet_spec.substr(0, sep)
		var facet_value: float = float(facet_spec.substr(sep + 1))
		entity.personality.axes[facet_name] = facet_value
		print_output("Set facet %s = %.3f" % [facet_name, facet_value])
		return

	print_output("Usage: trait entity:<id> list|facet:<name>:<v>", Color.RED)


func _cmd_entity(args: Dictionary) -> void:
	if _entity_manager == null:
		print_output("entity_manager not connected", Color.RED)
		return

	var subcommand: String = ""
	var subvalue: String = ""
	var positional = args.get("_pos", [])
	if positional is Array and not positional.is_empty():
		subcommand = str(positional[0]).to_lower()
		if positional.size() > 1:
			subvalue = str(positional[1])
	elif args.has("list"):
		subcommand = "list"
	elif args.has("info"):
		subcommand = "info"
		subvalue = str(args.get("info", ""))
	elif args.has("kill"):
		subcommand = "kill"
		subvalue = str(args.get("kill", ""))
	elif args.has("spawn"):
		subcommand = "spawn"
	else:
		subcommand = "list"

	match subcommand:
		"list":
			var alive: Array = _entity_manager.get_alive_entities()
			for i in range(alive.size()):
				var e = alive[i]
				print_output("%d: %s at (%d,%d)" % [e.id, e.entity_name, e.position.x, e.position.y])
		"info":
			if subvalue.is_empty():
				print_output("Usage: entity info <id>", Color.RED)
				return
			var entity = _get_entity(subvalue)
			if entity == null:
				return
			var stress_value: float = 0.0
			if entity.emotion_data != null:
				stress_value = entity.emotion_data.stress
			print_output("Entity %s" % entity.entity_name)
			print_output("id:%d stress:%.1f hunger:%.3f energy:%.3f" % [entity.id, stress_value, entity.hunger, entity.energy])
			print_output("job:%s settlement:%d scars:%d" % [entity.job, entity.settlement_id, entity.trauma_scars.size()])
			print_output("violation_history keys: %s" % str(entity.violation_history.keys()))
		"kill":
			if subvalue.is_empty():
				print_output("Usage: entity kill <id>", Color.RED)
				return
			if _sim_engine == null:
				print_output("sim_engine not connected", Color.RED)
				return
			var kill_id: int = subvalue.to_int()
			_entity_manager.kill_entity(kill_id, "debug_kill", _sim_engine.current_tick)
			print_output("Killed entity %d" % kill_id)
		"spawn":
			print_output("Entity spawn not implemented in debug console")
		_:
			print_output("Usage: entity list|info:<id>|kill:<id>|spawn", Color.RED)


func _cmd_time(args: Dictionary) -> void:
	if _sim_engine == null:
		print_output("sim_engine not connected", Color.RED)
		return

	if args.has("fast"):
		var years: int = int(str(args.get("fast", "1")))
		years = maxi(years, 0)
		var ticks_fast: int = years * TICKS_PER_YEAR
		if _sim_engine.has_method("advance_ticks"):
			_sim_engine.advance_ticks(ticks_fast)
		elif _sim_engine.has_method("_process_tick"):
			for i in range(ticks_fast):
				_sim_engine._process_tick()
		print_output("Advanced %d year(s)" % years)
		return

	if args.has("tick"):
		var ticks: int = int(str(args.get("tick", "1")))
		ticks = maxi(ticks, 0)
		if _sim_engine.has_method("advance_ticks"):
			_sim_engine.advance_ticks(ticks)
		elif _sim_engine.has_method("_process_tick"):
			for i in range(ticks):
				_sim_engine._process_tick()
		print_output("Advanced %d tick(s)" % ticks)
		return

	var positional = args.get("_pos", [])
	if positional is Array and not positional.is_empty():
		var mode: String = str(positional[0]).to_lower()
		if mode == "fast":
			var years_pos: int = 1
			if positional.size() > 1:
				years_pos = int(str(positional[1]))
			years_pos = maxi(years_pos, 0)
			var ticks_pos: int = years_pos * TICKS_PER_YEAR
			if _sim_engine.has_method("advance_ticks"):
				_sim_engine.advance_ticks(ticks_pos)
			elif _sim_engine.has_method("_process_tick"):
				for i in range(ticks_pos):
					_sim_engine._process_tick()
			print_output("Advanced %d year(s)" % years_pos)
			return
		if mode == "tick":
			var tick_count: int = 1
			if positional.size() > 1:
				tick_count = int(str(positional[1]))
			tick_count = maxi(tick_count, 0)
			if _sim_engine.has_method("advance_ticks"):
				_sim_engine.advance_ticks(tick_count)
			elif _sim_engine.has_method("_process_tick"):
				for i in range(tick_count):
					_sim_engine._process_tick()
			print_output("Advanced %d tick(s)" % tick_count)
			return
		if mode == "pause":
			_sim_engine.is_paused = true
			print_output("Paused")
			return
		if mode == "resume":
			_sim_engine.is_paused = false
			print_output("Resumed")
			return

	if args.has("pause"):
		_sim_engine.is_paused = true
		print_output("Paused")
		return

	if args.has("resume"):
		_sim_engine.is_paused = false
		print_output("Resumed")
		return

	print_output("Usage: time fast:<years>|tick:<n>|pause|resume", Color.RED)


func _cmd_log(_args: Dictionary) -> void:
	print_output("Log filtering not yet implemented. All logs go to Godot output.")


func _cmd_help(args: Dictionary) -> void:
	var topic: String = ""
	var positional = args.get("_pos", [])
	if positional is Array and not positional.is_empty():
		topic = str(positional[0]).to_lower()
	elif args.has("cmd"):
		topic = str(args.get("cmd", "")).to_lower()

	if topic.is_empty():
		print_output("COMMANDS:")
		print_output("  stress entity:<id> set:<v>|add:<v>|break|reset")
		print_output("  event entity:<id> id:<event_id> [context:key=val]")
		print_output("  violation entity:<id> action:<id> [witness:<rel>] [victim:<rel>] [count:<n>]")
		print_output("  scar entity:<id> add:<id>|remove:<id>|list|clear")
		print_output("  emotion entity:<id> set:<name>:<v>|reset")
		print_output("  trait entity:<id> list|facet:<name>:<v>")
		print_output("  entity list|info:<id>|kill:<id>")
		print_output("  time fast:<years>|tick:<n>|pause|resume")
		print_output("  log stress|violation|break|scar|all on|off")
		print_output("  help [cmd]")
		return

	match topic:
		"stress":
			print_output("stress entity:<id> set:<v>|add:<v>|break|reset")
		"event":
			print_output("event entity:<id> id:<event_id> [context:key=val]")
		"violation":
			print_output("violation entity:<id> action:<id> [witness:<rel>] [victim:<rel>] [count:<n>]")
		"scar":
			print_output("scar entity:<id> add:<id>|remove:<id>|list|clear")
		"emotion":
			print_output("emotion entity:<id> set:<name>:<v>|reset")
		"trait":
			print_output("trait entity:<id> list|facet:<name>:<v>")
		"entity":
			print_output("entity list|info:<id>|kill:<id>")
		"time":
			print_output("time fast:<years>|tick:<n>|pause|resume")
		"log":
			print_output("log stress|violation|break|scar|all on|off")
		"help":
			print_output("help [cmd]")
		_:
			print_output("No detailed help for: " + topic, Color.RED)
