extends Node2D

const ArtHashScript = preload("res://scripts/ui/art_hash.gd")
const Cold = preload("res://scripts/ui/cold_effects.gd")
const MorningChill = preload("res://scripts/ui/morning_chill.gd")
const FoundingGroups = preload("res://scripts/ui/founding_group_renderer.gd")
const CarryLoad = preload("res://scripts/ui/carry_load_layer.gd")
const Activity = preload("res://scripts/ui/agent_activity.gd")
const SpriteLibrary = preload("res://scripts/ui/art_sprite_library.gd")
const CampSeatsScript = preload("res://scripts/ui/camp_seats.gd")
const ArtDeclumpP12 = preload("res://scripts/ui/art_declump_p12.gd")
const PoseMoments = preload("res://scripts/ui/agent_pose_moments.gd")
const SettlementSelection = preload("res://scripts/core/settlement_selection.gd")
const HoverMarker = preload("res://scripts/ui/offscreen_marker.gd")
const RING_COLOR := Color("D7BD77")
const SETTLEMENT_OUTLINE = preload("res://shaders/settlement_outline.gdshader")
const TILE_SIZE := 16
const SPRITE_ORIGIN_X := 448
const SPRITE_ORIGIN_Y := 28
const ICON_OFFSET_PX := Vector2(0, -30)
const ICON_SIZE_PX := Vector2(16, 16)
const RECALL_CUE_FRAMES := 36
const COMBAT_CUE_FRAMES := 36
const RECALL_CUE_TINT := Color(0.45, 0.65, 1.0, 1.0)
const COMBAT_CUE_TINT := Color(1.0, 0.3, 0.3, 1.0)
const SIM_TICK_DURATION: float = 1.0 / 30.0
const _SEEN_RECALL_MAX := 512
const _SEEN_COMBAT_MAX := 512
var world_sim: WorldSimNode
var _objects: Node2D
var _shadow_layer: Node2D
var _sprites: Dictionary = {}
var _activity_icons: Dictionary = {}
var _shadows: Dictionary = {}
var _cues: Dictionary = {}
var _sprite_pool: Array[Sprite2D] = []
var _shadow_pool: Array[Sprite2D] = []
var _animation_time: float = 0.0
var _render_positions: Dictionary = {}
var _drawn_positions: Dictionary = {}
var _walking_directions: Dictionary = {}
var _walking_tiles: Dictionary = {}
var _stationary_seconds: Dictionary = {}
var _camp_seats = CampSeatsScript.new()
var _pose_moments = PoseMoments.new()
var _morning_chill = MorningChill.new()
var _opaque_rects: Dictionary = {}
var _idle_seconds: Dictionary = {}
var _frame_groups: Dictionary = {}
var _selection_ring: Line2D
var _capture_visible: bool = true
var _lerp_accumulator: float = 0.0
var _move_duration: float = SIM_TICK_DURATION
var _snapshot_checksum: int = -1
var _prev_positions: Dictionary = {}
var _curr_positions: Dictionary = {}
var _recalling_agents: Dictionary = {}
var _combating_agents: Dictionary = {}
var _seen_recall_event_ids: Dictionary = {}
var _seen_combat_event_ids: Dictionary = {}
var _last_causal_tick: int = -1
var _causal_rearm: bool = false
var _talk_xs := PackedInt32Array()
var _talk_ys := PackedInt32Array()
var _talk_deltas: Dictionary = {}
var _base_looks: Dictionary = {}
var _spatial_hash_enabled: bool = true
var _declump_kernel_enabled_p12: bool = OS.get_environment("P12_DECLUMP_KERNEL") == "1"
var _declump_fixture_sink_p12: RefCounted
var _hovered_person_p12: int = -1
var _hover_owner_p12: int = 0
var _hover_marker_p12: Dictionary = {}
var _raised_sprite_p12: Sprite2D
var _raised_icon_p12: Sprite2D
var _raised_sprite_z_p12: int = 0
var _raised_icon_z_p12: int = 0
var _activity_oracle_enabled: bool = not OS.get_environment("P10_ACTIVITY_OUT").is_empty()

const SelectionIndex = preload("res://scripts/ui/agent_selection_index_p10.gd")
var _selection_index = SelectionIndex.new()
var _selection_lookup_enabled: bool = true
const FrameChoices = preload("res://scripts/ui/agent_frame_choices_p10.gd")
var _frame_choices = FrameChoices.new()
var _frame_choices_ready: bool = false
var _allocation_candidate: bool = true
var _pose_output: Dictionary = {}

func set_perf_candidate_p12(change_id: String, enabled: bool) -> bool:
	if change_id != "declump-kernel":
		return false
	_declump_kernel_enabled_p12 = enabled
	return true

func get_perf_candidate_p12() -> Dictionary:
	return {"change_id": "declump-kernel", "candidate_enabled": _declump_kernel_enabled_p12}

func set_perf_candidate_p10(change_id: String, enabled: bool) -> bool:
	if change_id != "allocation-cache":
		return false
	_camp_seats.set_input_cache_enabled(true)
	_spatial_hash_enabled = true
	_allocation_candidate = enabled
	_selection_lookup_enabled = enabled
	_selection_index.begin_frame()
	_frame_choices_ready = false
	SpriteLibrary.set_variant_regions_cache_enabled(enabled)
	return true

func get_perf_candidate_p10() -> Dictionary:
	return {"change_id": "allocation-cache", "candidate_enabled": _allocation_candidate}

func _ready() -> void:
	set_perf_candidate_p10("allocation-cache", _allocation_candidate)
	_objects = SpriteLibrary.shared_layer(self, "WorldObjects", 4, true)
	_shadow_layer = SpriteLibrary.shared_layer(self, "Shadows", 3)
	var founding_layer := FoundingGroups.new()
	founding_layer.name = "FoundingGroups"
	add_child(founding_layer)
	world_sim = get_node_or_null("../WorldSim") as WorldSimNode
	if world_sim != null:
		world_sim.connect("world_reset", _on_world_reset)

