extends Node2D

const ArtHashScript = preload("res://scripts/ui/art_hash.gd")
const SpriteLibrary = preload("res://scripts/ui/art_sprite_library.gd")
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
const LOOKS: Array[String] = ["direction_09_villager_gatherer", "direction_10_villager_builder", "direction_11_villager_elder"]
var world_sim: WorldSimNode
var _objects: Node2D
var _shadow_layer: Node2D
var _sprites: Dictionary = {}
var _shadows: Dictionary = {}
var _cues: Dictionary = {}
var _sprite_pool: Array[Sprite2D] = []
var _shadow_pool: Array[Sprite2D] = []
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
	_clear_expired_recalling()
	_clear_expired_combating()
	var snap: Dictionary = world_sim.get_agent_snapshot()
	var ids: PackedInt64Array = snap.get("ids", PackedInt64Array())
	var agent_ids: PackedInt64Array = snap.get("agent_ids", PackedInt64Array())
	var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
	var states: PackedByteArray = snap.get("states", PackedByteArray())
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
	for i in n:
		var agent_id: int = int(agent_ids[i])
		live[agent_id] = true
		var curr_pos: Vector2 = _curr_positions.get(agent_id, Vector2.ZERO)
		var prev_pos: Vector2 = _prev_positions.get(agent_id, curr_pos)
		var ipos: Vector2 = prev_pos.lerp(curr_pos, lerp_t)
		var sprite: Sprite2D = _ensure_agent(agent_id)
		var look: Dictionary = ArtHashScript.look(agent_id)
		var child: bool = snap.has("life_stages") and i < snap["life_stages"].size() and int(snap["life_stages"][i]) == 1
		var sprite_id: String = "direction_12_villager_child" if child else LOOKS[posmod(ArtHashScript.mix(agent_id), LOOKS.size())]
		if str(sprite.get_meta("sprite_id", "")) != sprite_id:
			SpriteLibrary.apply(sprite, sprite_id)
		sprite.position = ipos.round()
		if not is_equal_approx(curr_pos.x, prev_pos.x):
			sprite.flip_h = curr_pos.x < prev_pos.x
		sprite.visible = _capture_visible
		sprite.set_meta("entity_bits", int(ids[i]))
		sprite.set_meta("body", "child" if child else ("adult_a" if int(look["body"]) == 0 else "adult_b"))
		sprite.set_meta("variant", {"skin": look["skin"], "hair": look["hair"], "clothing": look["clothing"]})
		sprite.set_meta("tile_x", int(xs[i]))
		sprite.set_meta("tile_y", int(ys[i]))
		sprite.set_meta("activity", "walk" if i < states.size() and states[i] == 1 else "idle")
		var shadow: Sprite2D = _shadows[agent_id]
		shadow.position = sprite.position
		shadow.visible = sprite.visible
		_update_cue(agent_id, sprite.position, _recalling_agents.has(agent_ids[i]), _combating_agents.has(agent_ids[i]))
	_retire_missing(live)
	_ingest_memory_recalls(ids, xs, ys, n)
	_ingest_combat_events(ids, agent_ids, xs, ys, n)

func _ensure_agent(agent_id: int) -> Sprite2D:
	if _sprites.has(agent_id):
		return _sprites[agent_id] as Sprite2D
	var sprite: Sprite2D = _sprite_pool.pop_back() if not _sprite_pool.is_empty() else Sprite2D.new()
	if sprite.get_parent() == null:
		_objects.add_child(sprite)
	sprite.name = "Agent_%d" % agent_id
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

func refresh_snapshot() -> void:
	_prev_positions.clear()
	_curr_positions.clear()
	_snapshot_checksum = -1
	_process(0.0)

func head_icon_position(world_pos: Vector2) -> Vector2:
	return world_pos + ICON_OFFSET_PX

func _on_world_reset() -> void:
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
