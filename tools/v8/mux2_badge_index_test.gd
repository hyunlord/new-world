# M-UX2 badge broad-phase and exhaustion proof regression against the original exhaustive search.
# Bridge methods: none. Synthetic geometry cannot replace GPU or FPS acceptance.
extends SceneTree

const Layout = preload("res://scripts/world/colony/marker_layout.gd")
var _failures: int = 0
var _checks: int = 0


func _init() -> void:
	call_deferred("_run")


func _check(label: String, valid: bool) -> void:
	_checks += 1
	_failures += 0 if valid else 1
	print("MUX2_BADGE_INDEX_%s %s" % ["PASS" if valid else "FAIL", label])


func _reference(figure: Rect2, size: Vector2, viewport: Rect2, blocked: Array[Rect2]) -> Rect2:
	var preferred: Vector2 = Vector2(figure.get_center().x - size.x * 0.5, figure.position.y - 4.0 - size.y).round()
	var nearby: Array[Vector2] = [preferred, Vector2(figure.position.x - 4.0 - size.x, figure.get_center().y - size.y * 0.5), Vector2(figure.end.x + 4.0, figure.get_center().y - size.y * 0.5), Vector2(figure.get_center().x - size.x * 0.5, figure.end.y + 4.0)]
	for point: Vector2 in nearby:
		var candidate: Rect2 = Rect2(point.round(), size)
		if _free(candidate, figure, viewport, blocked):
			return candidate
	var step: float = maxf(size.x, size.y) + 4.0
	var limit: int = ceili(maxf(viewport.size.x, viewport.size.y) / step)
	for ring: int in range(1, limit + 1):
		for y: int in range(-ring, ring + 1):
			for x: int in range(-ring, ring + 1):
				if maxi(absi(x), absi(y)) != ring:
					continue
				var candidate: Rect2 = Rect2((preferred + Vector2(x, y) * step).round(), size)
				if _free(candidate, figure, viewport, blocked):
					return candidate
	return Rect2(preferred, size)


func _free(label: Rect2, figure: Rect2, viewport: Rect2, blocked: Array[Rect2]) -> bool:
	if not viewport.encloses(label.grow(1.0)):
		return false
	var leader: PackedVector2Array = Layout.badge_leader(figure, label)
	for obstacle: Rect2 in blocked:
		if label.intersects(obstacle.grow(2.0)) or Layout.segment_hits_rect(leader[0], leader[1], obstacle.grow(2.5)):
			return false
	return true