func _process(delta: float) -> void:
	if world_sim == null:
		return
	var world_info: Dictionary = SimReads.read(world_sim, "get_world_info")
	_move_duration = float(maxi(1, int(world_info.get("move_interval_ticks", 1)))) * SIM_TICK_DURATION
	_lerp_accumulator += delta
	_animation_time += delta
	_clear_expired_recalling()
	_clear_expired_combating()
	var snap: Dictionary = SimReads.read(world_sim, "get_agent_snapshot")
	var society: Dictionary = SimReads.read(world_sim, "get_society_snapshot")
	var founders: PackedInt64Array = society.get("founder_agent_ids", PackedInt64Array())
	var ids: PackedInt64Array = snap.get("ids", PackedInt64Array())
	var agent_ids: PackedInt64Array = snap.get("agent_ids", PackedInt64Array())
	var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
	var states: PackedByteArray = snap.get("states", PackedByteArray())
	var life_stages: Variant = snap.get("life_stages", PackedByteArray())
	var n: int = ids.size()
	if xs.size() != n or ys.size() != n or agent_ids.size() != n:
		push_error("AgentRenderer: parallel-array length mismatch")
		return
	if _selection_lookup_enabled:
		_selection_index.begin_frame()
	var activity_snapshot: Dictionary = SimReads.read(world_sim, "get_activity_snapshot")
	var daily_rows: Dictionary = Activity.activity_rows(activity_snapshot)
	_prepare_frame_choices()
	_prepare_talk_cache(xs, ys)
	var new_checksum: int = _snapshot_checksum_from(agent_ids, xs, ys, n)
	if new_checksum != _snapshot_checksum:
		_snapshot_checksum = new_checksum
		var previous_blend: float = clampf(_lerp_accumulator / _move_duration, 0.0, 1.0)
		for moving_id: int in _curr_positions:
			var previous_target: Vector2 = _curr_positions[moving_id]
			var previous_origin: Vector2 = _prev_positions.get(moving_id, previous_target)
			_prev_positions[moving_id] = previous_origin.lerp(previous_target, previous_blend)
		_curr_positions.clear()
		for i in n:
			var look: Dictionary = _base_look(int(agent_ids[i]))
			_curr_positions[int(agent_ids[i])] = Vector2(SPRITE_ORIGIN_X + xs[i] * TILE_SIZE + 8 + int(look["jx"]), SPRITE_ORIGIN_Y + ys[i] * TILE_SIZE + 14 + int(look["jy"]))
		for old_id: int in _prev_positions.keys():
			if not _curr_positions.has(old_id):
				_prev_positions.erase(old_id)
		_lerp_accumulator = 0.0
	var lerp_t: float = clampf(_lerp_accumulator / _move_duration, 0.0, 1.0)
	var live: Dictionary = {}
	var context: Dictionary = _activity_context()
	var tile_contexts: Dictionary = {}
	var time: Dictionary = SimReads.read(world_sim, "get_time_snapshot")
	var tick: int = int(time.get("tick", 0))
	ingest_chill_snapshot(snap, tick)
	var hour: float = float(time.get("time_of_day", 12.0))
	var draw_agents: Array[Dictionary] = []
	for i in n:
		var agent_id: int = int(agent_ids[i])
		live[agent_id] = true
		var curr_pos: Vector2 = _curr_positions.get(agent_id, Vector2.ZERO)
		var prev_pos: Vector2 = _prev_positions.get(agent_id, curr_pos)
		var ipos: Vector2 = prev_pos.lerp(curr_pos, lerp_t)
		var sprite: Sprite2D = _ensure_agent(agent_id)
		var look: Dictionary = _base_look(agent_id)
		var life_stage: int = int(life_stages[i]) if i < life_stages.size() else 0
		look = Activity.elder_look(look, agent_id, life_stage)
		var child: bool = life_stage == 1
		var previous_position: Vector2 = _render_positions.get(agent_id, ipos)
		var movement: Vector2 = ipos - previous_position
		var moving: bool = movement.length_squared() > 0.000001
		if moving and not _walking_directions.has(agent_id):
			_walking_directions[agent_id] = movement.normalized()
		_stationary_seconds[agent_id] = 0.0 if moving else float(_stationary_seconds.get(agent_id, 0.0)) + delta
		_render_positions[agent_id] = ipos
		var tile_position := Vector2i(xs[i], ys[i])
		if not tile_contexts.has(tile_position):
			tile_contexts[tile_position] = _tile_context(tile_position, context)
		var tile: Dictionary = tile_contexts[tile_position]
		var state: int = int(states[i]) if i < states.size() else 0
		var target: int = Activity.snapshot_target(snap, i)
		var has_targets: bool = target >= 0
		var carried: float = float(snap["carried_foods"][i]) if snap.has("carried_foods") and i < snap["carried_foods"].size() else 0.0
		var wood: int = Activity.snapshot_wood(snap, i)
		var moments: Dictionary
		if _allocation_candidate:
			_pose_moments.update_into(agent_id, i, snap, Vector2i(xs[i], ys[i]), not moving, bool(tile.get("wood", false)), context["hearths"], delta, _pose_output)
			moments = _pose_output
		else:
			moments = _pose_moments.update(agent_id, i, snap, Vector2i(xs[i], ys[i]), not moving, bool(tile.get("wood", false)), context["hearths"], delta)
		var daily: Dictionary = daily_rows.get(int(ids[i]), {})
		var daily_kind: int = int(daily.get("kind", -1))
		var arrived: bool = not daily.is_empty() and int(daily.get("until_tick", -1)) >= 0
		var daily_active: bool = not daily.is_empty() and (state == 4 or target == 8)
		var overrides: Array[String] = []
		var activity: String = Activity.resolve(state, target, has_targets, moving, carried, tile, hour, wood, daily_kind, arrived)
		if bool(moments["pickup"]):
			activity = "gather_wood"
			overrides.append("pickup")
		_idle_seconds[agent_id] = Activity.idle_elapsed(activity, movement, float(_idle_seconds.get(agent_id, 0.0)), delta)
		var partner_delta: Vector2 = _cached_talk_delta(i, xs, ys) if activity == "talk" else Vector2.ZERO
		var previous_facing: String = str(sprite.get_meta("facing", "S"))
		var override_facing: String = ""
		if daily_active and arrived and not bool(moments["pickup"]):
			override_facing = _daily_facing(daily_kind, daily, agent_id, context, previous_facing)
		var facing: String = Activity.render_facing(activity, movement, partner_delta, previous_facing, float(_idle_seconds[agent_id]), override_facing)
		var pose: String = Activity.camp_pose(activity, life_stage, agent_id, int(time.get("day_index", 0)), hour, bool(tile.get("near_camp", false)))
		var body: String = "child" if child else ("adult_a" if int(look["body"]) == 0 else "adult_b")
		if daily_active and not bool(moments["pickup"]):
			var presentation: Dictionary = _frame_choices.daily_pose(body, daily_kind, facing, _animation_time, arrived) if _allocation_candidate else Activity.daily_pose(_frame_groups, body, daily_kind, facing, _animation_time, arrived)
			pose = str(presentation["pose"])
			facing = str(presentation["facing"])
			if not arrived:
				activity = pose
		if bool(moments["meal"]):
			pose = "eat"
			overrides.append("meal")
		var cold: bool = Cold.snapshot_cold(snap, i)
		var base_pose: String = pose
		pose = Cold.pose(pose, cold or _morning_chill.alpha(agent_id, tick) > 0.0, moving)
		if pose != base_pose:
			if cold:
				overrides.append("cold")
			if _morning_chill.alpha(agent_id, tick) > 0.0:
				overrides.append("morning_chill")
		var choice: Dictionary = frame_choice(body, pose, facing, float(look["phase"]))
		if daily_active and not bool(moments["pickup"]) and not bool(moments["meal"]):
			choice = _frame_choices.with_daily_icon(choice, daily_kind) if _allocation_candidate else Activity.with_daily_icon(choice, daily_kind)
		var sprite_id: String = str(choice.get("sprite_id", ""))
		if sprite_id.is_empty():
			sprite.texture = null
			sprite.set_meta("sprite_id", "")
		elif str(sprite.get_meta("sprite_id", "")) != sprite_id:
			SpriteLibrary.apply(sprite, sprite_id)
		SpriteLibrary.apply_variant(sprite, look)
		sprite.flip_h = facing == "W"
		var sheltered_sleep: bool = Activity.hidden(activity, has_targets, tile)
		sprite.set_meta("sheltered_sleep", sheltered_sleep)
		sprite.visible = _capture_visible and not sheltered_sleep and not sprite_id.is_empty()
		sprite.set_meta("p10_activity_schema", 1)
		sprite.set_meta("activity_kind", daily_kind)
		sprite.set_meta("activity_place_known", arrived)
		sprite.set_meta("activity_overrides", overrides)
		sprite.set_meta("activity_frame_view", str(choice.get("view", "")))
		sprite.set_meta("activity_frame_activity", str(choice.get("activity", "")))
		if _activity_oracle_enabled:
			sprite.set_meta("activity_oracle_inputs", {"schema": 1,
				"animation_seconds": _animation_time, "previous_facing": previous_facing,
				"movement": [movement.x, movement.y], "pickup": bool(moments["pickup"]), "meal": bool(moments["meal"]),
				"morning_chill_alpha": _morning_chill.alpha(agent_id, tick), "capture_visible": _capture_visible,
				"parent_visible_in_tree": (sprite.get_parent() as CanvasItem).is_visible_in_tree()})
		sprite.set_meta("entity_bits", int(ids[i]))
		if _selection_lookup_enabled:
			_selection_index.observe(int(ids[i]), agent_id)
		sprite.set_meta("body", body)
		sprite.set_meta("variant", {"tone": look["tone"], "hair": look["hair"]})
		sprite.set_meta("tile_x", int(xs[i]))
		sprite.set_meta("tile_y", int(ys[i]))
		sprite.set_meta("activity", activity)
		sprite.set_meta("pose", pose)
		sprite.set_meta("cold_pose", str(choice.get("activity", pose)))
		sprite.set_meta("cold_base_pose", base_pose)
		sprite.set_meta("moving", moving)
		sprite.set_meta("facing", facing)
		sprite.set_meta("fallback_kind", str(choice.get("fallback_kind", "")))
		sprite.set_meta("qa_status", str(choice.get("qa_status", "unavailable")))
		CarryLoad.apply(sprite, activity)
		CarryLoad.apply_founder(sprite, founders.has(agent_id), activity, carried, wood, base_pose)
		_update_activity_icon(agent_id, sprite, choice)
		draw_agents.append({"id": agent_id, "ipos": ipos, "tile": Vector2i(xs[i], ys[i]), "visible": sprite.visible, "moving": moving, "stationary_seconds": _stationary_seconds[agent_id], "pose": pose, "activity": activity, "cold": cold, "meal": bool(moments["meal"]), "walking_direction": _walking_directions.get(agent_id, Vector2.ZERO), "activity_place_known": arrived})
	_apply_drawn_positions(draw_agents, context, delta)
	for row: Dictionary in draw_agents:
		var agent_id: int = int(row["id"])
		var sprite: Sprite2D = _sprites[agent_id]
		Cold.apply(sprite, bool(row.get("cold", false)), bool(row["moving"]), agent_id, _animation_time)
		_morning_chill.apply(sprite, agent_id, tick, _animation_time)
		sprite.set_meta("drawn_position", sprite.position)
		_apply_settlement_outline(sprite, agent_id, agent_id == _hovered_person_p12)
		var shadow: Sprite2D = _shadows[agent_id]
		shadow.position = sprite.position
		shadow.visible = sprite.visible
		_update_cue(agent_id, sprite.position, _recalling_agents.has(agent_id), _combating_agents.has(agent_id))
	_retire_missing(live)
	_update_selection_ring()
	_update_link_hover_p12()
	_ingest_causal_events(tick, xs, ys)

