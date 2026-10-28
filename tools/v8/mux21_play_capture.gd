# Real seed0 play evidence; preparation is not performance measurement.
# Bridge reads: get_agent_mind/detail/m5, get_chronicle_since; SimReads snapshots,
# clock, story, steward, drafted. Mutations: reset_world, advance_ticks, save/load_world.
# No injected simulation state. Parent must serialize bridge builds and windowed captures.
# Args: prepare OUT [MAX_DAYS=45] | scenario OUT MANIFEST work|hunger|flee|defence
#       chronicle OUT (fresh seed0, exactly three elapsed days).
extends "res://tools/v8/mux2_text_capture.gd"

const ChronicleText = preload("res://scripts/ui/chronicle_text.gd")
var world: Node
var camera: Camera2D
var hud: Node
var inspector: Control
var causal: Control
var started: int
var mode: String
var evidence: Dictionary = {"complete": false, "visual_verdict": "PENDING_REVIEW", "scenarios": {}, "shots": []}
var inputs: Dictionary = {}
var ended: bool = false

func click_point(point: Vector2) -> void:
	# Keep native pointer motion consistent with the injected press/release pair.
	# Otherwise an OS motion event can move hover away between those two frames.
	root.warp_mouse(point)
	await frames(3)
	await super.click_point(point)

func _hashes() -> Dictionary:
	var result: Dictionary = {}
	result["res://assets/ui/chronicle_feed.json"] = FileAccess.get_sha256("res://assets/ui/chronicle_feed.json")
	result["res://scripts/ui/mux/mux_hud.gd"] = FileAccess.get_sha256("res://scripts/ui/mux/mux_hud.gd")
	for path: String in [get_script().resource_path, "res://tools/v8/mux2_text_capture.gd", "res://scripts/ui/panels/agent_inspector_panel.gd", "res://scripts/ui/panels/causal_panel.gd", "res://scripts/ui/panels/chronicle_panel.gd", "res://sim_bridge.gdextension", "res://rust/target/debug/libsim_bridge.dylib", "res://rust/target/release/libsim_bridge.dylib"]:
		result[path] = FileAccess.get_sha256(path) if FileAccess.file_exists(path) else "ABSENT"
	return result

func _write(name: String, value: Variant) -> void:
	var file: FileAccess = FileAccess.open(out.path_join(name), FileAccess.WRITE)
	check("write " + name, file != null)
	if file != null:
		file.store_string(JSON.stringify(value, "  "))
		file.close()

func _finish() -> void:
	if ended: return
	ended = true
	var after: Dictionary = _hashes()
	check("input hashes unchanged", inputs == after)
	evidence["inputs_start"] = inputs
	evidence["inputs_end"] = after
	evidence["mode"] = mode
	evidence["failures"] = failures
	evidence["complete"] = failures == 0
	evidence["inventory"] = inventory
	evidence["elapsed_seconds"] = (Time.get_ticks_msec() - started) / 1000.0
	_write("evidence.json", evidence)
	print("MUX21_PLAY_DONE " + JSON.stringify({"mode": mode, "complete": failures == 0, "failures": failures, "out": out}))
	quit(0 if failures == 0 else 1)

func _watchdog() -> void:
	if not ended:
		check("1800 second watchdog", false)
		_finish()

