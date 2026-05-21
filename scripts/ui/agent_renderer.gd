extends Node2D

# V7 Phase 4-γ — AgentRenderer (MultiMeshInstance2D driver).
#
# Pulls a per-frame `(ids, xs, ys)` snapshot from `WorldSim` via the new
# `get_agent_snapshot()` FFI and drives a single `MultiMeshInstance2D` to
# draw every agent in one batched call.
#
# Per-instance palette variation is deterministic from `Entity::to_bits()`:
# hash → (hair_col, body_col, skin_col) → normalised Vector4 written to
# `MultiMesh.set_instance_custom_data` and consumed by the modified
# `palette_swap.gdshader` via `INSTANCE_CUSTOM.rgb`.
#
# Coordinate frame: mirrors `world_renderer.gd` so both layers share the
# same `(SPRITE_ORIGIN_X, SPRITE_ORIGIN_Y)` + `TILE_SIZE` mapping. The
# sprite is centred on the tile.

const TILE_SIZE := 16
const SPRITE_ORIGIN_X := 448
const SPRITE_ORIGIN_Y := 28
const SPRITE_W := 64
const SPRITE_H := 72
# Sprite is scaled so the 64-px-wide character fits inside one 16-px
# world tile while keeping its 8:9 aspect ratio. 16/64 = 0.25.
const SPRITE_SCALE := 0.25
const PALETTE_HAIR_COLS := 8
const PALETTE_BODY_COLS := 4
const PALETTE_SKIN_COLS := 8

# V7 Phase 8-δ — memory recall visual indicator.
# When a `memory_recalled` causal event fires for an agent, mark it
# briefly so the renderer can apply a transient visual cue (~0.5–1.0s
# wall-time, scale boost so the cue is visible without a shader edit
# and without extending the AgentSnapshotRow `state_tag` byte —
# preserving the Phase 7-δ A22 contract `state_tag ∈ {0,1,2,3}`).
const RECALL_CUE_FRAMES := 36         # ~0.6s at 60 FPS — within plan's [0.5, 1.0]
const RECALL_CUE_SCALE_BOOST := 1.25  # 25% scale pulse, clearly visible
const RECALL_CUE_TINT := Color(0.45, 0.65, 1.0, 1.0)  # blue cue (for future shader use)

# V7 Phase 9-δ — combat cue visual indicator.
# When a `combat_started` causal event fires for an agent (attacker side),
# mark it briefly with a red-tinted scale pulse (~0.6s) to surface combat
# activity without extending the AgentSnapshotRow `state_tag` byte.
const COMBAT_CUE_FRAMES := 36             # ~0.6s at 60 FPS
const COMBAT_CUE_SCALE_BOOST := 1.3      # 30% scale pulse
const COMBAT_CUE_TINT := Color(1.0, 0.3, 0.3, 1.0)  # red cue (future shader)

# V7 Phase 11-α — position interpolation (Gaffer accumulator) + state_tag tint.
const SIM_TICK_DURATION: float = 1.0 / 30.0  # nominal 30 TPS
# 4-entry tint palette matching the 4 locked state_tag values (0-3):
#   0=Idle, 1=Seeking, 2=Consuming(Agent)=socializing, 3=Consuming(other)
const STATE_TINTS: Array = [
	Color(1.0, 1.0, 1.0, 1.0),   # 0: Idle — white
	Color(1.0, 0.9, 0.2, 1.0),   # 1: Seeking — yellow
	Color(0.9, 0.5, 0.8, 1.0),   # 2: Consuming(Agent)/Socializing — pink
	Color(0.4, 0.9, 0.4, 1.0),   # 3: Consuming(other)/Eating/Building/Sleeping — green
]
var _lerp_accumulator: float = 0.0
var _snapshot_checksum: int = -1
var _prev_positions: Dictionary = {}  # agent_id (int) → Vector2 pixel
var _curr_positions: Dictionary = {}  # agent_id (int) → Vector2 pixel

var world_sim: WorldSimNode
var multi_mesh_inst: MultiMeshInstance2D
var multi_mesh: MultiMesh

# V7 Phase 8-δ (code-attempt 3) — domain note: this dictionary is keyed
# by `AgentId` (the `Agent.id` field surfaced as `agent_ids` in the
# snapshot dict), NOT by `entity_bits`. `CausalEvent::MemoryRecalled.agent`
# is an `AgentId` and the FFI dict serialises it as `"agent_id"`, so the
# producer (`_ingest_memory_recalls` → `mark_agent_recalling(agent_id)`)
# and the consumer (`_process` → `_recalling_agents.has(agent_ids[i])`)
# both use the AgentId domain. Mixing this with `ids[i]` (entity bits)
# would silently break the cue, which is the regression code-attempt 2
# shipped.
var _recalling_agents: Dictionary = {}

