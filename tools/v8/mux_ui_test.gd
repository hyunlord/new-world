# M-UX A11: windowed integration against the real scene, with synthetic UI input and explicit assertions.
# Bridge methods: SimReads snapshots, advance_ticks/cmd_set_steward; paged get_chronicle_since.
# Run after fresh bridge build: godot --path . --script tools/v8/mux_ui_test.gd -- /absolute/evidence/dir
# Synthetic gestures do not establish physical macOS trackpad acceptance. Settings use an isolated file.
extends SceneTree

const ArtData = preload("res://scripts/world/art/art_data.gd")
const OrderBar = preload("res://scripts/ui/hud/order_bar.gd")
const Economy = preload("res://scripts/ui/economy_text.gd")
const Hud = preload("res://scripts/ui/mux/mux_hud.gd")
const Store = preload("res://scripts/ui/mux/settings_store.gd")
const FactsAdapter = preload("res://scripts/ui/mux/ui_facts_adapter.gd")
const AlertCases = preload("res://tools/v8/mux_alerts_test.gd")
const Chronicle = preload("res://scripts/ui/chronicle_text.gd")
const Neighbours = preload("res://scripts/ui/neighbour_text.gd")
const View = preload("res://scripts/world/colony/view.gd")
var _main: Node
var _hud: Hud
var _camera: Camera2D
var _world: Node
var _checks: int = 0
var _failures: int = 0
var _out: String
var _rows: Array[Dictionary] = []
var _jump: Dictionary = {}
var _live_routes: Dictionary = {}
var _observe_routes: bool = false

func _init() -> void:
	call_deferred("_run")

func _check(label: String, passed: bool) -> void:
	_checks += 1
	if not passed: _failures += 1
	_rows.append({"check": label, "passed": passed})
	print("MUX_UI_%s %s" % ["PASS" if passed else "FAIL", label])

func _frames(count: int = 3) -> void:
	for _i: int in count:
		await process_frame
		if _observe_routes and is_instance_valid(_hud) and _hud.is_node_ready():
			_observe_live_routes()

func _mouse(point: Vector2, button: MouseButton, down: bool) -> void:
	var event: InputEventMouseButton = InputEventMouseButton.new()
	event.position = point
	event.button_index = button
	event.pressed = down
	Input.parse_input_event(event)

func _motion(point: Vector2, relative: Vector2 = Vector2.ZERO) -> void:
	var event: InputEventMouseMotion = InputEventMouseMotion.new()
	event.position = point
	event.relative = relative
	Input.parse_input_event(event)

func _key(code: Key) -> void:
	var event: InputEventKey = InputEventKey.new()
	event.keycode = code
	event.pressed = true
	Input.parse_input_event(event)
	event = InputEventKey.new()
	event.keycode = code
	Input.parse_input_event(event)

func _capture(name: String) -> void:
	await _frames(3)
	if name.begins_with("steward-completed-house-"):
		var provenance: Dictionary = _restore_live_alert_facts("before " + name)
		var evidence: FileAccess = FileAccess.open(_out.path_join(name + "-alerts.json"), FileAccess.WRITE)
		_check("completed house alert provenance opened", evidence != null)
		if evidence != null:
			evidence.store_string(JSON.stringify(provenance, "\t"))
			evidence.close()
	if DisplayServer.get_name() == "headless": return
	await RenderingServer.frame_post_draw
	var screenshot: Image = root.get_texture().get_image()
	_check("capture " + name, screenshot.save_png(_out.path_join(name + ".png")) == OK)

func _run() -> void:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	_out = args[0] if not args.is_empty() else "/tmp/worldsim-mux-ui-%d" % OS.get_process_id()
	_check("external evidence directory", _out.is_absolute_path() and not _out.begins_with(ProjectSettings.globalize_path("res://")))
	if _failures > 0:
		quit(1)
		return
	_check("evidence directory created", DirAccess.make_dir_recursive_absolute(_out) == OK)
	_check("windowed renderer", DisplayServer.get_name() != "headless")
	_main = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	_hud = _main.get_node("UI/MuxHud") as Hud
	_hud.store = Store.new(_out.path_join("settings-%d.cfg" % OS.get_process_id()))
	_camera = _main.get_node("Camera2D") as Camera2D
	root.add_child(_main)
	if not bool(_main.get("boot_ok")):
		push_error(str(_main.get("boot_error")))
		quit(1)
		return
	_world = _main.get_node("WorldSim")
	_camera.call("set_paused", true)
	_check("attach establishes notification baseline before first tick", _hud._chronicle_cursor >= 0 and _hud._sequence == 0)
	SimReads.mutate(_world, "advance_ticks", [1]) # Buildings register on their first simulation tick.
	var first_state: Dictionary = _world.get_steward()
	Chronicle.steward_labels = {} # Polling must use its own supplied snapshot, even before outer HUD processing.
	_hud._poll_steward_notifications(first_state)
	_check("first attach batch is retained before scheduled poll", _hud._sequence == int(first_state.get("sequence", 0)) and (_main.get_node("UI/OrderBar").get("_background_toasts") as Array).size() == int(first_state.get("sequence", 0)))
	_check("first actual notifications include localized action labels without global cache", _queued_action_labels_valid(first_state))
	await _frames(20)
	_check("real HUD attached", _hud.is_inside_tree() and _hud.controls != null and _hud.minimap != null)
	_check("isolated learning starts at camera", _hud.helper.current_id() == "camera")
	await _camera_checks()
	await _control_checks()
	await _minimap_checks()
	await _frames()
	var order_bar: Control = _main.get_node("UI/OrderBar").get("_bar") as Control
	_check("learning card clears order bar", not _hud.helper.get_global_rect().intersects(order_bar.get_global_rect()))
	_check("alerts clear minimap header", not _hud.alerts.get_global_rect().intersects(_hud.minimap.get_global_rect()))
	await _capture("hud")
	await _help_checks()
	await _lesson_checks()
	await _alert_checks()
	await _tooltip_checks()
	_notification_checks()
	await _locale_checks()
	await _steward_checks()
	await _finish_live_routes()
	await _notification_lifecycle_checks()
	_panel_event_checks()
	_check("at least forty actual assertions", _checks >= 40)
	var report: FileAccess = FileAccess.open(_out.path_join("mux-ui-results.json"), FileAccess.WRITE)
	if report != null:
		report.store_string(JSON.stringify({"checks": _checks, "failures": _failures, "input": "synthetic windowed", "results": _rows, "minimap_cost": _hud.minimap.performance_stats()}, "\t"))
	print("MUX_UI_DONE checks=%d failures=%d evidence=%s" % [_checks, _failures, _out])
	quit(0 if _failures == 0 else 1)

