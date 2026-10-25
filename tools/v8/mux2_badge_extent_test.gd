# M-UX2 aggregate extents: independent cell-occupancy oracle, clipping and live parameter refresh.
# Bridge methods: none. No raster rounding; fractional source coordinates remain exact.
extends SceneTree

const Layout = preload("res://scripts/world/colony/marker_layout.gd")
const Agents = preload("res://scripts/world/colony/agent_layer.gd")
const Art = preload("res://scripts/world/art/art_data.gd")
var _checks: int = 0
var _failures: int = 0


func _init() -> void:
	call_deferred("_run")


func _check(label: String, valid: bool) -> void:
	_checks += 1
	_failures += 0 if valid else 1
	print("MUX2_BADGE_EXTENT_%s %s" % ["PASS" if valid else "FAIL", label])


# Comparison representation only: coalesce adjacent collinear cell edges.
func _canonical(points: PackedVector2Array) -> Array:
	var lines: Dictionary = {}
	for i: int in range(0, points.size(), 2):
		var a: Vector2 = points[i]
		var b: Vector2 = points[i + 1]
		var horizontal: bool = a.y == b.y
		var key: Vector2 = Vector2(0, a.y) if horizontal else Vector2(1, a.x)
		var interval: Vector2 = Vector2(minf(a.x, b.x), maxf(a.x, b.x)) if horizontal else Vector2(minf(a.y, b.y), maxf(a.y, b.y))
		if not lines.has(key):
			lines[key] = []
		lines[key].append(interval)
	var keys: Array = lines.keys()
	keys.sort_custom(func(a: Vector2, b: Vector2) -> bool: return a.x < b.x or (a.x == b.x and a.y < b.y))
	var result: Array = []
	for key: Vector2 in keys:
		var intervals: Array = lines[key]
		intervals.sort_custom(func(a: Vector2, b: Vector2) -> bool: return a.x < b.x or (a.x == b.x and a.y < b.y))
		var merged: Array[Vector2] = []
		for interval: Vector2 in intervals:
			if merged.is_empty() or interval.x > merged[-1].y:
				merged.append(interval)
			else:
				merged[-1] = Vector2(merged[-1].x, maxf(merged[-1].y, interval.y))
		result.append([key, merged])
	return result


# Independent oracle: coordinate-compressed occupied cells, then occupied/empty adjacencies.
# It does not use the production spatial index, edge subtraction or leader helpers.
func _reference(figures: Array, padding: float) -> PackedVector2Array:
	var xs: Array[float] = []
	var ys: Array[float] = []
	var rectangles: Array[Rect2] = []
	for figure: Rect2 in figures:
		var rectangle: Rect2 = figure.grow(padding)
		if rectangle.size.x <= 0.0 or rectangle.size.y <= 0.0:
			continue
		rectangles.append(rectangle)
		for x: float in [rectangle.position.x, rectangle.end.x]:
			if not xs.has(x):
				xs.append(x)
		for y: float in [rectangle.position.y, rectangle.end.y]:
			if not ys.has(y):
				ys.append(y)
	xs.sort()
	ys.sort()
	var occupied: Dictionary = {}
	for y: int in range(ys.size() - 1):
		for x: int in range(xs.size() - 1):
			var center: Vector2 = Vector2((xs[x] + xs[x + 1]) * 0.5, (ys[y] + ys[y + 1]) * 0.5)
			for rectangle: Rect2 in rectangles:
				if rectangle.has_point(center):
					occupied[Vector2i(x, y)] = true
	var result: PackedVector2Array = PackedVector2Array()
	for cell: Vector2i in occupied:
		var a: Vector2 = Vector2(xs[cell.x], ys[cell.y])
		var b: Vector2 = Vector2(xs[cell.x + 1], ys[cell.y + 1])
		for direction: Vector2i in [Vector2i.UP, Vector2i.DOWN, Vector2i.LEFT, Vector2i.RIGHT]:
			if occupied.has(cell + direction):
				continue
			if direction == Vector2i.UP or direction == Vector2i.DOWN:
				var y: float = a.y if direction == Vector2i.UP else b.y
				result.append_array(PackedVector2Array([Vector2(a.x, y), Vector2(b.x, y)]))
			else:
				var x: float = a.x if direction == Vector2i.LEFT else b.x
				result.append_array(PackedVector2Array([Vector2(x, a.y), Vector2(x, b.y)]))
	return result


func _compare(label: String, figures: Array, padding: float = 0.0) -> void:
	var actual: PackedVector2Array = Layout.aggregate_contours(figures, padding)
	_check(label + "_independent_cell_oracle", _canonical(actual) == _canonical(_reference(figures, padding)))
	var reversed: Array = figures.duplicate()
	reversed.reverse()
	_check(label + "_order_independent", actual == Layout.aggregate_contours(reversed, padding))