func _ensure_agent(agent_id: int) -> Sprite2D:
	if _sprites.has(agent_id):
		return _sprites[agent_id] as Sprite2D
	var sprite: Sprite2D = _sprite_pool.pop_back() if not _sprite_pool.is_empty() else Sprite2D.new()
	if sprite.get_parent() == null:
		_objects.add_child(sprite)
	sprite.name = "Agent_%d" % agent_id
	sprite.set_meta("facing", "S")
	sprite.set_meta("art_kind", "agent")
	sprite.set_meta("art_id", agent_id)
	_sprites[agent_id] = sprite
	var shadow: Sprite2D = _shadow_pool.pop_back() if not _shadow_pool.is_empty() else Sprite2D.new()
	if shadow.get_parent() == null:
		_shadow_layer.add_child(shadow)
	shadow.texture = load("res://assets/art/contact_shadow.png") as Texture2D
	shadow.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	shadow.modulate = Color(1.0, 1.0, 1.0, 0.4)
	shadow.set_meta("art_kind", "agent_shadow")
	shadow.set_meta("agent_shadow", true)
	shadow.set_meta("art_id", agent_id)
	_shadows[agent_id] = shadow
	return sprite

func _update_cue(agent_id: int, feet: Vector2, recalling: bool, combating: bool) -> void:
	if not recalling and not combating:
		if _cues.has(agent_id):
			(_cues[agent_id] as Line2D).visible = false
		return
	if not _cues.has(agent_id):
		var cue: Line2D = Line2D.new()
		cue.width = 1.0
		cue.points = PackedVector2Array([Vector2(-5, -2), Vector2(5, -2), Vector2(5, 2), Vector2(-5, 2), Vector2(-5, -2)])
		_shadow_layer.add_child(cue)
		_cues[agent_id] = cue
	var cue: Line2D = _cues[agent_id]
	cue.position = feet
	cue.default_color = COMBAT_CUE_TINT if combating else RECALL_CUE_TINT
	cue.visible = _capture_visible