func _panel_event_checks() -> void:
	# Open and close synchronously, with HUD polling disabled: no frame or 0.5-second poll can rescue a missed signal.
	_hud.set_process(false)
	var poll_left: float = _hud._poll_left
	for node_name: String in Hud.LEARNING_PANELS:
		(_main.get_node("UI/" + node_name) as Control).hide()
	var isolated: Store = Store.new(_out.path_join("panel-events-%d.cfg" % OS.get_process_id()))
	_hud.helper.setup(isolated)
	for node_name: String in Hud.LEARNING_PANELS:
		var panel: Control = _main.get_node("UI/" + node_name) as Control
		var event_id: String = str(Hud.LEARNING_PANELS[node_name])
		_check(node_name + " startup wired actual visibility signal", panel.visibility_changed.is_connected(_hud._learning_panel_visibility_changed.bind(panel)))
		var connections: int = panel.visibility_changed.get_connections().size()
		_hud._learning_panel_entered(panel)
		_hud._learning_panel_entered(panel)
		_check(node_name + " reconnect does not duplicate visibility listeners", panel.visibility_changed.get_connections().size() == connections)
		_check(node_name + " hidden wiring does not record an opening", not (isolated.progress()["events"] as Dictionary).has(event_id))
		panel.show()
		panel.hide()
		_check(node_name + " brief opening latched before any poll", (isolated.progress()["events"] as Dictionary).has(event_id))
	var reopened: Store = Store.new(isolated.path)
	_check("all five brief panel openings persist across settings reload", (reopened.progress()["events"] as Dictionary).size() == Hud.LEARNING_PANELS.size())
	_check("panel signals leave expensive fact poll timer unchanged", _hud._poll_left == poll_left)
	_check("panel opening observations do not advance lessons", (isolated.progress()["completed"] as PackedStringArray).is_empty())
	_hud.helper.setup(_hud.store)
	_hud.set_process(true)

func _camera_checks() -> void:
	_camera.call("center_on_tile", Vector2i(96, 96))
	var point: Vector2 = root.get_visible_rect().size * Vector2(0.52, 0.45)
	_motion(point)
	await _frames()
	_check("map pointer outside GUI", root.gui_get_hovered_control() == null)
	var before: Vector2 = _camera.position
	var pan: InputEventPanGesture = InputEventPanGesture.new()
	pan.position = point
	pan.delta = Vector2(2, 1)
	Input.parse_input_event(pan)
	await _frames()
	_check("pan gesture moves real camera", before.distance_to(_camera.position) > 1.0)
	var level: float = float(_camera.call("get_zoom_level"))
	var pinch: InputEventMagnifyGesture = InputEventMagnifyGesture.new()
	pinch.position = point
	pinch.factor = 1.6
	Input.parse_input_event(pinch)
	await _frames(1)
	var ladder: Array = _camera.call("zoom_steps")
	var next_level: float = float(ladder[mini(ladder.find(level) + 1, ladder.size() - 1)])
	_check("pinch advances exactly one zoom step", is_equal_approx(float(_camera.call("get_zoom_level")), next_level))
	await create_timer(0.18).timeout
	_check("zoom animation reaches target", is_equal_approx(_camera.zoom.x, float(_camera.call("get_zoom_level"))))
	for button: MouseButton in [MOUSE_BUTTON_LEFT, MOUSE_BUTTON_RIGHT]:
		_main.get_node("ColonyView/OrderTool").call("set_mode", 0)
		before = _camera.position
		_motion(point)
		await _frames()
		_mouse(point, button, true)
		_motion(point + Vector2(70, 30), Vector2(70, 30))
		_mouse(point + Vector2(70, 30), button, false)
		await _frames()
		_check("map drag %d moves camera" % button, before.distance_to(_camera.position) > 1.0)

func _control_checks() -> void:
	_hud.controls.button_for("pause").pressed.emit()
	_check("pause button resumes", not bool(_camera.call("is_paused")))
	_hud.controls.button_for("pause").pressed.emit()
	_check("pause button pauses", bool(_camera.call("is_paused")))
	for speed: int in range(1, 5):
		_hud.controls.button_for("speed_%d" % speed).pressed.emit()
		_check("speed %d actual camera state" % speed, int(_camera.call("get_speed")) == speed and not bool(_camera.call("is_paused")))
		_check("speed %d highlighted" % speed, _hud.controls.button_for("speed_%d" % speed).button_pressed)
	_camera.call("set_paused", true)
	_hud.controls.button_for("zoom_out").pressed.emit()
	await create_timer(0.18).timeout
	_check("zoom out button", is_equal_approx(_camera.zoom.x, 1.0))
	_hud.controls.button_for("zoom_in").pressed.emit()
	await create_timer(0.18).timeout
	_check("zoom in button", is_equal_approx(_camera.zoom.x, 2.0))
	var buildings: Dictionary = SimReads.read(_world, "get_building_snapshot")
	var kinds: PackedByteArray = buildings.get("kinds", PackedByteArray())
	var xs: PackedInt32Array = buildings.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = buildings.get("ys", PackedInt32Array())
	var camp: Vector2i = Vector2i(-1, -1)
	for index: int in mini(kinds.size(), mini(xs.size(), ys.size())):
		if bool(ArtData.binding("buildings", kinds[index]).get("light", false)):
			camp = Vector2i(xs[index], ys[index])
			break
	_check("home fixture has registered camp hearth", camp.x >= 0)
	_camera.call("center_on_tile", camp + Vector2i(20, 20))
	_hud.controls.button_for("home").pressed.emit()
	_check("home button centers on actual hearth", camp.x >= 0 and _camera.position.distance_to(View.tile_centre(camp)) < 2.0)

