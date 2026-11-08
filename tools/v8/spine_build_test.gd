# M-SPINE real-chain native input, window capture and construction regression checks.
# One fresh process per --window-size; no synthetic gallery or fabricated completion.
extends SceneTree

const View = preload("res://scripts/world/colony/view.gd")
const Content = preload("res://scripts/ui/content_context.gd")
const ItemIcons = preload("res://scripts/ui/item_icons.gd")
const STAGES: Array[String] = ["00placed", "01half", "02complete", "03zonegrown", "04upgraded"]
var _out: String = ""
var _saves: String = ""
var _livestock_saves: bool = false
var _size := Vector2i.ZERO
var _main: Node
var _world: Node
var _panel: Control
var _tool: Node
var _camera: Camera2D
var _locale: Node
var _rows: Array[Dictionary] = []
var _manifests: Dictionary = {}
var _failures: int = 0
var _gaps: int = 0
var _preview_changes: int = 0
var _pacing: Dictionary = {}

func _init() -> void:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	for i: int in args.size() - 1:
		if args[i] == "--out": _out = args[i + 1]
		elif args[i] == "--saves": _saves = args[i + 1]
		elif args[i] == "--livestock-saves":
			_saves = args[i + 1]
			_livestock_saves = true
		elif args[i] == "--window-size":
			var parts: PackedStringArray = args[i + 1].split("x")
			if parts.size() == 2: _size = Vector2i(int(parts[0]), int(parts[1]))
	_run.call_deferred()

func _check(label: String, ok: bool, observed: Dictionary = {}) -> void:
	_rows.append({"check": label, "passed": ok, "observed": observed})
	print("SPINE_BUILD %s %s" % ["PASS" if ok else "FAIL", label])
	if not ok: _failures += 1

func _gap(label: String) -> void:
	_gaps += 1
	_rows.append({"check": label, "status": "NOT_RUN_FIXTURE_GAP"})
	print("SPINE_BUILD NOT_RUN_FIXTURE_GAP " + label)

func _frames(count: int = 4) -> void:
	for _i: int in count: await process_frame

func _key(code: Key) -> void:
	for pressed: bool in [true, false]:
		var event := InputEventKey.new()
		event.keycode = code
		event.physical_keycode = code
		event.pressed = pressed
		Input.parse_input_event(event)
		await process_frame

func _point(logical: Vector2, click: bool = true) -> void:
	var point: Vector2 = root.get_final_transform() * logical
	var motion := InputEventMouseMotion.new()
	motion.position = point
	Input.parse_input_event(motion)
	await process_frame
	_rows.append({"native_pointer": str(point), "hovered_path": str(root.gui_get_hovered_control().get_path()) if root.gui_get_hovered_control() != null else "", "click": click})
	if not click: return
	for pressed: bool in [true, false]:
		var event := InputEventMouseButton.new()
		event.position = point
		event.button_index = MOUSE_BUTTON_LEFT
		event.pressed = pressed
		Input.parse_input_event(event)
		await process_frame
	await _frames()

func _click(button: Button) -> void:
	if button == null:
		_check("native target exists", false)
		return
	var node: Node = button.get_parent()
	while node != null:
		if node is ScrollContainer:
			node.ensure_control_visible(button)
			await _frames(2)
		node = node.get_parent()
	await _point(button.get_global_rect().get_center())

func _tile(tile: Vector2i, click: bool = true) -> void:
	await _point(_camera.get_canvas_transform() * View.tile_centre(tile), click)

func _capture(label: String) -> void:
	await RenderingServer.frame_post_draw
	var image: Image = root.get_texture().get_image()
	_check("capture " + label, image != null and image.save_png(_out.path_join(label + ".png")) == OK)

func _run() -> void:
	var settings: String = OS.get_environment("WORLDSIM_TEST_SETTINGS_DIR")
	var valid: bool = DisplayServer.get_name() != "headless" and _out.is_absolute_path() and _saves.is_absolute_path()
	valid = valid and not _out.begins_with(ProjectSettings.globalize_path("res://")) and settings.is_absolute_path() and DirAccess.dir_exists_absolute(settings)
	valid = valid and _size in [Vector2i(1920, 1080), Vector2i(1440, 900)]
	_check("realwindow external paths and requested supported size", valid)
	if not valid: _finish(); return
	DirAccess.make_dir_recursive_absolute(_out)
	_main = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(_main)
	await _frames(20)
	root.size = _size
	await _frames()
	_check("actual window equals requested after startup", DisplayServer.window_get_size() == _size)
	_world = _main.get_node("WorldSim")
	_camera = _main.get_node("Camera2D") as Camera2D
	_tool = _main.get_node("ColonyView/OrderTool")
	_locale = root.get_node("Locale")
	_panel = _main.get_node_or_null("UI/BuildPanel") as Control
	_check("integrated actual BuildPanel exists", _panel != null)
	if _panel == null: _finish(); return
	if _livestock_saves:
		await _livestock()
		_finish()
		return
	_option_unit_decision()
	_camera.call("set_paused", true)
	for language: String in ["ko", "en"]:
		_locale.call("set_language", language)
		for stage: String in STAGES:
			if not (await _load(stage)): continue
			_panel.call("open")
			await _frames()
			var groups: Array = _panel.get("_groups")
			_check(language + " " + stage + " three groups", groups.size() == 3)
			_fixed_labels(language)
			_option_membership(language + " " + stage)
			if stage == "00placed": await _reserved_zones(language)
			var count: int = (_world.call("get_content_catalog").get("buildings", []) as Array).size()
			_check("one catalogue row in available or next", groups.size() == 3 and groups[1].get_child_count() + groups[2].get_child_count() == count)
			await _capture(language + "_" + stage + "_groups")
			await _inert(language + "_" + stage)
			await _completed(language, stage)
			_panel.hide()
			if stage == "03zonegrown": await _housing(language)
			if stage == "04upgraded":
				await _history(language)
				await _home(language)
		if await _load("03-ready-upgrade"):
			var aux: Dictionary = _manifests["03-ready-upgrade"]
			_check("real ready-upgrade fixture", aux.get("ui", {}).get("native_action") == "upgrade" and bool(aux.get("ui", {}).get("upgrade_stock_checked", false)) and aux["selected_site"] == _manifests["03zonegrown"]["selected_site"])
			await _completed(language, "03-ready-upgrade")
			_panel.hide()
		if await _load("02complete"):
			await _placement(language)
			await _pace(language)
		await _cord_world(language)
	_finish()