func _retire_missing(live: Dictionary) -> void:
	for agent_id: int in _base_looks.keys():
		if not live.has(agent_id):
			_base_looks.erase(agent_id)
	for agent_id: int in _sprites.keys():
		if live.has(agent_id):
			continue
		var sprite: Sprite2D = _sprites[agent_id]
		sprite.visible = false
		_sprite_pool.append(sprite)
		_sprites.erase(agent_id)
		if _activity_icons.has(agent_id):
			(_activity_icons[agent_id] as Sprite2D).queue_free()
			_activity_icons.erase(agent_id)
		_render_positions.erase(agent_id)
		_drawn_positions.erase(agent_id)
		_walking_directions.erase(agent_id)
		_walking_tiles.erase(agent_id)
		_stationary_seconds.erase(agent_id)
		_idle_seconds.erase(agent_id)
		_pose_moments.retire(agent_id)
		var shadow: Sprite2D = _shadows[agent_id]
		shadow.visible = false
		_shadow_pool.append(shadow)
		_shadows.erase(agent_id)
		if _cues.has(agent_id):
			(_cues[agent_id] as Line2D).queue_free()
			_cues.erase(agent_id)

func set_capture_visible(shown: bool) -> void:
	_capture_visible = shown
	_process(0.0)

func ingest_chill_snapshot(snapshot: Dictionary, tick: int) -> void:
	_morning_chill.ingest(snapshot, tick)
	var ids: PackedInt64Array = snapshot.get("agent_ids", PackedInt64Array())
	var xs: PackedInt32Array = snapshot.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snapshot.get("ys", PackedInt32Array())
	if xs.size() != ids.size() or ys.size() != ids.size():
		return
	for index: int in ids.size():
		var agent_id: int = int(ids[index])
		var tile := Vector2(xs[index], ys[index])
		if _walking_tiles.has(agent_id):
			var direction: Vector2 = tile - Vector2(_walking_tiles[agent_id])
			if not direction.is_zero_approx():
				_walking_directions[agent_id] = direction.normalized()
		_walking_tiles[agent_id] = tile


func refresh_snapshot() -> void:
	_prev_positions.clear()
	_curr_positions.clear()
	_snapshot_checksum = -1
	_process(0.0)

func head_icon_position(world_pos: Vector2) -> Vector2:
	return world_pos + ICON_OFFSET_PX

func _on_world_reset() -> void:
	clear_hovered_person_p12(_hover_owner_p12)
	_frame_choices_ready = false
	_pose_output.clear()
	_selection_index.begin_frame()
	_talk_xs = PackedInt32Array()
	_talk_ys = PackedInt32Array()
	_talk_deltas.clear()
	_base_looks.clear()
	_morning_chill.clear()
	_render_positions.clear()
	_drawn_positions.clear()
	_walking_directions.clear()
	_walking_tiles.clear()
	_stationary_seconds.clear()
	_camp_seats = CampSeatsScript.new()
	_opaque_rects.clear()
	_idle_seconds.clear()
	_pose_moments.clear()
	_animation_time = 0.0
	_prev_positions.clear()
	_curr_positions.clear()
	_snapshot_checksum = -1
	_lerp_accumulator = 0.0
	_recalling_agents.clear()
	_combating_agents.clear()
	_seen_recall_event_ids.clear()
	_seen_combat_event_ids.clear()
	_last_causal_tick = -1
	_causal_rearm = false
	_retire_missing({})
	_process(0.0)

func mark_agent_recalling(agent_id: int) -> void:
	_recalling_agents[agent_id] = RECALL_CUE_FRAMES

func recall_cue_remaining(agent_id: int) -> int:
	return int(_recalling_agents.get(agent_id, 0))

func _ingest_causal_events(tick: int, xs: PackedInt32Array, ys: PackedInt32Array) -> void:
	if world_sim == null or (tick == _last_causal_tick and not _causal_rearm):
		return
	if not world_sim.has_method("get_tile_causal_events"):
		return
	_last_causal_tick = tick
	_causal_rearm = false
	var visited: Dictionary = {}
	var tile_xs := PackedInt32Array()
	var tile_ys := PackedInt32Array()
	for i: int in range(mini(xs.size(), ys.size())):
		var tile := Vector2i(xs[i], ys[i])
		if visited.has(tile):
			continue
		visited[tile] = true
		tile_xs.append(tile.x)
		tile_ys.append(tile.y)
	var events: Array = SimReads.causal_events(world_sim, tile_xs, tile_ys, PackedStringArray(["memory_recalled", "combat_started"]))
	_ingest_memory_recalls(events)
	_ingest_combat_events(events)

