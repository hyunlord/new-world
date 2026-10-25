# mart2_ui_test.gd — the M-ART2 UI test (goal §3.6 and §3.5's night check; windowed: real clicks, frames and pixels).
# Purpose: on a seed 0 day save (`--day`, a .wsave from capture.gd --mart2-set) and the day-6 02:00 save (`--night`),
#   with the clock stopped, checks as a player would: a click on a person opens the inspector with that person's
#   POLISH2 portrait (128 px, their look folder and face variant) and the look in words; clicking a second person
#   changes it; the selected and the drafted outline; the HUD zoom indicator through the wheel and the keys (1×, ½×, ¼×,
#   2×), half-scale figures and quarter-scale markers; F12 re-reads render_params.json, shows "렌더 설정 다시 읽음" and hides it; every
#   item kind and every order-bar icon resolves to imported art; workshop recipe buttons carry their output's icon;
#   the order tool's hover, footprint and wall-drag preview work on 64-px tiles with 64-px ghost pieces; at night the
#   global tone is the night colour (value × 0.55 with the blue shift), the hearth light is on (off by day), and people
#   stay separable from the ground (Rec.709 luminance contrast between a person's opaque pixels and the ground beside
#   them, median over the people on the night screen ≥ 0.15).
#   Prints MART2_UI PASS/FAIL lines and MART2_UI_DONE checks=N failures=M; exits 1 on any failure.
#   `Godot --path . --rendering-driver opengl3 --resolution 1920x1080 --script res://tools/v8/mart2_ui_test.gd -- \
#      --day <abs .wsave> --night <abs .wsave> [--shots <abs dir>]`
# Bridge methods called: load_world, cmd_draft, advance_ticks, get_place_preview (directly); the panels and layers
#   read the rest through SimReads.
extends SceneTree

