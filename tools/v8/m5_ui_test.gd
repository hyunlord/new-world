# tools/v8/m5_ui_test.gd — headless check of the M5 UI paths (goal §3.5, §10), asserted through the bridge reads.
# Purpose: boot res://scenes/main.tscn with the clock paused (Space) on the live default world and, as a player would
#   through input events: F7 then 3 and a click on a deer marks it (get_animals `marked`, the ring drawn); F7 then 4 and
#   a drag add a hunt zone (get_animals `zones`, drawn); the cancel mode (F6) drag lifts both; a person in the inspector
#   and D drafts them (get_drafted, the red outline, the inspector's draft line, the HUD's drafted list); right clicks
#   on a tile, a deer and another person give cmd_draft_move / cmd_draft_attack / cmd_draft_attack_person (get_agent_m5
#   `draft_order` and target), to the inspector's person or, with the inspector closed, to every drafted person; D
#   releases, 모두 해제 releases all; M toggles the mood overlay (tints by `mood_bands`, the legend); the inspector's
#   mood and apparel lines and the 이야기 tab carry no raw key; P and the "모두 가죽 옷" toggle send cmd_set_dress_order
#   (get_story `dress_order` flips both ways); the teach dialog pre-selects get_teach_suggestion; chronicle kinds 29–37
#   and the M5 quarrel render without raw keys in ko and en. Then the clock is advanced (advance_ticks, a day at a
#   time) until the storyteller's first wolf pack and first cold snap: the wolves' banner "늑대 무리가 다가온다 (n)"
#   with n = get_story `wolves` and its pulse, the wolves drawn, the cold-snap banner with `days_of`, `firewood` and
#   `firewood_day_need`; a sick person's badge; a chronicle entry with a cause shows "→ 왜?" and a click opens the
#   chain (UI/WhyPanel, one line per get_chronicle_why link); the 이야기 tab lists that person's lines; the year
#   summary heads the chronicle once a year is over.
# Bridge methods called: via SimReads get_story, get_animals, get_drafted, get_agent_snapshot, get_world_info,
#   get_time_snapshot, get_colony_overlay, advance_ticks; get_agent_m5, get_agent_detail, get_chronicle_since,
#   get_chronicle_why, get_teach_suggestion, get_tech_node, get_year_summary (direct); and the UI's own calls.
# Run with WORLDSIM_TEST_SETTINGS_DIR set to an existing external absolute directory.
# bash tools/v8/run_bounded.sh 900 "$GODOT_BIN" --path . --script res://tools/v8/m5_ui_test.gd
# Output: one "M5_UI <PASS|FAIL|INFO> <step> <detail>" line per check, then "M5_UI_DONE failures=N"; quits 0 / 1.
extends SceneTree