func _minimap_checks() -> void:
	for _frame: int in range(240):
		if _hud.minimap.map_image() != null: break
		await process_frame
	_check("live minimap published", _hud.minimap.map_image() != null)
	var canvas: Control = _hud.minimap.get("_canvas") as Control
	var rect: Rect2 = _hud.minimap._map_rect()
	var point: Vector2 = canvas.get_global_transform_with_canvas() * (rect.position + rect.size * 0.5)
	_motion(point)
	await _frames()
	_mouse(point, MOUSE_BUTTON_LEFT, true)
	_mouse(point, MOUSE_BUTTON_LEFT, false)
	await _frames()
	var dimensions: Vector2i = _hud.minimap.map_image().get_size() if _hud.minimap.map_image() != null else Vector2i.ZERO
	var tile: Vector2i = Vector2i(Vector2(dimensions) / 2.0)
	_check("minimap dispatched click centers real camera", _camera.position.distance_to(View.tile_centre(tile)) < 2.0)
	_mouse(point, MOUSE_BUTTON_LEFT, true)
	var destination: Vector2 = canvas.get_global_transform_with_canvas() * (rect.position + rect.size * Vector2(0.6, 0.55))
	_motion(destination, destination - point)
	_mouse(destination, MOUSE_BUTTON_LEFT, false)
	await _frames()
	var dragged: Vector2i = Vector2i(floori(dimensions.x * 0.6), floori(dimensions.y * 0.55))
	_check("minimap dispatched drag centers real camera", _camera.position.distance_to(View.tile_centre(dragged)) < 2.0)
	_hud.minimap.set_collapsed(true)
	_check("minimap collapse hides canvas", not canvas.visible)
	_hud.minimap.set_collapsed(false)
	_check("minimap expands canvas", canvas.visible)

func _help_checks() -> void:
	(_hud.find_child("HelpButton", true, false) as Button).pressed.emit()
	_check("help button opens overlay", _hud.help.visible)
	_help_rect_diagnostic("opened")
	var grid: GridContainer = _hud.help.get("_rows") as GridContainer
	_check("help generated from entire catalog", grid.get_child_count() == _hud.catalog.entries.size() * 2)
	await _frames()
	_help_rect_diagnostic("before_original_assert")
	_check("help overlay fits viewport", root.get_visible_rect().encloses(_hud.help.get_global_rect()))
	await _capture("help")
	_help_rect_diagnostic("after_capture_frame_post_draw")
	_check("rendered help overlay fits viewport", root.get_visible_rect().encloses(_hud.help.get_global_rect()))
	(_hud.help.find_child("Close", true, false) as Button).pressed.emit()
	_check("help close button", not _hud.help.visible)
	_key(KEY_SLASH)
	await _frames()
	_check("slash shortcut opens help", _hud.help.visible)
	_key(KEY_ESCAPE)
	await _frames()
	_check("escape closes help", not _hud.help.visible)

func _lesson_checks() -> void:
	_check("real camera and minimap complete camera lesson", _hud.helper.can_advance())
	(_hud.helper.find_child("Next", true, false) as Button).pressed.emit()
	_check("next advances to speed lesson", _hud.helper.current_id() == "speed")
	_check("real speed changes complete speed lesson", _hud.helper.can_advance())
	(_hud.helper.find_child("Next", true, false) as Button).pressed.emit()
	_check("next advances to person lesson", _hud.helper.current_id() == "person")
	await _capture("lesson")

func _alert_checks() -> void:
	_hud.alerts.set_process(false)
	_hud.alerts.jump_requested.connect(func(target: Dictionary) -> void: _jump = target)
	var people: Dictionary = SimReads.read(_world, "get_agent_snapshot")
	var ids: PackedInt64Array = people.get("agent_ids", PackedInt64Array())
	_check("live person available for alert fixture", not ids.is_empty())
	if ids.is_empty(): return
	for test_case: Dictionary in AlertCases._cases():
		for child: Node in _main.get_node("UI").get_children():
			if child is Control and str(child.name).ends_with("Panel"): child.hide()
		var facts: Dictionary = AlertCases._facts()
		var row: Dictionary = (test_case["row"] as Dictionary).duplicate(true)
		row["id"] = ids[0] if test_case["kind"] == "person" else row["id"]
		if test_case["kind"] == "technique":
			var tech_ids: PackedInt64Array = SimReads.read(_world, "get_tech").get("ids", PackedInt64Array())
			_check(str(test_case["id"]) + " valid technique fixture", not tech_ids.is_empty())
			if not tech_ids.is_empty():
				row["id"] = tech_ids[0]
				if row.has("node_id"): row["node_id"] = tech_ids[0]
		row["x"] = 80
		row["y"] = 82
		facts[test_case["table"]] = [row]
		_check(str(test_case["id"]) + " real panel accepts fixture", _hud.alerts.refresh_from_facts(facts))
		var button: Button = _hud.alerts.find_child(str(test_case["id"]), true, false) as Button
		_check(str(test_case["id"]) + " real alert button exists", button != null)
		if button == null: continue
		_jump = {}
		button.pressed.emit()
		_check(str(test_case["id"]) + " routes real camera", _camera.position.distance_to(View.tile_centre(Vector2i(80, 82))) < 2.0)
		_check(str(test_case["id"]) + " routes target kind", _jump.get("kind") == test_case["kind"])
		if test_case["kind"] == "person":
			var inspector: Control = _main.get_node("UI/AgentInspectorPanel") as Control
			_check(str(test_case["id"]) + " selects live person", inspector.visible and int(inspector.get("_selected_person")) == ids[0])
		elif test_case["kind"] in ["tile", "animal"]:
			var causal: Control = _main.get_node("UI/CausalPanel") as Control
			_check(str(test_case["id"]) + " navigates without opening why", not causal.visible)
			if str(test_case["id"]) == "food_low":
				await _capture("causal-alert-target")
		elif test_case["kind"] == "technique":
			var tech: Control = _main.get_node("UI/TechPanel") as Control
			_check(str(test_case["id"]) + " selects technique", tech.visible and int(tech.call("selected_node")) == int(_jump.get("id", -1)))
	await _capture("alerts-synthetic-causes")

