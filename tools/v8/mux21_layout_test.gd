# Exact marker-placement regression oracle; bridge methods: none.
# Frozen reference: git3814828c06d1a7a211b13b54368e4e0a488cc0e3:scripts/world/colony/marker_layout.gd.
# Source SHA256:47c98348eec4f93f267669874935f2918447a0e4f773272c59201d56c0683904.
# Frozen place/_free/_reserve SHA256:1a9f17b56dd70bc790b54d9c7589391ba565772e634583e3d28de3a4ebe6e192.
# Run against an imported project using --script ABSOLUTE_PATH. Geometry-only mode supports headless.
# --cache also checks real MultiMesh getters and requires a windowed renderer; not FPS/visual acceptance.
extends SceneTree

const Layout = preload("res://scripts/world/colony/marker_layout.gd")
const Markers = preload("res://scripts/world/colony/lod_markers.gd")
var _checks: int = 0
var _failures: int = 0
var _cases: int = 0
var _reference_usec: int = 0
var _candidate_usec: int = 0

func _init() -> void:
	call_deferred("_run")

func _check(label: String, passed: bool) -> void:
	_checks += 1
	if not passed:
		_failures += 1
		push_error("MUX21_LAYOUT_FAIL " + label)

func _case(label: String, points: PackedVector2Array, ids: PackedInt64Array, extent: float, glyphs: bool, viewport: Rect2, blocked: Array[Rect2], gap: float) -> void:
	_cases += 1
	var original: PackedVector2Array = points.duplicate()
	var original_ids: PackedInt64Array = ids.duplicate()
	var original_blocked: Array[Rect2] = blocked.duplicate()
	var started: int = Time.get_ticks_usec()
	var expected: PackedVector2Array = place(points, ids, extent, glyphs, viewport, blocked, gap)
	_reference_usec += Time.get_ticks_usec() - started
	started = Time.get_ticks_usec()
	var actual: PackedVector2Array = Layout.place(points, ids, extent, glyphs, viewport, blocked, gap)
	_candidate_usec += Time.get_ticks_usec() - started
	_check(label + "/exact", actual == expected)
	_check(label + "/count", actual.size() == points.size())
	_check(label + "/inputs", points == original and ids == original_ids and blocked == original_blocked)
	_check(label + "/repeat", Layout.place(points, ids, extent, glyphs, viewport, blocked, gap) == expected)
	var reversed_points: PackedVector2Array = points.duplicate()
	var reversed_ids: PackedInt64Array = ids.duplicate()
	reversed_points.reverse()
	reversed_ids.reverse()
	var reversed: PackedVector2Array = Layout.place(reversed_points, reversed_ids, extent, glyphs, viewport, blocked, gap)
	reversed.reverse()
	_check(label + "/stable_identity", reversed == expected)
	print("MUX21_LAYOUT_CASE " + label)