func _clipping() -> void:
	var line: PackedVector2Array = PackedVector2Array([Vector2(0, 5), Vector2(20, 5)])
	var viewport: Rect2 = Rect2(2, 0, 16, 10)
	var empty_bodies: Dictionary = Layout.aggregate_body_obstacles({}, Transform2D.IDENTITY, 0.0)
	_check("viewport_fraction_preserved", Layout.clipped_aggregate(line, viewport, {}, empty_bodies, []) == PackedVector2Array([Vector2(2, 5), Vector2(18, 5)]))
	var blockers: Array[Rect2] = [Rect2(6, 0, 4, 10)]
	var expected: PackedVector2Array = PackedVector2Array([Vector2(2, 5), Vector2(6, 5), Vector2(10, 5), Vector2(18, 5)])
	_check("ui_and_label_gap", Layout.clipped_aggregate(line, viewport, Layout.badge_index(blockers), empty_bodies, []) == expected)
	var bodies: Dictionary = Layout.aggregate_body_obstacles({1: blockers[0]}, Transform2D.IDENTITY, 0.0)
	_check("member_body_does_not_cut_own_scope", Layout.clipped_aggregate(line, viewport, {}, bodies, [1]) == PackedVector2Array([Vector2(2, 5), Vector2(18, 5)]))
	_check("nonmember_body_cuts_scope", Layout.clipped_aggregate(line, viewport, {}, bodies, []) == expected)
	bodies = Layout.aggregate_body_obstacles({1: blockers[0], 2: blockers[0]}, Transform2D.IDENTITY, 0.0)
	_check("equal_rect_nonmember_owner_retained", Layout.clipped_aggregate(line, viewport, {}, bodies, [1]) == expected)
	_check("covered_scope_empty", Layout.clipped_aggregate(line, viewport, Layout.badge_index([viewport]), empty_bodies, []).is_empty())
	var scope: PackedVector2Array = Layout.aggregate_contours([Rect2(0, 0, 10, 10), Rect2(30, 0, 10, 10)], 0.0)
	var leader: PackedVector2Array = Layout.aggregate_leader(scope, Rect2(17, 2, 6, 4))
	_check("leader_uses_actual_edge_not_gap_envelope", leader == PackedVector2Array([Vector2(10, 4), Vector2(17, 4)]))
	leader = Layout.aggregate_leader(PackedVector2Array([Vector2(80, -1), Vector2(80, 0), Vector2(10, 40), Vector2(10, 41)]), Rect2(0, 2, 100, 10))
	_check("leader_nearest_label_boundary_not_center", leader == PackedVector2Array([Vector2(80, 0), Vector2(80, 2)]))
	_check("empty_scope_has_no_fictitious_leader", Layout.aggregate_leader(PackedVector2Array(), Rect2(0, 0, 10, 10)).is_empty())


func _person(bits: int, at: Vector2) -> Agents.Person:
	var person: Agents.Person = Agents.Person.new()
	person.bits = bits
	person.agent_id = bits
	person.position = at
	person.body = Sprite2D.new()
	person.body.centered = false
	person.body.offset = Vector2(-16, -64)
	var image: Image = Image.create(32, 64, false, Image.FORMAT_RGBA8)
	image.fill(Color.WHITE)
	person.body.texture = ImageTexture.create_from_image(image)
	person.add_child(person.body)
	return person