func _tooltip_checks() -> void:
	for counter: String in ["_agents_label", "_clock_label", "_wood_label", "_cold_label"]:
		var label: Control = _main.get_node("UI/HudTopbar").get(counter) as Control
		_check(counter + " has localized HUD explanation", label != null and not label.tooltip_text.is_empty() and not "MUX_" in label.tooltip_text)
	for panel_name: String in ["HudTopbar", "OrderBar", "AgentInspectorPanel", "TechPanel", "TeachPanel", "RoomPanel", "PriorityPanel", "NeighboursPanel", "MigrantPanel", "TradePanel", "WorkshopPanel", "ChroniclePanel"]:
		var panel: Node = _main.get_node_or_null("UI/" + panel_name)
		_check(panel_name + " available for tooltip check", panel != null)
		if panel == null: continue
		var buttons: Array[Node] = panel.find_children("*", "Button", true, false)
		var covered: bool = true
		for node: Node in buttons:
			var button: Button = node as Button
			button.mouse_entered.emit()
			covered = covered and not button.tooltip_text.strip_edges().is_empty() and not "MUX_" in button.tooltip_text
		_check(panel_name + " instantiated buttons have localized tooltips", covered)

func _steward_checks() -> void:
	_observe_routes = true
	for child: Node in _main.get_node("UI").get_children():
		if child is Control and str(child.name).ends_with("Panel"): child.hide()
	_hud.alerts.setup(_world)
	_hud.alerts.set_process(true)
	var choice: OptionButton = _hud.find_child("Steward", true, false) as OptionButton
	choice.item_selected.emit(3)
	_check("steward toggle off reaches simulation", not bool(SimReads.read(_world, "get_steward").get("enabled", true)))
	choice.item_selected.emit(1)
	var state: Dictionary = SimReads.read(_world, "get_steward")
	_check("steward housing mode reaches simulation", bool(state.get("enabled", false)) and int(state.get("scope", -1)) == 1)
	var before: int = int(state.get("sequence", 0))
	for _batch: int in range(12):
		SimReads.mutate(_world, "advance_ticks", [60])
		await _frames()
		state = SimReads.read(_world, "get_steward")
		if int(state.get("sequence", 0)) > before: break
	_check("bounded 720 ticks produces real steward order", int(state.get("sequence", 0)) > before)
	_check("housing rule has accepted provenance", (state.get("accepted_rules", PackedInt32Array()) as PackedInt32Array).has(3))
	var rows: Array[Dictionary] = []
	var cursor: int = 0
	for _page: int in range(32):
		var page: Dictionary = _world.call("get_chronicle_since", cursor, 512)
		rows.append_array(Chronicle.rows(page))
		var next: int = int(page.get("next", cursor))
		if next <= cursor or next >= int(page.get("total", next)): break
		cursor = next
	var found: bool = false
	var formatted: String = ""
	for row: Dictionary in rows:
		if int(row.get("kind", -1)) == 45:
			found = true
			formatted = Chronicle.line(row, -1, root.get_node("Locale"), SimReads.read(_world, "get_world_info"))
	_check("steward order exists in real chronicle", found)
	var bar: Node = _main.get_node("UI/OrderBar")
	bar.call("clear_background_toasts")
	_hud._sequence = before
	_hud._chronicle_cursor = 0
	var expected: int = 0
	for row: Dictionary in rows:
		if int(row.get("kind", -1)) == 45 and int(row.get("value", -1)) > before:
			expected += 1
	for _page: int in range(32):
		_hud._poll_steward_notifications(state)
		if _hud._chronicle_cursor >= rows.size(): break
	_check("every new real Steward record queued", (bar.get("_background_toasts") as Array).size() == expected and expected > 0)
	_check("every queued actual notification includes localized action", _queued_action_labels_valid(state))
	_hud._poll_steward_notifications(state)
	_check("repeated chronicle poll does not duplicate", (bar.get("_background_toasts") as Array).size() == expected)
	_hud._world_reset()
	_hud._poll_steward_notifications(state)
	_check("reset attach baseline does not flood history", (bar.get("_background_toasts") as Array).is_empty() and _hud._sequence == int(state.get("sequence", -1)))
	_check("steward chronicle formatter gives localized sentence", formatted.replace("\u2060", "").contains(str(root.get_node("Locale").call("ltr", "MUX_STEWARD_ACTION"))) and not "MUX_" in formatted)
	var chronicle: Control = _main.get_node("UI/ChroniclePanel") as Control
	chronicle.show()
	chronicle.call("_process", 0.0)
	chronicle.call("_refresh")
	_check("chronicle panel includes steward line", str(chronicle.call("plain_text")).replace("\u2060", "").contains(str(root.get_node("Locale").call("ltr", "MUX_STEWARD_ACTION"))))
	chronicle.hide()
	var housing: Dictionary = _housing_evidence()
	while not bool(housing["has_wall_and_bed_plans"]) and int(housing["tick"]) < 14400:
		SimReads.mutate(_world, "advance_ticks", [mini(60, 14400 - int(housing["tick"]))])
		await _frames()
		housing = _housing_evidence()
	_check("by tick 14400 steward housing provenance has actual wall and bed plans", bool(housing["has_wall_and_bed_plans"]) and (housing["steward"]["accepted_rules"] as PackedInt32Array).has(3))
	var first_plans: Dictionary = housing.duplicate(true)
	while not bool(housing["construction_started"]) and int(housing["tick"]) < 14400:
		SimReads.mutate(_world, "advance_ticks", [mini(60, 14400 - int(housing["tick"]))])
		await _frames()
		housing = _housing_evidence()
	_check("by tick 14400 steward housing construction has started", bool(housing["construction_started"]))
	housing["first_plans_observed"] = first_plans
	print("MUX_STEWARD_HOUSING_EVIDENCE tick=%d wall_plans=%d bed_plans=%d delivered=%d progress=%d built_walls=%d built_beds=%d construction_started=%s evidence=%s" % [housing["tick"], housing["wall_plans"], housing["bed_plans"], housing["delivered_total"], housing["progress_total"], housing["built_walls"], housing["built_beds"], housing["construction_started"], _out.path_join("steward-housing-state.json")])
	var evidence: FileAccess = FileAccess.open(_out.path_join("steward-housing-state.json"), FileAccess.WRITE)
	if evidence != null:
		evidence.store_string(JSON.stringify(housing, "\t"))
		evidence.close()
	_camera.call("step_zoom", -1)
	await create_timer(0.18).timeout
	var plans: Array = first_plans["housing_sites"]
	if not plans.is_empty():
		var centre: Vector2 = Vector2.ZERO
		for site: Dictionary in plans: centre += Vector2(float(site["x"]), float(site["y"]))
		centre /= float(plans.size())
		_camera.call("center_on_tile", Vector2i(roundi(centre.x), roundi(centre.y)))
	else:
		_camera.call("center_on_camp")
	await _capture("steward-housing-sites-tick-%d" % int(housing["tick"]))
	await _completed_housing_check()