func run() -> void:
	started = Time.get_ticks_msec()
	var args: PackedStringArray = OS.get_cmdline_user_args()
	if args.size() < 2:
		push_error("expected prepare OUT [MAX_DAYS] | scenario OUT MANIFEST NAME | chronicle OUT")
		quit(1)
		return
	mode = args[0]
	out = args[1].simplify_path().trim_suffix("/")
	var project: String = ProjectSettings.globalize_path("res://").trim_suffix("/")
	if not mode in ["prepare", "scenario", "chronicle"] or not out.is_absolute_path() or out == project or out.begins_with(project + "/") or DirAccess.dir_exists_absolute(out):
		push_error("mode must be valid and OUT must be a new external absolute directory")
		quit(1)
		return
	if DirAccess.make_dir_recursive_absolute(out) != OK:
		quit(1)
		return
	inputs = _hashes()
	_write("evidence.json", evidence)
	create_timer(1800.0).timeout.connect(_watchdog)
	check("windowed capture", mode == "prepare" or DisplayServer.get_name() != "headless")
	main = load("res://scenes/main.tscn").instantiate()
	hud = main.get_node("UI/MuxHud")
	hud.set("store", Store.new(out.path_join("settings.cfg")))
	root.add_child(main)
	hud.get("store").set_value("helper_on", false)
	evidence["capture_preferences"] = {"helper_on": false}
	check("real scene boot", bool(main.get("boot_ok")))
	if failures > 0:
		_finish()
		return
	world = main.get_node("WorldSim")
	camera = main.get_node("Camera2D") as Camera2D
	inspector = main.get_node("UI/AgentInspectorPanel") as Control
	causal = main.get_node("UI/CausalPanel") as Control
	camera.call("set_paused", true)
	world.process_mode = Node.PROCESS_MODE_DISABLED
	root.get_node("Locale").call("set_language", "ko")
	await frames(10)
	if mode == "prepare":
		await _prepare(clampi(args[2].to_int(), 3, 45) if args.size() > 2 else 45)
	elif mode == "chronicle":
		await _chronicle()
	elif args.size() == 4:
		var parsed: Variant = JSON.parse_string(FileAccess.get_file_as_string(args[2]))
		check("preparation manifest parses", parsed is Dictionary)
		if parsed is Dictionary:
			evidence["manifest"] = {"path": args[2], "sha256": FileAccess.get_sha256(args[2])}
			var scenarios: Dictionary = parsed.get("scenarios", {})
			check("requested real scenario exists", scenarios.has(args[3]))
			if scenarios.has(args[3]): await _scenario(args[3], scenarios[args[3]])
	else:
		check("scenario requires manifest and name", false)
	_finish()

func _tick() -> int:
	return int(SimReads.read(world, "get_time_snapshot").get("tick", -1))

func _reset() -> void:
	check("reset seed0", bool(SimReads.mutate(world, "reset_world", [0])))
	await frames(3)
	evidence["world_info"] = SimReads.read(world, "get_world_info")
	check("actual seed0 tick0", int(evidence["world_info"].get("seed", -1)) == 0 and _tick() == 0)
	evidence["initial_steward"] = SimReads.read(world, "get_steward")

func _advance(count: int) -> void:
	var before: int = _tick()
	var reached: int = int(SimReads.mutate(world, "advance_ticks", [count]))
	check("advance exact ticks", reached == before + count)
	await frames(1)

func _rows(start: int = 0) -> Array[Dictionary]:
	var rows: Array[Dictionary] = []
	var cursor: int = start
	for _page: int in 10000:
		var page: Dictionary = world.get_chronicle_since(cursor, 512)
		rows.append_array(ChronicleText.rows(page))
		var next: int = int(page.get("next", cursor))
		if next >= int(page.get("total", next)): return rows
		if next <= cursor:
			check("chronicle cursor advances", false)
			return rows
		cursor = next
	check("chronicle pagination bound", false)
	return rows

func _save(name: String, extra: Dictionary = {}) -> void:
	var path: String = out.path_join(name + ".wsave")
	check("save " + name, bool(SimReads.mutate(world, "save_world", [path])))
	var row: Dictionary = extra.duplicate(true)
	row.merge({"save": path, "sha256": FileAccess.get_sha256(path), "tick": _tick(), "clock": SimReads.read(world, "get_time_snapshot")})
	evidence["scenarios"][name] = row
	_write("manifest.json", {"seed": 0, "scenarios": evidence["scenarios"], "inputs": inputs, "policy": "natural seed0 default settings; sampled every30ticks; manual tick advance only; no state injection"})

func _person_record(bits: int) -> Dictionary:
	var detail: Dictionary = world.get_agent_detail(bits)
	return {"bits": bits, "agent_id": int(detail.get("agent_id", -1)), "detail": detail, "mind": world.get_agent_mind(bits), "m5": world.get_agent_m5(int(detail.get("agent_id", -1)))}

func _category(record: Dictionary, wolves: int) -> String:
	var mind: Dictionary = record["mind"]
	var behaviour: String = str(mind.get("behaviour", ""))
	if not bool(mind.get("found", false)): return ""
	if behaviour == "do_job" and int(record["detail"].get("target_kind", -1)) == 9 and int(record["detail"].get("state_tag", -1)) == 3:
		return "work"
	for fact: Dictionary in mind.get("facts", []):
		if behaviour == "eat" and str(fact.get("name", "")) == "hunger_actionable" and int(fact.get("value", 0)) == 1:
			return "hunger"
		if behaviour == "flee" and str(fact.get("name", "")) == "fleeing" and int(fact.get("value", 0)) == 1 and bool(record["m5"].get("fleeing", false)) and wolves > 0:
			return "flee"
	return ""