func _reserved_zones(language: String) -> void:
	var zones: Array = _world.call("get_content_catalog").get("building_zones", [])
	_check(language + " three authored zone kinds", zones.size() == 3 and (_panel.get("_zone_buttons") as Array).size() == zones.size())
	var expected: Dictionary = {"housing": "Connected", "storage": "Inert", "workshop": "Inert"}
	_check(language + " authored zone order and identity", zones.map(func(row: Dictionary) -> String: return str(row["id"])) == expected.keys())
	var before: Dictionary = _world.call("get_command_log", 0)
	var sites: String = JSON.stringify(_world.call("get_building_snapshot").get("template_sites", []))
	for row: Dictionary in zones:
		var id: String = str(row["id"])
		var button: Button = _panel.find_child("Zone_" + id, true, false) as Button
		_check(language + " actual zone status " + id, expected.get(id) == row.get("status") and button != null)
		if button == null: continue
		var inert: bool = str(row.get("status")) == "Inert"
		var name: String = str(_locale.call("ltr", row["name_key"]))
		_check(language + " localized zone control " + id, name != str(row["name_key"]) and button.text.contains(name) and button.disabled == inert)
		if inert:
			_check(language + " reserved grey preparation and debt " + id, button.get_theme_color("font_disabled_color") == Color("807970") and button.text.contains(str(_locale.call("ltr", "BUILD_LOCK_INERT"))) and not str(row.get("debt", "")).is_empty() and button.tooltip_text.contains(str(row["debt"])))
			await _click(button)
			_check(language + " reserved native click no mutation " + id, _panel.visible and not bool(_tool.call("template_active")) and _world.call("get_command_log", 0) == before and JSON.stringify(_world.call("get_building_snapshot").get("template_sites", [])) == sites)

# Synthetic DTO decision check only; no simulated-world eligibility or capture claim.
func _option_unit_decision() -> void:
	var saved: Dictionary = _panel.get("_snapshot")
	var options: Array[Dictionary] = [{"id": "fixture", "station": "knapping_spot", "known": true, "affordable": false}, {"id": "fixture", "station": "wood_bench", "known": true, "affordable": true}]
	_panel.set("_snapshot", {"template_options": options})
	var best: Dictionary = _panel.call("_option", "fixture")
	var exact: Dictionary = _panel.call("_option", "fixture", "knapping_spot")
	var missing: Dictionary = _panel.call("_option", "fixture", "absent")
	options[1]["affordable"] = false
	var fallback: Dictionary = _panel.call("_option", "fixture")
	_panel.set("_snapshot", saved)
	_check("UNIT_DECISION synthetic DTO chooses affordable alternative", str(best.get("station", "")) == "wood_bench")
	_check("UNIT_DECISION explicit station stays exact or absent", str(exact.get("station", "")) == "knapping_spot" and missing.is_empty())
	_check("UNIT_DECISION all unavailable falls back to first", str(fallback.get("station", "")) == "knapping_spot")

func _option_membership(label: String) -> void:
	var snapshot: Dictionary = _world.call("get_building_snapshot")
	var groups: Array = _panel.get("_groups")
	for row: Dictionary in _world.call("get_content_catalog").get("buildings", []):
		var available: bool = false
		for option: Dictionary in snapshot.get("template_options", []):
			if str(option.get("id", "")) == str(row["id"]) and bool(option.get("known", false)) and bool(option.get("affordable", false)): available = true
		var card: Button = _panel.find_child("Template_" + str(row["id"]), true, false) as Button
		var expected: int = 1 if bool(row.get("connected", false)) and available else 2
		_check(label + " real option membership " + str(row["id"]), card != null and groups.size() == 3 and card.get_parent() == groups[expected])

