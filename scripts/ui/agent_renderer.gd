extends Node2D

const ArtHashScript = preload("res://scripts/ui/art_hash.gd")
const Cold = preload("res://scripts/ui/cold_effects.gd")
const MorningChill = preload("res://scripts/ui/morning_chill.gd")
const CarryLoad = preload("res://scripts/ui/carry_load_layer.gd")
const Activity = preload("res://scripts/ui/agent_activity.gd")
const SpriteLibrary = preload("res://scripts/ui/art_sprite_library.gd")
const CampSeatsScript = preload("res://scripts/ui/camp_seats.gd")
const PoseMoments = preload("res://scripts/ui/agent_pose_moments.gd")
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

func _ready() -> void:
	_objects = SpriteLibrary.shared_layer(self, "WorldObjects", 4, true)
	_shadow_layer = SpriteLibrary.shared_layer(self, "Shadows", 3)
	world_sim = get_node_or_null("../WorldSim") as WorldSimNode
	if world_sim != null:
		world_sim.connect("world_reset", _on_world_reset)

func _process(delta: float) -> void:
	if world_sim == null:
		return
	var world_info: Dictionary = world_sim.get_world_info()
	_move_duration = float(maxi(1, int(world_info.get("move_interval_ticks", 1)))) * SIM_TICK_DURATION
	_lerp_accumulator += delta
	_animation_time += delta
	_clear_expired_recalling()
	_clear_expired_combating()
	var snap: Dictionary = world_sim.get_agent_snapshot()
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
			var look: Dictionary = ArtHashScript.look(int(agent_ids[i]))
			_curr_positions[int(agent_ids[i])] = Vector2(SPRITE_ORIGIN_X + xs[i] * TILE_SIZE + 8 + int(look["jx"]), SPRITE_ORIGIN_Y + ys[i] * TILE_SIZE + 14 + int(look["jy"]))
		for old_id: int in _prev_positions.keys():
			if not _curr_positions.has(old_id):
				_prev_positions.erase(old_id)
		_lerp_accumulator = 0.0
	var lerp_t: float = clampf(_lerp_accumulator / _move_duration, 0.0, 1.0)
	var live: Dictionary = {}
	var context: Dictionary = _activity_context()
	var time: Dictionary = world_sim.get_time_snapshot()
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
		var look: Dictionary = ArtHashScript.look(agent_id)
		var life_stage: int = int(life_stages[i]) if i < life_stages.size() else 0
		look = Activity.elder_look(look, agent_id, life_stage)
		var child: bool = life_stage == 1
		var previous_position: Vector2 = _render_positions.get(agent_id, ipos)
		var movement: Vector2 = ipos - previous_position
		var moving: bool = movement.length_squared() > 0.000001
		_stationary_seconds[agent_id] = 0.0 if moving else float(_stationary_seconds.get(agent_id, 0.0)) + delta
		_render_positions[agent_id] = ipos
		var tile: Dictionary = _tile_context(Vector2i(xs[i], ys[i]), context)
		var state: int = int(states[i]) if i < states.size() else 0
		var target: int = Activity.snapshot_target(snap, i)
		var has_targets: bool = target >= 0
		var carried: float = float(snap["carried_foods"][i]) if snap.has("carried_foods") and i < snap["carried_foods"].size() else 0.0
		var wood: int = Activity.snapshot_wood(snap, i)
		var moments: Dictionary = _pose_moments.update(agent_id, i, snap, Vector2i(xs[i], ys[i]), not moving, bool(tile.get("wood", false)), context["hearths"], delta)
		var activity: String = Activity.resolve(state, target, has_targets, movement.length_squared() > 0.000001, carried, tile, hour, wood)
		if bool(moments["pickup"]):
			activity = "gather_wood"
		_idle_seconds[agent_id] = Activity.idle_elapsed(activity, movement, float(_idle_seconds.get(agent_id, 0.0)), delta)
		var partner_delta: Vector2 = _talk_delta(i, xs, ys) if activity == "talk" else Vector2.ZERO
		var facing: String = Activity.render_facing(activity, movement, partner_delta, str(sprite.get_meta("facing", "S")), float(_idle_seconds[agent_id]))
		var pose: String = Activity.camp_pose(activity, life_stage, agent_id, int(time.get("day_index", 0)), hour, bool(tile.get("near_camp", false)))
		if bool(moments["meal"]):
			pose = "eat"
		var cold: bool = Cold.snapshot_cold(snap, i)
		var base_pose: String = pose
		pose = Cold.pose(pose, cold or _morning_chill.alpha(agent_id, tick) > 0.0, moving)
		var body: String = "child" if child else ("adult_a" if int(look["body"]) == 0 else "adult_b")
		var choice: Dictionary = frame_choice(body, pose, facing, float(look["phase"]))
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
		sprite.set_meta("entity_bits", int(ids[i]))
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
		_update_activity_icon(agent_id, sprite, choice)
		draw_agents.append({"id": agent_id, "ipos": ipos, "tile": Vector2i(xs[i], ys[i]), "visible": sprite.visible, "moving": moving, "stationary_seconds": _stationary_seconds[agent_id], "pose": pose, "activity": activity, "cold": cold, "meal": bool(moments["meal"])})
	_apply_drawn_positions(draw_agents, context, delta)
	for row: Dictionary in draw_agents:
		var agent_id: int = int(row["id"])
		var sprite: Sprite2D = _sprites[agent_id]
		Cold.apply(sprite, bool(row.get("cold", false)), bool(row["moving"]), agent_id, _animation_time)
		_morning_chill.apply(sprite, agent_id, tick, _animation_time)
		sprite.set_meta("drawn_position", sprite.position)
		var shadow: Sprite2D = _shadows[agent_id]
		shadow.position = sprite.position
		shadow.visible = sprite.visible
		_update_cue(agent_id, sprite.position, _recalling_agents.has(agent_id), _combating_agents.has(agent_id))
	_retire_missing(live)
	_update_selection_ring()
	_ingest_memory_recalls(ids, xs, ys, n)
	_ingest_combat_events(ids, agent_ids, xs, ys, n)

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

