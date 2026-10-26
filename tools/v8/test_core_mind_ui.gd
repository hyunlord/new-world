# Native M-CORE QA: real scene, real selected person, current records and rendered frames.
# Bridge: get_agent_mind/get_agent_detail directly; get_agent_snapshot/get_time_snapshot/
# get_world_info through SimReads; advance_ticks, reset_world and optional load_world through SimReads.
# No synthetic facts, injected state, renderer substitutes or policy evaluation in the UI.
# Run windowed after the production mind switch and bridge/locale rebuild. User args: OUT [SAVE].
# CORE_MIND_UI_TICKS bounds natural transition observation (default 1440; maximum 28800).
extends SceneTree

const Store = preload("res://scripts/ui/mux/settings_store.gd")
var _main: Node
var _world: Node
var _inspector: Control
var _causal: Control
var _out: String
var _bits: int = -1
var _failures: Array[String] = []
var _samples: Array[Dictionary] = []
var _captures: Array[Dictionary] = []
var _observed: Dictionary = {"hunger": false, "threat": false}

func _init() -> void:
	call_deferred("_run")

func _frames(count: int = 5) -> void:
	for _index: int in count:
		await process_frame

func _check(label: String, success: bool) -> void:
	print("CORE_MIND_UI %s %s" % ["PASS" if success else "FAIL", label])
	if not success:
		_failures.append(label)

func _texts(node: Node) -> Array[Dictionary]:
	var result: Array[Dictionary] = []
	if node is Control and (node as Control).is_visible_in_tree() and (node is Label or node is Button):
		var control: Control = node as Control
		var text: String = str(node.get("text")).replace("\u2060", "")
		if not text.is_empty():
			result.append({"path": str(node.get_path()), "text": text, "rect": str(control.get_global_rect())})
	for child: Node in node.get_children():
		result.append_array(_texts(child))
	return result

func _click(control: Control) -> void:
	var point: Vector2 = control.get_global_rect().get_center()
	var motion := InputEventMouseMotion.new()
	motion.position = point
	Input.parse_input_event(motion)
	await _frames(1)
	for down: bool in [true, false]:
		var event := InputEventMouseButton.new()
		event.position = point
		event.button_index = MOUSE_BUTTON_LEFT
		event.pressed = down
		Input.parse_input_event(event)
		await _frames(2)

func _capture(name: String, mind: Dictionary) -> void:
	await _frames(8)
	await RenderingServer.frame_post_draw
	var screenshot: String = _out.path_join(name + ".png")
	_check("screenshot " + name, root.get_texture().get_image().save_png(screenshot) == OK)
	var labels: Array[Dictionary] = _texts(_inspector)
	labels.append_array(_texts(_causal))
	var chronicle: Node = _main.get_node("UI/ChroniclePanel")
	labels.append_array(_texts(chronicle))
	for row: Dictionary in labels:
		_check("localized " + str(row["path"]), not str(row["text"]).contains("MIND_") and not str(row["text"]).contains("UI_UNKNOWN"))
	_captures.append({"name": name, "png": screenshot, "mind": mind, "detail": _world.get_agent_detail(_bits), "labels": labels, "chronicle_text": str(chronicle.call("plain_text")), "viewport": str(root.size), "display": DisplayServer.get_name()})

# Check the real recorded predicate directions independently of summary formatting.
func _check_reason_metadata(mind: Dictionary) -> void:
	var facts: Array = mind.get("facts", [])
	_check("selected facts have explanatory metadata", not facts.is_empty())
	for fact: Dictionary in facts:
		_check("fact metadata present " + str(fact.get("name", "")), fact.has("predicates") and fact.has("score_terms") and fact.has("reason_kind") and fact.has("score_contribution"))
		var value: int = int(fact.get("value", 0))
		var predicates: Array = fact.get("predicates", [])
		for predicate: Dictionary in predicates:
			var threshold: int = int(predicate["threshold"]) if predicate.get("threshold") != null else 0
			var matches_record: bool = false
			match str(predicate.get("op", "")):
				"has": matches_record = value == 1
				"not": matches_record = value == 0
				"<": matches_record = value < threshold
				"<=": matches_record = value <= threshold
				">": matches_record = value > threshold
				">=": matches_record = value >= threshold
				"==": matches_record = value == threshold
			_check("selected predicate actually holds " + str(fact.get("name", "")), matches_record)
		var sum: int = 0
		var terms: Array = fact.get("score_terms", [])
		for term: Dictionary in terms:
			sum += int(term.get("contribution", 0))
		_check("recorded score contribution sum " + str(fact.get("name", "")), sum == int(fact.get("score_contribution", 0)))