func _load(stage: String) -> bool:
	var path: String = _saves.path_join(stage + ".json")
	var manifest: Variant = JSON.parse_string(FileAccess.get_file_as_string(path)) if FileAccess.file_exists(path) else null
	_check("canonical manifest " + stage, manifest is Dictionary)
	if manifest is not Dictionary: return false
	_manifests[stage] = manifest
	var ok: bool = bool(SimReads.mutate(_world, "load_world", [_saves.path_join(stage + ".wsave")]))
	_check("canonical save " + stage, ok)
	if not ok: return false
	_camera.call("set_paused", true)
	await _frames()
	_check("manifest tick " + stage, int(_world.call("get_time_snapshot").get("tick", -1)) == int(manifest["tick"]))
	return true

func _inert(language: String) -> void:
	var definitions: Array = _world.call("get_content_catalog").get("buildings", [])
	var inert: Dictionary = {}
	for value: Dictionary in definitions:
		if not bool(value.get("connected", false)): inert = value; break
	_check("actual inert catalogue row", not inert.is_empty())
	if inert.is_empty(): return
	var before: Dictionary = _world.call("get_command_log", 0)
	await _click(_panel.find_child("Template_" + str(inert["id"]), true, false) as Button)
	var inert_card: Button = _panel.find_child("Template_" + str(inert["id"]), true, false) as Button
	_check("grey actual inert placeholder", inert_card != null and inert_card.icon != null and inert_card.get_theme_color("font_color") == Color("807970"))
	var place: Button = _panel.get("_place") as Button
	_check("inert action disabled and debt", place.disabled and str((_panel.get("_gate") as Label).text).contains(str(inert["debt"])), {"selected": _panel.get("_selected"), "gate": (_panel.get("_gate") as Label).text, "debt": inert["debt"], "disabled": place.disabled, "card_rect": str(inert_card.get_global_rect()) if inert_card != null else "", "mouse": str(root.get_mouse_position()), "panel_rect": str(_panel.get_global_rect())})
	_check("inert card remains inspectable without placement", _panel.visible and not bool(_tool.call("template_active")))
	await _click(place)
	_check("native inert action cannot submit", _world.call("get_command_log", 0) == before)
	var selected: Array = [_panel.get("_selected"), _panel.get("_site"), _panel.get("_station")]
	var current: String = str(_locale.get("_current_lang"))
	for translated: String in ["en" if current == "ko" else "ko", current]:
		_locale.call("set_language", translated)
		await _frames()
		_fixed_labels(translated)
		_check("same panel locale preserves selection", selected == [_panel.get("_selected"), _panel.get("_site"), _panel.get("_station")])
		_check("same panel locale preserves inert lock and debt", place.disabled and str((_panel.get("_gate") as Label).text).contains(str(inert["debt"])), {"selected": _panel.get("_selected"), "gate": (_panel.get("_gate") as Label).text, "debt": inert["debt"], "disabled": place.disabled, "locale": translated})
	await _capture(language + "_inert")

func _fixed_labels(language: String) -> void:
	var labels: Array = _panel.get("_group_labels")
	var keys: Array[String] = ["BUILD_GROUP_BUILT", "BUILD_GROUP_AVAILABLE", "BUILD_GROUP_NEXT"]
	for index: int in keys.size():
		var expected: String = str(_locale.call("ltr", keys[index]))
		_check(language + " fixed " + keys[index], labels.size() == 3 and (labels[index] as Label).text == expected and expected != keys[index])
	var controls: Dictionary = {"_heading": "UI_ORDER_BUILD", "_housing": "ZONE_HOUSING", "_close": "UI_CLOSE", "_place": "BUILD_PLACE", "_upgrade": "BUILD_UPGRADE", "_why": "BUILD_SOURCE"}
	for field: String in controls:
		var control: Control = _panel.get(field) as Control
		var expected: String = str(_locale.call("ltr", controls[field]))
		_check(language + " fixed " + str(controls[field]), control != null and str(control.get("text")) == expected and expected != str(controls[field]))

func _selected(stage: String) -> Dictionary:
	var id: int = int((_manifests[stage] as Dictionary).get("selected_site", -1))
	for value: Dictionary in _world.call("get_building_snapshot").get("template_sites", []):
		if int(value["id"]) == id: return value
	return {}

func _completed(language: String, stage: String) -> void:
	var site: Dictionary = _selected(stage)
	_check("saved selected site " + stage, not site.is_empty())
	if site.is_empty() or str(site.get("status", "")) != "complete": return
	_panel.call("open_site", int(site["id"]))
	await _frames()
	await _click(_panel.get("_why") as Button)
	var why: Control = _main.get_node("UI/WhyPanel") as Control
	var cause: Dictionary = _world.call("get_building_cause", int(site["id"]), int(site["source_record"]))
	_check("native source opens exact building cause", bool(cause.get("valid", false)) and why.visible and str(why.call("plain_text")).replace("\u2060", "").contains(str(_locale.call("ltr", cause.get("name_key", "UI_UNKNOWN")))), {"cause": cause, "visible": why.visible, "body": (why.get("_body") as RichTextLabel).text, "plain": why.call("plain_text"), "bbcode": (why.get("_body") as RichTextLabel).bbcode_enabled, "expected": _locale.call("ltr", cause.get("name_key", "UI_UNKNOWN")), "locale": _locale.get("_current_lang")})
	await _capture(language + "_" + stage + "_source")
	why.call("close")
	_panel.call("open_site", int(site["id"]))
	var upgrade: Button = _panel.get("_upgrade") as Button
	if stage == "03-ready-upgrade":
		_check("real quarry-funded upgrade enabled", upgrade.visible and not upgrade.disabled)
		if not upgrade.visible or upgrade.disabled: return
		await _click(upgrade)
		var after: Dictionary = _selected(stage)
		_check("native upgrade starts same saved site", str(after.get("status", "")) == "building" and after.get("width") == site.get("width") and after.get("height") == site.get("height"))