func _ingest_memory_recalls(events: Array) -> void:
	for ev: Dictionary in events:
		if String(ev.get("kind", "")) != "memory_recalled":
			continue
		var event_id: int = int(ev.get("id", -1))
		if event_id < 0 or _seen_recall_event_ids.has(event_id):
			continue
		_seen_recall_event_ids[event_id] = true
		var agent_id: int = int(ev.get("agent_id", -1))
		if agent_id >= 0:
			mark_agent_recalling(agent_id)
	if _seen_recall_event_ids.size() > _SEEN_RECALL_MAX:
		_seen_recall_event_ids.clear()
		_causal_rearm = true

func _clear_expired_recalling() -> void:
	if _recalling_agents.is_empty():
		return
	var expired: Array = []
	for eid in _recalling_agents:
		var remaining: int = int(_recalling_agents[eid]) - 1
		if remaining <= 0:
			expired.append(eid)
		else:
			_recalling_agents[eid] = remaining
	for eid in expired:
		_recalling_agents.erase(eid)

func mark_agent_in_combat(agent_id: int) -> void:
	_combating_agents[agent_id] = COMBAT_CUE_FRAMES

func mark_agent_combating(agent_id: int) -> void:
	mark_agent_in_combat(agent_id)

func combat_cue_remaining(agent_id: int) -> int:
	return int(_combating_agents.get(agent_id, 0))

func _ingest_combat_events(events: Array) -> void:
	for ev: Dictionary in events:
		if String(ev.get("kind", "")) != "combat_started":
			continue
		var event_id: int = int(ev.get("id", -1))
		if event_id < 0 or _seen_combat_event_ids.has(event_id):
			continue
		_seen_combat_event_ids[event_id] = true
		var agent_id: int = int(ev.get("agent_id", -1))
		if agent_id >= 0:
			mark_agent_in_combat(agent_id)
	if _seen_combat_event_ids.size() > _SEEN_COMBAT_MAX:
		_seen_combat_event_ids.clear()
		_causal_rearm = true

func _clear_expired_combating() -> void:
	if _combating_agents.is_empty():
		return
	var expired: Array = []
	for eid in _combating_agents:
		var remaining: int = int(_combating_agents[eid]) - 1
		if remaining <= 0:
			expired.append(eid)
		else:
			_combating_agents[eid] = remaining
	for eid in expired:
		_combating_agents.erase(eid)

func _snapshot_checksum_from(agent_ids: PackedInt64Array, xs: PackedInt32Array, ys: PackedInt32Array, n: int) -> int:
	if n == 0:
		return 0
	var h: int = n * 2654435761
	for i in n:
		var aid: int = int(agent_ids[i])
		h = (h * 1000003) ^ (aid * 73856093) ^ (int(xs[i]) * 19349663) ^ (int(ys[i]) * 83492791) ^ (i * 2654435761)
	return h

func _prepare_frame_choices() -> void:
	if _frame_groups.is_empty():
		_frame_groups = Activity.build_frame_groups(SpriteLibrary.entries())
	if _allocation_candidate and not _frame_choices_ready:
		_frame_choices.configure(_frame_groups, SpriteLibrary.entries())
		_frame_choices_ready = true

func frame_choice(body: String, activity: String, facing: String, phase: float) -> Dictionary:
	_prepare_frame_choices()
	if _allocation_candidate:
		return _frame_choices.choose(body, activity, facing, _animation_time, phase)
	return Activity.choose_frame(_frame_groups, body, activity, facing, _animation_time, phase)

func _animation_frame(body: String, activity: String, facing: String, phase: float, _agent_id: int) -> String:
	return str(frame_choice(body, activity, facing, phase).get("sprite_id", ""))

func _update_activity_icon(agent_id: int, sprite: Sprite2D, choice: Dictionary) -> void:
	var camera: Camera2D = get_node_or_null("../Camera2D") as Camera2D
	var zoom: float = camera.zoom.x if camera != null else 1.0
	var show_icon: bool = Activity.show_activity_icon(choice, zoom) and sprite.visible
	if not show_icon:
		if _activity_icons.has(agent_id):
			(_activity_icons[agent_id] as Sprite2D).visible = false
		return
	if not _activity_icons.has(agent_id):
		var icon: Sprite2D = Sprite2D.new()
		sprite.add_child(icon)
		icon.set_meta("art_kind", "activity_icon")
		icon.set_meta("art_id", agent_id)
		_activity_icons[agent_id] = icon
	var icon: Sprite2D = _activity_icons[agent_id]
	Activity.apply_activity_icon(sprite, icon, choice, zoom)

func _activity_context() -> Dictionary:
	var context: Dictionary = {"shelters": {}, "hearths": [], "stockpiles": [], "sites": {}, "food": {}, "wood": {}, "water": {}, "seat_blocked": {}}
	var buildings: Dictionary = SimReads.read(world_sim, "get_building_snapshot")
	var xs: PackedInt32Array = buildings.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = buildings.get("ys", PackedInt32Array())
	var kinds: PackedByteArray = buildings.get("kinds", PackedByteArray())
	for i: int in range(mini(xs.size(), mini(ys.size(), kinds.size()))):
		var tile: Vector2i = Vector2i(xs[i], ys[i])
		context["seat_blocked"][tile] = true
		if kinds[i] == 0:
			context["hearths"].append(tile)
		elif kinds[i] == 1:
			context["shelters"][tile] = true
		elif kinds[i] == 2:
			context["stockpiles"].append(tile)
	for source: String in ["sites", "food"]:
		var snap: Dictionary = SimReads.read(world_sim, "get_construction_snapshot") if source == "sites" else SimReads.read(world_sim, "get_resource_snapshot")
		var sx: PackedInt32Array = snap.get("xs", PackedInt32Array())
		var sy: PackedInt32Array = snap.get("ys", PackedInt32Array())
		var sk: PackedInt32Array = snap.get("kinds", PackedInt32Array())
		for i: int in range(mini(sx.size(), sy.size())):
			if source == "sites" or (i < sk.size() and sk[i] == 3):
				context["seat_blocked"][Vector2i(sx[i], sy[i])] = true
			if source == "food" and i < sk.size() and sk[i] == 3:
				context["wood"][Vector2i(sx[i], sy[i])] = true
			if source == "sites" or (i < sk.size() and sk[i] == 0):
				context[source][Vector2i(sx[i], sy[i])] = true
	var terrain: Dictionary = SimReads.read(world_sim, "get_terrain_snapshot")
	context["terrain"] = terrain
	return context