func _run() -> void:
	var viewport: Rect2 = Rect2(-40, -30, 400, 300)
	for case_id: int in range(32):
		var figure: Rect2 = Rect2(-30 + (case_id * 43) % 330, -20 + (case_id * 31) % 230, 16, 32)
		var blocked: Array[Rect2] = [figure]
		for j: int in range(case_id % 12):
			blocked.append(Rect2(-40 + (j * 67 + case_id * 13) % 370, -30 + (j * 41 + case_id * 17) % 270, 19 + j, 33))
		var size: Vector2 = Vector2(24.5, 30.5)
		var index: Dictionary = Layout.badge_index(blocked)
		_check("reference_equivalence_%d" % case_id, Layout.badge(figure, size, viewport, blocked, index) == _reference(figure, size, viewport, blocked))
		var inserted: Rect2 = Rect2(figure.position - Vector2(10, 40), Vector2(42, 35))
		blocked.append(inserted)
		Layout.badge_index_add(index, inserted)
		_check("insert_equivalence_%d" % case_id, Layout.badge(figure, size, viewport, blocked, index) == _reference(figure, size, viewport, blocked))
	var figure: Rect2 = Rect2(100, 100, 16, 32)
	var covering: Array[Rect2] = [figure.grow(4)]
	_check("full_perimeter_proves_exhaustion", Layout.badge_start_blocked(figure, Layout.badge_index(covering)))
	_check("exhausted_reference_equivalence", Layout.badge(figure, Vector2(24, 30), viewport, covering) == _reference(figure, Vector2(24, 30), viewport, covering))
	var own: Array[Rect2] = [figure]
	_check("own_figure_not_false_exhaustion", not Layout.badge_start_blocked(figure, Layout.badge_index(own)))
	var partial: Array[Rect2] = [Rect2(90, 90, 40, 20)]
	_check("partial_perimeter_not_exhaustion", not Layout.badge_start_blocked(figure, Layout.badge_index(partial)))
	var split: Array[Rect2] = [figure, Rect2(90, 90, 15, 50), Rect2(110, 90, 20, 50)]
	_check("closed_interval_union_proves_exhaustion", Layout.badge_start_blocked(figure, Layout.badge_index(split)))
	var slit: Array[Rect2] = [figure, Rect2(90, 90, 14.9, 50), Rect2(110.1, 90, 20, 50)]
	_check("subpixel_gap_prevents_false_exhaustion", not Layout.badge_start_blocked(figure, Layout.badge_index(slit)))
	var split_without_body: Array[Rect2] = [Rect2(90, 90, 15, 50), Rect2(110, 90, 20, 50)]
	_check("union_coverage_without_enclosing_body", Layout.badge_start_blocked(figure, Layout.badge_index(split_without_body)))
	_check("union_coverage_reference", Layout.badge(figure, Vector2(24, 30), viewport, split_without_body) == _reference(figure, Vector2(24, 30), viewport, split_without_body))
	var large: Rect2 = Rect2(100, 100, 100, 100)
	var hollow: Array[Rect2] = [Rect2(90, 90, 120, 12), Rect2(90, 198, 120, 12), Rect2(90, 90, 12, 120), Rect2(198, 90, 12, 120)]
	_check("covered_perimeter_with_interior_hole_not_proof", not Layout.badge_start_blocked(large, Layout.badge_index(hollow)))
	var segments: Array[PackedVector2Array] = [
		PackedVector2Array([Vector2(0, 0), Vector2(400, 400)]),
		PackedVector2Array([Vector2(400, 0), Vector2(0, 400)]),
		PackedVector2Array([Vector2(-128, -128), Vector2(128, 128)]),
		PackedVector2Array([Vector2(128, -128), Vector2(-128, 128)]),
		PackedVector2Array([Vector2(64, 64), Vector2(64, 64)]),
		PackedVector2Array([Vector2(64, -100), Vector2(64, 400)]),
		PackedVector2Array([Vector2(-100, 64), Vector2(400, 64)]),
		PackedVector2Array([Vector2(127.999, -300), Vector2(128.001, 900)])]
	var contacts: Array[Rect2] = [Rect2(66.5, 66.5, 8, 8), Rect2(66.5, 53.5, 8, 8), Rect2(-61.5, -61.5, 8, 8), Rect2(125, 700, 12, 30), Rect2(900, 900, 20, 20)]
	for i: int in segments.size():
		for j: int in contacts.size():
			var single: Array[Rect2] = [contacts[j]]
			var segment: PackedVector2Array = segments[i]
			var expected: bool = Layout.segment_hits_rect(segment[0], segment[1], contacts[j].grow(2.5))
			_check("lazy_segment_reference_%d_%d" % [i, j], Layout.badge_segment_blocked(segment[0], segment[1], Layout.badge_index(single)) == expected)
	var corner_contact: Array[Rect2] = [Rect2(66.5, 66.5, 10, 10)]
	_check("mixed_direction_corner_contact_included", Layout.badge_segment_blocked(Vector2(128, 0), Vector2(0, 128), Layout.badge_index(corner_contact)))
	var drift_contact: Array[Rect2] = [Rect2(642.5, 2.5, 8, 8)]
	_check("float32_endpoint_corner_contact", Layout.badge_segment_blocked(Vector2(0, 448), Vector2(640, 0), Layout.badge_index(drift_contact)))
	_check("float32_interior_corner_contact", Layout.badge_segment_blocked(Vector2(0, 448), Vector2(1280, -448), Layout.badge_index(drift_contact)))
	print("MUX2_BADGE_INDEX_DONE checks=%d failures=%d" % [_checks, _failures])
	quit(0 if _failures == 0 else 1)