# V7 Phase 8-δ — id-based dedupe set for `memory_recalled` causal events
# observed via `WorldSimNode.get_tile_causal_history()`. A given event
# `id` must fire `mark_agent_recalling()` exactly once even though the
# tile's causal ring keeps returning it across frames. Bounded prune at
# `_SEEN_RECALL_MAX` to cap GDScript memory; oldest entries are dropped
# wholesale because causal-event ids are monotonically increasing within
# a session so re-observing an evicted id is unlikely.
const _SEEN_RECALL_MAX := 512
var _seen_recall_event_ids: Dictionary = {}

# V7 Phase 9-δ — combat cue state (mirrors _recalling_agents pattern).
var _combating_agents: Dictionary = {}       # agent_id → frames remaining
const _SEEN_COMBAT_MAX := 512
var _seen_combat_event_ids: Dictionary = {}  # dedupe set

func _ready() -> void:
	print("AgentRenderer ready (V7 Phase 4-γ — MultiMeshInstance2D)")
	world_sim = get_node("../WorldSim") as WorldSimNode
	if world_sim == null:
		push_error("WorldSim node not found at ../WorldSim")
		return

	var quad := QuadMesh.new()
	quad.size = Vector2(SPRITE_W, SPRITE_H)

	multi_mesh = MultiMesh.new()
	multi_mesh.transform_format = MultiMesh.TRANSFORM_2D
	multi_mesh.use_colors = true
	multi_mesh.use_custom_data = true
	multi_mesh.mesh = quad
	multi_mesh.instance_count = 0

	var agent_tex: Texture2D = load("res://assets/sprites/agent_base.png") as Texture2D
	var palette_tex: Texture2D = load("res://assets/sprites/palette_lut.png") as Texture2D
	var shader: Shader = load("res://shaders/palette_swap.gdshader") as Shader

	var mat := ShaderMaterial.new()
	mat.shader = shader
	if palette_tex != null:
		mat.set_shader_parameter("palette_lut", palette_tex)

	multi_mesh_inst = MultiMeshInstance2D.new()
	multi_mesh_inst.multimesh = multi_mesh
	multi_mesh_inst.texture = agent_tex
	multi_mesh_inst.material = mat
	add_child(multi_mesh_inst)