const View = preload("res://scripts/world/colony/view.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
const ItemIcons = preload("res://scripts/ui/item_icons.gd")
const Daylight = preload("res://scripts/world/daylight_controller.gd")
const AgentLayer = preload("res://scripts/world/colony/agent_layer.gd")
const MIN_CHECKS: int = 25
const NIGHT_CONTRAST_BAR: float = 0.15

var _checks: int = 0
var _failures: int = 0
var _main: Node
var _world: Node
var _camera: Camera2D
var _agents: Node
var _shots: String = ""


func _init() -> void:
	call_deferred("_run")


func _arg(name: String) -> String:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	var i: int = args.find(name)
	return args[i + 1] if i >= 0 and i + 1 < args.size() else ""


func _check(name: String, ok: bool, detail: String) -> void:
	_checks += 1
	print("MART2_UI %s %s — %s" % ["PASS" if ok else "FAIL", name, detail])
	if not ok:
		_failures += 1


func _frames(n: int) -> void:
	for _i: int in n:
		await process_frame


func _screen(world_pos: Vector2) -> Vector2:
	return root.get_viewport().get_canvas_transform() * world_pos


func _motion(at: Vector2, held: bool = false) -> void:
	var m: InputEventMouseMotion = InputEventMouseMotion.new()
	m.position = at
	m.global_position = at
	if held:
		m.button_mask = MOUSE_BUTTON_MASK_LEFT
	Input.parse_input_event(m)
	await process_frame


func _button(at: Vector2, index: MouseButton, pressed: bool) -> void:
	var b: InputEventMouseButton = InputEventMouseButton.new()
	b.position = at
	b.global_position = at
	b.button_index = index
	b.pressed = pressed
	Input.parse_input_event(b)
	await process_frame


func _click(at: Vector2) -> void:
	await _motion(at)
	await _button(at, MOUSE_BUTTON_LEFT, true)
	await _button(at, MOUSE_BUTTON_LEFT, false)


func _key(code: Key) -> void:
	for pressed: bool in [true, false]:
		var k: InputEventKey = InputEventKey.new()
		k.keycode = code
		k.physical_keycode = code
		k.pressed = pressed
		Input.parse_input_event(k)
		await process_frame


func _load(path: String) -> bool:
	if path == "" or not FileAccess.file_exists(path):
		return false
	var r: Variant = _world.call("load_world", path)
	_world.process_mode = Node.PROCESS_MODE_DISABLED
	await _frames(40)
	return r is bool and bool(r)


func _shot(name: String) -> void:
	if _shots == "":
		return
	await RenderingServer.frame_post_draw
	root.get_viewport().get_texture().get_image().save_png(_shots.path_join(name))


func _run() -> void:
	_shots = _arg("--shots")
	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
	_main = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(_main)
	_world = _main.get_node("WorldSim")
	_camera = _main.get_node("Camera2D")
	_agents = _main.get_node("ColonyView/AgentLayer")
	await _frames(10)
	var loaded: bool = await _load(_arg("--day"))
	_check("day_save_loads", loaded, _arg("--day"))
	if loaded:
		await _people_and_portraits()
		await _zoom_and_lod()
		await _params_reload()
		await _icons()
		await _preview()
		await _night(_arg("--night"))
	var enough: bool = _checks >= MIN_CHECKS
	print("MART2_UI_DONE checks=%d failures=%d%s" % [_checks, _failures, "" if enough else " (fewer than %d checks)" % MIN_CHECKS])
	quit(1 if _failures > 0 or not enough else 0)


## People drawn now (awake only when asked), nearest the camera first: [bits, feet].
func _people(awake_only: bool) -> Array:
	var out: Array = []
	var persons: Dictionary = _agents.get("_persons")
	for b: int in persons.keys():
		var look: Dictionary = _agents.call("person_look", b)
		if look.is_empty() or (awake_only and str(look.get("activity", "")) == "sleep"):
			continue
		out.append([b, look["feet"]])
	var c: Vector2 = _camera.position
	out.sort_custom(func(a: Array, d: Array) -> bool: return (a[1] as Vector2).distance_to(c) < (d[1] as Vector2).distance_to(c))
	return out


## People whose activity carries an LOD glyph (agent_layer.gd GLYPH_OF).
func _glyph_people() -> int:
	var n: int = 0
	var glyph_of: Dictionary = AgentLayer.GLYPH_OF
	for b: int in (_agents.get("_persons") as Dictionary).keys():
		if glyph_of.has(str((_agents.call("person_look", b) as Dictionary).get("activity", ""))):
			n += 1
	return n


func _outline(bits: int) -> Color:
	var persons: Dictionary = _agents.get("_persons")
	var p: Object = persons.get(bits)
	return p.get("outline") if p != null else Color(0, 0, 0, 0)


func _people_and_portraits() -> void:
	_camera.zoom = Vector2.ONE
	var snap: Dictionary = SimReads.read(_world, "get_agent_snapshot")
	var snap_ids: PackedInt64Array = snap.get("ids", PackedInt64Array())
	await _frames(5)
	_check("people_drawn_equal_snapshot", snap_ids.size() > 0 and int(_agents.call("person_count")) == snap_ids.size(), "%d drawn, %d in the snapshot" % [_agents.call("person_count"), snap_ids.size()])
	var people: Array = _people(true)
	if people.size() < 2:
		_check("two_awake_people", false, "%d awake people" % people.size())
		return
	_camera.position = people[0][1]
	await _frames(10)
	var inspector: Control = _main.get_node("UI/AgentInspectorPanel")
	var first: int = people[0][0]
	await _click(_screen((_agents.call("drawn_rect", first) as Rect2).get_center()))
	await _frames(6)
	var selected: int = int(inspector.call("get_selected_entity_bits"))
	_check("click_opens_inspector", inspector.visible and selected == first, "visible %s, selected %d (clicked %d)" % [inspector.visible, selected, first])
	var look: Dictionary = _agents.call("person_look", first)
	var state: Dictionary = inspector.call("portrait_state")
	var want: String = ArtData.portrait_path(str(look["frames"]), "v%d" % (int(look["variant"]) + 1))
	_check("portrait_shown", str(state["path"]).ends_with(".png"), "portrait %s" % state["path"])
	_check("portrait_is_the_persons_look_and_face", want != "" and state["path"] == want, "%s vs %s" % [state["path"], want])
	_check("portrait_128px", state["size"] == Vector2(128, 128), "texture %s" % state["size"])
	var portrait: Control = inspector.find_child("Portrait", true, false) as Control
	_check("portrait_on_screen", portrait != null and portrait.is_visible_in_tree() and portrait.size.x >= 128.0, "rect %s" % (portrait.get_global_rect() if portrait != null else Rect2()))
	var row: Dictionary = ArtData.look(int(look["look"]))
	var locale: Node = root.get_node_or_null("Locale")
	var top_word: String = str(locale.call("ltr", "LOOK_TOP_" + str(row.get("top", "")).to_upper())) if locale != null else ""
	var hair_word: String = str(locale.call("ltr", "LOOK_HAIR_" + str(row.get("hair", "")).to_upper())) if locale != null else ""
	var semantic_look: String = str(state["look"]).replace("\u2060", "")
	_check("look_line_names_the_top", top_word != "" and not top_word.begins_with("LOOK_") and semantic_look.contains(top_word.replace("\u2060", "")), "line '%s', top '%s'" % [state["look"], top_word])
	_check("look_line_names_the_hair", hair_word != "" and not hair_word.begins_with("LOOK_") and semantic_look.contains(hair_word.replace("\u2060", "")), "hair '%s'" % hair_word)
	_check("look_line_names_the_face", str(state["look"]).contains(str(int(look["variant"]) + 1)), "face %d" % (int(look["variant"]) + 1))
	var sel: Color = _outline(first)
	_check("selected_outline", sel.a > 0.0, "outline %s" % sel)
	await _shot("mart2_ui_01_inspector.png")
	var other: int = -1
	for p: Array in people.slice(1):
		var l: Dictionary = _agents.call("person_look", p[0])
		if int(l["look"]) != int(look["look"]) or int(l["variant"]) != int(look["variant"]):
			other = p[0]
			_camera.position = p[1]
			break
	await _frames(10)
	if other >= 0:
		await _click(_screen((_agents.call("drawn_rect", other) as Rect2).get_center()))
		await _frames(6)
		var state2: Dictionary = inspector.call("portrait_state")
		_check("portrait_follows_selection", state2["path"] != state["path"] and int(inspector.call("get_selected_entity_bits")) == other, "%s → %s" % [str(state["path"]).get_file(), state2["path"]])
		_check("selection_outline_moves", _outline(first).a == 0.0 and _outline(other).a > 0.0, "first %s, second %s" % [_outline(first), _outline(other)])
	else:
		_check("portrait_follows_selection", false, "no second look or face among %d people" % people.size())
	# Drafting one more adult marks them with the drafted outline (a different colour from the selection).
	var ids: PackedInt64Array = snap.get("agent_ids", PackedInt64Array())
	var stages: PackedByteArray = snap.get("life_stages", PackedByteArray())
	var di: int = -1
	for i: int in mini(ids.size(), mini(snap_ids.size(), stages.size())):
		if stages[i] == 0 and snap_ids[i] != other and snap_ids[i] != first and (_agents.get("_persons") as Dictionary).has(snap_ids[i]):
			di = i
			break
	if di >= 0:
		var r: Dictionary = _world.call("cmd_draft", ids[di], true)
		_world.call("advance_ticks", 1)
		await _frames(6)
		var marks: Dictionary = _agents.call("person_marks", snap_ids[di])
		var o: Color = _outline(snap_ids[di])
		_check("drafted_outline", bool(r.get("accepted", false)) and bool(marks.get("drafted", false)) and o.a > 0.0 and o != _outline(other), "drafted %s, outline %s vs selected %s" % [marks.get("drafted"), o, _outline(other)])
		_world.call("cmd_draft", ids[di], false)
		_world.call("advance_ticks", 1)
	else:
		_check("drafted_outline", false, "no third adult")
	inspector.call("close")
	await _frames(3)


func _zoom_and_lod() -> void:
	var hud: Node = _main.get_node("UI/HudTopbar")
	var markers: Node = _agents.get_node("LodMarkers")
	_camera.zoom = Vector2.ONE
	_camera.call("center_on_camp")
	await _frames(5)
	_check("zoom_indicator_1x", str(hud.call("zoom_text")).contains("1×") and not bool(_agents.call("lod_active")), "'%s', lod %s" % [hud.call("zoom_text"), _agents.call("lod_active")])
	var centre: Vector2 = Vector2(960, 600)
	await _motion(centre)
	for z: Array in [[0.5, "½×"], [0.25, "¼×"]]:
		await _button(centre, MOUSE_BUTTON_WHEEL_DOWN, true)
		await _frames(4)
		_check("wheel_out_to_%s" % str(z[1]), absf(_camera.zoom.x - float(z[0])) < 0.001 and str(hud.call("zoom_text")).contains(str(z[1])), "zoom %.2f, '%s'" % [_camera.zoom.x, hud.call("zoom_text")])
		var counts: Array = markers.call("counts")
		var marker_mode: bool = float(z[0]) == 0.25
		_check("lod_at_%s" % str(z[1]), bool(_agents.call("lod_active")) == marker_mode and int(counts[0]) == (int(_agents.call("person_count")) if marker_mode else 0), "lod %s, markers %s of %d people" % [_agents.call("lod_active"), counts, _agents.call("person_count")])
		if marker_mode:
			var expected: int = _glyph_people()
			_check("activity_glyphs_at_¼×", int(counts[1]) == expected, "glyphs %d, people working/carrying/asleep/fleeing %d" % [int(counts[1]), expected])
		else:
			await _shot("mart2_ui_02_half_figures.png")
	for _k: int in 3:
		await _key(KEY_EQUAL)
	await _frames(4)
	_check("keys_in_to_2x", absf(_camera.zoom.x - 2.0) < 0.001 and str(hud.call("zoom_text")).contains("2×") and not bool(_agents.call("lod_active")) and int((markers.call("counts") as Array)[0]) == 0, "zoom %.2f '%s' lod %s" % [_camera.zoom.x, hud.call("zoom_text"), _agents.call("lod_active")])
	await _key(KEY_MINUS)
	await _frames(3)
	_check("minus_steps_out", absf(_camera.zoom.x - 1.0) < 0.001, "zoom %.2f" % _camera.zoom.x)
	var label: Control = hud.find_child("Zoom", true, false) as Control
	_check("zoom_label_in_top_bar", label != null and label.is_visible_in_tree() and label.get_global_rect().position.y < 160.0, "rect %s" % (label.get_global_rect() if label != null else Rect2()))


func _params_reload() -> void:
	var ground: Node = _main.get_node("ColonyView/GroundLayer")
	var strips_before: int = int(ground.call("strip_count"))
	var version: int = ArtData.params_version()
	await _key(KEY_F12)
	await _frames(2)
	var sets: int = (ground.call("wall_baker").get("sets") as Dictionary).size()
	_check("f12_keeps_walls", strips_before > 0 and int(ground.call("strip_count")) == strips_before and sets > 0, "wall/hill strips %d → %d, wall sets %d" % [strips_before, int(ground.call("strip_count")), sets])
	_check("f12_refreshes_hill_cap", bool(ground.call("hill_cap_is_current")), "hill cap = the re-prepared hill ground")
	var toast: String = str(_camera.call("toast_text"))
	_check("f12_rereads_params", ArtData.params_version() == version + 1, "version %d → %d" % [version, ArtData.params_version()])
	_check("f12_toast", toast == "렌더 설정 다시 읽음", "toast '%s'" % toast)
	var t0: int = Time.get_ticks_msec()
	while Time.get_ticks_msec() - t0 < 3500:
		await process_frame
	_check("toast_hides", str(_camera.call("toast_text")) == "", "after 3.5 s: '%s'" % _camera.call("toast_text"))


func _icons() -> void:
	var missing: Array[int] = []
	var codes: int = 0
	for r: Variant in (ArtData.bindings().get("items", {}) as Dictionary).get("codes", []):
		var code: int = int((r as Dictionary)["code"])
		codes += 1
		var rect: TextureRect = ItemIcons.icon(code)
		if rect.texture == null or rect.custom_minimum_size != Vector2(32, 32):
			missing.append(code)
		rect.free()
	_check("item_icons_resolve", codes >= 10 and missing.is_empty(), "%d item kinds, missing %s" % [codes, missing])
	var order: Dictionary = ArtData.bindings().get("order_icons", {})
	var named: int = 0
	var bad: Array[String] = []
	for k: String in order.keys():
		if k == "about" or order[k] == null:
			continue
		named += 1
		if ArtData.texture(str(order[k])) == null:
			bad.append(k)
	var with_icon: int = 0
	for n: Node in _main.get_node("UI/OrderBar").find_children("*", "Button", true, false):
		if (n as Button).icon != null:
			with_icon += 1
	_check("order_bar_icons", named >= 5 and bad.is_empty() and with_icon >= named, "%d bound icons (missing %s), %d order buttons carry one" % [named, bad, with_icon])
	var shops: Dictionary = SimReads.read(_world, "get_workshops")
	var panel: Node = _main.get_node("UI/WorkshopPanel")
	var sids: PackedInt32Array = shops.get("ids", PackedInt32Array())
	if sids.size() > 0:
		panel.call("display_workshop", sids[0])
		await _frames(4)
		panel.call("_toggle_recipes")
		await _frames(4)
		var with_item: int = 0
		var total: int = 0
		for n: Node in panel.find_children("Recipe_*", "Button", true, false):
			total += 1
			var b: Button = n
			if b.icon != null and b.icon == ArtData.texture(ItemIcons.asset_of(int(b.get_meta("item_kind", -1)))):
				with_item += 1
		_check("workshop_recipe_icons", total > 0 and with_item == total, "%d of %d recipe buttons carry their output's icon" % [with_item, total])
		panel.call("close")
		await _frames(2)
	else:
		_check("workshop_recipe_icons", false, "no workshop in the day save")


func _preview() -> void:
	var tool: Node = _main.get_node("ColonyView/OrderTool")
	_camera.zoom = Vector2.ONE
	_camera.call("center_on_camp")
	await _frames(5)
	# A free row of four tiles below the camera centre: the bridge's own preview read decides.
	var centre: Vector2i = View.world_to_tile(_camera.position)
	tool.call("set_build", "log_wall")
	var a: Vector2i = Vector2i(-1, -1)
	for dy: int in range(2, 9):
		for dx: int in range(-6, 6):
			var t: Vector2i = centre + Vector2i(dx, dy)
			var p: Dictionary = _world.call("get_place_preview", "log_wall", t.x, t.y, t.x + 3, t.y, 1)
			var ok: PackedByteArray = p.get("ok", PackedByteArray())
			if ok.size() == 4 and ok.count(1) == 4:
				a = t
				break
		if a.x >= 0:
			break
	if a.x < 0:
		_check("wall_drag_preview", false, "no free 4-tile row near the camp")
		tool.call("set_mode", 0)
		return
	var sa: Vector2 = _screen(View.tile_centre(a))
	await _motion(sa)
	await _frames(2)
	_check("tool_hover_on_64px_tiles", tool.get("_hover") == a, "hover %s, tile %s (screen %s)" % [tool.get("_hover"), a, sa])
	await _button(sa, MOUSE_BUTTON_LEFT, true)
	var sb: Vector2 = _screen(View.tile_centre(a + Vector2i(3, 0)))
	for k: int in 4:
		await _motion(sa.lerp(sb, float(k + 1) / 4.0), true)
	await _frames(2)
	var prev: Dictionary = tool.call("preview")
	var pieces: Array = tool.get("_preview_draw")
	var sized: int = 0
	for piece: Array in pieces:
		var tex: Texture2D = piece[1]
		if tex != null and tex.get_width() == View.TILE:
			sized += 1
	var tiles: int = (prev.get("xs", PackedInt32Array()) as PackedInt32Array).size()
	_check("wall_drag_preview", tiles == 4, "preview tiles %d" % tiles)
	_check("preview_ghosts_64px", pieces.size() == 4 and sized == 4, "%d pieces, %d of them %d px wide" % [pieces.size(), sized, View.TILE])
	await _shot("mart2_ui_04_wall_preview.png")
	# Right-click leaves the mode without placing; the release then does nothing.
	await _button(sb, MOUSE_BUTTON_RIGHT, true)
	await _button(sb, MOUSE_BUTTON_RIGHT, false)
	await _button(sb, MOUSE_BUTTON_LEFT, false)
	var after: Dictionary = _world.call("get_place_preview", "log_wall", a.x, a.y, a.x + 3, a.y, 1)
	var still_free: bool = (after.get("ok", PackedByteArray()) as PackedByteArray).count(1) == 4
	_check("right_click_cancels_the_drag", int(tool.get("mode")) == 0 and still_free, "mode %d, row still free %s" % [int(tool.get("mode")), still_free])
	tool.call("set_build", "wood_bench")
	await _motion(sa + Vector2(1, 1))
	await _motion(sa)
	_check("workshop_footprint_and_origin", tool.call("build_size") == Vector2i(2, 1) and tool.call("build_origin", a) == a, "size %s origin %s" % [tool.call("build_size"), tool.call("build_origin", a)])
	tool.call("set_mode", 0)
	await _frames(2)


func _night(path: String) -> void:
	var daylight: Node = _main.get_node("DaylightController")
	var day_lights: Array = daylight.call("lights")
	var day_on: int = 0
	for l: Dictionary in day_lights:
		day_on += 1 if bool(l["enabled"]) else 0
	_check("hearth_light_off_by_day", day_lights.size() > 0 and day_on == 0, "%d lights, %d on by day" % [day_lights.size(), day_on])
	_camera.zoom = Vector2.ONE
	_camera.call("center_on_camp")
	await _frames(10)
	await _separability("day")
	var loaded: bool = await _load(path)
	_check("night_save_loads", loaded, path)
	if not loaded:
		return
	_camera.zoom = Vector2.ONE
	_camera.call("center_on_camp")
	await _frames(30)
	var mod: Color = (daylight.get_node("CanvasModulate") as CanvasModulate).color
	var want: Color = Daylight.night_colour()
	_check("night_tone", absf(mod.r - want.r) < 0.02 and absf(mod.b - want.b) < 0.02 and mod.b > mod.r, "modulate %s, night colour %s" % [mod, want])
	var lights: Array = daylight.call("lights")
	var on: int = 0
	for l: Dictionary in lights:
		on += 1 if bool(l["enabled"]) else 0
	_check("hearth_light_on_at_night", on > 0, "%d of %d lights on" % [on, lights.size()])
	await _shot("mart2_ui_03_night.png")
	_camera.zoom = Vector2(0.25, 0.25)
	await _frames(4)
	var sleepers: int = _glyph_people()
	var night_glyphs: int = int((_agents.get_node("LodMarkers").call("counts") as Array)[1])
	_check("sleep_glyphs_at_night_¼×", sleepers > 0 and night_glyphs == sleepers, "glyphs %d for %d people with a glyph activity" % [night_glyphs, sleepers])
	_camera.zoom = Vector2.ONE
	await _frames(6)
	var night: Array = await _separability("night")
	var edges: Array[float] = []
	for r: Dictionary in night:
		edges.append(float(r["contrast"]))
	edges.sort()
	@warning_ignore("integer_division")
	var median: float = edges[edges.size() / 2] if not edges.is_empty() else -1.0
	var lo: float = edges[0] if not edges.is_empty() else -1.0
	_check("night_person_ground_contrast", edges.size() >= 3 and median >= NIGHT_CONTRAST_BAR, "%d people on screen, silhouette (fill or rim) vs ground contrast min %.3f median %.3f (bar %.2f)" % [edges.size(), lo, median, NIGHT_CONTRAST_BAR])


## Per person on the screen now, Rec.709 luminance means of: their opaque frame pixels (fill), the 1-px rim just
## outside their silhouette (where a selection, drafted or night-halo outline is drawn; plain ground when none), and
## the ground 4–6 px outside the silhouette that no person or rim covers. Prints one MART2_UI_INFO line per person.
func _separability(tag: String) -> Array:
	await RenderingServer.frame_post_draw
	var img: Image = root.get_viewport().get_texture().get_image()
	var size: Vector2i = img.get_size()
	var persons: Dictionary = _agents.get("_persons")
	var cover: Dictionary = {}
	var shapes: Array = []
	for b: int in persons.keys():
		var px: Dictionary = _silhouette(img, persons[b] as Node)
		shapes.append([b, px])
		for q: Vector2i in px.keys():
			for dy: int in range(-1, 2):
				for dx: int in range(-1, 2):
					cover[q + Vector2i(dx, dy)] = true
	var out: Array = []
	for pair: Array in shapes:
		var px: Dictionary = pair[1]
		if px.size() < 40:
			continue
		var fill: float = 0.0
		var rim: Dictionary = {}
		var near: Dictionary = {}
		var far: Dictionary = {}
		for q: Vector2i in px.keys():
			fill += _lum(img.get_pixelv(q))
			var on_edge: bool = false
			for d: Vector2i in [Vector2i(1, 0), Vector2i(-1, 0), Vector2i(0, 1), Vector2i(0, -1)]:
				if not px.has(q + d):
					on_edge = true
			if not on_edge:
				continue
			for dy: int in range(-6, 7):
				for dx: int in range(-6, 7):
					var g: Vector2i = q + Vector2i(dx, dy)
					if px.has(g) or g.x < 0 or g.y < 0 or g.x >= size.x or g.y >= size.y:
						continue
					var d: int = maxi(absi(dx), absi(dy))
					if d == 1:
						rim[g] = true
					if d <= 3:
						near[g] = true
					else:
						far[g] = true
		var ground: float = 0.0
		var n_ground: int = 0
		for g: Vector2i in far.keys():
			if not near.has(g) and not cover.has(g):
				ground += _lum(img.get_pixelv(g))
				n_ground += 1
		if n_ground < 40 or rim.is_empty():
			continue
		var rim_l: float = 0.0
		for g: Vector2i in rim.keys():
			rim_l += _lum(img.get_pixelv(g))
		var row: Dictionary = {"bits": pair[0], "activity": str((_agents.call("person_look", pair[0]) as Dictionary).get("activity", "")),
			"fill": fill / px.size(), "rim": rim_l / rim.size(), "ground": ground / n_ground}
		row["contrast"] = maxf(absf(float(row["fill"]) - float(row["ground"])), absf(float(row["rim"]) - float(row["ground"])))
		print("MART2_UI_INFO %s %s fill %.3f rim %.3f ground %.3f contrast %.3f" % [tag, row["activity"], row["fill"], row["rim"], row["ground"], row["contrast"]])
		out.append(row)
	return out


## The screen pixels a person's opaque frame pixels cover ({Vector2i: true}); {} when hidden or off screen.
func _silhouette(img: Image, person: Node) -> Dictionary:
	var out: Dictionary = {}
	var body: Sprite2D = person.get("body")
	if body == null or body.texture == null or not body.is_visible_in_tree():
		return out
	var frame: Image = body.texture.get_image()
	if frame == null:
		return out
	if frame.is_compressed():
		frame.decompress()
	var xf: Transform2D = body.get_global_transform_with_canvas()
	var rect: Rect2 = body.get_rect()
	var w: int = frame.get_width()
	for fy: int in frame.get_height():
		for fx: int in w:
			if frame.get_pixel(fx, fy).a < 0.5:
				continue
			var lx: float = float(w - 1 - fx) if body.flip_h else float(fx)
			var sp: Vector2 = xf * (rect.position + Vector2(lx + 0.5, fy + 0.5))
			if sp.x < 0 or sp.y < 0 or sp.x >= img.get_width() or sp.y >= img.get_height():
				return {}
			out[Vector2i(sp)] = true
	return out


func _lum(c: Color) -> float:
	return 0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b
