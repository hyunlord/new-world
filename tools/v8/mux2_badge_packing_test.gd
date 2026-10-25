# M-UX2 label-slot policy and clipped-leader geometry controls.
# Bridge methods: none. Actual images and FPS remain separate acceptance gates.
extends SceneTree

const Layout = preload("res://scripts/world/colony/marker_layout.gd")
var _checks: int = 0
var _failures: int = 0


func _init() -> void:
	call_deferred("_run")


func _check(label: String, valid: bool) -> void:
	_checks += 1
	_failures += 0 if valid else 1
	print("MUX2_BADGE_PACKING_%s %s" % ["PASS" if valid else "FAIL", label])


func _packed(figure: Rect2, size: Vector2, viewport: Rect2, blocked: Array[Rect2]) -> Rect2:
	return Layout.badge_packed(figure, size, viewport, Layout.badge_index(blocked), Layout.badge_grid(size, viewport, blocked))


func _run() -> void:
	var viewport: Rect2 = Rect2(0, 0, 800, 600)
	var figure: Rect2 = Rect2(400, 300, 16, 32)
	var own: Array[Rect2] = [figure]
	_check("preferred_above_preserved", _packed(figure, Vector2(24, 30), viewport, own) == Rect2(396, 266, 24, 30))
	var neighbor: Rect2 = Rect2(398, 267, 19, 33)
	var neighbors: Array[Rect2] = [figure, neighbor]
	var side: Rect2 = _packed(figure, Vector2(24, 30), viewport, neighbors)
	_check("adjacent_third_person_not_covered", not side.intersects(neighbor.grow(2)))
	_check("left_preferred_after_above_blocked", side == Rect2(372, 301, 24, 30))
	var crowd: Rect2 = Rect2(300, 200, 200, 200)
	var ui: Rect2 = Rect2(0, 0, 800, 80)
	var blocked: Array[Rect2] = [crowd, ui]
	var index: Dictionary = Layout.badge_index(blocked)
	var grid: Dictionary = Layout.badge_grid(Vector2(44, 30), viewport, blocked)
	var placements: Array[Rect2] = []
	for i: int in 100:
		var size: Vector2 = Vector2(24 + (i % 6) * 4, 30)
		var placed: Rect2 = Layout.badge_packed(figure, size, viewport, index, grid)
		var clear: bool = viewport.encloses(placed.grow(1))
		for obstacle: Rect2 in blocked:
			clear = clear and not placed.intersects(obstacle.grow(2))
		_check("dense_label_clear_%d" % i, clear)
		placements.append(placed)
		blocked.append(placed)
		Layout.badge_index_add(index, placed)
	_check("dense_no_exhaustion", int(grid["exhausted"]) == 0)
	var repeated: Array[Rect2] = [crowd, ui]
	var repeated_index: Dictionary = Layout.badge_index(repeated)
	var repeated_grid: Dictionary = Layout.badge_grid(Vector2(44, 30), viewport, repeated)
	var deterministic: bool = true
	for i: int in placements.size():
		var next: Rect2 = Layout.badge_packed(figure, Vector2(24 + (i % 6) * 4, 30), viewport, repeated_index, repeated_grid)
		deterministic = deterministic and next == placements[i]
		Layout.badge_index_add(repeated_index, next)
	_check("stable_order_deterministic", deterministic)
	var full: Array[Rect2] = [viewport]
	var exhausted: Dictionary = Layout.badge_grid(Vector2(24, 30), viewport, full)
	var fallback: Rect2 = Layout.badge_packed(figure, Vector2(24, 30), viewport, Layout.badge_index(full), exhausted)
	_check("exhaustion_observable", int(exhausted["exhausted"]) == 1)
	_check("exhaustion_retains_count_cue_rectangle", fallback == Rect2(396, 266, 24, 30))
	var negative_viewport: Rect2 = Rect2(-400.5, -300.5, 800, 600)
	var edge: Rect2 = Rect2(-400, -300, 16, 32)
	var edge_blocked: Array[Rect2] = [edge, Rect2(-400, -300, 100, 80)]
	var edge_label: Rect2 = _packed(edge, Vector2(24.5, 30.5), negative_viewport, edge_blocked)
	_check("negative_fractional_viewport", negative_viewport.encloses(edge_label.grow(1)) and not edge_label.intersects(edge_blocked[1].grow(2)))
	var blockers: Array[Rect2] = [Rect2(40, -10, 20, 20), Rect2(55, -10, 20, 20)]
	var clipped: PackedVector2Array = Layout.clipped_leader(Vector2(0, 0), Vector2(100, 0), Layout.badge_index(blockers))
	_check("overlapping_clip_intervals_merge", clipped.size() == 4 and clipped[0] == Vector2(0, 0) and clipped[1] == Vector2(37.5, 0) and clipped[2] == Vector2(77.5, 0) and clipped[3] == Vector2(100, 0))
	var reverse: PackedVector2Array = Layout.clipped_leader(Vector2(100, 0), Vector2(0, 0), Layout.badge_index(blockers))
	_check("reverse_clip_same_visible_geometry", reverse.size() == 4 and reverse[0].is_equal_approx(clipped[3]) and reverse[1].is_equal_approx(clipped[2]) and reverse[2].is_equal_approx(clipped[1]) and reverse[3].is_equal_approx(clipped[0]))
	_check("zero_length_has_no_stroke", Layout.clipped_leader(Vector2(0, 0), Vector2(0, 0), index).is_empty())
	var contact: Array[Rect2] = [Rect2(642.5, 2.5, 8, 8)]
	for endpoints: PackedVector2Array in [PackedVector2Array([Vector2(0, 448), Vector2(1280, -448)]), PackedVector2Array([Vector2(1280, -448), Vector2(0, 448)]), PackedVector2Array([Vector2(400, 400), Vector2(0, 0)])]:
		var pieces: PackedVector2Array = Layout.clipped_leader(endpoints[0], endpoints[1], Layout.badge_index(contact))
		var safe: bool = pieces.size() % 2 == 0
		for i: int in range(0, pieces.size(), 2):
			safe = safe and not Layout.segment_hits_rect(pieces[i], pieces[i + 1], contact[0].grow(2.0))
		_check("corner_negative_direction_stroke_clear_%s" % str(endpoints[0]), safe)
	print("MUX2_BADGE_PACKING_DONE checks=%d failures=%d" % [_checks, _failures])
	quit(0 if _failures == 0 else 1)