func _process(delta: float) -> void:
	if world_sim == null or multi_mesh == null:
		return
	# V7 Phase 11-α — advance Gaffer accumulator every frame.
	_lerp_accumulator += delta
	# V7 Phase 8-δ — decrement / clear expired recall-cue timers BEFORE
	# building this frame's transforms so an entry that just dropped to 0
	# does not render with the boosted scale on its final frame.
	_clear_expired_recalling()
	_clear_expired_combating()
	var snap: Dictionary = world_sim.get_agent_snapshot()
	var ids: PackedInt64Array = snap.get("ids", PackedInt64Array())
	var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
	# V7 Phase 11-α — state_tag per agent (0-3), used for color tint.
	var states: PackedByteArray = snap.get("states", PackedByteArray())
	# V7 Phase 8-δ (code-attempt 3) — parallel array of `Agent.id` per row.
	# Used as the lookup key for `_recalling_agents` so the renderer's check
	# domain matches the producer's mark domain (both AgentId, not
	# entity_bits). See header comment on `_recalling_agents`.
	var agent_ids: PackedInt64Array = snap.get("agent_ids", PackedInt64Array())
	var n: int = ids.size()
	if xs.size() != n or ys.size() != n or agent_ids.size() != n:
		push_error("AgentRenderer: parallel-array length mismatch")
		return
	# V7 Phase 11-α — detect snapshot tick boundary by checksum.
	# If positions changed, promote curr→prev, rebuild curr, reset accumulator.
	# Identity (`agent_ids`) participates in the hash so a snapshot whose row
	# order changes — or whose agent set changes — flips the checksum even
	# when raw position bytes are unchanged. See A19 in the harness.
	var new_checksum: int = _snapshot_checksum_from(agent_ids, xs, ys, n)
	if new_checksum != _snapshot_checksum:
		_snapshot_checksum = new_checksum
		_prev_positions = _curr_positions.duplicate()
		_curr_positions.clear()
		for i in n:
			var tile_x: int = xs[i]
			var tile_y: int = ys[i]
			var cpx: float = float(SPRITE_ORIGIN_X + tile_x * TILE_SIZE + TILE_SIZE / 2)
			var cpy: float = float(SPRITE_ORIGIN_Y + tile_y * TILE_SIZE + TILE_SIZE / 2)
			_curr_positions[int(agent_ids[i])] = Vector2(cpx, cpy)
		_lerp_accumulator = 0.0
	var lerp_t: float = clampf(_lerp_accumulator / SIM_TICK_DURATION, 0.0, 1.0)
	if multi_mesh.instance_count != n:
		multi_mesh.instance_count = n
	for i in n:
		var agent_id: int = int(agent_ids[i])
		var curr_pos: Vector2 = _curr_positions.get(agent_id, Vector2.ZERO)
		# V7 Phase 11-α — interpolate between prev and curr pixel position.
		var prev_pos: Vector2 = _prev_positions.get(agent_id, curr_pos)
		var ipos: Vector2 = prev_pos.lerp(curr_pos, lerp_t)
		var px: float = ipos.x
		var py: float = ipos.y
		# V7 Phase 8-δ — boost the per-instance scale by RECALL_CUE_SCALE_BOOST
		# while the agent is in the recall window. Visible without a shader
		# change. NOT tied to `state_tag` so Phase 7-δ A22 stays intact.
		# Lookup keyed by `agent_ids[i]` (AgentId) so it matches the FFI
		# `event.agent_id` value used by `_ingest_memory_recalls`.
		var boost: float = 1.0
		if _recalling_agents.has(agent_ids[i]):
			boost = max(boost, RECALL_CUE_SCALE_BOOST)
		if _combating_agents.has(agent_ids[i]):
			boost = max(boost, COMBAT_CUE_SCALE_BOOST)
		var scale_mul: float = SPRITE_SCALE * boost
		var xform := Transform2D(0.0, Vector2(scale_mul, scale_mul), 0.0, Vector2(px, py))
		multi_mesh.set_instance_transform_2d(i, xform)
		multi_mesh.set_instance_custom_data(i, _palette_for_id(ids[i]))
		# V7 Phase 11-α — apply state_tag color tint via instance color.
		var tag: int = clampi(int(states[i]) if i < states.size() else 0, 0, 3)
		multi_mesh.set_instance_color(i, STATE_TINTS[tag])
	# V7 Phase 8-δ — after rendering this frame, ingest fresh
	# `memory_recalled` events from the causal log so the next frame can
	# pulse the cue. We poll each unique tile occupied by an agent because
	# WorldSimNode does not (yet) emit a Godot signal for causal events;
	# the tile causal history is the canonical source. Dedupe by event id
	# so a single recall fires `mark_agent_recalling()` exactly once.
	_ingest_memory_recalls(ids, xs, ys, n)
	# V7 Phase 9-δ — ingest combat events after memory recalls so both cues
	# are updated in the same frame pass.
	_ingest_combat_events(ids, agent_ids, xs, ys, n)

# V7 Phase 8-δ — public API: an external dispatcher (currently
# `_ingest_memory_recalls` reading `memory_recalled` causal events, or
# any future signal-based dispatcher) calls this once when a recall
# fires for a given agent. The timer is overwritten if it was already
# active, so back-to-back recalls extend the cue rather than ending it
# early.
#
# Domain: `agent_id` is the `AgentId` (the `Agent.id` field, surfaced as
# `agent_ids` in the snapshot dict and as `agent_id` in the FFI causal
# event dict). NOT `entity_bits`. Mixing the two domains was the
# regression code-attempt 2 shipped.
func mark_agent_recalling(agent_id: int) -> void:
	_recalling_agents[agent_id] = RECALL_CUE_FRAMES

# V7 Phase 8-δ — read-only accessor used by tests and the SimulationBus
# relay to confirm a cue is active. Returns 0 when the agent has no
# active recall cue. Domain: `agent_id` is `AgentId`, see
# `mark_agent_recalling`.
func recall_cue_remaining(agent_id: int) -> int:
	return int(_recalling_agents.get(agent_id, 0))