func _completed_housing_check() -> void:
	var state: Dictionary = _completed_housing_evidence()
	var clock: Dictionary = state["clock"]
	var ticks_per_day: int = int(clock.get("ticks_per_day", 0))
	# Match the probe deadline; display clock offset is evidence only.
	var deadline: int = 10 * ticks_per_day
	while bool(state["facts_valid"]) and not bool(state["complete"]) and int(state["tick"]) < deadline:
		SimReads.mutate(_world, "advance_ticks", [mini(60, deadline - int(state["tick"]))])
		await _frames()
		state = _completed_housing_evidence()
	state["deadline_tick"] = deadline
	state["deadline_simulation_days"] = 10
	var proved: bool = bool(state["complete"]) and int(state["tick"]) <= deadline
	state["completed_within_deadline"] = proved
	_check("completed housing observer fields available", bool(state["facts_valid"]))
	_check("by simulation day 10 roofed built-bed capacity covers all living people", proved)
	var evidence: FileAccess = FileAccess.open(_out.path_join("steward-housing-completion-state.json"), FileAccess.WRITE)
	_check("housing completion evidence file opened", evidence != null)
	if evidence != null:
		evidence.store_string(JSON.stringify(state, "\t"))
		evidence.close()
	print("MUX_HOUSING_COMPLETION tick=%d complete=%s deadline=%d living=%d assigned=%d" % [state["tick"], proved, deadline, state["living_count"], state["roofed_bed_assigned_count"]])
	if proved:
		var centre: Vector2i = Vector2i(state["capture_x"], state["capture_y"])
		_camera.call("center_on_tile", centre)
		await _capture("steward-completed-house-tick-%d" % int(state["tick"]))

func _completed_housing_evidence() -> Dictionary:
	var raw: Dictionary = SimReads.read(_world, "get_ui_facts")
	var rooms: Dictionary = SimReads.read(_world, "get_rooms")
	var space: Dictionary = SimReads.read(_world, "get_space")
	var people: Dictionary = raw.get("people", {})
	var result: Dictionary = {"facts_valid": false, "complete": false, "tick": int(raw.get("tick", -1)), "living_count": 0, "roofed_bed_assigned_count": 0, "capture_x": 0, "capture_y": 0, "raw_rooms": rooms, "raw_space": space, "raw_people": people, "clock": {"ticks_per_day": raw.get("ticks_per_day", 0), "clock_offset_ticks": raw.get("clock_offset_ticks", 0), "day_index": raw.get("day", -1)}, "source": "Unmodified simulation and steward orders. Completion matches walled_beds_for_all: built beds inside roofed rooms >= living population. Assigned-bed coverage is recorded separately, not required (infants may share). No completed screenshot is emitted on missing facts or timeout."}
	if not raw.has_all(["tick", "ticks_per_day", "clock_offset_ticks", "day"]) or int(raw["ticks_per_day"]) <= 0:
		return result
	if not _aligned_columns(people, ["ids", "bed_xs", "bed_ys"]) or not _aligned_columns(rooms, ["ids", "roofed", "centre_xs", "centre_ys"]) or not _aligned_columns(rooms, ["tile_xs", "tile_ys", "tile_rooms"]) or not _aligned_columns(space, ["bed_xs", "bed_ys"]):
		return result
	result["facts_valid"] = true
	var roofed_rooms: Dictionary = {}
	for i: int in rooms["ids"].size():
		if rooms["roofed"][i]:
			roofed_rooms[rooms["ids"][i]] = Vector2i(rooms["centre_xs"][i], rooms["centre_ys"][i])
	var roofed_tiles: Dictionary = {}
	for i: int in rooms["tile_xs"].size():
		if roofed_rooms.has(rooms["tile_rooms"][i]):
			roofed_tiles[Vector2i(rooms["tile_xs"][i], rooms["tile_ys"][i])] = rooms["tile_rooms"][i]
	var built_beds: Dictionary = {}
	for i: int in space["bed_xs"].size():
		built_beds[Vector2i(space["bed_xs"][i], space["bed_ys"][i])] = true
	var roofed_beds: int = 0
	for bed: Vector2i in built_beds:
		if roofed_tiles.has(bed):
			roofed_beds += 1
			var centre: Vector2i = roofed_rooms[roofed_tiles[bed]]
			result["capture_x"] = centre.x
			result["capture_y"] = centre.y
	var missing: Array[int] = []
	for i: int in people["ids"].size():
		var bed: Vector2i = Vector2i(people["bed_xs"][i], people["bed_ys"][i])
		if built_beds.has(bed) and roofed_tiles.has(bed):
			result["roofed_bed_assigned_count"] += 1
			var centre: Vector2i = roofed_rooms[roofed_tiles[bed]]
			result["capture_x"] = centre.x
			result["capture_y"] = centre.y
		else:
			missing.append(people["ids"][i])
	result["living_count"] = people["ids"].size()
	result["without_roofed_bed_assignment_ids"] = missing
	result["roofed_built_bed_count"] = roofed_beds
	result["roofed_room_count"] = roofed_rooms.size()
	result["complete"] = not roofed_rooms.is_empty() and people["ids"].size() > 0 and roofed_beds >= people["ids"].size()
	return result