const Chronicle = preload("res://scripts/ui/chronicle_text.gd")
const StoryText = preload("res://scripts/ui/story_text.gd")
const EconomyText = preload("res://scripts/ui/economy_text.gd")
const View = preload("res://scripts/world/colony/view.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
const AnimalLayer = preload("res://scripts/world/colony/animal_layer.gd")
const OverlayLayer = preload("res://scripts/world/colony/overlay_layer.gd")
const KEY_RE: String = "[A-Z][A-Z0-9]*(_[A-Z0-9]+)+"
const DOORWAY: int = 10
## How far the clock may be advanced looking for the first pack and cold snap (display days).
const MAX_DAYS: int = 60

var _failures: Array[String] = []
var _checks: int = 0
var _main: Node
var _world: Node
var _locale: Node
var _raw: RegEx
var _tool: Node
var _bar: Node
var _inspector: Node
var _agents: Node
var _animals: Node
var _banner: Node
var _camera: Node


func _init() -> void:
	call_deferred("_run")


func _check(step: String, ok: bool, detail: String = "") -> void:
	_checks += 1
	print("M5_UI %s %s %s" % ["PASS" if ok else "FAIL", step, detail])
	if not ok:
		_failures.append(step)


func _info(step: String, detail: String) -> void:
	print("M5_UI INFO %s %s" % [step, detail])


func _frames(n: int) -> void:
	for _i: int in n:
		await process_frame


func _key(code: Key) -> void:
	for pressed: bool in [true, false]:
		var event: InputEventKey = InputEventKey.new()
		event.keycode = code
		event.physical_keycode = code
		event.pressed = pressed
		Input.parse_input_event(event)
		await process_frame


func _motion(at: Vector2) -> void:
	var motion: InputEventMouseMotion = InputEventMouseMotion.new()
	motion.position = at
	motion.global_position = at
	Input.parse_input_event(motion)
	await process_frame


func _button(at: Vector2, index: MouseButton, pressed: bool) -> void:
	var click: InputEventMouseButton = InputEventMouseButton.new()
	click.button_index = index
	click.pressed = pressed
	click.position = at
	click.global_position = at
	Input.parse_input_event(click)
	await process_frame


## A click with `index` at screen point `at`: motion, press, release.
func _click(at: Vector2, index: MouseButton = MOUSE_BUTTON_LEFT) -> void:
	await _motion(at)
	await _button(at, index, true)
	await _button(at, index, false)


## A left drag from `a` to `b` (screen points).
func _drag(a: Vector2, b: Vector2) -> void:
	await _motion(a)
	await _button(a, MOUSE_BUTTON_LEFT, true)
	await _motion((a + b) * 0.5)
	await _motion(b)
	await _button(b, MOUSE_BUTTON_LEFT, false)


## The window point of world position `p` under the current camera (the window may be smaller than the 1920×1080
## canvas headless: the stretch transform maps canvas pixels to window pixels, as input events arrive).
func _screen(p: Vector2) -> Vector2:
	return root.get_final_transform() * (root.get_canvas_transform() * p)


## The window point of a UI control's centre.
func _ui(control: Control) -> Vector2:
	return root.get_final_transform() * control.get_global_rect().get_center()


func _tile_screen(t: Vector2i) -> Vector2:
	return _screen(Vector2(t) * View.TILE + Vector2(View.TILE, View.TILE) * 0.5)


## Centre the camera on `tile` and let it settle.
func _look_at(tile: Vector2i) -> void:
	_camera.call("center_on_tile", tile)
	(_camera as Camera2D).reset_smoothing()
	(_camera as Camera2D).force_update_scroll()
	await _frames(3)


## Whether `text` is non-empty and holds no key-shaped token.
func _readable(text: String) -> bool:
	return text.strip_edges() != "" and _raw.search(text) == null


## A BBCode line's visible text.
func _plain(bbcode: String) -> String:
	var label: RichTextLabel = RichTextLabel.new()
	label.bbcode_enabled = true
	label.text = bbcode
	var out: String = label.get_parsed_text()
	label.free()
	return out


func _hearth() -> Vector2i:
	var b: Dictionary = SimReads.read(_world, "get_building_snapshot")
	var kinds: PackedByteArray = b.get("kinds", PackedByteArray())
	var xs: PackedInt32Array = b.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = b.get("ys", PackedInt32Array())
	for i: int in mini(kinds.size(), mini(xs.size(), ys.size())):
		if kinds[i] == 0:
			return Vector2i(xs[i], ys[i])
	return Vector2i(96, 96)


func _tick() -> int:
	return int(SimReads.read(_world, "get_time_snapshot").get("tick", 0))


## Adults by AgentId with their entity bits and tiles: [{id, bits, tile}], AgentId order.
func _adults() -> Array[Dictionary]:
	var snap: Dictionary = SimReads.read(_world, "get_agent_snapshot")
	var ids: PackedInt64Array = snap.get("agent_ids", PackedInt64Array())
	var bits: PackedInt64Array = snap.get("ids", PackedInt64Array())
	var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
	var stages: PackedByteArray = snap.get("life_stages", PackedByteArray())
	var out: Array[Dictionary] = []
	for i: int in mini(mini(ids.size(), bits.size()), mini(xs.size(), ys.size())):
		if i < stages.size() and stages[i] == 0:
			out.append({"id": ids[i], "bits": bits[i], "tile": Vector2i(xs[i], ys[i])})
	out.sort_custom(func(a: Dictionary, b: Dictionary) -> bool: return int(a["id"]) < int(b["id"]))
	return out


## The deer nearest `from` that the animal layer draws: {id, tile} or {}.
func _deer_near(from: Vector2i) -> Dictionary:
	var read: Dictionary = SimReads.read(_world, "get_animals")
	var ids: PackedInt64Array = read.get("ids", PackedInt64Array())
	var kinds: PackedInt64Array = read.get("kinds", PackedInt64Array())
	var xs: PackedInt32Array = read.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = read.get("ys", PackedInt32Array())
	var best: Dictionary = {}
	var best_d: int = 1 << 30
	for i: int in mini(mini(ids.size(), kinds.size()), mini(xs.size(), ys.size())):
		var t: Vector2i = Vector2i(xs[i], ys[i])
		var d: int = (t - from).length_squared()
		if kinds[i] == StoryText.DEER and d < best_d:
			best_d = d
			best = {"id": ids[i], "tile": t}
	return best


func _m5(agent: int) -> Dictionary:
	var read: Variant = _world.call("get_agent_m5", agent)
	return read if read is Dictionary else {}


func _drafted_ids() -> PackedInt64Array:
	return SimReads.read(_world, "get_drafted").get("ids", PackedInt64Array())


## Show `agent` in the inspector (as a click on them does).
func _inspect(person: Dictionary) -> void:
	_inspector.call("display_agent", _world.call("get_agent_detail", int(person["bits"])))
	await _frames(2)


## A tile near `from` with nobody and no animal drawn on it.
func _empty_tile(from: Vector2i) -> Vector2i:
	for r: int in range(2, 9):
		for off: Vector2i in [Vector2i(r, 0), Vector2i(0, r), Vector2i(-r, 0), Vector2i(0, -r), Vector2i(r, r)]:
			var t: Vector2i = from + off
			var p: Vector2 = Vector2(t) * View.TILE + Vector2(View.TILE, View.TILE) * 0.5
			if int(_agents.call("entity_at", p)) < 0 and int(_animals.call("animal_at", p)) < 0:
				return t
	return from + Vector2i(3, 3)


func _run() -> void:
	var settings_dir: String = OS.get_environment("WORLDSIM_TEST_SETTINGS_DIR")
	var isolated: bool = settings_dir.is_absolute_path() and not settings_dir.begins_with(ProjectSettings.globalize_path("res://")) and DirAccess.dir_exists_absolute(settings_dir)
	_check("isolated_settings", isolated, settings_dir)
	if not isolated:
		print("M5_UI_DONE failures=1 checks=1 isolated_settings")
		quit(1)
		return
	_raw = RegEx.new()
	_raw.compile(KEY_RE)
	_main = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(_main)
	await _frames(20)
	_world = _main.get_node_or_null("WorldSim")
	_locale = root.get_node_or_null("Locale")
	_tool = _main.get_node_or_null("ColonyView/OrderTool")
	_bar = _main.get_node_or_null("UI/OrderBar")
	_inspector = _main.get_node_or_null("UI/AgentInspectorPanel")
	_agents = _main.get_node_or_null("ColonyView/AgentLayer")
	_animals = _main.get_node_or_null("ColonyView/AnimalLayer")
	_banner = _main.get_node_or_null("UI/StoryBanner")
	_camera = _main.get_node_or_null("Camera2D")
	var why: Node = _main.get_node_or_null("UI/WhyPanel")
	var chronicle: Node = _main.get_node_or_null("UI/ChroniclePanel")
	var priority: Node = _main.get_node_or_null("UI/PriorityPanel")
	var teach: Node = _main.get_node_or_null("UI/TeachPanel")
	var ok_nodes: bool = _world != null and _locale != null and _tool != null and _bar != null and _inspector != null and _agents != null \
		and _animals != null and _banner != null and _camera != null and why != null and chronicle != null and priority != null and teach != null
	_check("scene", ok_nodes, "animal layer %s banner %s why %s" % [_animals != null, _banner != null, why != null])
	if not ok_nodes:
		_finish()
		return
	_locale.call("set_language", "ko")
	await _key(KEY_SPACE)
	await _frames(2)
	var story: Dictionary = SimReads.read(_world, "get_story")
	var info: Dictionary = SimReads.read(_world, "get_world_info")
	_info("window", "size %s final %s" % [root.size, root.get_final_transform()])
	_info("world", "seed %d tick %d story on %s dress %s" % [int(info.get("seed", -1)), _tick(), story.get("on", false), story.get("dress_order", false)])
	_check("story_on", bool(story.get("on", false)), str(story.keys().size()))
	var animals: Dictionary = SimReads.read(_world, "get_animals")
	var count: int = (animals.get("ids", PackedInt64Array()) as PackedInt64Array).size()
	_check("animals_drawn", count > 0 and int((_animals.call("counts") as Dictionary)["animals"]) == count, "read %d drawn %s" % [count, _animals.call("counts")])
	# M-ART2: every animal pose (all states, both facings, still and moving) and the dead frame resolve to manifest
	# frames through bindings.json; the M5 item kinds resolve to manifest sprites in every count bucket and have names.
	var sprites_ok: bool = true
	var missing: Array[String] = []
	for kind: int in [0, 1]:
		for state: int in 6:
			for facing: int in 2:
				for moving: bool in [false, true]:
					var fid: String = AnimalLayer.frame_id(kind, state, facing, moving)
					if not ArtData.has(fid):
						missing.append(fid)
	for kind: int in range(11, 17):
		for n_items: int in [1, 5, 20]:
			var look: Array = OverlayLayer.stack_look(kind, n_items)
			if not ArtData.has(str(look[0])):
				missing.append(str(look[0]))
		var dead: String = OverlayLayer.carcass_id(kind)
		if dead != "" and not ArtData.has(dead):
			missing.append(dead)
		sprites_ok = sprites_ok and _readable(EconomyText.item(_locale, kind))
	_check("m5_sprites_and_items", sprites_ok and missing.is_empty() and OverlayLayer.carcass_id(11) != "", "missing %s" % [missing])
	var hearth: Vector2i = _hearth()
	await _hunt_orders(hearth)
	await _draft_orders(hearth)
	await _mood_and_inspector()
	await _dress_and_teach(priority, teach)
	_chronicle_texts()
	await _advance_and_story(chronicle, why)
	_finish()


## F7-3 click on a deer, F7-4 drag a zone, F6 drag lifts both.
func _hunt_orders(hearth: Vector2i) -> void:
	var deer: Dictionary = _deer_near(hearth)
	if deer.is_empty():
		_check("deer_present", false, "no deer")
		return
	var tile: Vector2i = deer["tile"]
	await _look_at(tile)
	await _key(KEY_F7)
	var menu_ok: bool = bool(_bar.call("dig_menu_open")) and (_bar.get("_dig_buttons") as Array).size() == 4
	await _key(KEY_3)
	var hint: String = (_bar.get("_hint") as Label).text
	_check("f7_3_hunt_mode", menu_ok and int(_tool.get("mode")) == 10 and not bool(_bar.call("dig_menu_open")) and _readable(hint), "mode %d hint '%s'" % [int(_tool.get("mode")), hint])
	var drawn: Vector2 = _animals.call("drawn_position", int(deer["id"]))
	await _click(_screen(drawn + Vector2(View.TILE, View.TILE) * 0.5))
	await _frames(2)
	var read: Dictionary = SimReads.read(_world, "get_animals")
	var i: int = (read.get("ids", PackedInt64Array()) as PackedInt64Array).find(int(deer["id"]))
	var marked: bool = i >= 0 and (read.get("marked", PackedByteArray()) as PackedByteArray)[i] == 1
	var toast: Label = _bar.get("_toast") as Label
	_check("hunt_click_marks_deer", marked and bool(_animals.call("marked_shown", int(deer["id"]))) and _readable(toast.text), "deer %d at %s marked %s toast '%s'" % [int(deer["id"]), tile, marked, toast.text])
	await _key(KEY_ESCAPE)
	# F7-4: a hunt zone around the deer.
	await _key(KEY_F7)
	await _key(KEY_4)
	_check("f7_4_zone_mode", int(_tool.get("mode")) == 11 and _readable((_bar.get("_hint") as Label).text), (_bar.get("_hint") as Label).text)
	var a: Vector2i = tile - Vector2i(3, 3)
	var b: Vector2i = tile + Vector2i(3, 3)
	await _drag(_tile_screen(a), _tile_screen(b))
	await _frames(2)
	var zones: PackedInt32Array = SimReads.read(_world, "get_animals").get("zones", PackedInt32Array())
	_check("f7_4_drag_adds_zone", zones.size() == 4 and zones[0] == a.x and zones[1] == a.y and zones[2] == b.x and zones[3] == b.y \
		and int((_animals.call("counts") as Dictionary)["zones"]) == 1, "zones %s toast '%s'" % [zones, toast.text])
	await _key(KEY_ESCAPE)
	# F6 cancel over the same rectangle lifts the zone and the mark.
	await _key(KEY_F6)
	await _drag(_tile_screen(a - Vector2i(1, 1)), _tile_screen(b + Vector2i(1, 1)))
	await _frames(2)
	read = SimReads.read(_world, "get_animals")
	var marks: int = 0
	for m: int in read.get("marked", PackedByteArray()):
		marks += m
	_check("cancel_lifts_hunt", (read.get("zones", PackedInt32Array()) as PackedInt32Array).is_empty() and marks == 0 and _readable(toast.text),
		"zones %s marks %d toast '%s'" % [read.get("zones", []), marks, toast.text])
	await _key(KEY_ESCAPE)
	# Leave one deer marked for the rest of the run (hunters may take it while the clock runs later).
	await _key(KEY_F7)
	await _key(KEY_3)
	drawn = _animals.call("drawn_position", int(deer["id"]))
	await _click(_screen(drawn + Vector2(View.TILE, View.TILE) * 0.5))
	await _key(KEY_ESCAPE)


## D drafts, right clicks order, D and 모두 해제 release.
func _draft_orders(hearth: Vector2i) -> void:
	var adults: Array[Dictionary] = _adults()
	if adults.size() < 3:
		_check("adults", false, "%d adults" % adults.size())
		return
	var first: Dictionary = adults[0]
	var second: Dictionary = adults[1]
	var other: Dictionary = adults[2]
	await _look_at(first["tile"])
	await _inspect(first)
	await _key(KEY_D)
	await _frames(2)
	var drafted: PackedInt64Array = _drafted_ids()
	var marks: Dictionary = _agents.call("person_marks", int(first["bits"]))
	_check("d_drafts_selected", drafted.size() == 1 and drafted[0] == int(first["id"]) and bool(marks.get("drafted", false)),
		"drafted %s marks %s" % [drafted, marks])
	var lines: Dictionary = _inspector.call("m5_lines")
	_check("inspector_draft_line", _readable(str(lines["draft"])), "'%s'" % lines["draft"])
	await _frames(2)
	var rows: Array = _banner.call("drafted_texts")
	_check("hud_drafted_list", rows.size() == 1 and _readable(str(rows[0])), str(rows))
	# Right click on an empty tile: the inspector's drafted person moves there.
	var spot: Vector2i = _empty_tile(first["tile"])
	await _click(_tile_screen(spot), MOUSE_BUTTON_RIGHT)
	await _frames(2)
	var m5: Dictionary = _m5(int(first["id"]))
	_check("right_click_move", int(m5.get("draft_order", -1)) == 1 and int(m5.get("draft_x", -1)) == spot.x and int(m5.get("draft_y", -1)) == spot.y,
		"order %d (%d, %d) want %s toast '%s'" % [int(m5.get("draft_order", -1)), int(m5.get("draft_x", -1)), int(m5.get("draft_y", -1)), spot, (_bar.get("_toast") as Label).text])
	# Right click on another person: attack them.
	await _look_at(other["tile"])
	var other_drawn: Vector2 = _agents.call("drawn_position", int(other["bits"]))
	await _click(_screen(other_drawn + Vector2(View.TILE, View.TILE) * 0.5), MOUSE_BUTTON_RIGHT)
	await _frames(2)
	m5 = _m5(int(first["id"]))
	_check("right_click_attack_person", int(m5.get("draft_order", -1)) == 3 and int(m5.get("draft_x", -1)) == int(other["id"]),
		"order %d target %d want %d" % [int(m5.get("draft_order", -1)), int(m5.get("draft_x", -1)), int(other["id"])])
	# Draft a second person, close the inspector: a right click on a deer orders both.
	await _inspect(second)
	await _key(KEY_D)
	_inspector.call("close")
	await _frames(2)
	var deer: Dictionary = _deer_near(hearth)
	if not deer.is_empty():
		await _look_at(deer["tile"])
		var drawn: Vector2 = _animals.call("drawn_position", int(deer["id"]))
		await _click(_screen(drawn + Vector2(View.TILE, View.TILE) * 0.5), MOUSE_BUTTON_RIGHT)
		await _frames(2)
		var a: Dictionary = _m5(int(first["id"]))
		var b: Dictionary = _m5(int(second["id"]))
		_check("right_click_attack_animal_all", _drafted_ids().size() == 2 and int(a.get("draft_order", -1)) == 2 and int(b.get("draft_order", -1)) == 2
			and int(a.get("draft_x", -1)) == int(deer["id"]) and _readable((_bar.get("_toast") as Label).text),
			"drafted %s orders %d/%d target %d deer %d toast '%s'" % [_drafted_ids(), int(a.get("draft_order", -1)), int(b.get("draft_order", -1)),
			int(a.get("draft_x", -1)), int(deer["id"]), (_bar.get("_toast") as Label).text])
	await _frames(2)
	rows = _banner.call("drafted_texts")
	var rows_ok: bool = rows.size() == 2
	for row: String in rows:
		rows_ok = rows_ok and _readable(row)
	_check("hud_drafted_rows", rows_ok, " | ".join(rows))
	# D on the first again: released.
	await _inspect(first)
	await _key(KEY_D)
	await _frames(2)
	_check("d_releases", _drafted_ids().size() == 1 and _drafted_ids()[0] == int(second["id"]) and not bool((_agents.call("person_marks", int(first["bits"])) as Dictionary).get("drafted", true)),
		"drafted %s" % _drafted_ids())
	_inspector.call("close")
	# 모두 해제 (a click on the drafted list's button).
	await _frames(2)
	var undraft: Button = (_banner as Control).find_child("UndraftAll", true, false) as Button
	if undraft != null and undraft.is_visible_in_tree():
		await _click(_ui(undraft))
		await _frames(2)
	_check("undraft_all_button", undraft != null and _drafted_ids().is_empty() and _readable((_bar.get("_toast") as Label).text),
		"drafted %s toast '%s'" % [_drafted_ids(), (_bar.get("_toast") as Label).text])


## M: the mood overlay; the inspector's mood and apparel lines; the 이야기 tab.
func _mood_and_inspector() -> void:
	# MoodSystem runs at minute 7 of each hour (M5 DECISIONS D18): give it its first run before reading moods.
	SimReads.mutate(_world, "advance_ticks", [10])
	await _frames(2)
	await _key(KEY_M)
	await _frames(2)
	var snap: Dictionary = SimReads.read(_world, "get_agent_snapshot")
	var bits: PackedInt64Array = snap.get("ids", PackedInt64Array())
	var bands: PackedByteArray = snap.get("mood_bands", PackedByteArray())
	var tinted: int = 0
	for i: int in mini(bits.size(), bands.size()):
		var marks: Dictionary = _agents.call("person_marks", bits[i])
		if marks.get("tint", Color.BLACK) == StoryText.band_tint(bands[i]):
			tinted += 1
	var legend: Label = (_banner as Control).find_child("MoodLegend", true, false) as Label
	_check("m_mood_overlay", bool(_agents.call("mood_shown")) and tinted == bits.size() and legend != null and legend.is_visible_in_tree() and _readable(legend.text),
		"tinted %d of %d bands %s legend '%s'" % [tinted, bits.size(), Array(bands), legend.text if legend != null else ""])
	await _key(KEY_M)
	_check("m_toggles_off", not bool(_agents.call("mood_shown")), "")
	var adults: Array[Dictionary] = _adults()
	await _inspect(adults[0])
	var lines: Dictionary = _inspector.call("m5_lines")
	var m5: Dictionary = _m5(int(adults[0]["id"]))
	_check("inspector_mood_line", _readable(str(lines["mood"])) and str(lines["mood"]).contains(str(int(m5.get("mood", -1)))), "'%s' mood %d mods %s" % [lines["mood"], int(m5.get("mood", -1)), m5.get("modifier_names", [])])
	var want_top: String = StoryText.apparel_line(_locale, m5)
	_check("inspector_apparel_line", _readable(str(lines["apparel"])) and str(lines["apparel"]) == want_top, "'%s' top %d permille %d" % [lines["apparel"], int(m5.get("top", -1)), int(m5.get("top_cold_permille", -1))])
	var wrap: String = StoryText.apparel_line(_locale, {"top": 1, "top_cold_permille": 700})
	_check("apparel_wrap_text", wrap.contains("30") and _readable(wrap), "'%s'" % wrap)
	_inspector.call("close")


## P: the dress toggle; K: the teach dialog's suggestion.
func _dress_and_teach(priority: Node, teach: Node) -> void:
	await _key(KEY_P)
	await _frames(3)
	var dress: Button = (priority as Control).find_child("DressOrder", true, false) as Button
	var before: bool = bool(SimReads.read(_world, "get_story").get("dress_order", false))
	if dress != null and dress.is_visible_in_tree():
		await _click(_ui(dress))
		await _frames(2)
	var after: bool = bool(SimReads.read(_world, "get_story").get("dress_order", before))
	var pressed_after: bool = dress != null and dress.button_pressed
	if dress != null and dress.is_visible_in_tree():
		await _click(_ui(dress))
		await _frames(2)
	var back: bool = bool(SimReads.read(_world, "get_story").get("dress_order", not before))
	_check("dress_toggle_sends_order", dress != null and _readable(dress.text) and after != before and pressed_after == after and back == before,
		"'%s' %s -> %s -> %s" % [dress.text if dress != null else "", before, after, back])
	var hunt_title: Label = null
	for label: Node in (priority as Control).find_children("*", "Label", true, false):
		if (label as Label).text == EconomyText.work(_locale, "hunt"):
			hunt_title = label as Label
	_check("priority_hunt_column", hunt_title != null and _readable(hunt_title.text), hunt_title.text if hunt_title != null else "")
	await _key(KEY_P)
	teach.call("open_for", DOORWAY)
	await _frames(3)
	var chosen: Dictionary = teach.call("suggestion")
	var want: int = int(_world.call("get_teach_suggestion", int(chosen.get("teacher", -1)), DOORWAY))
	_check("teach_preselects_suggestion", want >= 0 and int(chosen.get("chosen", -2)) == want and int(chosen.get("suggested", -2)) == want, "%s want %d" % [chosen, want])
	teach.call("close")


## Kinds 29–37 and the M5 quarrel/fight in ko and en; the self-undraft toast; why lines from a synthetic chain.
func _chronicle_texts() -> void:
	var p: int = int(_adults()[0]["id"])
	var q: int = int(_adults()[1]["id"])
	var rows: Array[Dictionary] = [
		{"kind": 29, "subject": -1, "other": -1, "cause": 0, "value": 0}, {"kind": 29, "subject": -1, "other": -1, "cause": 1, "value": 1},
		{"kind": 29, "subject": -1, "other": -1, "cause": 2, "value": 3}, {"kind": 29, "subject": p, "other": -1, "cause": 3, "value": 0},
		{"kind": 30, "subject": -1, "other": -1, "cause": 0, "value": 1}, {"kind": 31, "subject": p, "other": -1, "cause": 1, "value": 3},
		{"kind": 31, "subject": p, "other": -1, "cause": 0, "value": 0}, {"kind": 32, "subject": p, "other": q, "cause": 1, "value": 4},
		{"kind": 33, "subject": p, "other": -1, "cause": 3, "value": 5 | (3 << 8) | (6 << 16)}, {"kind": 34, "subject": p, "other": -1, "cause": 0, "value": 0},
		{"kind": 34, "subject": p, "other": -1, "cause": 1, "value": 0}, {"kind": 34, "subject": p, "other": -1, "cause": 2, "value": 0},
		{"kind": 35, "subject": p, "other": -1, "cause": 0, "value": 2}, {"kind": 35, "subject": -1, "other": -1, "cause": 0, "value": 0},
		{"kind": 36, "subject": p, "other": -1, "cause": 0, "value": 0}, {"kind": 36, "subject": p, "other": q, "cause": 0, "value": 0},
		{"kind": 37, "subject": p, "other": -1, "cause": 0, "value": 0},
	]
	for c: int in range(1, 7):
		rows.append({"kind": 19, "subject": p, "other": q, "cause": c, "value": -60, "index": 3, "link": 1})
	var info: Dictionary = SimReads.read(_world, "get_world_info")
	for language: String in ["ko", "en"]:
		_locale.call("set_language", language)
		var texts: Array[String] = []
		var ok: bool = true
		for row: Dictionary in rows:
			var plain: String = _plain(Chronicle.line(row, -1, _locale, info, true, true))
			texts.append(plain)
			ok = ok and _readable(plain)
		var toast: String = Chronicle.self_undraft_plain(rows[11], _locale)
		ok = ok and _readable(toast)
		var why: Array[String] = Chronicle.why_lines({"links": PackedInt32Array([0, 1, 2, 1]), "codes": PackedInt64Array([0, 3, 0, 100]),
			"ticks": PackedInt64Array([100, -1, -1, -1]), "kinds": PackedInt32Array([19, -1, -1, -1]), "subjects": PackedInt64Array([p, -1, -1, -1]),
			"others": PackedInt64Array([q, -1, -1, -1]), "causes": PackedInt64Array([1, -1, -1, -1]), "values": PackedInt64Array([-60, 0, 0, 0])}, _locale, info)
		for line: String in why:
			ok = ok and _readable(_plain(line))
		_check("chronicle_m5_kinds_%s" % language, ok and rows.size() == 23 and why.size() == 4, " || ".join(texts) + " || " + toast + " || " + " ".join(why))
	_locale.call("set_language", "ko")
	var notable: Array = (load("res://scripts/ui/panels/chronicle_panel.gd") as GDScript).get_script_constant_map().get("NOTABLE", [])
	var all_kinds: bool = true
	for k: int in range(29, 38):
		all_kinds = all_kinds and notable.has(k)
	_check("chronicle_notable_m5", all_kinds and Chronicle.LAST_KIND >= 37, str(notable))


## Advance a day at a time to the first wolf pack and the first cold snap; check the banners, the pulse, the sick
## badge, a "→ 왜?" chain, the 이야기 tab and the year summary.
func _advance_and_story(chronicle: Node, why: Node) -> void:
	var info: Dictionary = SimReads.read(_world, "get_world_info")
	var per_day: int = maxi(1, int(info.get("ticks_per_day", 1440)))
	var started: int = Time.get_ticks_msec()
	var seen_wolves: bool = false
	var seen_cold: bool = false
	var seen_sick: bool = false
	var seen_year: bool = false
	var why_done: bool = false
	for day: int in MAX_DAYS:
		SimReads.mutate(_world, "advance_ticks", [per_day])
		await _frames(2)
		var story: Dictionary = SimReads.read(_world, "get_story")
		var kinds: PackedInt64Array = story.get("kinds", PackedInt64Array())
		if not seen_wolves and int(story.get("wolves", 0)) > 0:
			seen_wolves = true
			await _check_wolves(story)
		if not seen_cold and kinds.has(StoryText.COLD_SNAP):
			seen_cold = true
			_check_cold(story)
		if not seen_sick and int(story.get("sick", 0)) > 0:
			seen_sick = _check_sick()
		if not why_done:
			why_done = await _check_why(chronicle, why)
		if not seen_year:
			seen_year = _check_year(chronicle)
		if seen_wolves and seen_cold and why_done and seen_year:
			break
	_info("advance", "tick %d after %d ms; wolves %s cold %s sick %s why %s year %s" % [_tick(), Time.get_ticks_msec() - started, seen_wolves, seen_cold,
		seen_sick, why_done, seen_year])
	_check("reached_wolves", seen_wolves, "by display day %d" % [Chronicle.day(_tick(), info)])
	_check("reached_cold_snap", seen_cold, "by display day %d" % [Chronicle.day(_tick(), info)])
	_check("why_chain_seen", why_done, "")
	_check("year_summary_seen", seen_year, "")
	if not seen_sick:
		_info("sick_badge", "no illness in force on a day boundary before day %d" % Chronicle.day(_tick(), info))


func _check_wolves(story: Dictionary) -> void:
	await _frames(2)
	var texts: Array = _banner.call("banner_texts")
	var want: String = StoryText.trf(_locale, "M5_BANNER_WOLVES_FMT", {"count": int(story.get("wolves", 0))})
	_check("wolves_banner", texts.has(want) and _readable(want), "%s want '%s' (wolves %d pack %d at %d,%d)" % [texts, want, int(story.get("wolves", 0)),
		int(story.get("pack_size", 0)), int(story.get("wolves_x", -1)), int(story.get("wolves_y", -1))])
	var pulse: Dictionary = _banner.call("pulse")
	_check("wolves_pulse", bool(pulse.get("shown", false)), str(pulse))
	var animals: Dictionary = SimReads.read(_world, "get_animals")
	var wolves: int = 0
	for k: int in animals.get("kinds", PackedInt64Array()):
		if k == StoryText.WOLF:
			wolves += 1
	_check("wolves_drawn", wolves == int(story.get("wolves", -1)) and int((_animals.call("counts") as Dictionary)["animals"]) == (animals.get("ids", PackedInt64Array()) as PackedInt64Array).size(),
		"wolves %d drawn %s" % [wolves, _animals.call("counts")])


func _check_cold(story: Dictionary) -> void:
	var kinds: PackedInt64Array = story.get("kinds", PackedInt64Array())
	var days: PackedInt64Array = story.get("days_of", PackedInt64Array())
	var i: int = kinds.find(StoryText.COLD_SNAP)
	var want: String = StoryText.trf(_locale, "M5_BANNER_COLD_FMT", {"day": int(days[i]) if i < days.size() else -1, "wood": int(story.get("firewood", -1)),
		"need": int(story.get("firewood_day_need", -1))})
	var banners: Array[Dictionary] = StoryText.banners(_locale, story)
	var texts: Array[String] = []
	for banner: Dictionary in banners:
		texts.append(str(banner["text"]))
	_check("cold_snap_banner", texts.has(want) and _readable(want) and int(story.get("firewood_day_need", 0)) > 0, "%s want '%s'" % [texts, want])


func _check_sick() -> bool:
	var snap: Dictionary = SimReads.read(_world, "get_agent_snapshot")
	var bits: PackedInt64Array = snap.get("ids", PackedInt64Array())
	var sick: PackedByteArray = snap.get("sick", PackedByteArray())
	var ids: PackedInt64Array = snap.get("agent_ids", PackedInt64Array())
	for i: int in mini(bits.size(), sick.size()):
		if sick[i] != 0:
			var marks: Dictionary = _agents.call("person_marks", bits[i])
			var line: String = StoryText.sick_line(_locale, _m5(ids[i]), 60)
			_check("sick_badge_and_line", bool(marks.get("sick", false)) and _readable(line), "agent %d marks %s line '%s'" % [ids[i], marks, line])
			return true
	return false


## The first chronicle entry with a cause link: its "→ 왜?" in the panel opens the chain; the 이야기 tab of its subject.
func _check_why(chronicle: Node, why: Node) -> bool:
	var page: Dictionary = _world.call("get_chronicle_since", 0, 512)
	var rows: Array[Dictionary] = Chronicle.rows(page)
	var pick: Dictionary = {}
	for row: Dictionary in rows:
		if int(row.get("link", -1)) >= 0 and int(row.get("kind", -1)) in [19, 30, 32, 33, 35, 36] and int(row.get("subject", -1)) >= 0:
			pick = row
			break
	if pick.is_empty():
		for row: Dictionary in rows:
			if int(row.get("link", -1)) >= 0:
				pick = row
				break
	if pick.is_empty():
		return false
	if not (chronicle as Control).visible:
		await _key(KEY_C)
	await _frames(2)
	var feed: String = str(chronicle.call("plain_text"))
	var link: String = StoryText.ltr(_locale, "M5_CHRON_WHY")
	var body: RichTextLabel = chronicle.get("_body") as RichTextLabel
	_check("chronicle_why_link_shown", feed.contains(link) and _readable(feed.substr(0, 400)), "kind %d index %d: '%s'" % [int(pick["kind"]), int(pick["index"]), feed.substr(0, 300)])
	body.meta_clicked.emit("why:%d" % int(pick["index"]))
	await _frames(2)
	var read: Dictionary = _world.call("get_chronicle_why", int(pick["index"]))
	var lines: Array = why.call("lines")
	var plain: String = str(why.call("plain_text"))
	_check("why_chain_opens", (why as Control).visible and int(why.call("current_index")) == int(pick["index"]) \
		and lines.size() == (read.get("links", PackedInt32Array()) as PackedInt32Array).size() and lines.size() >= 2 and _readable(plain),
		"links %s lines %d: '%s'" % [read.get("links", []), lines.size(), plain.replace("\n", " ")])
	why.call("close")
	await _key(KEY_C)
	# The 이야기 tab of the entry's subject (or other).
	var who: int = int(pick.get("subject", -1))
	if who < 0:
		who = int(pick.get("other", -1))
	if who >= 0:
		_inspector.call("show_person", who)
		_inspector.call("select_tab", 2)
		await _frames(3)
		var story: String = str(_inspector.call("story_text"))
		var expect_link: bool = int(_inspector.get("_selected_entity_bits")) >= 0
		_check("story_tab_lists_lines", int(_inspector.call("current_tab")) == 2 and _readable(story) and (not expect_link or story.contains(link)),
			"person %d alive %s: '%s'" % [who, expect_link, story.substr(0, 300).replace("\n", " ")])
		_inspector.call("select_tab", 0)
		_inspector.call("close")
	return true


func _check_year(chronicle: Node) -> bool:
	var info: Dictionary = SimReads.read(_world, "get_world_info")
	var per_year: int = int(info.get("ticks_per_year", 0))
	if per_year <= 0 or _tick() < per_year + 60:
		return false
	chronicle.call("toggle_visible")
	chronicle.call("_refresh")
	var shown: Dictionary = chronicle.call("year_summary")
	var summary: Dictionary = _world.call("get_year_summary", 0)
	chronicle.call("toggle_visible")
	_check("year_summary_header", int(shown.get("year", -1)) == 0 and _readable(str(shown.get("text", ""))) and str(shown["text"]) == StoryText.year_line(_locale, summary),
		"%s" % shown)
	return true


func _finish() -> void:
	print("M5_UI_DONE failures=%d checks=%d %s" % [_failures.size(), _checks, ",".join(_failures)])
	quit(0 if _failures.is_empty() else 1)