func refresh_snapshot() -> void:
	_prev_positions.clear()
	_curr_positions.clear()
	_snapshot_checksum = -1
	_process(0.0)

func head_icon_position(world_pos: Vector2) -> Vector2:
	return world_pos + ICON_OFFSET_PX

func _on_world_reset() -> void:
	_morning_chill.clear()
	_render_positions.clear()
	_drawn_positions.clear()
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
	_retire_missing({})
	_process(0.0)

func mark_agent_recalling(agent_id: int) -> void:
	_recalling_agents[agent_id] = RECALL_CUE_FRAMES

func recall_cue_remaining(agent_id: int) -> int:
	return int(_recalling_agents.get(agent_id, 0))

func _ingest_memory_recalls(_ids: PackedInt64Array, xs: PackedInt32Array, ys: PackedInt32Array, n: int) -> void:
	if world_sim == null:
		return
	if not world_sim.has_method("get_tile_causal_history"):
		return
	var visited_tiles: Dictionary = {}
	for i in n:
		var key: int = (int(xs[i]) << 20) | int(ys[i])
		if visited_tiles.has(key):
			continue
		visited_tiles[key] = true
		var history: Array = world_sim.get_tile_causal_history(int(xs[i]), int(ys[i]))
		for ev in history:
			if not (ev is Dictionary):
				continue
			var dev: Dictionary = ev
			if String(dev.get("kind", "")) != "memory_recalled":
				continue
			var event_id: int = int(dev.get("id", -1))
			if event_id < 0:
				continue
			if _seen_recall_event_ids.has(event_id):
				continue
			_seen_recall_event_ids[event_id] = true
			var agent_id: int = int(dev.get("agent_id", -1))
			if agent_id >= 0:
				mark_agent_recalling(agent_id)
	if _seen_recall_event_ids.size() > _SEEN_RECALL_MAX:
		_seen_recall_event_ids.clear()

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