func _aligned_columns(table: Dictionary, columns: Array[String]) -> bool:
	var length: int = -1
	for column: String in columns:
		if not table.has(column): return false
		var values: Variant = table[column]
		if not (values is Array or values is PackedInt32Array or values is PackedInt64Array or values is PackedByteArray): return false
		if length >= 0 and values.size() != length: return false
		length = values.size()
	return true

func _housing_evidence() -> Dictionary:
	var overlay: Dictionary = SimReads.read(_world, "get_colony_overlay")
	var kinds: PackedInt64Array = overlay.get("site_kinds", PackedInt64Array())
	var sites: Array[Dictionary] = []
	var walls: int = 0
	var beds: int = 0
	var delivered: int = 0
	var progress: int = 0
	for index: int in kinds.size():
		var definition: Dictionary = ArtData.binding("blueprints", kinds[index])
		var wall: bool = str(definition.get("kind", "")) == "wall"
		var bed: bool = bool(definition.get("bed", false))
		if not wall and not bed: continue
		walls += int(wall)
		beds += int(bed)
		delivered += int(overlay["site_delivered"][index])
		progress += int(overlay["site_progress"][index])
		sites.append({"blueprint_code": kinds[index], "category": "wall" if wall else "bed", "x": overlay["site_xs"][index], "y": overlay["site_ys"][index], "delivered": overlay["site_delivered"][index], "needed": overlay["site_needed"][index], "progress": overlay["site_progress"][index], "required": overlay["site_required"][index]})
	var space: Dictionary = SimReads.read(_world, "get_space")
	var built_walls: int = (space.get("wall_xs", PackedInt32Array()) as PackedInt32Array).size()
	var built_beds: int = (space.get("bed_xs", PackedInt32Array()) as PackedInt32Array).size()
	return {"construction_started": delivered > 0 or progress > 0 or (built_walls > 0 and built_beds > 0), "delivered_total": delivered, "progress_total": progress, "built_walls": built_walls, "built_beds": built_beds, "tick": int(SimReads.read(_world, "get_time_snapshot").get("tick", -1)), "steward": SimReads.read(_world, "get_steward"), "has_wall_and_bed_plans": walls > 0 and beds > 0, "wall_plans": walls, "bed_plans": beds, "housing_sites": sites, "registered_buildings": SimReads.read(_world, "get_building_snapshot"), "built_space": space, "source": "Actual paused simulation, advanced in batches of at most 60 ticks; categories resolved from blueprint registry. No player construction commands submitted. Accepted housing rule plus observed plans is causal evidence; bridge exposes no per-site ownership IDs. Plans are not a completed-house claim."}


# Temporary layout timing evidence; preserve the original assertion until the observed rectangles explain it.
func _help_rect_diagnostic(phase: String) -> void:
	print("MUX_HELP_RECT phase=%s process_frame=%d viewport=%s help=%s minimum=%s" % [phase, Engine.get_process_frames(), root.get_visible_rect(), _hud.help.get_global_rect(), _hud.help.get_combined_minimum_size()])

func _notification_checks() -> void:
	var bar: Node = _main.get_node("UI/OrderBar")
	bar.set_process(false)
	bar.call("clear_background_toasts")
	bar.call("_process", 100.0)
	bar.call("queue_background_toast", "first", 101)
	bar.call("queue_background_toast", "second", 102)
	bar.call("queue_background_toast", "first", 103)
	bar.call("_process", 0.0)
	_check("background first action visible", int(bar.get("_background_sequence")) == 101)
	bar.call("show_toast", "player feedback")
	_check("player feedback immediately replaces background", (bar.get("_toast") as Label).text == "player feedback")
	bar.call("_process", 100.0)
	_check("interrupted action resumes before next", int(bar.get("_background_sequence")) == 101)
	bar.call("_process", 100.0)
	_check("second action gets distinct visible toast", int(bar.get("_background_sequence")) == 102)
	bar.call("_process", 100.0)
	_check("same rule action retains distinct sequence", int(bar.get("_background_sequence")) == 103)
	bar.call("clear_background_toasts")
	_check("reset clears pending and visible background", (bar.get("_background_toasts") as Array).is_empty() and int(bar.get("_background_sequence")) == -1)
	bar.set_process(true)

func _locale_checks() -> void:
	var locale: Node = root.get_node("Locale")
	var original: String = str(locale.get("_current_lang"))
	var bar: Node = _main.get_node("UI/OrderBar")
	var old_mode: int = int(bar.call("current_mode"))
	var tool: Node = bar.get("_tool")
	var selected_kind: String = str(tool.get("build_kind"))
	tool.call("set_build", selected_kind)
	for language: String in ["en", "ko"]:
		locale.call("set_language", language)
		await _frames()
		_orderbar_locale_assertions(language, selected_kind)
		_check(language + " camp label refreshes", _hud.controls.button_for("home").text == str(locale.call("ltr", "UI_MUX_CAMP")))
		_check(language + " camera tooltip refreshes", _hud.controls.button_for("home").tooltip_text == _hud.catalog.tooltip("home"))
		_check(language + " minimap label refreshes", (_hud.minimap.get("_toggle") as Button).text == str(locale.call("ltr", "UI_MUX_MINIMAP")))
		_check(language + " minimap hint refreshes", (_hud.minimap.get("_canvas") as Control).tooltip_text == str(locale.call("ltr", "UI_MUX_MINIMAP_HINT")))
		for entry: Array in [["PrioritiesButton", "priorities"], ["TechButton", "tech"], ["NeighboursButton", "neighbours"]]:
			_check(language + " action tooltip " + str(entry[1]), (_hud.find_child(str(entry[0]), true, false) as Button).tooltip_text == _hud.catalog.tooltip(str(entry[1])))
		if language == "en":
			for child: Node in _main.get_node("UI").get_children():
				if child is Control and str(child.name).ends_with("Panel"): child.hide()
			await _newborn_title_fit_check()
			await _capture("english-controls")
	locale.call("set_language", original)
	bar.call("select_mode", old_mode)

