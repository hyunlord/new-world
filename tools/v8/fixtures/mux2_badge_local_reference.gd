# Frozen pre-local-optimization badge update and measurement bodies.
# Bridge methods: none. Eager lookups, repeated measurement and duplicate obstacles remain frozen.
# The separately tested aggregate extent/text policy is adapted identically after the frozen preparation.
extends "res://scripts/world/colony/agent_layer.gd"


func _update_coincident_badges(rectangles: Dictionary) -> void:
	var phase: int = Time.get_ticks_usec() if _render_profiling else 0
	var groups: Dictionary = {}
	for person: Person in _persons.values():
		var cell: Vector2i = Vector2i((person.position * _zoom).round())
		if not groups.has(cell):
			groups[cell] = []
		groups[cell].append(person.bits)
	var has_group: bool = false
	for group: Array in groups.values():
		has_group = has_group or group.size() > 1
	phase = _profile_span("badge_grouping_usec", phase)
	if not has_group:
		if not _coincident_badges.is_empty():
			_clear_coincident_badges()
			queue_redraw()
		return
	var badges: Array[Dictionary] = []
	var canvas: Transform2D = get_global_transform_with_canvas()
	var inverse: Transform2D = canvas.affine_inverse()
	var blocked: Array[Rect2] = []
	for rectangle: Rect2 in rectangles.values():
		blocked.append(canvas * rectangle)
	var ui_rectangles: Array[Rect2] = []
	var ui: Node = get_node_or_null("/root/Main/UI")
	if ui != null:
		MarkerLayout.ui_rects(ui, ui_rectangles)
	blocked.append_array(ui_rectangles)
	var badge_index: Dictionary = MarkerLayout.badge_index(blocked)
	phase = _profile_span("badge_obstacles_usec", phase)
	var descriptors: Array[Dictionary] = []
	var agent_ids: Dictionary = {}
	var maximum_size: Vector2 = Vector2.ONE
	for group: Array in groups.values():
		if group.size() < 2:
			continue
		var members: Array[int] = []
		for bits: int in group:
			members.append(bits)
			agent_ids[bits] = (_persons[bits] as Person).agent_id
		members.sort_custom(func(a: int, b: int) -> bool: return (_persons[a] as Person).agent_id < (_persons[b] as Person).agent_id)
		var bounds: Rect2 = rectangles.get(members[0], drawn_rect(members[0]))
		for bits: int in members:
			bounds = bounds.merge(rectangles.get(bits, drawn_rect(bits)))
		var text: String = str(members.size())
		descriptors.append({"key": int(agent_ids[members[0]]), "members": members, "text": text, "size": _reference_badge_screen_size(text, canvas), "figure": canvas * bounds})
	descriptors.sort_custom(func(a: Dictionary, b: Dictionary) -> bool: return int(a["key"]) < int(b["key"]))
	_coincident_groups = descriptors.duplicate()
	phase = _profile_span("badge_prepare_usec", phase)
	descriptors = MarkerLayout.aggregate_badges(descriptors, agent_ids)
	var extent_style: Dictionary = _aggregate_style()
	var extent_counters: Dictionary = {"source_rectangles": 0} if _render_profiling else {}
	var extent_started: int = Time.get_ticks_usec() if _render_profiling else 0
	for descriptor: Dictionary in descriptors:
		descriptor["text"] = _aggregate_text(descriptor)
		descriptor["scope"] = MarkerLayout.aggregate_contours(descriptor["source_figures"], float(extent_style["padding"]), extent_counters) if (descriptor["source_groups"] as Array).size() > 1 else PackedVector2Array()
		descriptor["size"] = _reference_badge_screen_size(str(descriptor["text"]), canvas)
		maximum_size = maximum_size.max(descriptor["size"])
	if _render_profiling:
		extent_counters["build_usec"] = Time.get_ticks_usec() - extent_started
		extent_counters["cache_reuses"] = 0
		mux2_render_profile["badge_source_groups"] = _coincident_groups.size()
		mux2_render_profile["badge_aggregate_groups"] = descriptors.size()
		mux2_render_profile["badge_source_members"] = agent_ids.size()
		var aggregate_members: int = 0
		var membership: Array[Dictionary] = []
		for descriptor: Dictionary in descriptors:
			aggregate_members += (descriptor["members"] as Array).size()
			membership.append({"members": descriptor["members"], "source_groups": descriptor["source_groups"]})
		mux2_render_profile["badge_aggregate_members"] = aggregate_members
		mux2_render_profile["badge_memberships"] = membership
	phase = _profile_span("badge_aggregate_usec", phase)
	var grid: Dictionary = MarkerLayout.badge_grid(maximum_size, get_viewport_rect(), blocked)
	phase = _profile_span("badge_grid_usec", phase)
	for descriptor: Dictionary in descriptors:
		var figure: Rect2 = descriptor["figure"]
		var exhausted_before: int = int(grid["exhausted"])
		var placed: Rect2 = MarkerLayout.badge_packed(figure, descriptor["size"], get_viewport_rect(), badge_index, grid)
		MarkerLayout.badge_index_add(badge_index, placed)
		var aggregate: bool = (descriptor["source_groups"] as Array).size() > 1
		var leader: PackedVector2Array = MarkerLayout.aggregate_leader(descriptor["scope"], placed) if aggregate else MarkerLayout.badge_leader(figure, placed)
		if leader.is_empty():
			leader = PackedVector2Array([placed.get_center(), placed.get_center()])
			if _render_profiling:
				mux2_render_profile["empty_aggregate_scopes"] = int(mux2_render_profile.get("empty_aggregate_scopes", 0)) + 1
		badges.append({"members": descriptor["members"], "member_count": (descriptor["members"] as Array).size(), "aggregate": aggregate, "screen_scope": descriptor["scope"], "source_groups": descriptor["source_groups"], "anchor": inverse * Vector2(placed.get_center().x, placed.end.y), "text": descriptor["text"], "rect": inverse * placed, "leader_from": inverse * leader[0], "leader_to": inverse * leader[1], "screen_leader": leader, "exhausted": int(grid["exhausted"]) != exhausted_before})
	phase = _profile_span("badge_layout_usec", phase)
	var segments: PackedVector2Array = PackedVector2Array()
	var aggregate_segments: PackedVector2Array = PackedVector2Array()
	var active_segments: PackedVector2Array = PackedVector2Array()
	var half_stroke: float = float(extent_style["dark_width"]) * 0.5
	var has_aggregate: bool = false
	for badge: Dictionary in badges:
		has_aggregate = has_aggregate or bool(badge["aggregate"])
	var common_rectangles: Array[Rect2] = []
	for rectangle: Rect2 in ui_rectangles:
		common_rectangles.append(rectangle.grow(float(extent_style["ui_clearance"]) + half_stroke))
	for badge: Dictionary in badges:
		common_rectangles.append((canvas * (badge["rect"] as Rect2)).grow(float(extent_style["ui_clearance"]) + half_stroke))
	var common_index: Dictionary = MarkerLayout.badge_index(common_rectangles)
	var body_obstacles: Dictionary = MarkerLayout.aggregate_body_obstacles(rectangles, canvas, float(extent_style["body_clearance"]) + half_stroke) if has_aggregate else {}
	var mouse: Vector2 = get_local_mouse_position()
	var mouse_ui: bool = false
	for rectangle: Rect2 in ui_rectangles:
		mouse_ui = mouse_ui or rectangle.has_point(canvas * mouse)
	for badge: Dictionary in badges:
		var leader: PackedVector2Array = badge["screen_leader"]
		var clipped: PackedVector2Array = MarkerLayout.clipped_leader(leader[0], leader[1], badge_index)
		if bool(badge["aggregate"]):
			clipped = MarkerLayout.clipped_aggregate(clipped, get_viewport_rect().grow(-half_stroke), common_index, body_obstacles, badge["members"], extent_counters)
			clipped.append_array(MarkerLayout.clipped_aggregate(badge["screen_scope"], get_viewport_rect().grow(-half_stroke), common_index, body_obstacles, badge["members"], extent_counters))
			badge["active"] = (badge["members"] as Array).has(_selected_bits) or (not mouse_ui and (badge["rect"] as Rect2).has_point(mouse))
			for point: Vector2 in clipped:
				if bool(badge["active"]):
					active_segments.append(inverse * point)
				else:
					aggregate_segments.append(inverse * point)
		else:
			for point: Vector2 in clipped:
				segments.append(inverse * point)
	if _render_profiling:
		mux2_render_profile["aggregate_extent"] = extent_counters
	badge_layout_exhausted = int(grid["exhausted"])
	if _render_profiling:
		mux2_render_profile["badge_layout_exhausted"] = badge_layout_exhausted
	if _badge_leaders != null:
		var width: float = 1.0 / maxf(_zoom, 0.01)
		if _badge_leaders.segments != segments or _badge_leaders.light != _half_rim or _badge_leaders.scale_width != width or _badge_leaders.aggregate_segments != aggregate_segments or _badge_leaders.active_segments != active_segments or _badge_leaders.extent_style != extent_style:
			_badge_leaders.segments = segments
			_badge_leaders.aggregate_segments = aggregate_segments
			_badge_leaders.active_segments = active_segments
			_badge_leaders.extent_style = extent_style
			_badge_leaders.light = _half_rim
			_badge_leaders.scale_width = width
			_badge_leaders.queue_redraw()
			queue_redraw()
	if badges != _coincident_badges:
		_coincident_badges = badges
		queue_redraw()
	_profile_span("badge_leader_clip_usec", phase)


func _reference_badge_screen_size(text: String, canvas: Transform2D) -> Vector2:
	var px: int = int(ArtData.param("labels.font_px", 24))
	var pad: float = floorf(float(px) / 4.0)
	var size: Vector2 = _font.get_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, -1, px) if _font != null else Vector2(px, px)
	var inv: float = 1.0 / maxf(_zoom, 0.01) if bool(ArtData.param("labels.follow_camera", true)) else 1.0
	return (canvas * Rect2(Vector2.ZERO, Vector2(size.x + pad * 2.0, px + pad) * inv)).size