func _check_name_key(key: String, identity: String) -> void:
	var locale: Node = root.get_node("Locale")
	var translated: String = str(locale.call("ltr", key))
	var unknown: String = str(locale.call("ltr", "UI_UNKNOWN"))
	_check("record locale key " + identity, not identity.is_empty() and not key.is_empty() and key != "UI_UNKNOWN" and translated != key and translated != unknown and not translated.is_empty())

func _check_record_names(mind: Dictionary) -> void:
	_check_name_key(str(mind.get("name_key", "")), str(mind.get("node", "")))
	for candidate: Dictionary in mind.get("top3", []):
		_check_name_key(str(candidate.get("name_key", "")), str(candidate.get("node", "")))
	for fact: Dictionary in mind.get("facts", []):
		var identity: String = str(fact.get("name", ""))
		var key: String = str(fact.get("name_key", ""))
		_check("fact locale identity " + identity, key == "MIND_FACT_" + identity.to_upper())
		_check_name_key(key, identity)

# These are observed selected inputs, not inferred tile proximity or injected needs.
func _scenario(mind: Dictionary) -> String:
	for fact: Dictionary in mind.get("facts", []):
		var name: String = str(fact.get("name", ""))
		var value: int = int(fact.get("value", 0))
		if str(mind.get("behaviour", "")) == "eat" and name == "hunger_actionable" and value == 1:
			return "hunger"
		if str(mind.get("behaviour", "")) == "flee" and name == "fleeing" and value == 1:
			return "threat"
	return ""

func _people_records() -> Dictionary:
	var records: Dictionary = {}
	var snapshot: Dictionary = SimReads.read(_world, "get_agent_snapshot")
	var entities: PackedInt64Array = snapshot.get("ids", PackedInt64Array())
	for entity: int in entities:
		var mind: Dictionary = _world.get_agent_mind(entity)
		if bool(mind.get("found", false)):
			records[entity] = mind
	return records

func _present(name: String, mind: Dictionary) -> void:
	_check_record_names(mind)
	_check_reason_metadata(mind)
	_causal.hide()
	_inspector.call("display_agent", _world.get_agent_detail(_bits))
	await _frames()
	var label: Label = _inspector.find_child("MindReason", true, false) as Label
	_check(name + " mind reason visible", label != null and label.is_visible_in_tree() and not label.text.is_empty())
	_check(name + " mind reason matches selected record", label != null and label.text == str(_causal.call("mind_summary", mind)))
	await _capture(name + "-person", mind)
	var why: Button = _inspector.find_child("Why", true, false) as Button
	_check(name + " Why button exists", why != null)
	if why != null:
		_check(name + " Why follows active locale", why.text.replace("\u2060", "") == str(root.get_node("Locale").call("ltr", "MUX_WHY")))
		var scroll: ScrollContainer = _inspector.get("_scroll") as ScrollContainer
		scroll.ensure_control_visible(why)
		await _frames()
		await _click(why)
		_check(name + " Why panel opened by input", _causal.is_visible_in_tree())
		var candidates: Array = mind.get("top3", [])
		_check(name + " recorded candidates present", not candidates.is_empty() and candidates.size() <= 3)
		var rendered: Array[Dictionary] = _texts(_causal)
		var locale: Node = root.get_node("Locale")
		for index: int in candidates.size():
			var candidate: Dictionary = candidates[index]
			var choice: String = str(locale.call("ltr", str(candidate.get("name_key", ""))))
			var expected: String = str(locale.call("ltr", "MIND_CANDIDATE_FMT")).format({"rank": index + 1, "choice": choice, "score": int(candidate.get("score", 0))})
			var matched: bool = false
			for row: Dictionary in rendered:
				matched = matched or str(row["text"]) == expected
			_check(name + " candidate matches actual record " + str(index + 1), matched)
		await _capture(name + "-why", mind)

func _finish() -> void:
	for category: String in _observed:
		_check("required transition " + category + " " + ("OBSERVED" if _observed[category] else "NOT_OBSERVED"), bool(_observed[category]))
	var evidence := FileAccess.open(_out.path_join("evidence.json"), FileAccess.WRITE)
	if evidence == null:
		_failures.append("write evidence")
	else:
		evidence.store_string(JSON.stringify({"failures": _failures, "samples": _samples, "captures": _captures, "transitions_observed": _observed, "visual_verdict": "PENDING_REVIEW", "world_info": SimReads.read(_world, "get_world_info") if _world != null else {}}, "\t"))
	for category: String in _observed:
		print("CORE_MIND_UI EXPOSURE %s %s" % [category, "OBSERVED" if _observed[category] else "NOT_OBSERVED"])
	print("CORE_MIND_UI_DONE failures=%d visual=PENDING_REVIEW" % _failures.size())
	quit(0 if _failures.is_empty() else 1)