func _observe_live_routes() -> void:
	# Snapshot identities are real; only the alert condition itself is injected.
	var rooms: Dictionary = SimReads.read(_world, "get_rooms")
	if not _live_routes.has("room") and not rooms.get("ids", []).is_empty():
		_live_alert_route("storehouse_full", int(rooms["ids"][0]), Vector2i(rooms["centre_xs"][0], rooms["centre_ys"][0]))
	var animals: Dictionary = SimReads.read(_world, "get_animals")
	if not _live_routes.has("animal") and not animals.get("ids", []).is_empty():
		_live_alert_route("wolves_approaching", int(animals["ids"][0]), Vector2i(animals["xs"][0], animals["ys"][0]))
	var neighbours: Dictionary = SimReads.read(_world, "get_neighbours")
	if not _live_routes.has("migrant"):
		for wave: Dictionary in Neighbours.waves(neighbours):
			if int(wave["state"]) == Neighbours.WAITING and int(wave["answer"]) == Neighbours.ANSWER_NONE:
				_live_alert_route("migrants_waiting", int(wave["id"]), wave["tile"])
				break
	if not _live_routes.has("trader"):
		for caravan: Dictionary in Neighbours.caravans(neighbours):
			if int(caravan["state"]) == Neighbours.WAITING:
				_live_alert_route("trader_present", int(caravan["id"]), caravan["tile"])
				break

func _live_alert_route(alert_id: String, identity: int, tile: Vector2i) -> void:
	for test_case: Dictionary in AlertCases._cases():
		if str(test_case["id"]) != alert_id: continue
		var kind: String = str(test_case["kind"])
		var facts: Dictionary = AlertCases._facts()
		var row: Dictionary = (test_case["row"] as Dictionary).duplicate(true)
		row.merge({"id": identity, "x": tile.x, "y": tile.y}, true)
		facts[test_case["table"]] = [row]
		_hud.alerts.refresh_from_facts(facts)
		var button: Button = _hud.alerts.find_child(alert_id, true, false) as Button
		_check("live " + kind + " alert exists", button != null)
		if button == null:
			_restore_live_alert_facts("missing live " + kind + " button")
			return
		button.pressed.emit()
		if kind == "animal":
			var causal: Control = _main.get_node("UI/CausalPanel") as Control
			_check("live animal navigates without opening why", not causal.visible)
			var half_view: Vector2 = root.get_visible_rect().size / (2.0 * _camera.zoom.x)
			_check("live animal alert brings tile into camera view", Rect2(_camera.position - half_view, half_view * 2.0).has_point(View.tile_centre(tile)))
			_live_routes[kind] = identity
			causal.hide()
			_restore_live_alert_facts("after live " + kind + " route")
			return
		var panel_name: String = {"room": "RoomPanel", "migrant": "MigrantPanel", "trader": "TradePanel"}[kind]
		var getter: String = {"room": "current_room", "migrant": "shown_wave", "trader": "shown_caravan"}[kind]
		var panel: Control = _main.get_node("UI/" + panel_name) as Control
		_check("live " + kind + " alert selects exact identity", panel.visible and int(panel.call(getter)) == identity)
		_check("live " + kind + " alert reaches camera tile", _camera.position.distance_to(View.tile_centre(tile)) < 2.0)
		_live_routes[kind] = identity
		panel.hide()
		_restore_live_alert_facts("after live " + kind + " route")

func _restore_live_alert_facts(context: String) -> Dictionary:
	var before: Array[Dictionary] = _hud.alerts.evaluator.active_alerts()
	var elapsed: float = _hud.alerts.evaluator._elapsed
	var raw: Dictionary = SimReads.read(_world, "get_ui_facts")
	var facts: Dictionary = FactsAdapter.adapt(raw)
	_check(context + " accepts current production facts", _hud.alerts.refresh_from_facts(facts))
	var after: Array[Dictionary] = _hud.alerts.evaluator.active_alerts()
	var actual_full: bool = false
	for room: Dictionary in facts.get("storehouses", []):
		if int(room.get("role", -1)) == 2 and int(room.get("capacity", 0)) > 0 and int(room.get("used", 0)) >= int(room["capacity"]):
			actual_full = true
	var displayed_full: bool = false
	for alert: Dictionary in after:
		if str(alert["id"]) == "storehouse_full": displayed_full = true
	_check(context + " full alert requires real capacity evidence, not synthetic 8/8", not displayed_full or actual_full)
	_check(context + " preserves production poll phase", _hud.alerts.evaluator._elapsed == elapsed)
	var provenance: Dictionary = {"context": context, "source": "SimReads.get_ui_facts -> production FactsAdapter -> alerts.refresh_from_facts", "tick": raw.get("tick", -1), "process_frame": Engine.get_process_frames(), "poll_elapsed": elapsed, "before": before, "after": after, "raw": raw, "storehouses": facts.get("storehouses", []), "synthetic_fixture": {"role": 2, "used": 8, "capacity": 8}, "synthetic_fixture_is_not_evidence": true}
	print("MUX_ALERT_PROVENANCE " + JSON.stringify(provenance))
	return provenance

func _finish_live_routes() -> void:
	var info: Dictionary = SimReads.read(_world, "get_world_info")
	var deadline: int = 30 * int(info.get("ticks_per_day", 1440))
	for _batch: int in range(360):
		if _live_routes.size() >= 4 or int(SimReads.read(_world, "get_time_snapshot").get("tick", 0)) >= deadline:
			break
		SimReads.mutate(_world, "advance_ticks", [120])
		await _frames(1)
	for kind: String in ["room", "migrant", "trader", "animal"]:
		_check("real valid alert fixture reached: " + kind, _live_routes.has(kind))