func _prepare(max_days: int) -> void:
	await _reset()
	var per_day: int = int(SimReads.read(world, "get_time_snapshot").get("ticks_per_day", 1440))
	var defence: Dictionary = {}
	var cursor: int = 0
	while _tick() < max_days * per_day and failures == 0:
		var next_target: int = 3 * per_day if _tick() < 3 * per_day else max_days * per_day
		await _advance(mini(30, next_target - _tick()))
		var snapshot: Dictionary = SimReads.read(world, "get_agent_snapshot")
		var bits: PackedInt64Array = snapshot.get("ids", PackedInt64Array())
		var stages: PackedByteArray = snapshot.get("life_stages", PackedByteArray())
		var wolves: int = int(SimReads.read(world, "get_story").get("wolves", 0))
		for i: int in mini(bits.size(), stages.size()):
			if stages[i] != 0: continue
			var person: Dictionary = _person_record(bits[i])
			var category: String = _category(person, wolves)
			if not category.is_empty() and not evidence["scenarios"].has(category):
				person["wolves"] = wolves
				person["life_stage"] = stages[i]
				_save(category, person)
		for row: Dictionary in _rows(cursor):
			cursor = maxi(cursor, int(row.get("index", -1)) + 1)
			if int(row.get("kind", -1)) == 46 and defence.is_empty(): defence = row
		if _tick() == 3 * per_day: _save("day3")
		if not defence.is_empty() and evidence["scenarios"].has("work") and evidence["scenarios"].has("hunger") and evidence["scenarios"].has("flee") and evidence["scenarios"].has("day3"): break
	if not defence.is_empty():
		await _reset()
		# Systems emit at current_tick before the engine increments it.
		var target: int = int(defence["tick"])
		check("defence has replayable tick", target >= 0)
		while _tick() < target and failures == 0: await _advance(mini(240, target - _tick()))
		if _tick() == target: _save("defence", {"expected_event": defence, "advance_ticks": 1})
	for required: String in ["work", "hunger", "flee", "day3", "defence"]:
		check("observed " + required, evidence["scenarios"].has(required))

func _load(row: Dictionary) -> void:
	var path: String = str(row.get("save", ""))
	check("save checksum", not path.is_empty() and FileAccess.get_sha256(path) == str(row.get("sha256", "")))
	if failures > 0: return
	check("load real save", bool(SimReads.mutate(world, "load_world", [path])))
	await frames(10)
	check("loaded exact tick seed0", _tick() == int(row.get("tick", -2)) and int(SimReads.read(world, "get_world_info").get("seed", -1)) == 0)

func _fit_control(control: Control, label: String) -> void:
	check(label + " exists and visible", control != null and control.is_visible_in_tree())
	if control == null: return
	var rect: Rect2 = control.get_global_rect()
	check(label + " inside screen", root.get_visible_rect().encloses(rect))
	var parent: Node = control.get_parent()
	while parent != null:
		if parent is Control and (parent as Control).clip_contents:
			check(label + " inside " + str(parent.name), (parent as Control).get_global_rect().encloses(rect))
		parent = parent.get_parent()

func _shot(name: String, extra: Dictionary) -> void:
	var start: int = inventory.size()
	await capture(name)
	for index: int in range(start, inventory.size()):
		var text: String = str(inventory[index].get("raw", ""))
		check("localized " + str(inventory[index].get("path", "")), not text.contains("MIND_") and not text.contains("UI_UNKNOWN"))
	var row: Dictionary = extra.duplicate(true)
	row.merge({"name": name, "png": out.path_join(name + ".png"), "sha256": FileAccess.get_sha256(out.path_join(name + ".png")), "tick": _tick(), "viewport": str(root.size)})
	evidence["shots"].append(row)