func _housing(language: String) -> void:
	var overlay: Node = _main.get_node("ColonyView/OverlayLayer")
	var ui: Dictionary = (_manifests["03zonegrown"] as Dictionary).get("ui", {})
	if not ui.has("housing_tile"): _gap("manifest ui.housing_tile required for unoccluded native zone selection"); return
	var tile := Vector2i(int(ui["housing_tile"][0]), int(ui["housing_tile"][1]))
	var id: int = int(overlay.call("housing_zone_at", tile))
	_check("real housing designation at fixture tile", id >= 0 and id == int(ui.get("housing_zone_id", -1)))
	_camera.position = View.tile_centre(tile)
	_camera.force_update_scroll()
	await _frames()
	await _tile(tile)
	_check("native housing zone selection", int(overlay.call("selected_housing_zone")) == id)
	await _capture(language + "_housing")
	await _key(KEY_ESCAPE)
	_check("Esc clears housing selection", int(overlay.call("selected_housing_zone")) == -1)
	_panel.call("open")
	var housing_button: Button = null
	for button: Node in _panel.find_children("*", "Button", true, false):
		if str(button.get("text")) == str(_locale.call("ltr", "ZONE_HOUSING")):
			housing_button = button as Button
			break
	_check("native housing affordance exists", housing_button != null)
	if housing_button == null: return
	await _click(housing_button)
	_check("native housing affordance click enters existing area tool", int(_tool.get("mode")) == 15 and not _panel.visible)
	await _key(KEY_ESCAPE)

func _history(language: String) -> void:
	var old: Dictionary = _manifests.get("03zonegrown", {})
	for site: Dictionary in old.get("sites", []):
		if int(site["id"]) != int(old.get("selected_site", -1)): continue
		var cause: Dictionary = _world.call("get_building_cause", int(site["id"]), int(site["source_record"]))
		_check("old source resolves after upgrade", bool(cause.get("valid", false)) and str(cause.get("template_id", "")) == str(site["template"]))
		var current: Dictionary = _selected("04upgraded")
		_check("history is same upgraded zone-grown site", int(current.get("id", -1)) == int(site["id"]) and str(current.get("template_id", "")) != str(site["template"]) and str(cause.get("cause_kind", "")) == "zone")
		if site.get("completion_entry") == null: _gap("historical completion entry missing"); return
		var why: Control = _main.get_node("UI/WhyPanel") as Control
		_check("real old completion opens", bool(why.call("open", int(site["completion_entry"]))))
		var body: RichTextLabel = why.get("_body") as RichTextLabel
		await _frames()
		# Native click on the last why-chain line; assert outcome, never emit meta_clicked.
		await _point(body.global_position + Vector2(80, body.get_line_offset(maxi(0, body.get_line_count() - 1)) + 12))
		_check("native historical source link", not body.bbcode_enabled and str(why.call("plain_text")).replace("\u2060", "").contains(str(_locale.call("ltr", cause.get("name_key", "UI_UNKNOWN")))), {"cause": cause, "visible": why.visible, "body": body.text, "plain": why.call("plain_text"), "bbcode": body.bbcode_enabled, "expected": _locale.call("ltr", cause.get("name_key", "UI_UNKNOWN")), "locale": _locale.get("_current_lang")})
		await _capture(language + "_historical_source")
		why.call("close")

func _home(language: String) -> void:
	var candidates: Dictionary = (_manifests.get("04upgraded", {}) as Dictionary).get("ui", {})
	if not candidates.has("home_entity_bits"): _gap("04 manifest assigned-bed home entity not provided"); return
	var detail: Dictionary = _world.call("get_agent_detail", int(candidates["home_entity_bits"]))
	var economy: Dictionary = _world.call("get_agent_economy", int(detail.get("agent_id", -1)))
	var inspector: Control = _main.get_node("UI/AgentInspectorPanel") as Control
	inspector.call("display_agent", detail)
	await _frames()
	_check("actual assigned-bed home name", str(economy.get("dwelling_kind", "")) != "" and str((inspector.get("_dwelling_label") as Label).text).replace("\u2060", "").contains(str(_locale.call("ltr", economy["dwelling_name_key"]))), {"economy": economy, "text": (inspector.get("_dwelling_label") as Label).text, "expected": _locale.call("ltr", economy.get("dwelling_name_key", "UI_UNKNOWN")), "locale": _locale.get("_current_lang")})
	await _capture(language + "_home")
	inspector.call("close")