func _tile_context(tile: Vector2i, context: Dictionary) -> Dictionary:
	var near: bool = false
	for hearth: Vector2i in context["hearths"]:
		if maxi(absi(tile.x - hearth.x), absi(tile.y - hearth.y)) <= 3:
			near = true
	var near_camp: bool = near
	for shelter: Vector2i in context["shelters"]:
		if maxi(absi(tile.x - shelter.x), absi(tile.y - shelter.y)) <= 3:
			near_camp = true
	var terrain: Dictionary = context["terrain"]
	var width: int = int(terrain.get("width", 0))
	var height: int = int(terrain.get("height", 0))
	var cells: PackedByteArray = terrain.get("terrain", PackedByteArray())
	var water: bool = false
	for delta: Vector2i in [Vector2i.ZERO, Vector2i.LEFT, Vector2i.RIGHT, Vector2i.UP, Vector2i.DOWN]:
		var t: Vector2i = tile + delta
		var index: int = t.y * width + t.x
		if t.x >= 0 and t.x < width and t.y >= 0 and t.y < height and index < cells.size() and cells[index] == 4:
			water = true
	return {"shelter": context["shelters"].has(tile), "construction": context["sites"].has(tile), "food": context["food"].has(tile), "wood": context.get("wood", {}).has(tile), "water": water, "near_hearth": near, "near_camp": near_camp}

func _base_look(agent_id: int) -> Dictionary:
	if not _base_looks.has(agent_id):
		var look: Dictionary = ArtHashScript.look(agent_id)
		look.make_read_only()
		_base_looks[agent_id] = look
	return _base_looks[agent_id]

func _prepare_talk_cache(xs: PackedInt32Array, ys: PackedInt32Array) -> void:
	if xs != _talk_xs or ys != _talk_ys:
		_talk_xs = xs.duplicate()
		_talk_ys = ys.duplicate()
		_talk_deltas.clear()

func _cached_talk_delta(index: int, xs: PackedInt32Array, ys: PackedInt32Array) -> Vector2:
	if not _talk_deltas.has(index):
		_talk_deltas[index] = _talk_delta(index, xs, ys)
	return _talk_deltas[index]

func _talk_delta(index: int, xs: PackedInt32Array, ys: PackedInt32Array) -> Vector2:
	var nearest: Vector2 = Vector2.ZERO
	var distance: float = 3.0
	for i: int in range(xs.size()):
		if i == index:
			continue
		var delta: Vector2 = Vector2(xs[i] - xs[index], ys[i] - ys[index])
		if maxf(absf(delta.x), absf(delta.y)) <= 1.0 and delta.length_squared() < distance:
			nearest = delta
			distance = delta.length_squared()
	return nearest

func get_agent_sprite(agent_id: int) -> Sprite2D:
	return _sprites.get(agent_id) as Sprite2D

func _apply_drawn_positions(agents: Array[Dictionary], context: Dictionary, delta: float) -> void:
	var hearths: Array[Vector2i] = []
	for hearth: Vector2i in context["hearths"]:
		hearths.append(hearth)
	var blocked: Dictionary = context["seat_blocked"]
	var terrain: Dictionary = context["terrain"]
	var width: int = int(terrain.get("width", 0))
	var height: int = int(terrain.get("height", 0))
	var cells: PackedByteArray = terrain.get("terrain", PackedByteArray())
	# Only seat-adjacent terrain matters; never scan the whole world per frame.
	for hearth: Vector2i in hearths:
		for y: int in range(hearth.y - 5, hearth.y + 6):
			for x: int in range(hearth.x - 7, hearth.x + 8):
				var index: int = y * width + x
				if x < 0 or x >= width or y < 0 or y >= height or index >= cells.size() or cells[index] == 4:
					blocked[Vector2i(x, y)] = true
	var seats: Dictionary = _camp_seats.update(hearths, blocked, agents)
	var spread: Dictionary = CampSeatsScript.spread_offsets(agents, seats)
	var targets: Dictionary = {}
	var rects: Dictionary = {}
	for row: Dictionary in agents:
		var agent_id: int = int(row["id"])
		targets[agent_id] = seats.get(agent_id, Vector2(row["ipos"]) + Vector2(spread.get(agent_id, Vector2.ZERO)))
		var sprite: Sprite2D = _sprites[agent_id]
		var size: Vector2 = Vector2(sprite.get_meta("frame_size", Vector2i.ZERO))
		var anchor: Vector2 = sprite.get_meta("anchor", Vector2.ZERO)
		var origin: Vector2 = -anchor
		if sprite.flip_h:
			origin.x = anchor.x - size.x
		if sprite.flip_v:
			origin.y = anchor.y - size.y
		var footprint := Rect2(origin, size)
		var load_layer: Sprite2D = sprite.get_node_or_null("CarryLoad") as Sprite2D
		if load_layer != null and load_layer.visible:
			var load_rect: Rect2 = load_layer.get_rect()
			load_rect.position += load_layer.position
			footprint = footprint.merge(load_rect)
		rects[agent_id] = footprint
	if _declump_fixture_sink_p12 != null:
		_declump_fixture_sink_p12.call("record", agents, targets, terrain, blocked, rects)
	var declump: Dictionary = ArtDeclumpP12.declump(world_sim, agents, targets, terrain, blocked, rects, _spatial_hash_enabled) if _declump_kernel_enabled_p12 else CampSeatsScript.declump(agents, seats, targets, terrain, blocked, rects, _spatial_hash_enabled)
	for row: Dictionary in agents:
		var agent_id: int = int(row["id"])
		var target: Vector2 = Vector2(targets[agent_id]) + Vector2(declump.get(agent_id, Vector2.ZERO))
		var drawn: Vector2 = _drawn_positions.get(agent_id, target)
		drawn += (target - drawn) * (1.0 - exp(-delta / 0.08))
		if drawn.distance_to(target) < 0.5:
			drawn = target
		_drawn_positions[agent_id] = drawn
		var sprite: Sprite2D = _sprites[agent_id]
		sprite.position = drawn.round()
		sprite.set_meta("drawn_position", sprite.position)
		_apply_settlement_outline(sprite, agent_id, agent_id == _hovered_person_p12)
		sprite.set_meta("seated", seats.has(agent_id))
		if seats.has(agent_id):
			sprite.set_meta("seat", seats[agent_id])
		elif sprite.has_meta("seat"):
			sprite.remove_meta("seat")
		sprite.set_meta("drawn_target", target)
		sprite.set_meta("spread_offset", spread.get(agent_id, Vector2.ZERO))
		sprite.set_meta("declump_offset", declump.get(agent_id, Vector2.ZERO))
	var rounded: Dictionary = {}
	for row: Dictionary in agents:
		var agent_id: int = int(row["id"])
		rounded[agent_id] = (_sprites[agent_id] as Sprite2D).position
	var canvas: Transform2D = _objects.get_global_transform_with_canvas() if _objects != null else get_global_transform_with_canvas()
	var lateral: Dictionary = CampSeatsScript.walker_offsets(agents, rounded, rects, terrain, blocked, canvas, _spatial_hash_enabled)
	for row: Dictionary in agents:
		var agent_id: int = int(row["id"])
		var sprite: Sprite2D = _sprites[agent_id]
		var offset: Vector2 = lateral.get(agent_id, Vector2.ZERO)
		sprite.position += offset
		sprite.set_meta("walker_lateral_offset", offset)
		sprite.set_meta("walker_lateral_screen_offset", canvas.basis_xform(offset))
		sprite.set_meta("walking_direction", row.get("walking_direction", Vector2.ZERO))
		sprite.set_meta("drawn_position", sprite.position)

