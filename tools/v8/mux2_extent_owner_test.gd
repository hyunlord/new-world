# Exact endpoint-order oracle for per-call body-owner classification caching.
# Bridge methods: none. Baseline runs without --expect-cache; candidate adds one work-reduction check.
extends SceneTree

const Layout = preload("res://scripts/world/colony/marker_layout.gd")
const Reference = preload("res://tools/v8/fixtures/mux2_extent_owner_reference.gd")
var _checks: int = 0
var _failures: int = 0


func _init() -> void:
	call_deferred("_run")


func _check(label: String, valid: bool) -> void:
	_checks += 1
	_failures += 0 if valid else 1
	print("MUX2_EXTENT_OWNER_%s %s" % ["PASS" if valid else "FAIL", label])


func _compare(label: String, lines: PackedVector2Array, viewport: Rect2, common_rects: Array[Rect2], rectangles: Dictionary, members: Array, clearance: float = 0.0, canvas: Transform2D = Transform2D.IDENTITY) -> PackedVector2Array:
	var common: Dictionary = Layout.badge_index(common_rects)
	var bodies: Dictionary = Layout.aggregate_body_obstacles(rectangles, canvas, clearance)
	var actual_counters: Dictionary = {"clip_candidates": 0}
	var expected_counters: Dictionary = {"clip_candidates": 0}
	var actual: PackedVector2Array = Layout.clipped_aggregate(lines, viewport, common, bodies, members, actual_counters)
	var expected: PackedVector2Array = Reference.clipped_aggregate(lines, viewport, common, bodies, members, expected_counters)
	_check(label + "_exact_endpoint_order", actual == expected)
	_check(label + "_existing_counters", int(actual_counters.get("clip_candidates", 0)) == int(expected_counters.get("clip_candidates", 0)) and int(actual_counters.get("emitted_segments", 0)) == int(expected_counters.get("emitted_segments", 0)))
	_check(label + "_unprofiled_same", Layout.clipped_aggregate(lines, viewport, common, bodies, members) == expected)
	return actual


