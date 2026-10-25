# Frozen pre-cache person update and frame binding.
# Bridge methods: original update reads SimReads getters; tests supply synthetic nodes.
extends "res://scripts/world/colony/agent_layer.gd"


func _set_frame(person: Person, pose: String, bob: int) -> void:
	var facing: String = "E" if person.facing == "E" else "S"
	if person.pose != "%s/%s" % [facing, pose]:
		person.pose = "%s/%s" % [facing, pose]
		var tex: Texture2D = ArtData.texture_at(ArtData.frame_path(person.frames, facing, pose))
		person.body.texture = tex
		var mask: Texture2D = _mask(person.frames, facing, pose)
		person.mat.set_shader_parameter("mask_tex", mask)
		person.mat.set_shader_parameter("has_mask", mask != null)
		var a: Vector2i = ArtData.anchor("%s/%s/%s" % [person.frames, facing, pose])
		person.body.set_meta("anchor", a)
	var anchor: Vector2i = person.body.get_meta("anchor", Vector2i(40, 104))
	person.body.flip_h = person.west and facing == "E"
	var w: int = person.body.texture.get_width() if person.body.texture != null else 80
	var ax: int = (w - anchor.x) if person.body.flip_h else anchor.x
	person.body.offset = Vector2(-ax, -anchor.y - bob)