func has_drawn_position(agent_id: int) -> bool:
	return _drawn_positions.has(agent_id)

func get_drawn_position(agent_id: int) -> Vector2:
	if _sprites.has(agent_id):
		return (_sprites[agent_id] as Sprite2D).position
	return Vector2(_drawn_positions.get(agent_id, Vector2.ZERO)).round()

func drawn_entity_at(world_position: Vector2) -> int:
	var hits: PackedInt64Array = drawn_entities_at(world_position)
	return hits[0] if not hits.is_empty() else -1

func drawn_entities_at(world_position: Vector2) -> PackedInt64Array:
	var hits: Array[Dictionary] = []
	for agent_id: int in _sprites:
		var sprite: Sprite2D = _sprites[agent_id]
		if not sprite.visible or sprite.texture == null or not has_drawn_position(agent_id):
			continue
		var sprite_id: String = str(sprite.get_meta("sprite_id", ""))
		if not _opaque_rects.has(sprite_id):
			var pixels: Image = sprite.texture.get_image()
			if sprite.region_enabled:
				pixels = pixels.get_region(Rect2i(sprite.region_rect))
			_opaque_rects[sprite_id] = Rect2(pixels.get_used_rect())
		var opaque: Rect2 = _opaque_rects[sprite_id]
		var frame_rect: Rect2 = sprite.get_rect()
		if sprite.flip_h:
			opaque.position.x = frame_rect.size.x - opaque.end.x
		if sprite.flip_v:
			opaque.position.y = frame_rect.size.y - opaque.end.y
		opaque.position += frame_rect.position
		if opaque.has_point(sprite.to_local(world_position)):
			hits.append({"id": agent_id, "entity": int(sprite.get_meta("entity_bits", -1)), "y": sprite.position.y})
	hits.sort_custom(func(a: Dictionary, b: Dictionary) -> bool:
		return float(a["y"]) > float(b["y"]) if a["y"] != b["y"] else int(a["id"]) > int(b["id"]))
	var entities := PackedInt64Array()
	for hit: Dictionary in hits:
		if int(hit["entity"]) >= 0:
			entities.append(int(hit["entity"]))
	return entities

func _raise_selected_draw_order_p12(selected: Sprite2D) -> void:
	if _raised_sprite_p12 != selected:
		if is_instance_valid(_raised_sprite_p12):
			_raised_sprite_p12.z_index = _raised_sprite_z_p12
		_raised_sprite_p12 = selected
		if is_instance_valid(selected):
			_raised_sprite_z_p12 = selected.z_index
			selected.z_index = _raised_sprite_z_p12 + 1
	var icon: Sprite2D
	if is_instance_valid(selected):
		icon = _activity_icons.get(int(selected.get_meta("art_id", -1))) as Sprite2D
	if _raised_icon_p12 != icon:
		if is_instance_valid(_raised_icon_p12):
			_raised_icon_p12.z_index = _raised_icon_z_p12
		_raised_icon_p12 = icon
		if is_instance_valid(icon):
			_raised_icon_z_p12 = icon.z_index
			icon.z_index = _raised_icon_z_p12 + 1

