# Exact AgentLayer badge descriptors, packing and leaders against frozen pre-optimization bodies.
# Bridge methods: none. Runs on the baseline before production patch; --expect-lazy adds call-count controls.
extends SceneTree

const Agents = preload("res://scripts/world/colony/agent_layer.gd")
const Reference = preload("res://tools/v8/fixtures/mux2_badge_local_reference.gd")
const Layout = preload("res://scripts/world/colony/marker_layout.gd")
const Art = preload("res://scripts/world/art/art_data.gd")
var _checks: int = 0
var _failures: int = 0

class CountedAgents:
	extends "res://scripts/world/colony/agent_layer.gd"
	var fallback_calls: int = 0

	func drawn_rect(bits: int) -> Rect2:
		fallback_calls += 1
		return super.drawn_rect(bits)


func _init() -> void:
	call_deferred("_run")


func _check(label: String, valid: bool) -> void:
	_checks += 1
	_failures += 0 if valid else 1
	print("MUX2_BADGE_LOCAL_%s %s" % ["PASS" if valid else "FAIL", label])


func _populate(layer: Node2D, scenario: int) -> Dictionary:
	var people: Dictionary = {}
	var rectangles: Dictionary = {}
	var identity: int = 0
	for group: int in 3:
		var position: Vector2 = Vector2(240 + group * 300, 380 + group * 40)
		if scenario == 1:
			position = Vector2(400 + group * 32, 400 + group * 64)
		for member: int in (3 if group == 2 else 2):
			identity += 1
			var person: Agents.Person = Agents.Person.new()
			person.bits = identity
			person.agent_id = 100 - identity
			person.position = position
			person.activity = "sleep" if scenario == 2 else "idle"
			layer.add_child(person)
			people[identity] = person
			var rectangle: Rect2 = Rect2(position + Vector2(-16, -64), Vector2(32, 64))
			if scenario == 4:
				rectangle.position.x += float(member) * 0.0005
			if scenario != 3 and not (scenario == 2 and identity in [2, 6]):
				rectangles[identity] = rectangle
	layer.set("_persons", people)
	return rectangles


func _missing_calls(groups: Array, rectangles: Dictionary) -> int:
	var count: int = 0
	for group: Dictionary in groups:
		var members: Array = group["members"]
		if not rectangles.has(members[0]):
			count += 1
		for bits: int in members:
			if not rectangles.has(bits):
				count += 1
	return count


func _compare(actual: Node2D, expected: Node2D, rectangles: Dictionary, label: String) -> void:
	actual.set("fallback_calls", 0)
	expected.call("_update_coincident_badges", rectangles)
	actual.call("_update_coincident_badges", rectangles)
	var source: Array = actual.get("_coincident_groups")
	var badges: Array = actual.get("_coincident_badges")
	_check(label + "_source_descriptors", source == expected.get("_coincident_groups"))
	_check(label + "_packed_badge_descriptors", badges == expected.get("_coincident_badges"))
	var actual_leaders: Node2D = actual.get("_badge_leaders")
	var expected_leaders: Node2D = expected.get("_badge_leaders")
	_check(label + "_clipped_endpoint_pairs", actual_leaders.get("segments") == expected_leaders.get("segments") and actual_leaders.get("aggregate_segments") == expected_leaders.get("aggregate_segments") and actual_leaders.get("active_segments") == expected_leaders.get("active_segments"))
	_check(label + "_leader_style", actual_leaders.get("scale_width") == expected_leaders.get("scale_width") and actual_leaders.get("light") == expected_leaders.get("light") and actual_leaders.get("extent_style") == expected_leaders.get("extent_style"))
	_check(label + "_exhaustion", actual.get("badge_layout_exhausted") == expected.get("badge_layout_exhausted"))
	var members: Array[int] = []
	var total: int = 0
	for badge: Dictionary in badges:
		members.append_array(badge["members"])
		total += int(badge["member_count"])
	members.sort()
	_check(label + "_full_members_and_count", members == [1, 2, 3, 4, 5, 6, 7] and total == 7)
	if OS.get_cmdline_user_args().has("--expect-lazy"):
		_check(label + "_only_missing_rect_calls", int(actual.get("fallback_calls")) == _missing_calls(source, rectangles))


func _run() -> void:
	root.size = Vector2i(960, 540)
	var params: Dictionary = Art.params()
	var saved_labels: Variant = params.get("labels")
	var main: Node = Node.new()
	main.name = "Main"
	root.add_child(main)
	var ui: Control = Control.new()
	ui.name = "UI"
	main.add_child(ui)
	var panels: Array[Panel] = []
	for i: int in 2:
		var panel: Panel = Panel.new()
		ui.add_child(panel)
		panels.append(panel)
	var primary_font: Font = load("res://assets/fonts/galmuri/Galmuri11.ttf") as Font
	var other_font: Font = load("res://assets/fonts/galmuri/Galmuri9.ttf") as Font
	_check("both_font_fixtures_loaded", primary_font != null and other_font != null)
	for scenario: int in 7:
		var actual: Node2D = CountedAgents.new()
		var expected: Node2D = Reference.new()
		root.add_child(actual)
		root.add_child(expected)
		actual.set_process(false)
		expected.set_process(false)
		var rectangles: Dictionary = _populate(actual, scenario)
		_populate(expected, scenario)
		# Same instances, new parameters/font/zoom each invocation: no stale per-frame memo allowed.
		for phase: int in 4:
			var zoom: float = 0.75 if phase == 2 else 0.5
			var px: int = [24, 18, 31, 24][phase]
			params["labels"] = {"font_px": px, "follow_camera": phase != 1}
			var font: Font = null if phase == 2 else (other_font if phase == 3 else primary_font)
			for layer: Node2D in [actual, expected]:
				layer.set("_zoom", zoom)
				layer.scale = Vector2(zoom, zoom)
				layer.position = Vector2(0.25, -0.75) if scenario == 4 else Vector2.ZERO
				layer.set("_font", font)
			for panel: Panel in panels:
				panel.visible = scenario >= 5
				var panel_rect: Rect2 = actual.get_global_transform_with_canvas() * (rectangles[1] as Rect2) if scenario == 5 else actual.get_viewport_rect()
				panel.position = panel_rect.position
				panel.size = panel_rect.size
			if scenario == 6:
				var viewport: Rect2 = actual.get_viewport_rect()
				var observed_ui: Array[Rect2] = []
				Layout.ui_rects(ui, observed_ui)
				var covered: bool = observed_ui.size() == 2
				for rectangle: Rect2 in observed_ui:
					covered = covered and rectangle.encloses(viewport)
				_check("full_viewport_ui_collected_%d" % phase, actual.get_node_or_null("/root/Main/UI") == ui and covered)
				print("MUX2_BADGE_LOCAL_VIEWPORT %s blockers=%s" % [viewport, observed_ui])
			_compare(actual, expected, rectangles, "%d_%d" % [scenario, phase])
			if scenario == 6:
				_check("covered_viewport_reports_exhaustion_%d" % phase, int(actual.get("badge_layout_exhausted")) > 0)
		actual.free()
		expected.free()
	params["labels"] = saved_labels
	main.free()
	print("MUX2_BADGE_LOCAL_DONE checks=%d failures=%d" % [_checks, _failures])
	quit(0 if _failures == 0 else 1)