func _run() -> void:
	var viewport: Rect2 = Rect2(-2000, -2000, 4000, 4000)
	var common: Array[Rect2] = [Rect2(48, -4, 4, 8), Rect2(48, -4, 4, 8)]
	var repeated: PackedVector2Array = PackedVector2Array([Vector2(0, 0), Vector2(100, 0), Vector2(100, 0), Vector2(0, 0), Vector2(0, 0), Vector2(100, 0)])
	var bodies: Dictionary = {1: Rect2(20, -5, 20, 10), 2: Rect2(20, -5, 20, 10), 3: Rect2(60, -5, 20, 10)}
	for members: Array in [[], [1], [1, 2], [2, 1], [1, 2, 3], [3], [1]]:
		_compare("mixed_owners_%s" % str(members), repeated, viewport, common, bodies, members)
	var mixed: PackedVector2Array = _compare("mixed_first_owner_member", PackedVector2Array([Vector2(0, 0), Vector2(100, 0)]), viewport, [], {1: bodies[1], 2: bodies[2]}, [1])
	_check("nonmember_duplicate_still_occludes", mixed == PackedVector2Array([Vector2(0, 0), Vector2(20, 0), Vector2(40, 0), Vector2(100, 0)]))
	_compare("mixed_first_owner_nonmember", repeated, viewport, [], {2: bodies[2], 1: bodies[1]}, [1])
	_compare("empty_input", PackedVector2Array(), viewport, [], {}, [])
	_compare("zero_length_and_border_contact", PackedVector2Array([Vector2(20, 0), Vector2(20, 0), Vector2(0, -5), Vector2(100, -5)]), viewport, [], bodies, [])
	_compare("viewport_clipped", repeated, Rect2(10, -10, 80, 20), common, bodies, [1, 2])
	_compare("fully_covered", repeated, viewport, [Rect2(-1, -1, 102, 2)], bodies, [1, 2, 3])
	_compare("no_obstacles", repeated, viewport, [], {}, [])
	var corners: Array[PackedVector2Array] = [
		PackedVector2Array([Vector2(0, 448), Vector2(1280, -448)]),
		PackedVector2Array([Vector2(1280, -448), Vector2(0, 448)]),
		PackedVector2Array([Vector2(0, 448), Vector2(640, 0)]),
		PackedVector2Array([Vector2(640, 0), Vector2(0, 448)]),
		PackedVector2Array([Vector2(128, 128), Vector2(-128, -128)]),
		PackedVector2Array([Vector2(-128, -128), Vector2(128, 128)]),
		PackedVector2Array([Vector2(64, -128), Vector2(64, 128)]),
		PackedVector2Array([Vector2(128, -64), Vector2(-128, -64)])]
	for i: int in corners.size():
		_compare("grid_corner_%d" % i, corners[i], viewport, [], {1: Rect2(640, 0, 8, 8), 2: Rect2(640, 0, 8, 8), 3: Rect2(-64, -64, 64, 64)}, [1])
	var ring: Array[Rect2] = [Rect2(0, 0, 100, 10), Rect2(0, 90, 100, 10), Rect2(0, 10, 10, 80), Rect2(90, 10, 10, 80)]
	var contours: PackedVector2Array = Layout.aggregate_contours(ring, 0.0)
	_check("hole_fixture_has_inner_and_outer_edges", contours.size() == 16)
	_compare("hole_union", contours, viewport, [], {1: Rect2(8, 8, 4, 84), 2: Rect2(8, 8, 4, 84)}, [1])
	_compare("hole_clipped_by_ui_label", contours, Rect2(5, 5, 90, 90), [Rect2(45, 0, 10, 100)], bodies, [])
	_compare("fractional_transform_clearance", repeated, viewport, common, bodies, [1, 2], 2.5, Transform2D(Vector2(0.5, 0), Vector2(0, 0.75), Vector2(-0.25, 0.125)))
	# One thousand ordinary owners on a grid, with duplicate silhouettes and a mixed-owner boundary.
	var thousand: Dictionary = {}
	var members_all: Array[int] = []
	var members_some: Array[int] = []
	var normals: PackedVector2Array = PackedVector2Array()
	for i: int in 1000:
		var group: int = floori(float(i) / 5.0)
		thousand[i] = Rect2(Vector2((group % 25) * 24, floori(float(group) / 25.0) * 32), Vector2(18, 26))
		members_all.append(i)
		if i < 603:
			members_some.append(i)
	for y: int in 16:
		normals.append(Vector2(-20, y * 16 + 0.5))
		normals.append(Vector2(640, y * 16 + 0.5))
	for members: Array in [[], members_some, members_all]:
		_compare("1000_normal_%d_members" % members.size(), normals, Rect2(0, 0, 620, 260), [Rect2(300, 0, 10, 260)], thousand, members, 2.5)
	# Reclassify all1000 equal-rectangle owners for each new invocation, never each segment.
	var identical: Dictionary = {}
	for i: int in 1000:
		identical[i] = Rect2(20, -5, 20, 10)
	_compare("1000_all_member_duplicate", repeated, viewport, [], identical, members_all)
	_compare("1000_all_nonmember_duplicate", repeated, viewport, [], identical, [])
	_compare("1000_mixed_duplicate", repeated, viewport, [], identical, members_some)
	_compare("1000_members_changed_again", repeated, viewport, [], identical, members_all)
	if OS.get_cmdline_user_args().has("--expect-cache"):
		var index: Dictionary = Layout.aggregate_body_obstacles(identical, Transform2D.IDENTITY, 0.0)
		var counters: Dictionary = {"clip_candidates": 0}
		Layout.clipped_aggregate(repeated, viewport, Layout.badge_index([]), index, members_all, counters)
		var nonmember_counters: Dictionary = {"clip_candidates": 0}
		Layout.clipped_aggregate(repeated, viewport, Layout.badge_index([]), index, [], nonmember_counters)
		_check("classification_once_and_nonmember_early_exit", int(counters.get("owner_checks", 0)) == 1000 and int(counters.get("owner_cache_hits", 0)) == 2 and int(nonmember_counters.get("owner_checks", 0)) == 1 and int(nonmember_counters.get("owner_cache_hits", 0)) == 2)
	print("MUX2_EXTENT_OWNER_DONE checks=%d failures=%d" % [_checks, _failures])
	quit(0 if _failures == 0 else 1)