func _update(delta: float) -> void:
	var phase: int = Time.get_ticks_usec() if _render_profiling else 0
	if _params_version != ArtData.params_version():
		_params_version = ArtData.params_version()
		_make_shadow()
		_selection_rgba = _rgba("people.selection_rgba", Color(0.94, 0.89, 0.67))
		_drafted_rgba = _rgba("people.drafted_rgba", Color(0.77, 0.2, 0.16))
		_outline_px = float(ArtData.param("people.outline_px", 1))
		_drafted_px = float(ArtData.param("people.drafted_outline_px", 2))
		_halo_rgba = _rgba("people.night_halo_rgba", Color(0.91, 0.89, 0.81))
		_halo_alpha = clampf(float(ArtData.param("people.night_halo_alpha", 0.0)), 0.0, 1.0)
		_half_rim = _rgba("people.half_rim_rgba", Color(0.95, 0.95, 0.88, 1))
		_half_rim_px = clampf(float(ArtData.param("people.half_rim_px", 1)), 0.0, 2.0)
		_half_overlap_alpha = clampf(float(ArtData.param("people.half_overlap_alpha", 0.8)), 0.5, 1.0)
		for person: Person in _persons.values():
			person.shadow.texture = _shadow_tex
			person.outline = Color(-1, -1, -1, -1)
			person.mat.set_shader_parameter("outline_px", _outline_px)
	_frame += 1
	_observe_settlements()
	var speed: float = _speed()
	_clock += delta * speed
	var snap: Dictionary = SimReads.read(_world_sim, "get_agent_snapshot")
	var bits: PackedInt64Array = snap.get("ids", PackedInt64Array())
	var ids: PackedInt64Array = snap.get("agent_ids", PackedInt64Array())
	var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
	var stages: PackedByteArray = snap.get("life_stages", PackedByteArray())
	var carries: PackedInt32Array = snap.get("carry_counts", PackedInt32Array())
	var states: PackedByteArray = snap.get("states", PackedByteArray())
	var targets: PackedByteArray = snap.get("target_kinds", PackedByteArray())
	var drafted: PackedByteArray = snap.get("drafted", PackedByteArray())
	var sick: PackedByteArray = snap.get("sick", PackedByteArray())
	var bands: PackedByteArray = snap.get("mood_bands", PackedByteArray())
	var fleeing: PackedByteArray = snap.get("fleeing", PackedByteArray())
	var looks: PackedInt32Array = snap.get("look_ids", PackedInt32Array())
	var variants: PackedByteArray = snap.get("variants", PackedByteArray())
	var n: int = mini(mini(bits.size(), ids.size()), mini(xs.size(), ys.size()))
	var duration: float = _step_seconds(speed)
	var walk_hz: float = float(ArtData.param("people.walk_hz", 4.0))
	var work_hz: float = float(ArtData.param("people.work_hz", 2.0))
	var bob_px: int = int(ArtData.param("people.bob_px", 1))
	var inspector: Node = get_node_or_null("/root/Main/UI/AgentInspectorPanel")
	_selected_bits = int(inspector.call("get_selected_entity_bits")) if inspector != null and (inspector as Control).visible else -1
	_zoom = _camera_zoom()
	var lod: bool = _zoom <= float(ArtData.param("lod.marker_zoom_max", 0.25)) + 0.001
	if lod:
		_update_households(snap)
	phase = _profile_span("params_snapshot_usec", phase)
	var any_sleep: bool = false
	for i: int in n:
		var key: int = bits[i]
		var tile: Vector2i = Vector2i(xs[i], ys[i])
		var person: Person = _persons.get(key)
		if person == null:
			person = _make_person(key, ids[i], tile)
			_persons[key] = person
		elif tile != person.tile:
			person.from_pos = person.position
			person.to_pos = View.base_point(tile)
			if tile.x != person.tile.x:
				person.facing = "E"
				person.west = tile.x < person.tile.x
			else:
				person.facing = "S"
				person.west = false
			person.tile = tile
			person.elapsed = 0.0
			person.duration = duration
		person.seen = _frame
		var stage: int = int(stages[i]) if i < stages.size() else 0
		var look: int = looks[i] if i < looks.size() else posmod(ids[i], maxi(1, ArtData.look_count()))
		var variant: int = int(variants[i]) if i < variants.size() else posmod(ids[i], 6)
		_set_appearance(person, look, variant, stage == 2)
		var asleep: bool = i < states.size() and i < targets.size() and states[i] == STATE_CONSUMING and targets[i] == TARGET_SLEEP
		var moving: bool = person.elapsed < person.duration
		var carrying: bool = not asleep and i < carries.size() and carries[i] > 0
		var working: bool = not moving and i < states.size() and i < targets.size() and states[i] == STATE_CONSUMING and targets[i] == TARGET_JOB
		var phase_walk: bool = fmod(_clock * walk_hz, 1.0) >= 0.5
		var pose: String
		var bob: int = 0
		if asleep:
			pose = "sleep"
			person.activity = "sleep"
			any_sleep = true
		elif carrying:
			pose = "carry"
			bob = bob_px if moving and phase_walk else 0
			person.activity = "carry"
		elif moving:
			pose = "walk_a" if phase_walk else "idle"
			bob = bob_px if phase_walk else 0
			person.activity = "walk"
		elif working:
			pose = "work_a" if fmod(_clock * work_hz, 1.0) >= 0.5 else "idle"
			person.activity = "work"
		else:
			pose = "idle"
			person.activity = "idle"
		person.drafted = i < drafted.size() and drafted[i] != 0
		person.selected = key == _selected_bits
		var sick_now: bool = i < sick.size() and sick[i] != 0
		var flee_now: bool = not asleep and i < fleeing.size() and fleeing[i] != 0
		var marks_changed: bool = sick_now != person.sick or flee_now != person.fleeing or person.marks.visible == lod
		person.sick = sick_now
		person.fleeing = flee_now
		if person.fleeing:
			person.activity = "flee"
		person.modulate = StoryText.band_tint(int(bands[i]) if i < bands.size() else 0) if _mood_shown else Color.WHITE
		if person.visible == lod:
			person.visible = not lod
			person.body.visible = not lod
			person.marks.visible = not lod
		person.shadow.visible = not asleep and not lod
		if marks_changed:
			person.marks.queue_redraw()
		if lod:
			# Markers only (drawn by this node): no frame, mask or shader work for a hidden body.
			continue
		_set_frame(person, pose, bob)
		var readable: bool = _zoom < 1.0 and not lod
		if person.readable != readable:
			person.readable = readable
			person.mat.shader = _half_shader if readable else VARIANT_SHADER
			person.body.z_as_relative = not readable
			person.body.z_index = 4094 if readable else 0
			person.body.self_modulate = Color.WHITE
			person.outline = Color(-1, -1, -1, -1)
		var outline: Color = _no_outline
		if person.selected:
			outline = _selection_rgba
		elif person.drafted:
			outline = _drafted_rgba
		elif readable:
			outline = _half_rim
		elif _halo_alpha > 0.0 and _daylight != null:
			var a: float = roundf(_halo_alpha * float(_daylight.call("dark_at", person.position)) * float(HALO_STEPS)) / float(HALO_STEPS)
			if a > 0.0:
				outline = Color(_halo_rgba, a)
		if outline != person.outline:
			person.outline = outline
			person.mat.set_shader_parameter("outline_color", outline)
			# The drafted mark is wider (`people.drafted_outline_px`): a 1-px red line is lost on dark skin and fur at 1×.
			var width: float = _drafted_px if outline == _drafted_rgba else _outline_px
			if readable:
				width = maxf(width, _half_rim_px / maxf(_zoom, 0.01))
			person.mat.set_shader_parameter("outline_px", width)
	phase = _profile_span("person_reconcile_usec", phase)
	for key: int in _persons.keys():
		var person: Person = _persons[key]
		if person.seen != _frame:
			person.queue_free()
			_persons.erase(key)
	_place_sleepers(any_sleep)
	for person: Person in _persons.values():
		if person.activity == "sleep":
			person.position = person.to_pos + person.bed
		elif person.elapsed < person.duration:
			person.elapsed = minf(person.duration, person.elapsed + delta)
			person.position = person.from_pos.lerp(person.to_pos, person.elapsed / person.duration).round()
		else:
			person.position = person.to_pos
	phase = _profile_span("cleanup_placement_usec", phase)
	if not lod and _zoom < 1.0:
		_show_overlapping_figures()
	elif not _coincident_badges.is_empty():
		_clear_coincident_badges()
		queue_redraw()
	if lod or _lod:
		_lod = lod
		_refresh_markers()
	_lod = lod
	_profile_span("half_or_lod_total_usec", phase)