func _scenario(name: String, row: Dictionary) -> void:
	for factor: float in [1.0, 1.25]:
		await _load(row)
		if failures > 0: return
		hud.get("store").set_value("ui_scale", factor)
		await frames(5)
		if name == "defence":
			await _defence(row, factor)
			continue
		var bits: int = int(row.get("bits", -1))
		var person: Dictionary = _person_record(bits)
		check("saved person and mind retained", int(person["agent_id"]) == int(row.get("agent_id", -2)) and person["mind"].get("event_id") == row.get("mind", {}).get("event_id"))
		check("scenario still true", _category(person, int(SimReads.read(world, "get_story").get("wolves", 0))) == name)
		camera.zoom = Vector2.ONE
		camera.position = View.tile_centre(Vector2i(int(person["detail"]["x"]), int(person["detail"]["y"])))
		camera.call("_clamp_position")
		inspector.call("display_agent", person["detail"])
		await frames(5)
		var reason: Label = inspector.find_child("MindReason", true, false) as Label
		var why: Button = inspector.find_child("Why", true, false) as Button
		var scroll: ScrollContainer = inspector.get("_scroll") as ScrollContainer
		check("person reason matches recorded choice", reason != null and reason.text == str(causal.call("mind_summary", person["mind"])))
		if reason == null or why == null:
			check("person controls found", false)
			return
		scroll.ensure_control_visible(reason)
		await frames(5)
		_fit_control(reason, "mind reason")
		var suffix: String = "%s-%s" % [name, str(factor)]
		await _shot(suffix + "-person", {"scale": factor, "person": person})
		scroll.ensure_control_visible(why)
		await frames(5)
		_fit_control(why, "Why button")
		var signals: Array[String] = []
		var on_down: Callable = func() -> void: signals.append("down")
		var on_up: Callable = func() -> void: signals.append("up")
		var on_pressed: Callable = func() -> void: signals.append("pressed")
		why.button_down.connect(on_down)
		why.button_up.connect(on_up)
		why.pressed.connect(on_pressed)
		await click(why)
		var hovered: Control = root.gui_get_hovered_control()
		evidence.get_or_add("why_input", []).append({"scenario": suffix, "signals": signals, "button_rect": str(why.get_global_rect()), "hovered": str(hovered.get_path()) if hovered != null else "none", "window_focus": root.has_focus(), "popup_visible": causal.is_visible_in_tree(), "selected_bits": inspector.get("_selected_entity_bits")})
		why.button_down.disconnect(on_down)
		why.button_up.disconnect(on_up)
		why.pressed.disconnect(on_pressed)
		check("Why opened by real input", causal.is_visible_in_tree())
		_fit_control(causal, "Why popup")
		var candidates: Array = person["mind"].get("top3", [])
		check("recorded top three exist", not candidates.is_empty() and candidates.size() <= 3)
		var container: Node = causal.get("_history_container")
		for rank: int in candidates.size():
			var candidate: Dictionary = candidates[rank]
			var locale: Node = root.get_node("Locale")
			var choice: String = str(locale.call("ltr", str(candidate.get("name_key", ""))))
			var expected: String = str(locale.call("ltr", "MIND_CANDIDATE_FMT")).format({"rank": rank + 1, "choice": choice, "score": int(candidate.get("score", 0))})
			var found: bool = false
			for child: Node in container.get_children():
				if child is Label and (child as Label).text.replace("\u2060", "") == expected: found = true
			check("Why candidate equals retained record " + str(rank), found)
		await _shot(suffix + "-why", {"scale": factor, "person": person})
		causal.hide()
		inspector.call("close")

func _chronicle() -> void:
	await _reset()
	var per_day: int = int(SimReads.read(world, "get_time_snapshot").get("ticks_per_day", 1440))
	while _tick() < 3 * per_day and failures == 0: await _advance(mini(30, 3 * per_day - _tick()))
	await frames(3)
	var rows: Array[Dictionary] = _rows()
	var counts: Dictionary = {}
	for row: Dictionary in rows:
		var kind: String = str(row["kind"])
		counts[kind] = int(counts.get(kind, 0)) + 1
	evidence["chronicle"] = {"rows": rows, "kind_counts": counts, "mind_choice_kind": 47, "mind_choice_count": int(counts.get("47", 0)), "clock": SimReads.read(world, "get_time_snapshot"), "elapsed_days": 3}
	var panel: Control = main.get_node("UI/ChroniclePanel") as Control
	check("feed caught up at exact day3", int(panel.get("_next")) == rows.size() and _tick() == 3 * per_day)
	for factor: float in [1.0, 1.25]:
		hud.get("store").set_value("ui_scale", factor)
		await frames(5)
		if panel.visible: panel.hide()
		for down: bool in [true, false]:
			var event := InputEventKey.new()
			event.keycode = KEY_C
			event.pressed = down
			Input.parse_input_event(event)
			await frames(2)
		check("C opened default feed", panel.visible and int(panel.get("_filter")) == 0 and not bool(panel.get("_person_only")))
		await _shot("chronicle-day3-%s" % factor, {"scale": factor, "plain_text": panel.call("plain_text"), "entries": panel.get("_entries"), "cursor": panel.get("_next"), "total": panel.get("_total")})