func _update_selection_ring() -> void:
	if _selection_ring == null:
		_selection_ring = Line2D.new()
		_selection_ring.name = "SelectionRing"
		_selection_ring.width = 1.0
		_selection_ring.default_color = RING_COLOR
		_selection_ring.set_meta("art_kind", "selection_ring")
		_shadow_layer.add_child(_selection_ring)
	_selection_ring.visible = false
	var camera: Camera2D = get_node_or_null("../Camera2D") as Camera2D
	_selection_ring.width = 2.0 if camera == null or camera.zoom.x <= 2.0 else 1.5
	var inspector: Node = get_node_or_null("../UI/AgentInspectorPanel")
	if inspector == null or not _capture_visible:
		_raise_selected_draw_order_p12(null)
		return
	var selected: int = int(inspector.call("get_selected_entity_bits"))
	var selected_sprite: Sprite2D
	if _selection_lookup_enabled:
		selected_sprite = _selection_index.find(_sprites, selected)
	else:
		for candidate: Sprite2D in _sprites.values():
			if int(candidate.get_meta("entity_bits", -1)) == selected and candidate.visible:
				selected_sprite = candidate
				break
	_raise_selected_draw_order_p12(selected_sprite)
	if selected_sprite == null:
		return
	var sprite: Sprite2D = selected_sprite
	var pulse: int = posmod(floori(_animation_time * 2.0), 2)
	var extent: float = float(7 + pulse)
	_selection_ring.points = PackedVector2Array([Vector2(-extent, -1), Vector2(-extent + 2, -3), Vector2(extent - 2, -3), Vector2(extent, -1), Vector2(extent, 1), Vector2(extent - 2, 3), Vector2(-extent + 2, 3), Vector2(-extent, 1), Vector2(-extent, -1)])
	_selection_ring.position = sprite.position
	_selection_ring.set_meta("art_id", int(sprite.get_meta("art_id")))
	_selection_ring.set_meta("frame_id", pulse)
	_selection_ring.visible = true


func get_active_agent_sprites() -> Array[Sprite2D]:
	var active: Array[Sprite2D] = []
	for sprite: Sprite2D in _sprites.values():
		active.append(sprite)
	return active

func is_capture_visible() -> bool:
	return _capture_visible


static func _apply_settlement_outline(sprite: Sprite2D, agent_id: int, hovered: bool = false) -> void:
	sprite.set_meta("link_hover_outline", hovered)
	if not SettlementSelection.layers_enabled and not hovered:
		var existing: Sprite2D = sprite.get_node_or_null("SettlementOutline") as Sprite2D
		if existing != null:
			existing.hide()
		sprite.set_meta("settlement_outline", false)
		return
	var home: int = SettlementSelection.home_for_agent(agent_id)
	var selected: bool = SettlementSelection.layers_enabled and SettlementSelection.selected_id >= 0 and home == SettlementSelection.selected_id
	sprite.set_meta("home_settlement_id", home)
	sprite.set_meta("settlement_outline", selected)
	var outline: Sprite2D = sprite.get_node_or_null("SettlementOutline") as Sprite2D
	if (not selected and not hovered) or sprite.texture == null:
		if outline != null:
			outline.hide()
		return
	if outline == null:
		outline = Sprite2D.new()
		outline.name = "SettlementOutline"
		outline.show_behind_parent = true
		outline.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
		var material := ShaderMaterial.new()
		material.shader = SETTLEMENT_OUTLINE
		outline.material = material
		sprite.add_child(outline)
	outline.texture = sprite.texture
	outline.region_enabled = sprite.region_enabled
	outline.region_rect = sprite.region_rect.grow(1.0)
	outline.centered = sprite.centered
	outline.offset = sprite.offset - Vector2.ONE
	outline.flip_h = sprite.flip_h
	outline.flip_v = sprite.flip_v
	var material := outline.material as ShaderMaterial
	var size: Vector2 = sprite.texture.get_size()
	var rect: Rect2 = sprite.region_rect
	material.set_shader_parameter("atlas_rect", Vector4(rect.position.x / size.x, rect.position.y / size.y, rect.end.x / size.x, rect.end.y / size.y))
	material.set_shader_parameter("outline_color", RING_COLOR if hovered else SettlementSelection.color_for(home))
	outline.show()

func _daily_facing(kind: int, daily: Dictionary, agent_id: int, context: Dictionary, previous: String) -> String:
	var place: Vector2i = daily.get("place", Vector2i.ZERO)
	if kind == 5:
		return "W" if previous == "W" else "E"
	var points: Array = context["hearths"]
	if kind == 1:
		points = context["stockpiles"]
	elif kind == 2:
		points = context["shelters"].keys()
	else:
		var home: int = SettlementSelection.home_for_agent(agent_id)
		var home_tile: Vector2i = SettlementSelection.formation_for(home)
		if home >= 0 and points.has(home_tile):
			points = [home_tile]
	var nearest: Vector2i = place
	var best: float = INF
	for point: Vector2i in points:
		var distance: float = Vector2(place).distance_squared_to(Vector2(point))
		if distance < best or (is_equal_approx(distance, best) and (point.y < nearest.y or (point.y == nearest.y and point.x < nearest.x))):
			nearest = point
			best = distance
	var toward: Vector2 = Vector2(nearest - place)
	return Activity.facing(-toward if kind == 4 else toward, previous)


func set_hovered_person_p12(agent_id: int, owner: int) -> void:
	var previous: int = _hovered_person_p12
	_hovered_person_p12 = agent_id if _sprites.has(agent_id) else -1
	_hover_owner_p12 = owner
	for id: int in [previous, _hovered_person_p12]:
		if _sprites.has(id):
			_apply_settlement_outline(_sprites[id], id, id == _hovered_person_p12)
	_update_link_hover_p12()

func clear_hovered_person_p12(owner: int) -> void:
	if owner == _hover_owner_p12:
		set_hovered_person_p12(-1, 0)

func _update_link_hover_p12() -> void:
	if _hovered_person_p12 < 0 and _hover_marker_p12.is_empty():
		return
	_hover_marker_p12.clear()
	var sprite: Sprite2D = _sprites.get(_hovered_person_p12) as Sprite2D
	if is_instance_valid(sprite) and sprite.visible and _capture_visible:
		_hover_marker_p12 = HoverMarker.geometry(sprite.get_global_transform_with_canvas().origin, get_viewport_rect(), HoverMarker.playable_rect(self))
	queue_redraw()

func _draw() -> void:
	if not _hover_marker_p12.is_empty():
		HoverMarker.draw_marker(self, _hover_marker_p12, RING_COLOR)