func _open_templates_native() -> void:
	await _key(KEY_F3)
	var bar: Node = _main.get_node("UI/OrderBar")
	_check("native F3 preserves existing menu", bool(bar.call("build_menu_open")))
	await _click(bar.find_child("BuildTemplates", true, false) as Button)
	_check("native template entry opens real panel", _panel.visible and not bool(bar.call("build_menu_open")))

func _choose_available_native(id: String) -> void:
	var card: Button = _panel.find_child("Template_" + id, true, false) as Button
	var groups: Array = _panel.get("_groups")
	_check("native target is actual available card " + id, card != null and groups.size() == 3 and card.get_parent() == groups[1])
	var before: Dictionary = _world.call("get_command_log", 0)
	await _click(card)
	_check("available single-choice card directly enters preview " + id, bool(_tool.call("template_active")) and not _panel.visible and (_panel.get("_stations") as OptionButton).item_count <= 1)
	_check("card selection alone sends no build command", _world.call("get_command_log", 0) == before)

# No motion/track invocation here: only ordinary frames may drive the production caller.
func _caller_refresh(preview: Node, tile: Vector2i, tick: int, revision: int, before: int, label: String) -> void:
	await _frames()
	var expected: Array = [preview.get("template_id"), preview.call("cursor_origin", tile), preview.get("template_rotation"), preview.get("station_id"), tick, revision]
	var actual: Array = preview.get("_query")
	_check("stationary native caller refresh " + label, (preview as CanvasItem).visible and _tool.get("_hover") == tile and actual == expected and _preview_changes > before)
	_rows[-1]["observed"] = {"frame": Engine.get_process_frames(), "expected_query": expected, "actual_query": actual.duplicate(), "changed_before": before, "changed_after": _preview_changes}

func _placement(language: String) -> void:
	var ui: Dictionary = (_manifests["02complete"] as Dictionary).get("ui", {})
	if not ui.has("placement_template") or not ui.has("placement_cursor_tile"): _gap("manifest placement template/cursor required"); return
	await _open_templates_native()
	await _choose_available_native(str(ui["placement_template"]))
	var preview: Node = _tool.get_node_or_null("TemplatePreview")
	_check("real placement preview active", preview != null and bool(preview.call("active")))
	if preview == null or not bool(preview.call("active")): return
	var tile := Vector2i(int(ui["placement_cursor_tile"][0]), int(ui["placement_cursor_tile"][1]))
	_camera.position = View.tile_centre(tile)
	_camera.force_update_scroll()
	await _tile(tile, false)
	var rooms: Node = _main.get_node("ColonyView/OverlayLayer/Rooms")
	var rooms_before: bool = bool(rooms.call("is_shown"))
	await _key(KEY_R)
	_check("native R rotates once and preserves room view", int(preview.get("template_rotation")) == 1 and bool(rooms.call("is_shown")) == rooms_before)
	_check("actual cursor anchor", preview.get("anchor") == preview.call("cursor_origin", tile))
	var origin: Vector2i = preview.get("anchor")
	var station: String = str(preview.get("station_id"))
	var old_sites: Array = _world.call("get_building_snapshot").get("template_sites", [])
	var total: int = int(_world.call("get_command_log", 9223372036854775807).get("total", -1))
	await _tile(tile)
	var delta: Dictionary = _world.call("get_command_log", total)
	_check("native valid placement is one accepted top command", int(delta.get("total", -1)) == total + 1 and delta.get("kinds") == PackedStringArray(["BuildTemplate"]) and delta.get("accepted") == PackedByteArray([1]))
	var new_sites: Array = _world.call("get_building_snapshot").get("template_sites", [])
	var old_ids: Array = old_sites.map(func(row: Dictionary) -> int: return int(row["id"]))
	var added: Array = new_sites.filter(func(row: Dictionary) -> bool: return int(row["id"]) not in old_ids)
	_check("one aggregate with exact native pose", added.size() == 1 and new_sites.size() == old_sites.size() + 1 and str(added[0]["template_id"]) == str(ui["placement_template"]) and Vector2i(int(added[0]["x"]), int(added[0]["y"])) == origin and int(added[0]["rotation"]) == 1 and str(added[0]["station"]) == station)
	await _capture(language + "_native_placed")
	await _load("02complete")
	await _open_templates_native()
	await _choose_available_native(str(ui["placement_template"]))
	_camera.position = View.tile_centre(tile)
	_camera.force_update_scroll()
	await _tile(tile, false)
	await _key(KEY_R)
	preview.connect("changed", func() -> void: _preview_changes += 1)
	var tick: int = int(_world.call("get_time_snapshot")["tick"])
	var version: int = int(_world.call("get_colony_versions")["command_revision"])
	await _caller_refresh(preview, tile, tick, version, -1, "initial complete key")
	var before: int = _preview_changes
	preview.call("track", tile, tick, version)
	_check("UNIT_DECISION same-key direct helper reuses cache", _preview_changes == before)
	SimReads.mutate(_world, "advance_ticks", [1])
	var next_tick: int = int(_world.call("get_time_snapshot")["tick"])
	var next_version: int = int(_world.call("get_colony_versions")["command_revision"])
	await _caller_refresh(preview, tile, next_tick, next_version, before, "tick advances")
	_check("actual tick advances for native caller", next_tick > tick)
	if ui.has("moving_person_entity_bits") and ui.has("moving_observe_ticks"):
		var person: int = int(ui["moving_person_entity_bits"])
		var first: Dictionary = _world.call("get_agent_detail", person)
		var bounds := Rect2i(preview.get("anchor"), preview.call("rotated_size"))
		var was_inside: bool = bounds.has_point(Vector2i(int(first["x"]), int(first["y"])))
		before = _preview_changes
		SimReads.mutate(_world, "advance_ticks", [int(ui["moving_observe_ticks"])])
		var last: Dictionary = _world.call("get_agent_detail", person)
		var moved_tick: int = int(_world.call("get_time_snapshot")["tick"])
		var moved_version: int = int(_world.call("get_colony_versions")["command_revision"])
		await _caller_refresh(preview, tile, moved_tick, moved_version, before, "moving person at unchanged revision")
		_check("real person crosses footprint with unchanged command revision", was_inside != bounds.has_point(Vector2i(int(last["x"]), int(last["y"]))) and moved_version == next_version and _preview_changes > before)
		next_tick = moved_tick
		next_version = moved_version
	else: _gap("moving-person enters/leaves footprint fixture trajectory not supplied")
	if ui.has("cache_housing_rect"):
		var rect: Array = ui["cache_housing_rect"]
		before = _preview_changes
		var result: Dictionary = SimReads.mutate(_world, "cmd_add_housing_zone", [int(rect[0]), int(rect[1]), int(rect[2]), int(rect[3])])
		var paused_tick: int = int(_world.call("get_time_snapshot")["tick"])
		var paused_version: int = int(_world.call("get_colony_versions")["command_revision"])
		await _caller_refresh(preview, tile, paused_tick, paused_version, before, "paused command at unchanged tick")
		_check("real paused command revision invalidates preview", bool(result.get("accepted", false)) and paused_tick == next_tick and paused_version != next_version and _preview_changes > before)
	else: _gap("manifest cache_housing_rect needed for paused command parity")
	await _capture(language + "_rotated_preview")
	await _key(KEY_ESCAPE)
	_check("native Esc cancels template", not bool(preview.call("active")))
	await _open_templates_native()
	await _choose_available_native(str(ui["placement_template"]))
	await _tile(tile, false)
	await _load("02complete")
	_check("actual world reset clears active preview cache", not bool(preview.call("active")) and (preview.get("_query") as Array).is_empty())

