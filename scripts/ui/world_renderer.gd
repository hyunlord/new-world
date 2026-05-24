extends Node2D

# WorldRenderer — Node2D child of Main scene.
# T7.9.B render mechanism: pulls an influence channel overlay from WorldSim
# and uploads it to a Sprite2D texture every frame.
# T7.10.B1: SPACE toggles between Warmth (T7.10.A) and Light (T7.10.B) channels.
# T7.10.C: cycle extended to include Noise (linear-decay) — SPACE cycles
# Warmth → Light → Noise → Warmth so all three backend wirings can be
# confirmed visually in one F6 session.
# T7.10.D: cycle extended to include Danger (linear-decay alpha=5, cap=15) —
# SPACE cycles Warmth → Light → Noise → Danger → Warmth so all four backend
# wirings can be confirmed visually in one F6 session.
# T7.10.E: cycle extended to include Spiritual (BFS exponential k=0.08) —
# SPACE cycles Warmth → Light → Noise → Danger → Spiritual → Warmth so all
# five backend wirings can be confirmed visually in one F6 session.
# T7.10.F: cycle extended to include Beauty (BFS exponential k=0.12) —
# SPACE cycles Warmth → Light → Noise → Danger → Spiritual → Beauty → Warmth
# so all six stamped-channel backend wirings can be confirmed visually in
# one F6 session. T7.10.F completes the Phase 2 stamped-channel
# dispatch-shell escape (6/6 stamped channels wired).
#
# Bootstrap: places one building at (32, 32) radius 8 so the BuildingStamp
# system has something to drive. Initial channel = Warmth.

const TILE_SIZE := 16
const GRID_W := 64
const GRID_H := 64
const CHANNEL_WARMTH := 0
const CHANNEL_LIGHT := 1
const CHANNEL_NOISE := 2
const CHANNEL_DANGER := 4
const CHANNEL_SPIRITUAL := 6
const CHANNEL_BEAUTY := 7
const BOOTSTRAP_X := 32
const BOOTSTRAP_Y := 32
const BOOTSTRAP_RADIUS := 8
const SPRITE_ORIGIN_X := 448
const SPRITE_ORIGIN_Y := 28

# V7 Phase 12-β — TileMapLayer floor terrain + bootstrap building sprite +
# influence-overlay z/alpha layering. Section 1 scope: visible-delta
# replacement of the pre-β solid-black overlay backdrop with a tiled floor
# under one bootstrap building, with the influence overlay preserved on top
# at reduced alpha. Walls + multi-building rendering deferred to β.2.
const TERRAIN_TILESET_PATH := "res://assets/tilesets/world_terrain.tres"
const BUILDING_SPRITE_PATH := "res://assets/sprites/buildings/cairn/1.png"
const TERRAIN_SEED := 19349663  # deterministic seed for reproducible terrain
const OVERLAY_ALPHA := 0.65
const Z_TERRAIN := 0
const Z_BUILDING := 5
const Z_OVERLAY := 10
const TERRAIN_SOURCE_COUNT := 9  # 3 materials × 3 variants

var current_channel: int = CHANNEL_WARMTH
var world_sim: WorldSimNode
var sprite: Sprite2D
var texture: ImageTexture
var image: Image