func _ingest_combat_events(_ids: PackedInt64Array, _agent_ids: PackedInt64Array, xs: PackedInt32Array, ys: PackedInt32Array, n: int) -> void:
	if world_sim == null:
		return
	if not world_sim.has_method("get_tile_causal_history"):
		return
	var visited_tiles: Dictionary = {}
	for i in n:
		var key: int = (int(xs[i]) << 20) | int(ys[i])
		if visited_tiles.has(key):
			continue
		visited_tiles[key] = true
		var history: Array = world_sim.get_tile_causal_history(int(xs[i]), int(ys[i]))
		for ev in history:
			if not (ev is Dictionary):
				continue
			var dev: Dictionary = ev
			if String(dev.get("kind", "")) != "combat_started":
				continue
			var event_id: int = int(dev.get("id", -1))
			if event_id < 0:
				continue
			if _seen_combat_event_ids.has(event_id):
				continue
			_seen_combat_event_ids[event_id] = true
			var agent_id: int = int(dev.get("agent_id", -1))
			if agent_id >= 0:
				mark_agent_in_combat(agent_id)
	if _seen_combat_event_ids.size() > _SEEN_COMBAT_MAX:
		_seen_combat_event_ids.clear()

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

func frame_choice(body: String, activity: String, facing: String, phase: float) -> Dictionary:
	if _frame_groups.is_empty():
		_frame_groups = Activity.build_frame_groups(SpriteLibrary.entries())
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
	var context: Dictionary = {"shelters": {}, "hearths": [], "sites": {}, "food": {}, "wood": {}, "water": {}, "seat_blocked": {}}
	var buildings: Dictionary = world_sim.get_building_snapshot()
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
	for source: String in ["sites", "food"]:
		var snap: Dictionary = world_sim.get_construction_snapshot() if source == "sites" else world_sim.get_resource_snapshot()
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
	var terrain: Dictionary = world_sim.get_terrain_snapshot()
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
	var declump: Dictionary = CampSeatsScript.declump(agents, seats, targets, terrain, blocked, rects)
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
		sprite.set_meta("seated", seats.has(agent_id))
		if seats.has(agent_id):
			sprite.set_meta("seat", seats[agent_id])
		elif sprite.has_meta("seat"):
			sprite.remove_meta("seat")
		sprite.set_meta("drawn_target", target)
		sprite.set_meta("spread_offset", spread.get(agent_id, Vector2.ZERO))
		sprite.set_meta("declump_offset", declump.get(agent_id, Vector2.ZERO))

func has_drawn_position(agent_id: int) -> bool:
	return _drawn_positions.has(agent_id)

func get_drawn_position(agent_id: int) -> Vector2:
	if _sprites.has(agent_id):
		return (_sprites[agent_id] as Sprite2D).position
	return Vector2(_drawn_positions.get(agent_id, Vector2.ZERO)).round()

func drawn_entity_at(world_position: Vector2) -> int:
	var selected: int = -1
	var largest_y: float = -INF
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
		if opaque.has_point(sprite.to_local(world_position)) and sprite.position.y >= largest_y:
			largest_y = sprite.position.y
			selected = int(sprite.get_meta("entity_bits", -1))
	return selected

func _update_selection_ring() -> void:
	if _selection_ring == null:
		_selection_ring = Line2D.new()
		_selection_ring.name = "SelectionRing"
		_selection_ring.width = 1.0
		_selection_ring.default_color = Color("#D7BD77")
		_selection_ring.set_meta("art_kind", "selection_ring")
		_shadow_layer.add_child(_selection_ring)
	_selection_ring.visible = false
	var inspector: Node = get_node_or_null("../UI/AgentInspectorPanel")
	if inspector == null or not _capture_visible:
		return
	var selected: int = int(inspector.call("get_selected_entity_bits"))
	for sprite: Sprite2D in _sprites.values():
		if int(sprite.get_meta("entity_bits", -1)) != selected or not sprite.visible:
			continue
		var pulse: int = posmod(floori(_animation_time * 2.0), 2)
		var extent: float = float(7 + pulse)
		_selection_ring.points = PackedVector2Array([Vector2(-extent, -1), Vector2(-extent + 2, -3), Vector2(extent - 2, -3), Vector2(extent, -1), Vector2(extent, 1), Vector2(extent - 2, 3), Vector2(-extent + 2, 3), Vector2(-extent, 1), Vector2(-extent, -1)])
		_selection_ring.position = sprite.position
		_selection_ring.set_meta("art_id", int(sprite.get_meta("art_id")))
		_selection_ring.set_meta("frame_id", pulse)
		_selection_ring.visible = true
		return

func get_active_agent_sprites() -> Array[Sprite2D]:
	var active: Array[Sprite2D] = []
	for sprite: Sprite2D in _sprites.values():
		active.append(sprite)
	return active

func is_capture_visible() -> bool:
	return _capture_visible
