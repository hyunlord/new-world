# Windowed UI-scale acceptance against the actual main scene and actual GUI hit testing.
# Bridge methods: get_agent_snapshot/get_building_snapshot through SimReads, advance_ticks(1).
# Run after fresh bridge build: godot --path . --script res://tools/v8/mux_scale_test.gd -- /absolute/evidence/dir
extends SceneTree

const Hud = preload("res://scripts/ui/mux/mux_hud.gd")
const Store = preload("res://scripts/ui/mux/settings_store.gd")
var _out: String
var _main: Node
var _hud: Hud
var _checks: int = 0
var _failures: int = 0
var _rows: Array[Dictionary] = []
var _scales: Array[Dictionary] = []
var _prefix: String = ""


func _init() -> void:
	call_deferred("_run")


func _check(label: String, passed: bool) -> void:
	_checks += 1
	if not passed:
		_failures += 1
	_rows.append({"check": _prefix + label, "passed": passed})
	print("MUX_SCALE_%s %s%s" % ["PASS" if passed else "FAIL", _prefix, label])


func _frames(count: int = 4) -> void:
	for _i: int in count:
		await process_frame


func _rect(control: Control) -> Rect2:
	var result: Rect2 = control.get_global_rect()
	for child: Node in control.get_children():
		if child is Control and (child as Control).is_visible_in_tree():
			result = result.merge((child as Control).get_global_rect())
	return result


func _encoded(rect: Rect2) -> Dictionary:
	return {"x": rect.position.x, "y": rect.position.y, "width": rect.size.x, "height": rect.size.y}


func _bounded(label: String, control: Control) -> Rect2:
	var actual: Rect2 = _rect(control)
	_check(label + " visible", control.is_visible_in_tree())
	_check(label + " nonempty", actual.size.x > 0 and actual.size.y > 0)
	_check(label + " viewport bounded", root.get_visible_rect().grow(0.1).encloses(actual))
	return actual


func _hit_probe(button: Button, point: Vector2, label: String) -> void:
	var motion: InputEventMouseMotion = InputEventMouseMotion.new()
	motion.position = point
	Input.parse_input_event(motion)
	await _frames(2)
	var hovered: Control = root.gui_get_hovered_control()
	_check(label + " actual GUI hit", hovered == button or (hovered != null and button.is_ancestor_of(hovered)))


func _click(button: Button, label: String) -> void:
	var point: Vector2 = button.get_global_rect().get_center()
	await _hit_probe(button, point, label)
	for down: bool in [true, false]:
		var event: InputEventMouseButton = InputEventMouseButton.new()
		event.position = point
		event.button_index = MOUSE_BUTTON_LEFT
		event.pressed = down
		Input.parse_input_event(event)
		await _frames(1)
	await _frames()


func _wait_for_minimap() -> void:
	var deadline: int = Time.get_ticks_msec() + 10000
	while _hud.minimap.map_image() == null and Time.get_ticks_msec() < deadline:
		await process_frame
	var published: Image = _hud.minimap.map_image()
	_check("minimap published actual image", published != null and not published.is_empty())
	if published != null:
		_check("minimap has map dimensions", published.get_width() > 1 and published.get_height() > 1)


func _capture(name: String) -> void:
	await _frames()
	await RenderingServer.frame_post_draw
	var screenshot: Image = root.get_texture().get_image()
	_check("capture " + name, screenshot.save_png(_out.path_join(name + ".png")) == OK)


func _run() -> void:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	_out = (args[0] if not args.is_empty() else "/tmp/worldsim-mux-scale-%d" % OS.get_process_id()).simplify_path()
	var project_path: String = ProjectSettings.globalize_path("res://").simplify_path().trim_suffix("/")
	_check("external evidence directory", _out.is_absolute_path() and _out != project_path and not _out.begins_with(project_path + "/"))
	_check("windowed renderer", DisplayServer.get_name() != "headless")
	if _failures > 0:
		quit(1)
		return
	_check("create evidence directory", DirAccess.make_dir_recursive_absolute(_out) == OK)
	for factor: float in [1.0, 1.25, 1.5]:
		await _test_scale(factor)
	_prefix = ""
	var report: FileAccess = FileAccess.open(_out.path_join("mux-scale-results.json"), FileAccess.WRITE)
	_check("report writable", report != null)
	if report != null:
		report.store_string(JSON.stringify({"checks": _checks, "failures": _failures, "input": "synthetic mouse events against real window", "results": _rows, "scales": _scales, "visual_acceptance": "screenshots require independent visual verdict"}, "\t"))
	print("MUX_SCALE_DONE checks=%d failures=%d evidence=%s" % [_checks, _failures, _out])
	quit(0 if _failures == 0 else 1)