func _ready() -> void:
	print("WorldRenderer ready (T7.9.B render mechanism)")
	world_sim = get_node("../WorldSim") as WorldSimNode
	if world_sim == null:
		push_error("WorldSim node not found at ../WorldSim")
		return
	world_sim.on_building_placed(BOOTSTRAP_X, BOOTSTRAP_Y, BOOTSTRAP_RADIUS)
	image = Image.create(GRID_W, GRID_H, false, Image.FORMAT_L8)
	texture = ImageTexture.create_from_image(image)
	sprite = Sprite2D.new()
	sprite.texture = texture
	sprite.scale = Vector2(TILE_SIZE, TILE_SIZE)
	sprite.position = Vector2(960, 540)
	add_child(sprite)
	# V7 Phase 12-β — overlay layer above terrain + building, semi-transparent
	# so the new terrain layer shows through. Existing `sprite` is the
	# influence overlay; we only touch z_index + modulate alpha here.
	sprite.z_index = Z_OVERLAY
	sprite.modulate = Color(1.0, 1.0, 1.0, OVERLAY_ALPHA)

	# V7 Phase 12-β — TileMapLayer floor terrain.
	# Loads the new world_terrain TileSet (9 atlas sources = 3 materials ×
	# 3 variants) and populates every cell of the 64×64 grid with a
	# deterministically-chosen source. Seeded RNG keeps the visual stable
	# across launches so VLM verification + human review are reproducible.
	var terrain_set: TileSet = load(TERRAIN_TILESET_PATH) as TileSet
	if terrain_set != null:
		var terrain_layer := TileMapLayer.new()
		terrain_layer.tile_set = terrain_set
		terrain_layer.z_index = Z_TERRAIN
		terrain_layer.position = Vector2(SPRITE_ORIGIN_X, SPRITE_ORIGIN_Y)
		add_child(terrain_layer)
		var rng := RandomNumberGenerator.new()
		rng.seed = TERRAIN_SEED
		for tx in GRID_W:
			for ty in GRID_H:
				var source_id: int = rng.randi_range(0, TERRAIN_SOURCE_COUNT - 1)
				terrain_layer.set_cell(Vector2i(tx, ty), source_id, Vector2i(0, 0), 0)
	else:
		push_error("WorldRenderer: failed to load terrain TileSet at %s" % TERRAIN_TILESET_PATH)

	# V7 Phase 12-β — bootstrap building sprite at BOOTSTRAP_X/BOOTSTRAP_Y.
	# One building only; multi-building rendering requires a new
	# `collect_building_snapshot` SimBridge FFI and is deferred to Phase
	# 12-β.2. Position uses the same SPRITE_ORIGIN + TILE_SIZE basis as
	# the influence overlay so terrain + building + overlay align.
	var building_tex: Texture2D = load(BUILDING_SPRITE_PATH) as Texture2D
	if building_tex != null:
		var building_sprite := Sprite2D.new()
		building_sprite.texture = building_tex
		building_sprite.position = Vector2(
			SPRITE_ORIGIN_X + BOOTSTRAP_X * TILE_SIZE + TILE_SIZE / 2.0,
			SPRITE_ORIGIN_Y + BOOTSTRAP_Y * TILE_SIZE + TILE_SIZE / 2.0,
		)
		building_sprite.z_index = Z_BUILDING
		add_child(building_sprite)
	else:
		push_error("WorldRenderer: failed to load building sprite at %s" % BUILDING_SPRITE_PATH)

func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo:
		if event.keycode == KEY_SPACE:
			# T7.10.F: 6-state cycle Warmth → Light → Noise → Danger → Spiritual → Beauty → Warmth.
			var channel_name: String
			if current_channel == CHANNEL_WARMTH:
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
				current_channel = CHANNEL_WARMTH
				channel_name = "Warmth"
			print("Channel switched: ", channel_name)
	elif event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
		_handle_tile_click(event.position)

func _process(_delta: float) -> void:
	if world_sim == null:
		return
	var data: PackedByteArray = world_sim.get_influence_overlay(current_channel)
	if data.size() != GRID_W * GRID_H:
		return
	image = Image.create_from_data(GRID_W, GRID_H, false, Image.FORMAT_L8, data)
	texture.update(image)

func _handle_tile_click(pos: Vector2) -> void:
	var tile_x := int(floor((pos.x - SPRITE_ORIGIN_X) / float(TILE_SIZE)))
	var tile_y := int(floor((pos.y - SPRITE_ORIGIN_Y) / float(TILE_SIZE)))
	if tile_x < 0 or tile_x >= GRID_W or tile_y < 0 or tile_y >= GRID_H:
		return
	_fetch_causal_history(tile_x, tile_y)

func _fetch_causal_history(tx: int, ty: int) -> void:
	if world_sim == null:
		return
	var history: Array = world_sim.get_tile_causal_history(tx, ty)
	var panel := get_node_or_null("/root/Main/UI/CausalPanel")
	if panel != null and panel.has_method("display_history"):
		panel.call("display_history", history, tx, ty)
		panel.visible = true