## JSON stores numbers as floats. Validate every field before canonical integer comparison.
## Reject unsafe float integers instead of concealing lost precision or truncating fractions.
static func normalize_event(raw: Variant) -> Dictionary:
	if not raw is Dictionary or raw.size() != ChronicleText.COLUMNS.size():
		return {"valid": false}
	var normalized: Dictionary = {}
	for key: String in ChronicleText.COLUMNS:
		if not raw.has(key): return {"valid": false}
		var value: Variant = raw[key]
		if value is float:
			var number: float = value
			if not is_finite(number) or absf(number) > 9007199254740991.0 or floor(number) != number:
				return {"valid": false}
		elif not value is int:
			return {"valid": false}
		normalized[key] = int(value)
	return {"valid": true, "event": normalized}

func _defence(row: Dictionary, factor: float) -> void:
	var normalized: Dictionary = normalize_event(row.get("expected_event"))
	check("expected defence event has exact integral schema", bool(normalized.get("valid", false)))
	if not bool(normalized.get("valid", false)): return
	var expected: Dictionary = normalized["event"]
	var before: int = int(world.get_chronicle_since(0, 1).get("total", 0))
	await _advance(int(row.get("advance_ticks", 1)))
	var actual: Dictionary = {}
	for event: Dictionary in _rows(before):
		if int(event.get("kind", -1)) == 46: actual = event
	check("natural defence event reproduced", not actual.is_empty() and actual == expected)
	evidence["defence_comparison"] = {"expected": expected, "actual": actual, "all_fields_equal": actual == expected}
	var drafted: Dictionary = SimReads.read(world, "get_drafted")
	var ids: PackedInt64Array = drafted.get("ids", PackedInt64Array())
	check("natural defence has drafted people", not ids.is_empty())
	var snapshot: Dictionary = SimReads.read(world, "get_agent_snapshot")
	var agents: PackedInt64Array = snapshot.get("agent_ids", PackedInt64Array())
	var bits: PackedInt64Array = snapshot.get("ids", PackedInt64Array())
	var selected: int = -1
	for i: int in mini(agents.size(), bits.size()):
		if agents[i] in ids:
			selected = bits[i]
			break
	check("drafted person has live entity", selected >= 0)
	if selected < 0: return
	var detail: Dictionary = world.get_agent_detail(selected)
	camera.zoom = Vector2.ONE
	camera.position = View.tile_centre(Vector2i(int(detail["x"]), int(detail["y"])))
	camera.call("_clamp_position")
	await frames(8)
	var layer: Node = main.get_node("ColonyView/AgentLayer")
	var marks: Dictionary = layer.call("person_marks", selected)
	check("actual renderer drafted mark", bool(marks.get("drafted", false)))
	var drawn: Rect2 = layer.call("drawn_rect", selected)
	var screen_rect: Rect2 = (layer as Node2D).get_global_transform_with_canvas() * drawn
	check("drafted person projects inside screenshot", drawn.has_area() and root.get_visible_rect().encloses(screen_rect))
	var bar: Node = main.get_node("UI/OrderBar")
	var label: Label = bar.get("_toast") as Label
	var toast: Control = bar.get("_toast_panel") as Control
	var text: String = str(root.get_node("Locale").call("ltr", "MUX_STEWARD_DEFENCE_ACTION")).format({"count": int(expected.get("value", 0))})
	var deadline: int = Time.get_ticks_msec() + 30000
	while Time.get_ticks_msec() < deadline and not (toast.visible and label.text.replace("\u2060", "") == text): await frames(1)
	check("production defence toast displayed", toast.visible and label.text.replace("\u2060", "") == text)
	_fit_control(label, "defence toast")
	await _shot("defence-%s" % factor, {"scale": factor, "event": actual, "drafted": drafted, "selected_bits": selected, "marks": marks, "drawn_rect": str(layer.call("drawn_rect", selected)), "steward": SimReads.read(world, "get_steward"), "toast": label.text, "ring_pixel_verdict": "PENDING_REVIEW"})
	check("toast survived capture settling", toast.visible and label.text.replace("\u2060", "") == text)
