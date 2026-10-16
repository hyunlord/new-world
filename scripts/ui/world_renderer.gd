extends Node2D

# Influence overlay and click inspection only. All visible world objects come from snapshots.
const TILE_SIZE: int = 16
const GRID_W: int = 64
const GRID_H: int = 64
const SPRITE_ORIGIN_X := 448
const SPRITE_ORIGIN_Y := 28
const CHANNEL_OFF := -1
const CHANNEL_WARMTH := 0
const CHANNEL_LIGHT := 1
const CHANNEL_NOISE := 2
const CHANNEL_DANGER := 4
const CHANNEL_SPIRITUAL := 6
const CHANNEL_BEAUTY := 7
const OVERLAY_ALPHA: float = 0.65
const Z_OVERLAY: int = 10
const CLICK_RADIUS_WORLD_PX: float = float(TILE_SIZE)

var current_channel: int = CHANNEL_OFF
var world_sim: WorldSimNode
var sprite: Sprite2D
var texture: ImageTexture
var image: Image

func _ready() -> void:
	print("WorldRenderer ready (T7.9.B render mechanism)")
	world_sim = get_node("../WorldSim") as WorldSimNode
	image = Image.create(GRID_W, GRID_H, false, Image.FORMAT_L8)
	texture = ImageTexture.create_from_image(image)
	sprite = Sprite2D.new()
	sprite.texture = texture
	sprite.scale = Vector2(TILE_SIZE, TILE_SIZE)
	sprite.position = Vector2(960, 540)
	sprite.z_index = Z_OVERLAY
	sprite.modulate = Color(1.0, 1.0, 1.0, OVERLAY_ALPHA)
	sprite.visible = false
	add_child(sprite)
	world_sim.connect("world_reset", refresh_snapshot)

func refresh_snapshot() -> void:
	_process(0.0)

func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo:
		if event.keycode == KEY_SPACE:
			# fix-overlay-darkness: 7-state cycle including OFF —
			# OFF → Warmth → Light → Noise → Danger → Spiritual → Beauty → OFF.
			# OFF is both enterable (Beauty→OFF wrap) and exitable (OFF→Warmth
			# entry) so the overlay can always be re-hidden; a one-way OFF would
			# trap the user on a clean screen with no way back to analysis.
			var channel_name: String
			if current_channel == CHANNEL_OFF:
				current_channel = CHANNEL_WARMTH
				channel_name = "Warmth"
			elif current_channel == CHANNEL_WARMTH:
				current_channel = CHANNEL_LIGHT
				channel_name = "Light"
			elif current_channel == CHANNEL_LIGHT:
				current_channel = CHANNEL_NOISE
				channel_name = "Noise"
			elif current_channel == CHANNEL_NOISE:
				current_channel = CHANNEL_DANGER
				channel_name = "Danger"
			elif current_channel == CHANNEL_DANGER:
				current_channel = CHANNEL_SPIRITUAL
				channel_name = "Spiritual"
			elif current_channel == CHANNEL_SPIRITUAL:
				current_channel = CHANNEL_BEAUTY
				channel_name = "Beauty"
			else:
				current_channel = CHANNEL_OFF
				channel_name = "Off"
			print("Channel switched: ", channel_name)
	elif event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
		_handle_tile_click(get_global_mouse_position())

func _process(_delta: float) -> void:
	if world_sim == null:
		return
	if current_channel == CHANNEL_OFF:
		sprite.visible = false
	else:
		sprite.visible = true
		var data: PackedByteArray = world_sim.get_influence_overlay(current_channel)
		if data.size() == GRID_W * GRID_H:
			image = Image.create_from_data(GRID_W, GRID_H, false, Image.FORMAT_L8, data)
			texture.update(image)