func _notification_lifecycle_checks() -> void:
	_observe_routes = false
	_hud.set_process(false)
	var bar: Node = _main.get_node("UI/OrderBar")
	bar.set_process(false)
	SimReads.mutate(_world, "reset_world", [0])
	SimReads.mutate(_world, "reset_world", [0])
	_check("two same-frame resets have two pending native receipts", _hud._pending_native_resets == 2)
	_check("reset callback establishes tick-zero notification baseline", _hud._chronicle_cursor >= 0 and _hud._sequence == 0)
	SimReads.mutate(_world, "advance_ticks", [60])
	var state: Dictionary = _world.get_steward()
	_hud._poll_steward_notifications(state)
	var expected: int = int(state.get("sequence", 0))
	_check("first post-reset batch before polling remains queued", expected > 0 and (bar.get("_background_toasts") as Array).size() == expected)
	await _frames()
	_check("deferred reset receipts preserve new actions", _hud._pending_native_resets == 0 and (bar.get("_background_toasts") as Array).size() == expected)
	var save_path: String = _out.path_join("notification-load.wsave")
	_check("notification history save succeeds", bool(SimReads.mutate(_world, "save_world", [save_path])))
	bar.call("queue_background_toast", "stale prior world", 99999)
	_check("notification history load succeeds", bool(SimReads.mutate(_world, "load_world", [save_path])))
	await _frames()
	_hud._poll_steward_notifications(_world.get_steward())
	_check("actual load baselines history and clears prior queue", (bar.get("_background_toasts") as Array).is_empty() and _hud._sequence == expected)
	bar.call("queue_background_toast", "stale rollback", 99999)
	_hud._sequence = expected + 1
	_hud._poll_steward_notifications(_world.get_steward())
	_check("sequence rollback clears stale background queue", (bar.get("_background_toasts") as Array).is_empty())
	bar.call("queue_background_toast", "keep after failed load", 777)
	var pending_before: int = _hud._pending_native_resets
	_check("failed load keeps current world", not bool(SimReads.mutate(_world, "load_world", [_out.path_join("missing-notification-save.wsave")])))
	_check("failed load does not baseline or consume reset receipt", _hud._pending_native_resets == pending_before and (bar.get("_background_toasts") as Array).size() == 1)
	bar.call("clear_background_toasts")
	bar.set_process(true)
	_hud.set_process(true)

func _queued_action_labels_valid(state: Dictionary) -> bool:
	var labels: Dictionary = state.get("rules", {})
	var prefix: String = str(root.get_node("Locale").call("ltr", "MUX_STEWARD_ACTION")) + " "
	var queue: Array = _main.get_node("UI/OrderBar").get("_background_toasts")
	if queue.is_empty(): return false
	for notification: Dictionary in queue:
		var text: String = str(notification.get("text", ""))
		var matched: bool = false
		var defence_template: String = str(root.get_node("Locale").call("ltr", "MUX_STEWARD_DEFENCE_ACTION"))
		var parts: PackedStringArray = defence_template.split("{count}")
		var plain: String = text.replace("\u2060", "")
		if parts.size() == 2 and plain.begins_with(parts[0]) and plain.ends_with(parts[1]):
			var count: String = plain.trim_prefix(parts[0]).trim_suffix(parts[1])
			matched = count.is_valid_int() and int(count) > 0 and defence_template.format({"count": int(count)}) == plain
		for key: Variant in labels.values():
			var action: String = str(root.get_node("Locale").call("ltr", str(key)))
			if not action.strip_edges().is_empty() and action != str(key) and text.replace("\u2060", "") == prefix + action:
				matched = true
		if not matched: return false
	return true

func _orderbar_locale_assertions(language: String, selected_kind: String) -> void:
	var bar: Node = _main.get_node("UI/OrderBar")
	var locale: Node = root.get_node("Locale")
	var main_buttons: Array = bar.get("_buttons")
	var build_buttons: Array = bar.get("_menu_buttons")
	var gather_buttons: Array = bar.get("_dig_buttons")
	var main_ok: bool = true
	var build_ok: bool = true
	var gather_ok: bool = true
	for i: int in main_buttons.size():
		var entry: Dictionary = OrderBar.MODES[i]
		main_ok = main_ok and (main_buttons[i] as Button).text == "%s %s" % [OS.get_keycode_string(int(entry["hotkey"])), locale.call("ltr", str(entry["key"]))]
	for i: int in build_buttons.size():
		var kind: Dictionary = Economy.build_kind(Economy.menu_order()[i])
		build_ok = build_ok and (build_buttons[i] as Button).text == "%s %s" % [OrderBar.MENU_LABELS[i], locale.call("ltr", str(kind["key"]))]
	for i: int in gather_buttons.size():
		gather_ok = gather_ok and (gather_buttons[i] as Button).text == "%d %s" % [i + 1, locale.call("ltr", str(OrderBar.DIG_MENU[i]["key"]))]
	_check(language + " all order buttons refresh", main_ok)
	_check(language + " all build buttons refresh", build_ok)
	_check(language + " all gather buttons refresh", gather_ok)
	_check(language + " rooms label refreshes", (bar.get("_rooms_button") as Button).text == str(locale.call("ltr", "UI_ROOMS_TOGGLE")))
	_check(language + " current build hint refreshes", (bar.get("_hint") as Label).text == OrderBar.build_hint(locale, Economy.build_code(selected_kind)))
	_check(language + " language switch preserves mode and selected tool", int(bar.call("current_mode")) == OrderBar.BUILD_MODE and str((bar.get("_tool") as Node).get("build_kind")) == selected_kind)

func _newborn_title_fit_check() -> void:
	_hud.store.set_value("ui_scale", 1.0)
	await _frames()
	var button: Button = _hud.alerts.find_child("newborn_without_bed", true, false) as Button
	_check("newborn alert available at 1x", button != null and _hud.alerts.scale.is_equal_approx(Vector2.ONE))
	if button == null: return
	var font: Font = button.get_theme_font("font")
	var width: float = font.get_string_size(button.text, HORIZONTAL_ALIGNMENT_LEFT, -1, button.get_theme_font_size("font_size")).x
	var available: float = button.size.x - button.get_theme_stylebox("normal").get_minimum_size().x
	_check("English newborn title including count fits rendered button", width <= available)
	_check("newborn tooltip retains household explanation", button.tooltip_text.contains(str(root.get_node("Locale").call("ltr", "MUX_ALERT_NEWBORN_WITHOUT_BED_WHY"))))
	print("MUX_NEWBORN_TITLE_FIT text=%s text_px=%s available_px=%s" % [button.text, width, available])
