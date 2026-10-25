# M-UX2 layout regression: coincident people, stable identity, UI occlusion and viewport edges.
# Bridge methods: none. Geometry checks supplement, never replace, the rendered visibility oracle.
extends SceneTree

const Layout = preload("res://scripts/world/colony/marker_layout.gd")
var _failures: int = 0
var _checks: int = 0


func _init() -> void:
	call_deferred("_run")


func _check(label: String, valid: bool) -> void:
	_checks += 1
	if not valid:
		_failures += 1
	print("MUX2_MARKER_LAYOUT_%s %s" % ["PASS" if valid else "FAIL", label])


func _run() -> void:
	var viewport: Rect2 = Rect2(0, 0, 800, 600)
	var points: PackedVector2Array = PackedVector2Array([Vector2(400, 300), Vector2(400, 300), Vector2(-100, -100), Vector2(4, 4)])
	var ids: PackedInt64Array = PackedInt64Array([20, 10, 30, 40])
	var blocked: Array[Rect2] = [Rect2(380, 280, 80, 50)]
	var placed: PackedVector2Array = Layout.place(points, ids, 24, true, viewport, blocked, 4)
	_check("one_marker_per_person", placed.size() == points.size())
	_check("overlap_separated", placed[0].distance_to(placed[1]) >= 32.0)
	_check("offscreen_stays_offscreen", placed[2] == points[2])
	_check("input_anchors_unchanged", points[0] == Vector2(400, 300) and points[3] == Vector2(4, 4))
	for i: int in [0, 1, 3]:
		var reserved: Rect2 = Rect2(placed[i] - Vector2(16, 30), Vector2(46, 46))
		_check("viewport_%d" % i, viewport.encloses(reserved))
		_check("ui_exclusion_%d" % i, not reserved.intersects(blocked[0]))
	var reversed: PackedVector2Array = Layout.place(PackedVector2Array([points[1], points[0], points[2], points[3]]), PackedInt64Array([10, 20, 30, 40]), 24, true, viewport, blocked, 4)
	_check("id_priority_independent_of_row_order", reversed[0] == placed[1] and reversed[1] == placed[0])
	_check("repeat_deterministic", Layout.place(points, ids, 24, true, viewport, blocked, 4) == placed)
	var ui: Control = Control.new()
	ui.size = viewport.size
	root.add_child(ui)
	var panel: Panel = Panel.new()
	panel.position = Vector2(100, 100)
	panel.size = Vector2(50, 40)
	ui.add_child(panel)
	var rects: Array[Rect2] = []
	Layout.ui_rects(ui, rects)
	_check("wrapper_not_false_occluder", rects.size() == 1 and rects[0] == Rect2(100, 100, 50, 40))
	panel.hide()
	rects.clear()
	Layout.ui_rects(ui, rects)
	_check("hidden_panel_not_occluder", rects.is_empty())
	var figure: Rect2 = Rect2(1016, 548, 16, 32)
	var neighbor: Rect2 = Rect2(1014, 515, 19, 33)
	var obstacles: Array[Rect2] = [figure, neighbor]
	var badge: Rect2 = Layout.badge(figure, Vector2(24, 30), Rect2(0, 0, 1920, 1080), obstacles)
	_check("badge_avoids_adjacent_person_regression", not badge.intersects(neighbor) and not badge.intersects(figure))
	_check("badge_uses_side_when_above_is_occupied", badge.end.x < figure.position.x or badge.position.x > figure.end.x)
	var edge: Rect2 = Rect2(3, 3, 16, 32)
	var edge_obstacles: Array[Rect2] = [edge, Rect2(20, 0, 100, 45)]
	var edge_badge: Rect2 = Layout.badge(edge, Vector2(24, 30), viewport, edge_obstacles)
	_check("badge_respects_screen_bounds", viewport.encloses(edge_badge))
	_check("badge_respects_visible_ui", not edge_badge.intersects(edge_obstacles[1]))
	var obstacle: Rect2 = Rect2(18, 0, 104, 47)
	_check("diagonal_leader_can_clear_corner", not Layout.segment_hits_rect(Vector2(14, 38), Vector2(42, 173), obstacle))
	_check("crossing_leader_rejected", Layout.segment_hits_rect(Vector2(10, 20), Vector2(140, 20), obstacle))
	_check("leader_start_inside_rejected", Layout.segment_hits_rect(Vector2(30, 20), Vector2(150, 100), obstacle))
	print("MUX2_MARKER_LAYOUT_DONE checks=%d failures=%d" % [_checks, _failures])
	quit(0 if _failures == 0 else 1)