## At half zoom, let genuine coincident art show through; bodies keep their original feet and shadows.
## Cache opaque frame extents and use tile-size spatial buckets, avoiding all-pairs work at population1000.
func _show_overlapping_figures() -> void:
	var started: int = Time.get_ticks_usec() if _render_profiling else 0
	var rectangles: Dictionary = {}
	for person: Person in _persons.values():
		var texture: Texture2D = person.body.texture
		if texture == null:
			continue
		var key: int = texture.get_instance_id()
		if not _opaque_rects.has(key):
			var image: Image = texture.get_image()
			if image.is_compressed():
				image.decompress()
			_opaque_rects[key] = image.get_used_rect()
		var used: Rect2i = _opaque_rects[key]
		if person.body.flip_h:
			used.position.x = texture.get_width() - used.end.x
		var rectangle: Rect2 = Rect2(person.position + person.body.offset + Vector2(used.position), Vector2(used.size))
		rectangles[person.bits] = rectangle
	var overlaps: Dictionary = MarkerLayout.overlapping_people(rectangles, float(View.TILE))
	for person: Person in _persons.values():
		person.body.self_modulate = Color(1, 1, 1, _half_overlap_alpha if overlaps.has(person.bits) else 1.0)
	started = _profile_span("overlap_alpha_usec", started)
	_update_coincident_badges(rectangles)
	_profile_span("badges_total_usec", started)


## Counts come from the full person snapshot, never body visibility. Counterfactual body hides cannot alter a badge.
func _clear_coincident_badges() -> void:
	_coincident_badges.clear()
	_coincident_groups.clear()
	badge_layout_exhausted = 0
	if _badge_leaders != null:
		_badge_leaders.segments.clear()
		_badge_leaders.aggregate_segments.clear()
		_badge_leaders.active_segments.clear()
		_badge_leaders.queue_redraw()