func _run() -> void:
	var viewport: Rect2 = Rect2(0, 0, 800, 600)
	var none: Array[Rect2] = []
	_case("empty", PackedVector2Array(), PackedInt64Array(), 24, true, viewport, none, 4)
	var edge: PackedVector2Array = PackedVector2Array([Vector2(0, 0), Vector2(800, 600), Vector2(799.49, 0.49), Vector2(-11.49, 300), Vector2(-12.51, 300), Vector2(812.51, 300), Vector2(400, 611.49), Vector2(400, -12.51), Vector2(-100.125, -100.375), Vector2(400.5, 300.5)])
	var ids: PackedInt64Array = PackedInt64Array([80, 2, 90, 3, 70, 4, 60, 5, 50, 6])
	var walls: Array[Rect2] = [Rect2(0, 0, 180, 600), Rect2(300, 0, 500, 75), Rect2(600, 350, 200, 250), Rect2(370, 270, 80, 80)]
	for glyphs: bool in [false, true]:
		for extent: float in [14.0, 24.0, 36.0]:
			for gap: float in [0.0, 4.0, 7.5]:
				_case("edge_g%s_e%s_gap%s" % [glyphs, extent, gap], edge, ids, extent, glyphs, viewport, walls, gap)
	for count: int in [1, 10, 40, 200]:
		var crowded: PackedVector2Array = PackedVector2Array()
		var roster: PackedInt64Array = PackedInt64Array()
		for i: int in count:
			crowded.append(Vector2(400, 300))
			roster.append(10000 - i * 7)
		_case("coincident_%d" % count, crowded, roster, 24, true, viewport, walls, 4)
	for seed: int in range(8):
		var points: PackedVector2Array = PackedVector2Array()
		var roster: PackedInt64Array = PackedInt64Array()
		for i: int in 80:
			points.append(Vector2(float((i * 97 + seed * 31) % 1000) - 100.25, float((i * 53 + seed * 17) % 800) - 100.375))
			roster.append(20000 - i * 11)
		_case("distributed_%d" % seed, points, roster, 24, seed % 2 == 0, viewport, walls, 4)
	var tiny: Rect2 = Rect2(0, 0, 92, 92)
	var full: Array[Rect2] = [tiny]
	var cluster: PackedVector2Array = PackedVector2Array([Vector2(46, 46), Vector2(46, 46), Vector2(46.49, 46.49), Vector2(-100.25, -100.75)])
	var cluster_ids: PackedInt64Array = PackedInt64Array([4, 2, 3, 1])
	_case("exhaustion_full_ui", cluster, cluster_ids, 24, true, tiny, full, 4)
	_case("exhaustion_capacity", cluster, cluster_ids, 36, true, Rect2(0, 0, 70, 70), none, 7.5)
	var thousand: PackedVector2Array = PackedVector2Array()
	var thousand_ids: PackedInt64Array = PackedInt64Array()
	for i: int in 1000:
		thousand.append(Vector2(96, 54))
		thousand_ids.append(1000 - i)
	_case("exhaustion_1000_small_viewport", thousand, thousand_ids, 24, true, Rect2(0, 0, 192, 108), none, 4)
	var exhausted: PackedVector2Array = Layout.place(cluster, cluster_ids, 24, true, tiny, full, 4)
	_check("exhaustion_retains_rounded_visible_anchor", exhausted[2] == cluster[2].round())
	_check("offscreen_retains_unrounded_anchor", exhausted[3] == cluster[3])
	var shifted: PackedVector2Array = edge.duplicate()
	for i: int in shifted.size():
		shifted[i] += Vector2(50, 75)
	var shifted_walls: Array[Rect2] = []
	for rectangle: Rect2 in walls:
		shifted_walls.append(Rect2(rectangle.position + Vector2(50, 75), rectangle.size))
	_case("nonzero_viewport_origin", shifted, ids, 24, true, Rect2(50, 75, 800, 600), shifted_walls, 4)
	if OS.get_cmdline_user_args().has("--cache"):
		_cache_cases()
		await _marker_integration()
	print("MUX21_LAYOUT_DONE " + JSON.stringify({"cases": _cases, "checks": _checks, "failures": _failures, "reference_usec": _reference_usec, "candidate_usec": _candidate_usec, "display": DisplayServer.get_name(), "timing_scope": "geometry diagnostic only; not release FPS acceptance"}))
	quit(0 if _failures == 0 else 1)

func _cache_sample(markers: Node, label: String, points: PackedVector2Array, ids: PackedInt64Array, extent: float, glyphs: bool, viewport: Rect2, blocked: Array[Rect2], gap: float, should_hit: bool) -> void:
	var calls: int = int(markers.get("layout_calls"))
	var hits: int = int(markers.get("layout_cache_hits"))
	var expected: PackedVector2Array = place(points, ids, extent, glyphs, viewport, blocked, gap)
	var actual: PackedVector2Array = markers.call("_place_markers", points, ids, extent, glyphs, viewport, blocked, gap)
	_check(label + "/exact", actual == expected)
	_check(label + "/calls", int(markers.get("layout_calls")) == calls + (0 if should_hit else 1))
	_check(label + "/hits", int(markers.get("layout_cache_hits")) == hits + (1 if should_hit else 0))