func _test_scale(factor: float) -> void:
	_prefix = "%.2f " % factor
	var slug: String = "scale-%03d" % roundi(factor * 100)
	_main = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	_hud = _main.get_node("UI/MuxHud") as Hud
	# Inject before _ready: fixture writes only its external settings/progress file.
	_hud.store = Store.new(_out.path_join("%s-settings-%d.cfg" % [slug, OS.get_process_id()]))
	_check("isolated scale accepted", _hud.store.set_value("ui_scale", factor))
	_hud.store.set_value("steward_default", false)
	_hud.store.set_value("edge_pan", false)
	root.add_child(_main)
	var camera: Camera2D = _main.get_node("Camera2D") as Camera2D
	var world: Node = _main.get_node("WorldSim")
	camera.call("set_paused", true)
	SimReads.mutate(world, "advance_ticks", [1])
	await _frames(24)
	var buildings: Dictionary = SimReads.read(world, "get_building_snapshot")
	_check("actual first tick registered buildings", not buildings.get("kinds", PackedByteArray()).is_empty())
	_check("world paused", bool(camera.call("is_paused")))
	_check("scale preference retained", is_equal_approx(float(_hud.store.value("ui_scale")), factor))
	var zoom_before: Vector2 = camera.zoom
	var viewport_before: Rect2 = root.get_visible_rect()
	var people: Dictionary = SimReads.read(world, "get_agent_snapshot")
	var ids: PackedInt64Array = people.get("agent_ids", PackedInt64Array())
	_check("live person exists", not ids.is_empty())
	var inspector: Control = _main.get_node("UI/AgentInspectorPanel") as Control
	if not ids.is_empty():
		inspector.call("show_person", ids[0])
	await _frames(12)
	var dock: Control = _hud.get_node("MapDock") as Control
	var bar: Control = _main.get_node("UI/OrderBar/Bar") as Control
	var inspector_rect: Rect2 = _bounded("inspector", inspector)
	var dock_rect: Rect2 = _bounded("right dock", dock)
	var helper_rect: Rect2 = _bounded("helper", _hud.helper)
	var bar_rect: Rect2 = _bounded("order bar", bar)
	_check("inspector clears action header", not inspector_rect.intersects(_rect(_hud.get_node("Actions") as Control)))
	_check("inspector and right dock do not overlap", not inspector_rect.intersects(dock_rect))
	_check("helper and order bar do not overlap", not helper_rect.intersects(bar_rect))
	_check("inspector and alerts do not overlap", not inspector_rect.intersects(_rect(_hud.alerts)))
	_check("order bar and right dock do not overlap", not bar_rect.intersects(dock_rect))
	await _wait_for_minimap()
	await _capture(slug + "-hud")
	var record: Dictionary = {"requested": factor, "viewport": _encoded(viewport_before), "inspector": _encoded(inspector_rect), "dock": _encoded(dock_rect), "helper": _encoded(helper_rect), "order_bar": _encoded(bar_rect)}
	var effective: Dictionary = _hud.get("_legacy").call("layout_report")
	var caps: Dictionary = {}
	for key: String in effective:
		var item: Dictionary = effective[key]
		caps[key] = {"requested": item["requested"], "effective": item["effective"], "capped": item["capped"], "visible": item["visible"], "rect": _encoded(item["rect"])}
	record["legacy_effective_scales"] = caps
	# Help button invokes the actual modal; its close action uses GUI dispatch.
	(_hud.get_node("Actions/HelpButton") as Button).pressed.emit()
	await _frames(10)
	record["help"] = _encoded(_bounded("full help", _hud.help))
	await _capture(slug + "-help")
	var close: Button = _hud.help.find_child("Close", true, false) as Button
	_check("help close button exists", close != null)
	if close != null:
		await _click(close, "help close")
	_check("help close action hides modal", not _hud.help.visible)
	# Real build menu placement comes from OrderBar, then LegacyLayout adjusts its scale.
	inspector.hide()
	var orders: Control = _main.get_node("UI/OrderBar") as Control
	orders.call("open_build_menu")
	await _frames(8)
	var menu: Control = orders.get_node("BuildMenu") as Control
	var menu_rect: Rect2 = _bounded("build menu", menu)
	record["build_menu"] = _encoded(menu_rect)
	_check("build menu above order bar", menu_rect.end.y <= _rect(bar).position.y - 7.9)
	await _capture(slug + "-build-menu")
	var menu_buttons: Array[Node] = menu.find_children("*", "Button", true, false)
	_check("build menu has real actions", not menu_buttons.is_empty())
	for index: int in menu_buttons.size():
		var entry: Button = menu_buttons[index] as Button
		var entry_rect: Rect2 = entry.get_global_rect()
		await _hit_probe(entry, entry_rect.position + entry_rect.size * Vector2(0.25, 0.5), "build entry %d left quarter" % index)
		await _hit_probe(entry, entry_rect.get_center(), "build entry %d center" % index)
	if not menu_buttons.is_empty():
		await _click(menu_buttons[0] as Button, "build entry")
	_check("build selection closes menu", not menu.visible)
	_check("build selection activates order tool", int(orders.call("current_mode")) == 3)
	_main.get_node("ColonyView/OrderTool").call("set_mode", 0)
	orders.call("open_dig_menu")
	await _frames(8)
	var dig: Control = orders.get_node("DigMenu") as Control
	record["dig_menu"] = _encoded(_bounded("gather menu", dig))
	var dig_buttons: Array[Node] = dig.find_children("*", "Button", true, false)
	_check("gather menu has real actions", not dig_buttons.is_empty())
	if not dig_buttons.is_empty():
		await _click(dig_buttons[0] as Button, "gather entry")
	_check("gather selection closes menu", not dig.visible)
	_check("gather selection activates quarry", int(orders.call("current_mode")) == 6)
	_check("UI scale leaves camera zoom unchanged", camera.zoom.is_equal_approx(zoom_before))
	_check("UI scale leaves viewport unchanged", root.get_visible_rect() == viewport_before)
	_scales.append(record)
	_main.queue_free()
	await _frames(8)