# Actual ordinary-job save, distinct from the synthetic renderer gallery.
func _cord_world(language: String) -> void:
	if not (await _load("05cord-world")): return
	var ui: Dictionary = (_manifests["05cord-world"] as Dictionary).get("ui", {})
	var fields: bool = ui.has_all(["cord_tile", "cord_item_code", "cord_count", "cord_quality", "cord_material", "cord_created_before", "cord_created_after"])
	_check("actual crafted cord fixture fields", fields)
	if not fields: return
	var tile := Vector2i(int(ui["cord_tile"][0]), int(ui["cord_tile"][1]))
	var code: int = Content.code("items", "cord")
	_check("actual cord ledger increased", code == int(ui["cord_item_code"]) and int(ui["cord_created_before"]) == 0 and int(ui["cord_created_after"]) > 0)
	_camera.position = View.tile_centre(tile)
	_camera.force_update_scroll()
	await _tile(tile, false)
	var before: Dictionary = _world.call("get_command_log", 0)
	var data: Dictionary = _world.call("get_colony_overlay")
	var xs: PackedInt32Array = data.get("item_xs", PackedInt32Array())
	var ys: PackedInt32Array = data.get("item_ys", PackedInt32Array())
	var kinds: PackedInt64Array = data.get("item_kinds", PackedInt64Array())
	var counts: PackedInt32Array = data.get("item_counts", PackedInt32Array())
	var qualities: PackedInt32Array = data.get("item_qualities", PackedInt32Array())
	var found: int = -1
	for index: int in mini(mini(xs.size(), ys.size()), mini(kinds.size(), mini(counts.size(), qualities.size()))):
		if Vector2i(xs[index], ys[index]) == tile and kinds[index] == code: found = index; break
	_check("actual ground cord at authored fixture tile", found >= 0)
	if found < 0: return
	_check("actual cord count and quality match save", counts[found] == int(ui["cord_count"]) and qualities[found] == int(ui["cord_quality"]))
	var row: Dictionary = Content.row("items", code)
	_check("actual selected cord placeholder rod birch", row.get("art_shape") == "rod" and row.get("art_tint_material") == "birch" and ItemIcons.uses_placeholder(code) and ItemIcons.texture_of(code) != null)
	var overlay: Node = _main.get_node("ColonyView/OverlayLayer")
	var look: Array = overlay.call("stack_look", code, counts[found])
	var texture: Array = overlay.call("_stack_texture", str(look[0]), int(look[1]), code)
	var sprite: Sprite2D = (overlay.get("_sprites") as Dictionary).get("i%d_%d" % [tile.x, tile.y]) as Sprite2D
	_check("real cord world sprite uses pile texture pipeline", sprite != null and sprite.is_visible_in_tree() and sprite.texture != null and sprite.texture == texture[0])
	_rows[-1]["observed"] = {"fixture": ui, "actual_count": counts[found], "actual_quality": qualities[found]}
	await _capture(language + "_05cord-world_actual_pile")
	_check("cord observation preserves tick and command log", int(_world.call("get_time_snapshot")["tick"]) == int(_manifests["05cord-world"]["tick"]) and _world.call("get_command_log", 0) == before)