func _cache_cases() -> void:
	var script: Script = load("res://scripts/world/colony/lod_markers.gd") as Script
	var markers: Node = script.new()
	if not markers.has_method("_place_markers"):
		_check("cache_api_available", false)
		markers.free()
		return
	var points: PackedVector2Array = PackedVector2Array([Vector2(300.1, 200.1), Vector2(300.1, 200.1), Vector2(-100.1, -100.1)])
	var ids: PackedInt64Array = PackedInt64Array([2, 1, 3])
	var viewport: Rect2 = Rect2(0, 0, 800, 600)
	var blocked: Array[Rect2] = [Rect2(0, 0, 100, 100)]
	_cache_sample(markers, "cache_cold", points, ids, 24, true, viewport, blocked, 4, false)
	_cache_sample(markers, "cache_repeat", points, ids, 24, true, viewport, blocked, 4, true)
	points[0] = Vector2(300.2, 200.2)
	points[2] = Vector2(-120.125, -140.875)
	_cache_sample(markers, "cache_rounded_and_offscreen_fractional", points, ids, 24, true, viewport, blocked, 4, true)
	points[0] = Vector2(301, 200)
	_cache_sample(markers, "cache_movement", points, ids, 24, true, viewport, blocked, 4, false)
	points[2] = Vector2(400, 300)
	_cache_sample(markers, "cache_enter_view", points, ids, 24, true, viewport, blocked, 4, false)
	ids[0] = 20
	_cache_sample(markers, "cache_id_mutation", points, ids, 24, true, viewport, blocked, 4, false)
	blocked[0] = Rect2(280, 180, 80, 80)
	_cache_sample(markers, "cache_ui_mutation", points, ids, 24, true, viewport, blocked, 4, false)
	_cache_sample(markers, "cache_extent", points, ids, 28, true, viewport, blocked, 4, false)
	_cache_sample(markers, "cache_glyphs", points, ids, 28, false, viewport, blocked, 4, false)
	_cache_sample(markers, "cache_gap", points, ids, 28, false, viewport, blocked, 6, false)
	viewport.size = Vector2(640, 480)
	_cache_sample(markers, "cache_viewport", points, ids, 28, false, viewport, blocked, 6, false)
	points.reverse()
	ids.reverse()
	_cache_sample(markers, "cache_row_order", points, ids, 28, false, viewport, blocked, 6, false)
	points.append(Vector2(300, 200))
	ids.append(40)
	_cache_sample(markers, "cache_population", points, ids, 28, false, viewport, blocked, 6, false)
	var other: Node = script.new()
	_cache_sample(other, "cache_second_instance_cold", points, ids, 28, false, viewport, blocked, 6, false)
	_cache_sample(markers, "cache_first_instance_preserved", points, ids, 28, false, viewport, blocked, 6, true)
	other.free()
	markers.free()