func _run() -> void:
	var arguments: PackedStringArray = OS.get_cmdline_user_args()
	_out = arguments[0] if not arguments.is_empty() else "/tmp/core-mind-ui-%d" % OS.get_process_id()
	if not _out.is_absolute_path() or _out.begins_with(ProjectSettings.globalize_path("res://")):
		push_error("An external absolute evidence directory is required")
		quit(1)
		return
	DirAccess.make_dir_recursive_absolute(_out)
	_check("native window", DisplayServer.get_name() != "headless")
	if not _failures.is_empty():
		_finish()
		return
	_main = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	_main.get_node("UI/MuxHud").set("store", Store.new(_out.path_join("settings.cfg")))
	root.add_child(_main)
	await _frames(25)
	_check("real scene boot", bool(_main.get("boot_ok")))
	if not _failures.is_empty():
		_finish()
		return
	_world = _main.get_node("WorldSim")
	_main.get_node("Camera2D").call("set_paused", true)
	_inspector = _main.get_node("UI/AgentInspectorPanel") as Control
	_causal = _main.get_node("UI/CausalPanel") as Control
	if arguments.size() > 1:
		_check("load supplied real mind save", bool(SimReads.mutate(_world, "load_world", [arguments[1]])))
		await _frames()
	else:
		SimReads.mutate(_world, "reset_world", [0])
		await _frames()
	var info: Dictionary = SimReads.read(_world, "get_world_info")
	var seed: int = int(info.get("seed", -1))
	_check("calibration seed only", seed >= 0 and seed <= 7)
	if not _failures.is_empty():
		_finish()
		return
	SimReads.mutate(_world, "advance_ticks", [2])
	await _frames()
	var snapshot: Dictionary = SimReads.read(_world, "get_agent_snapshot")
	var entities: PackedInt64Array = snapshot.get("ids", PackedInt64Array())
	for entity: int in entities:
		var mind: Dictionary = _world.get_agent_mind(entity)
		if bool(mind.get("found", false)):
			_bits = entity
			break
	_check("production world has actual mind record", _bits >= 0)
	if _bits < 0:
		_finish()
		return
	for language: String in ["ko", "en"]:
		root.get_node("Locale").call("set_language", language)
		await _frames()
		await _present(language, _world.get_agent_mind(_bits))
	root.get_node("Locale").call("set_language", "ko")
	var budget: int = 1440
	if not OS.get_environment("CORE_MIND_UI_TICKS").is_empty():
		budget = clampi(OS.get_environment("CORE_MIND_UI_TICKS").to_int(), 0, 28800)
	var previous: Dictionary = _people_records()
	for _elapsed: int in range(budget):
		SimReads.mutate(_world, "advance_ticks", [1])
		await _frames(1)
		var current_people: Dictionary = _people_records()
		for entity: int in current_people:
			var current: Dictionary = current_people[entity]
			if not previous.has(entity):
				continue
			var before: Dictionary = previous[entity]
			if current.get("agent_id") != before.get("agent_id") or current.get("event_id") == before.get("event_id"):
				continue
			var category: String = _scenario(current)
			if category.is_empty() or bool(_observed[category]) or _scenario(before) == category:
				continue
			_bits = entity
			_samples.append({"category": category, "entity_bits": entity, "before": before, "after": current})
			_observed[category] = true
			for language: String in ["ko", "en"]:
				root.get_node("Locale").call("set_language", language)
				await _frames()
				var live: Dictionary = _world.get_agent_mind(entity)
				_check("paused transition record unchanged " + category + " " + language, live.get("event_id") == current.get("event_id"))
				await _present(category + "-transition-" + language, current)
				if category == "threat":
					_causal.hide()
					var chronicle: Control = _main.get_node("UI/ChroniclePanel") as Control
					if not chronicle.visible:
						chronicle.call("toggle_visible")
					await _frames()
					_check(language + " chronicle recognizes current kinds", int(chronicle.get("_unknown_kinds")) == 0)
					var phrase: String = str(root.get_node("Locale").call("ltr", "CHRON_MIND_CRITICAL"))
					_check(language + " actual critical choice in feed", str(chronicle.call("plain_text")).replace("\u2060", "").contains(phrase))
					await _capture(category + "-transition-" + language + "-chronicle", current)
					chronicle.hide()
		previous = current_people
		if bool(_observed["hunger"]) and bool(_observed["threat"]):
			break
	_finish()