func _handle_tile_click(pos: Vector2) -> void:
	# V7 Phase 14-γ — agent probe takes priority over the tile dispatch.
	# When an agent is within CLICK_RADIUS_WORLD_PX of the click, the
	# inspector panel is shown and the tile causal-history dispatch is
	# skipped (mutually exclusive — see Assertion 22).
	if _try_agent_click(pos):
		return
	# Phase 12-β.2 tile causal-history fallback — preserved bounds check
	# and dispatch when no agent is within radius.
	var tile_x: int = int(floor((pos.x - SPRITE_ORIGIN_X) / float(TILE_SIZE)))
	var tile_y: int = int(floor((pos.y - SPRITE_ORIGIN_Y) / float(TILE_SIZE)))
	if tile_x < 0 or tile_x >= GRID_W or tile_y < 0 or tile_y >= GRID_H:
		return
	_fetch_causal_history(tile_x, tile_y)


# V7 Phase 14-γ — agent click probe.
#
# First selects the topmost drawn opaque villager rectangle, then falls
# back to the most recent agent snapshot (parallel arrays of
# entity_bits / xs / ys) and finds the agent whose tile centre is closest
# to the mouse world coords, within CLICK_RADIUS_WORLD_PX. On a hit,
# queries `get_agent_detail` for the 8-field row and forwards it to the
# AgentInspectorPanel via `display_agent`.
#
# Returns true iff an agent was found AND the inspector accepted the
# call (i.e. `found == true` in the detail dict). On a miss, the caller
# falls through to the tile causal-history dispatch.
#
# Tile-centre ties are resolved by lower snapshot iteration index because the
# comparison is strict less-than: the first candidate to set `best_idx`
# at a given distance wins (Assertion 21).
func _try_agent_click(world_pos: Vector2) -> bool:
	if world_sim == null:
		return false
	var agents: Node = get_node_or_null("../AgentRenderer")
	if agents != null:
		var drawn_entity: int = int(agents.call("drawn_entity_at", world_pos))
		if drawn_entity >= 0:
			var drawn_detail: Dictionary = world_sim.get_agent_detail(drawn_entity)
			var drawn_panel: Node = get_node_or_null("/root/Main/UI/AgentInspectorPanel")
			if bool(drawn_detail.get("found", false)) and drawn_panel != null and drawn_panel.has_method("display_agent"):
				drawn_panel.call("display_agent", drawn_detail)
				return true
	var snap: Dictionary = SimReads.read(world_sim, "get_agent_snapshot")
	var ids: PackedInt64Array = snap.get("ids", PackedInt64Array())
	var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
	var n: int = ids.size()
	if n == 0:
		return false
	# V7 Phase 14-γ — split cutoff (inclusive) from closer-wins (strict).
	# A click exactly at CLICK_RADIUS_WORLD_PX (d2 == max_dist2) is
	# INCLUDED via the `>` skip; inside the loop, strict `<` preserves
	# closer-wins behaviour, and ties (d2 == best_dist2) leave best_idx
	# unchanged so the first (lowest-index) match wins.
	var best_idx: int = -1
	var best_dist2: float = 0.0
	var max_dist2: float = CLICK_RADIUS_WORLD_PX * CLICK_RADIUS_WORLD_PX
	for i in n:
		var cpx: float = float(SPRITE_ORIGIN_X + xs[i] * TILE_SIZE) + float(TILE_SIZE) / 2.0
		var cpy: float = float(SPRITE_ORIGIN_Y + ys[i] * TILE_SIZE) + float(TILE_SIZE) / 2.0
		var dx: float = cpx - world_pos.x
		var dy: float = cpy - world_pos.y
		var d2: float = dx * dx + dy * dy
		if d2 > max_dist2:
			continue
		if best_idx < 0 or d2 < best_dist2:
			best_dist2 = d2
			best_idx = i
	if best_idx < 0:
		return false
	var detail: Dictionary = world_sim.get_agent_detail(int(ids[best_idx]))
	if not bool(detail.get("found", false)):
		return false
	var panel: Node = get_node_or_null("/root/Main/UI/AgentInspectorPanel")
	if panel != null and panel.has_method("display_agent"):
		panel.call("display_agent", detail)
		return true
	return false

func _fetch_causal_history(tx: int, ty: int) -> void:
	if world_sim == null:
		return
	var history: Array = world_sim.get_tile_causal_history(tx, ty)
	var panel: Node = get_node_or_null("/root/Main/UI/CausalPanel")
	if panel != null and panel.has_method("display_history"):
		panel.call("display_history", history, tx, ty)
		panel.visible = true