func _marker_integration() -> void:
	# Godot 4.6 dummy storage returns identity/empty values from per-instance getters.
	# Keep these real readback assertions: headless is an unsupported prerequisite, not PASS.
	_check("integration_requires_windowed_renderer", DisplayServer.get_name() != "headless")
	if DisplayServer.get_name() == "headless":
		return
	var viewport: SubViewport = SubViewport.new()
	viewport.size = Vector2i(800, 600)
	viewport.world_2d = World2D.new()
	viewport.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	root.add_child(viewport)
	var markers: Markers = Markers.new()
	viewport.add_child(markers)
	# Attach the viewport's canvas before setting its RenderingServer transform.
	await process_frame
	viewport.canvas_transform = Transform2D(Vector2(0.25, 0), Vector2(0, 0.25), Vector2.ZERO)
	await RenderingServer.frame_post_draw
	_check("integration_initial_canvas_transform", markers.get_global_transform_with_canvas() == viewport.canvas_transform)
	var points: PackedVector2Array = PackedVector2Array([Vector2(1200, 800), Vector2(1200, 800)])
	var colours: PackedColorArray = PackedColorArray([Color.WHITE, Color.WHITE])
	var rings: PackedColorArray = PackedColorArray([Color.RED, Color.RED])
	var glyph_ids: PackedInt32Array = PackedInt32Array([-1, 0])
	var ids: PackedInt64Array = PackedInt64Array([1, 2])
	markers.show_markers(points, colours, glyph_ids, 14, 0.25, true, Color.BLACK, rings, {}, ids)
	await RenderingServer.frame_post_draw
	var previous_point: Vector2 = markers.point_at(0)
	var discs: MultiMesh = (markers.get("_discs") as MultiMeshInstance2D).multimesh
	var glyphs: MultiMesh = (markers.get("_glyphs") as MultiMeshInstance2D).multimesh
	_check("integration_initial_instances", discs.instance_count == 2 and glyphs.instance_count == 1)
	_check("integration_initial_presentation", discs.get_instance_color(0) == Color.WHITE and markers.ring_of(0) == Color.RED and glyphs.get_instance_custom_data(0).r == 0.0)
	var calls: int = markers.layout_calls
	var hits: int = markers.layout_cache_hits
	# A subpixel camera pan and world movement retain rounded layout inputs, but change world-space hit positions.
	var panned: Transform2D = viewport.canvas_transform
	panned.origin = Vector2(0.125, 0.125)
	viewport.canvas_transform = panned
	await RenderingServer.frame_post_draw
	_check("integration_panned_canvas_transform", markers.get_global_transform_with_canvas() == panned)
	points[0] += Vector2(0.25, 0.25)
	colours[0] = Color(0.875, 1, 0.875, 1)
	rings[0] = Color.BLUE
	glyph_ids[0] = 3
	glyph_ids[1] = -1
	markers.show_markers(points, colours, glyph_ids, 14, 0.25, true, Color.BLACK, rings, {}, ids)
	await RenderingServer.frame_post_draw
	_check("integration_layout_cache_hit", markers.layout_calls == calls and markers.layout_cache_hits == hits + 1)
	_check("integration_current_anchors", markers.get("_anchors") == points)
	_check("integration_hit_position_changed", markers.point_at(0) != previous_point)
	var canvas: Transform2D = markers.get_global_transform_with_canvas()
	var screen: PackedVector2Array = PackedVector2Array()
	for point: Vector2 in points:
		screen.append(canvas * point)
	var blocked: Array[Rect2] = []
	var expected: PackedVector2Array = place(screen, ids, 24, true, markers.get_viewport_rect(), blocked, 4)
	for i: int in points.size():
		_check("integration_current_inverse_%d" % i, markers.point_at(i) == canvas.affine_inverse() * expected[i])
		_check("integration_disc_hit_position_%d" % i, discs.get_instance_transform_2d(i).origin == markers.point_at(i))
	_check("integration_colour_updates_on_hit", discs.get_instance_color(0) == colours[0])
	_check("integration_household_updates_on_hit", markers.ring_of(0) == Color.BLUE)
	_check("integration_glyph_identity_updates_on_hit", glyphs.instance_count == 1 and glyphs.get_instance_custom_data(0).r == 3.0)
	var glyph_offset: Vector2 = Vector2(76, -64.8)
	_check("integration_glyph_follows_current_person", glyphs.get_instance_transform_2d(0).origin.is_equal_approx(markers.point_at(0) + glyph_offset))
	var targets: PackedVector2Array = points.duplicate()
	for i: int in targets.size():
		targets[i] += Vector2(64, 0)
	markers.show_markers(points, colours, glyph_ids, 14, 0.25, true, Color.BLACK, rings, {}, ids, targets)
	await RenderingServer.frame_post_draw
	var target_position: Vector2 = markers.point_at(0)
	var old_anchor: Vector2 = points[0]
	calls = markers.layout_calls
	hits = markers.layout_cache_hits
	points[0] += Vector2(32, 0)
	markers.show_markers(points, colours, glyph_ids, 14, 0.25, true, Color.BLACK, rings, {}, ids, targets)
	await RenderingServer.frame_post_draw
	_check("integration_target_cache_ignores_interpolation", markers.layout_calls == calls and markers.layout_cache_hits == hits + 1)
	_check("integration_target_pick_position_stable", markers.point_at(0) == target_position and discs.get_instance_transform_2d(0).origin == target_position)
	var anchors: PackedVector2Array = markers.get("_anchors")
	_check("integration_target_leader_tracks_interpolation", anchors[0] == points[0] and anchors[0] != old_anchor and anchors[0] != target_position)
	screen.clear()
	for target: Vector2 in targets:
		screen.append(canvas * target)
	expected = place(screen, ids, 24, true, markers.get_viewport_rect(), blocked, 4)
	_check("integration_target_placement_matches_oracle", markers.point_at(0) == canvas.affine_inverse() * expected[0])
	targets[0] += Vector2(64, 0)
	markers.show_markers(points, colours, glyph_ids, 14, 0.25, true, Color.BLACK, rings, {}, ids, targets)
	await RenderingServer.frame_post_draw
	_check("integration_target_change_invalidates", markers.layout_calls == calls + 1)
	screen[0] = canvas * targets[0]
	expected = place(screen, ids, 24, true, markers.get_viewport_rect(), blocked, 4)
	_check("integration_target_change_updates_picking", markers.point_at(0) == canvas.affine_inverse() * expected[0] and discs.get_instance_transform_2d(0).origin == markers.point_at(0))
	viewport.free()