func _pace(language: String) -> void:
	_camera.call("set_paused", false)
	_camera.call("set_speed", 1.0)
	_panel.call("open")
	await _frames()
	var start_tick: int = int(_world.call("get_time_snapshot")["tick"])
	var samples: Array[int] = []
	var previous: int = Time.get_ticks_usec()
	for _i: int in 120:
		await process_frame
		var now: int = Time.get_ticks_usec()
		samples.append(now - previous)
		previous = now
	var mean: float = 0.0
	for sample: int in samples: mean += float(sample) / float(samples.size())
	var fps: float = 1000000.0 / mean if mean > 0.0 else 0.0
	var end_tick: int = int(_world.call("get_time_snapshot")["tick"])
	_pacing[language] = {"delta_us": samples, "observed_fps": fps, "target_fps": 60, "start_tick": start_tick, "end_tick": end_tick}
	_check("unpaused build panel tick advances and60fps " + language, end_tick > start_tick and fps >= 60.0)
	_camera.call("set_paused", true)
	_panel.hide()

func _finish() -> void:
	if _out.is_absolute_path() and DirAccess.dir_exists_absolute(_out):
		var file: FileAccess = FileAccess.open(_out.path_join("build_result.json"), FileAccess.WRITE)
		if file != null: file.store_string(JSON.stringify({"failures": _failures, "fixture_gaps": _gaps, "complete": _failures == 0 and _gaps == 0, "checks": _rows, "pacing": _pacing, "window_size": [DisplayServer.window_get_size().x, DisplayServer.window_get_size().y], "requested_window_size": [_size.x, _size.y], "screen_size": [DisplayServer.screen_get_size().x, DisplayServer.screen_get_size().y], "viewport_size": [root.get_texture().get_width(), root.get_texture().get_height()], "display": DisplayServer.get_name()}, "\t"))
	print("SPINE_BUILD_DONE failures=%d fixture_gaps=%d" % [_failures, _gaps])
	quit(1 if _failures > 0 else (2 if _gaps > 0 else 0))