# V7 Phase 8-δ — poll the tile causal log for each unique agent tile
# this frame. Any `memory_recalled` event whose id we have not yet seen
# is dispatched via `mark_agent_recalling()` for the event's `agent_id`.
# The dedupe set is bounded so a long session does not grow GDScript
# memory without limit.
func _ingest_memory_recalls(ids: PackedInt64Array, xs: PackedInt32Array, ys: PackedInt32Array, n: int) -> void:
	if world_sim == null:
		return
	if not world_sim.has_method("get_tile_causal_history"):
		return
	# Collect unique tile coordinates so we never query the same tile twice
	# per frame. With 10K agents and few colocated tiles this stays small.
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
			# First sighting of this recall — register the dedupe key and
			# fire the cue for the recalling agent.
			_seen_recall_event_ids[event_id] = true
			var agent_id: int = int(dev.get("agent_id", -1))
			if agent_id >= 0:
				# `agent_id` is `Entity::to_bits().get()` — the same key
				# used by `_recalling_agents` / `mark_agent_recalling()`.
				mark_agent_recalling(agent_id)
	# Bounded prune: when the dedupe set grows past _SEEN_RECALL_MAX we
	# clear it wholesale. Causal-event ids are monotonically increasing in
	# a session so any reused id post-clear is statistically unlikely.
	if _seen_recall_event_ids.size() > _SEEN_RECALL_MAX:
		_seen_recall_event_ids.clear()

func _clear_expired_recalling() -> void:
	# Decrement every active recall-cue timer; entries that hit zero are
	# erased so the boost scale stops being applied on the next frame.
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

# V7 Phase 9-δ — public API: mark attacker agent as in-combat for one cue
# window. Domain: `agent_id` is `AgentId` (same as `_recalling_agents`).
#
# Naming: `mark_agent_in_combat` is the primary public API the plan
# requires (mirrors `mark_agent_recalling`). `mark_agent_combating` is
# kept as a compatibility alias for any earlier callsites that may still
# reference the older spelling.
func mark_agent_in_combat(agent_id: int) -> void:
	_combating_agents[agent_id] = COMBAT_CUE_FRAMES

# V7 Phase 9-δ — backwards-compatible alias for `mark_agent_in_combat`.
# Forwards to the primary API so any older caller continues to work.
func mark_agent_combating(agent_id: int) -> void:
	mark_agent_in_combat(agent_id)

# V7 Phase 9-δ — read-only accessor; returns 0 when no active combat cue.
func combat_cue_remaining(agent_id: int) -> int:
	return int(_combating_agents.get(agent_id, 0))

# V7 Phase 9-δ — poll the tile causal log for `combat_started` events and
# dispatch `mark_agent_combating()` for the attacker (`agent_id` field).
# Mirrors `_ingest_memory_recalls` exactly.
func _ingest_combat_events(ids: PackedInt64Array, agent_ids: PackedInt64Array, xs: PackedInt32Array, ys: PackedInt32Array, n: int) -> void:
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
	# V7 Phase 11-α (code-attempt 3) — identity-aware tick-boundary checksum.
	#
	# Iterates EVERY row (no `mini(n, 32)` truncation) so a position change
	# on agent #33+ still flips the hash and triggers the interpolation
	# tick boundary. Folds `agent_ids[i]` (identity) and `i` (row index)
	# along with `xs[i]` / `ys[i]` (position) into a multiplicative
	# accumulator so the hash is sensitive to:
	#   • per-agent position changes (movement)
	#   • row-order changes (would cancel under pure XOR)
	#   • agent identity changes (entity set churn)
	#
	# `h * 1000003` (a 32-bit prime) advances the accumulator non-
	# commutatively, defeating the order-insensitive XOR pattern that the
	# previous version inherited. Knuth's φ-derived multiplier
	# `2654435761` plus three other large primes spread per-row bits across
	# the 64-bit accumulator before folding into `h`.
	if n == 0:
		return 0
	var h: int = n * 2654435761
	for i in n:
		var aid: int = int(agent_ids[i])
		h = (h * 1000003) ^ (aid * 73856093) ^ (int(xs[i]) * 19349663) ^ (int(ys[i]) * 83492791) ^ (i * 2654435761)
	return h

func _palette_for_id(eid: int) -> Color:
	# Hash splat — three independent multipliers give visually distinct
	# columns across the 8/4/8 palette layout. Stable per agent within a
	# session because `eid` is `Entity::to_bits().get()`.
	var h: int = absi(eid * 2654435761) % PALETTE_HAIR_COLS
	var b: int = absi(eid * 40503) % PALETTE_BODY_COLS
	var s: int = absi(eid * 2246822519) % PALETTE_SKIN_COLS
	return Color(
		float(h) / float(PALETTE_HAIR_COLS - 1),
		float(b) / float(PALETTE_BODY_COLS - 1),
		float(s) / float(PALETTE_SKIN_COLS - 1),
		0.0
	)