func _live_params() -> void:
	var agents: Node2D = Agents.new()
	root.add_child(agents)
	agents.set("_zoom", 0.5)
	var persons: Dictionary = {}
	for bits: int in [100, 200, 300, 400]:
		var person: Agents.Person = _person(bits, Vector2(400, 400) if bits <= 200 else Vector2(432, 464))
		agents.add_child(person)
		persons[bits] = person
	agents.set("_persons", persons)
	agents.call("_show_overlapping_figures")
	var badges: Array = agents.get("_coincident_badges")
	_check("live_aggregate_exists", badges.size() == 1 and bool(badges[0]["aggregate"]))
	if badges.size() != 1:
		agents.free()
		return
	_check("localized_group_and_people_counts", badges[0]["text"] == root.get_node("Locale").call("trf", "UI_COINCIDENT_AGGREGATE_FMT", {"groups": 2, "people": 4}) and badges[0]["member_count"] == 4)
	var old_scope: PackedVector2Array = badges[0]["screen_scope"]
	var old_path: String = Art.params_path
	var original: Dictionary = Art.params().duplicate(true)
	var changed: Dictionary = original.duplicate(true)
	changed["labels"]["aggregate"]["padding_px"] = 7.25
	changed["labels"]["aggregate"]["dark_width_px"] = 5.0
	var temp_path: String = "user://mux2_extent_params_test.json"
	var file: FileAccess = FileAccess.open(temp_path, FileAccess.WRITE)
	if file == null:
		_check("write_reload_fixture", false)
		agents.free()
		return
	file.store_string(JSON.stringify(changed))
	file.close()
	Art.params_path = temp_path
	var version: int = Art.params_version()
	_check("f12_reload_increments_version", Art.reload_params() and Art.params_version() == version + 1)
	agents.call("_show_overlapping_figures")
	badges = agents.get("_coincident_badges")
	_check("f12_padding_recomputes_fractional_scope", badges[0]["screen_scope"] != old_scope and (agents.call("_aggregate_style") as Dictionary)["padding"] == 7.25)
	var drawer: Node2D = agents.get("_badge_leaders")
	_check("f12_width_reaches_existing_draw_node", drawer.get("extent_style")["dark_width"] == 5.0)
	agents.set("_selected_bits", 100)
	agents.call("_show_overlapping_figures")
	_check("selected_aggregate_uses_active_contour", not (drawer.get("active_segments") as PackedVector2Array).is_empty() and (drawer.get("aggregate_segments") as PackedVector2Array).is_empty())
	agents.set("_zoom", 1.25)
	agents.call("_show_overlapping_figures")
	_check("camera_stroke_width_refresh", is_equal_approx(float(drawer.get("scale_width")), 0.8))
	_check("presentation_keeps_true_feet", persons[100].position == Vector2(400, 400) and persons[300].position == Vector2(432, 464))
	agents.set("_zoom", 0.5)
	# Invert stroke widths and put a real source edge at screen x=2.
	# The 12px light stroke requires a 6px viewport inset, even with a 3px dark stroke.
	changed["labels"]["aggregate"]["padding_px"] = 2.0
	changed["labels"]["aggregate"]["dark_width_px"] = 3.0
	changed["labels"]["aggregate"]["light_width_px"] = 12.0
	file = FileAccess.open(temp_path, FileAccess.WRITE)
	if file != null:
		file.store_string(JSON.stringify(changed))
		file.close()
	_check("inverted_width_parameters_reload", file != null and Art.reload_params())
	for bits: int in persons:
		persons[bits].position = Vector2(20, 400) if bits <= 200 else Vector2(52, 464)
	agents.call("_show_overlapping_figures")
	badges = agents.get("_coincident_badges")
	var edge_fixture: bool = badges.size() == 1 and bool(badges[0]["aggregate"])
	var has_edge_at_two: bool = false
	if edge_fixture:
		for point: Vector2 in badges[0]["screen_scope"]:
			has_edge_at_two = has_edge_at_two or is_equal_approx(point.x, 2.0)
	_check("inverted_width_fixture_has_near_viewport_source", edge_fixture and has_edge_at_two)
	var emitted: PackedVector2Array = drawer.get("active_segments")
	var inside_stroke_inset: bool = not emitted.is_empty()
	var safe: Rect2 = agents.get_viewport_rect().grow(-6.0)
	for point: Vector2 in emitted:
		var screen: Vector2 = agents.get_global_transform_with_canvas() * point
		inside_stroke_inset = inside_stroke_inset and screen.x >= safe.position.x - 0.001 and screen.y >= safe.position.y - 0.001 and screen.x <= safe.end.x + 0.001 and screen.y <= safe.end.y + 0.001
	_check("inverted_light_stroke_clips_to_full_visible_width", inside_stroke_inset)
	for bits: int in persons:
		persons[bits].position = Vector2(400, 400) if bits <= 200 else Vector2(432, 464)
	Art.params_path = old_path
	_check("restore_parameters", Art.reload_params())
	DirAccess.remove_absolute(ProjectSettings.globalize_path(temp_path))
	agents.call("_show_overlapping_figures")
	_check("restored_padding_recomputes_scope", agents.get("_coincident_badges")[0]["screen_scope"] == old_scope)
	agents.call("_on_world_reset")
	_check("reset_clears_contour_arrays", (drawer.get("active_segments") as PackedVector2Array).is_empty() and (drawer.get("aggregate_segments") as PackedVector2Array).is_empty())
	agents.free()


func _run() -> void:
	_compare("empty", [])
	_compare("single", [Rect2(0, 0, 10, 10)])
	_compare("disjoint", [Rect2(0, 0, 10, 10), Rect2(20, 0, 10, 10)])
	_compare("touching", [Rect2(0, 0, 10, 10), Rect2(10, 0, 10, 10)])
	_compare("corner", [Rect2(0, 0, 10, 10), Rect2(10, 10, 10, 10)])
	_compare("overlap", [Rect2(0, 0, 10, 10), Rect2(5, 5, 10, 10)])
	_compare("contained", [Rect2(0, 0, 10, 10), Rect2(2, 2, 3, 3)])
	_compare("duplicate", [Rect2(0, 0, 10, 10), Rect2(0, 0, 10, 10)])
	_compare("ring_hole", [Rect2(0, 0, 30, 5), Rect2(0, 25, 30, 5), Rect2(0, 5, 5, 20), Rect2(25, 5, 5, 20)])
	_compare("fractional_padding", [Rect2(-8.25, -5.5, 7.5, 9.25), Rect2(4.5, -1.25, 8.5, 10.25)], 1.25)
	for sample: int in 16:
		var figures: Array = []
		for i: int in 12:
			figures.append(Rect2(float((i * 17 + sample * 3) % 61 - 30) * 0.25, float((i * 13 + sample * 5) % 47 - 20) * 0.25, 4.25 + i % 3, 6.5))
		_compare("generated_%d" % sample, figures, 0.25)
	_clipping()
	_live_params()
	print("MUX2_BADGE_EXTENT_RESULT checks=%d failures=%d" % [_checks, _failures])
	quit(0 if _failures == 0 else 1)