# The following three functions are verbatim from the pinned reference, with no optimization.
static func place(points: PackedVector2Array, ids: PackedInt64Array, extent: float, glyphs: bool, viewport: Rect2, blocked: Array[Rect2], gap: float) -> PackedVector2Array:
	var result: PackedVector2Array = points.duplicate()
	var order: Array[int] = []
	for i: int in points.size():
		order.append(i)
	order.sort_custom(func(a: int, b: int) -> bool: return ids[a] < ids[b])
	var occupied: Dictionary = {}
	var radius: float = extent * 0.5 + gap
	var glyph: float = 14.0 if glyphs else 0.0
	var size: Vector2 = Vector2(radius * 2.0 + glyph, radius * 2.0 + glyph)
	var cell_size: float = maxf(size.x, size.y)
	for index: int in order:
		var anchor: Vector2 = points[index].round()
		if not Rect2(anchor - Vector2.ONE * extent * 0.5, Vector2.ONE * extent).intersects(viewport):
			continue
		var chosen: Vector2 = anchor
		var found: bool = false
		var limit: int = ceili(maxf(viewport.size.x, viewport.size.y) / cell_size)
		for ring: int in range(limit + 1):
			for y: int in range(-ring, ring + 1):
				for x: int in range(-ring, ring + 1):
					if maxi(absi(x), absi(y)) != ring:
						continue
					var candidate: Vector2 = anchor + Vector2(x, y) * cell_size
					var rectangle: Rect2 = Rect2(candidate - Vector2(radius, radius + glyph), size)
					if viewport.encloses(rectangle) and _free(rectangle, blocked, occupied, cell_size):
						chosen = candidate
						_reserve(rectangle, occupied, cell_size)
						found = true
						break
				if found:
					break
			if found:
				break
		# Exhaustion keeps the original visible marker; the visibility oracle still records any overlap.
		result[index] = chosen
	return result


static func _free(rectangle: Rect2, blocked: Array[Rect2], occupied: Dictionary, cell_size: float) -> bool:
	for obstacle: Rect2 in blocked:
		if rectangle.intersects(obstacle):
			return false
	var first: Vector2i = Vector2i((rectangle.position / cell_size).floor())
	var last: Vector2i = Vector2i((rectangle.end / cell_size).floor())
	for y: int in range(first.y, last.y + 1):
		for x: int in range(first.x, last.x + 1):
			for other: Rect2 in occupied.get(Vector2i(x, y), []):
				if rectangle.intersects(other):
					return false
	return true


static func _reserve(rectangle: Rect2, occupied: Dictionary, cell_size: float) -> void:
	var first: Vector2i = Vector2i((rectangle.position / cell_size).floor())
	var last: Vector2i = Vector2i((rectangle.end / cell_size).floor())
	for y: int in range(first.y, last.y + 1):
		for x: int in range(first.x, last.x + 1):
			var cell: Vector2i = Vector2i(x, y)
			if not occupied.has(cell):
				occupied[cell] = []
			occupied[cell].append(rectangle)