# C1-b: inspect actual chain checkpoints through the ordinary loader and native controls.
func _livestock() -> void:
	var stages: Array[String] = ["00-pen-placed", "01-capture", "02-two-adults", "03-kid", "04-milk", "05-slaughter"]
	var renderer: Node = _main.get_node("ColonyView/LivestockLayer")
	var goals: Control = _main.get_node("UI/MuxHud/Goals") as Control
	var cards: Control = _main.get_node("UI/MuxHud/CardPanel") as Control
	for language: String in ["ko", "en"]:
		_locale.call("set_language", language)
		for stage: String in stages:
			if not (await _load(stage)): continue
			var manifest: Dictionary = _manifests[stage]
			var ui: Dictionary = manifest.get("ui", {})
			var read: Dictionary = SimReads.read(_world, "get_livestock")
			var ids: PackedInt64Array = read.get("ids", PackedInt64Array())
			var expected: Array = ui.get("livestock", [])
			var observed_ids: Array[int] = []
			for row: Dictionary in expected: observed_ids.append(int(row["id"]))
			_check(stage + " exact loaded seed and living identities", int(_world.call("get_world_info").get("seed", -1)) == int(manifest["seed"]) and Array(ids) == observed_ids, {"expected": observed_ids, "actual": Array(ids)})
			_check(stage + " first kid stays pinned", int(read.get("first_kid_id", -1)) == int(ui.get("first_kid", -1) if ui.get("first_kid") != null else -1))
			var pen: int = int(manifest["selected_site"])
			var found: bool = false
			for actual: Dictionary in read.get("pens", []):
				if int(actual["site"]) == pen:
					found = true
					_check(stage + " real pen area and occupancy", bool(actual["intact"]) == (stage != stages[0]) and int(actual["count"]) == ids.size() and int(actual["available_grass"]) <= int(actual["grass_tiles"]), actual)
			_check(stage + " actual pen retained", found)
			_check(stage + " renderer draws all actual animals", int(renderer.call("counts").get("livestock", -1)) == ids.size())
			var anchor: Array = ui.get("pen_anchor", [])
			_check(stage + " actual pen framing supplied", anchor.size() == 2)
			if anchor.size() != 2: continue
			_camera.position = View.tile_centre(Vector2i(int(anchor[0]) + 2, int(anchor[1]) + 2))
			_camera.zoom = Vector2(2, 2)
			_camera.force_update_scroll()
			await _frames()
			_panel.hide()
			goals.hide()
			if ids.size() > 0:
				var xs: PackedInt64Array = read["xs"]
				var ys: PackedInt64Array = read["ys"]
				await _tile(Vector2i(xs[0], ys[0]), false)
			var cards_color: Color = cards.modulate
			cards.modulate = Color(cards_color, 0.0)
			await _capture(language + "_" + stage)
			cards.modulate = cards_color
			var before: Dictionary = _world.call("get_command_log", 0)
			var current_chapter: int = int(goals.call("current_chapter"))
			var has_capture: bool = int(read.get("first_capture", -1)) >= 0
			_check(stage + " capture fact controls chapter availability", (goals.call("available_chapters") as Array).has(4) == has_capture)
			if has_capture:
				await _key(KEY_G)
				_check("native chapter panel opened", goals.visible)
				_check("real chapter five selection", bool(goals.call("select_chapter", 4)) and str(goals.call("displayed_chapter").get("id", "")) == "pens_and_livestock")
				_check("chapter selection never skips prior goals", int(goals.call("current_chapter")) == current_chapter)
				await _capture(language + "_" + stage + "_chapter")
				goals.hide()
			_check(stage + " read-only inspection no commands", _world.call("get_command_log", 0) == before)
			for event: String in ["capture", "birth", "milk", "slaughter"]:
				var entry: int = int(ui.get(event + "_entry", -1) if ui.get(event + "_entry") != null else -1)
				if entry < 0: continue
				var why_read: Dictionary = _world.call("get_chronicle_why", entry)
				var evidence: Array = why_read.get("livestock_evidence", [])
				_check(stage + " typed terminal " + event, why_read.get("links", PackedInt32Array()) == PackedInt32Array([0, 4]) and evidence.size() == 2 and not (evidence[-1] as Dictionary).is_empty())
				var why: Control = _main.get_node("UI/WhyPanel") as Control
				_check(stage + " real Why " + event, bool(why.call("open", entry)))
				var plain: String = str(why.call("plain_text"))
				_check(stage + " localized Why " + event, not plain.contains("C1_") and not plain.contains(str(_locale.call("ltr", "UI_UNKNOWN"))), {"text": plain})
				if stage == "05-slaughter": await _capture(language + "_removed_" + event + "_why")
				why.call("close")
			if stage == "03-kid":
				await _livestock_card(cards, language, stage)
			elif stage == "05-slaughter":
				_check("removed first kid card does not retarget", not bool(cards.call("_check_bridge_trigger", "first_livestock_kid")) and not (cards.get("_active") as Dictionary).has("first_kid"))
			_check(stage + " inspection remains paused at exact checkpoint", int(_world.call("get_time_snapshot")["tick"]) == int(manifest["tick"]))

func _livestock_card(cards: Control, language: String, stage: String) -> void:
	var read: Dictionary = SimReads.read(_world, "get_livestock")
	var first: int = int(read.get("first_kid_id", -1))
	var active: Dictionary = cards.get("_active")
	_check("real first kid produces pinned decision card", active.has("first_kid") and int(active.get("first_kid", {}).get("trigger_data", {}).get("livestock_id", -1)) == first)
	if not active.has("first_kid"): return
	await _capture(language + "_first_kid_card")
	var before: Dictionary = _world.call("get_command_log", 0)
	await _click(_livestock_button(cards, "C1_CARD_FIRST_KID_KEEP"))
	_check("native Keep is command-free and retains kid", _world.call("get_command_log", 0) == before and SimReads.read(_world, "get_livestock").get("ids", PackedInt64Array()).has(first) and not (cards.get("_active") as Dictionary).has("first_kid"))
	if not (await _load(stage)): return
	# Exercise the actual expiry decision seam; no clock or world state is fabricated.
	before = _world.call("get_command_log", 0)
	cards.call("_expire_card", "first_kid")
	_check("expiry is command-free", _world.call("get_command_log", 0) == before and SimReads.read(_world, "get_livestock").get("ids", PackedInt64Array()).has(first))
	if not (await _load(stage)): return
	var start: int = int(_world.call("get_command_log", 9223372036854775807).get("total", -1))
	await _click(_livestock_button(cards, "C1_CARD_FIRST_KID_SLAUGHTER"))
	var delta: Dictionary = _world.call("get_command_log", start)
	_check("native Slaughter one accepted ordinary command", int(delta.get("total", -1)) == start + 1 and delta.get("kinds") == PackedStringArray(["DesignateSlaughter"]) and delta.get("accepted") == PackedByteArray([1]), delta)
	_check("job command preserves animal until execution", SimReads.read(_world, "get_livestock").get("ids", PackedInt64Array()).has(first))
	# The next checkpoint is the independently committed ordinary-job result.

func _livestock_button(cards: Control, key: String) -> Button:
	var text: String = str(_locale.call("ltr", key))
	for node: Node in cards.find_children("*", "Button", true, false):
		var button: Button = node as Button
		if button.text.replace("\u2060", "").begins_with(text): return button
	return null
