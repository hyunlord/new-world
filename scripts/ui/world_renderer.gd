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
# V7 Phase 13-α — bootstrap building uses campfire sprite to visually
# distinguish it from ConstructionSite (cairn) and Settlement centroid
# (hearth). Three distinct 32×32 sprites for three distinct semantic layers.
const BUILDING_SPRITE_PATH := "res://assets/sprites/buildings/campfire/1.png"
const TERRAIN_SEED := 19349663  # deterministic seed for reproducible terrain
const OVERLAY_ALPHA := 0.65
const Z_TERRAIN := 0
const Z_BUILDING := 5
const Z_OVERLAY := 10
const TERRAIN_SOURCE_COUNT := 9  # 3 materials × 3 variants

# V7 Phase 12-β.2 (A3) — construction-site rendering constants.
# Sites are drawn with a cairn placeholder sprite at z=5 (above terrain,
# below the influence overlay at z=10). Alpha is remapped from the raw
# `progress / required_progress` ratio into [0.3, 1.0] so freshly-started
# sites are still visible while completed sites become fully opaque.
const CONSTRUCTION_SPRITE_PATH := "res://assets/sprites/buildings/cairn/1.png"
const Z_CONSTRUCTION := 5
const CONSTRUCTION_ALPHA_MIN := 0.3
const CONSTRUCTION_ALPHA_MAX := 1.0

# V7 Phase 12-γ — Settlement furniture placeholder constants.
# A 32×32 hearth sprite is placed at the substrate-derived centroid of
# every Settlement that has at least one resolvable member agent. The
# sprite sits at z=4 — above the terrain TileMapLayer (z=0) and below
# the ConstructionSite layer (z=5) and the influence overlay (z=10).
const FURNITURE_SPRITE_PATH := "res://assets/sprites/furniture/hearth/1.png"
const Z_FURNITURE := 4

var current_channel: int = CHANNEL_WARMTH
var world_sim: WorldSimNode
var sprite: Sprite2D
var texture: ImageTexture
var image: Image

# V7 Phase 12-β.2 (A3) — entity_bits (int) → Sprite2D for that construction
# site. Keyed dictionary so the per-frame reaper can free Sprite2D nodes
# corresponding to despawned (completed or otherwise removed) sites.
var _construction_sprites: Dictionary = {}

# V7 Phase 12-γ — entity_bits (int) → Sprite2D for the corresponding
# Settlement's furniture placeholder. Same keyed-Dictionary pattern as
# the construction layer; the per-frame reaper frees sprites whose
# Settlement entity no longer appears in the snapshot (dissolved).
var _furniture_sprites: Dictionary = {}

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
	# V7 Phase 12-β.2 (A3) — ingest construction-site snapshot.
	_update_construction_sites()
	# V7 Phase 12-γ — ingest settlement snapshot for furniture placeholders.
	_update_settlement_furniture()

# V7 Phase 12-β.2 (A3) — pull the per-frame construction snapshot from
# SimBridge and reconcile against `_construction_sprites`:
#   - create a Sprite2D for any newly-seen entity_bits
#   - update position + modulate alpha for every visible entity
#   - reap (queue_free + erase) any keys not present this frame
# Alpha is `CONSTRUCTION_ALPHA_MIN + (MAX - MIN) * ratio`, where ratio is
# `clampf(progress / max(required_progress, 1), 0.0, 1.0)`. The
# `max(required_progress, 1)` guards against substrate sites with
# `required_progress == 0`.
func _update_construction_sites() -> void:
	var snap: Dictionary = world_sim.get_construction_snapshot()
	var ids: PackedInt64Array = snap.get("ids", PackedInt64Array())
	var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
	var progresses: PackedInt32Array = snap.get("progresses", PackedInt32Array())
	var required: PackedInt32Array = snap.get("required_progresses", PackedInt32Array())
	var n: int = ids.size()
	var seen: Dictionary = {}
	var tex: Texture2D = load(CONSTRUCTION_SPRITE_PATH) as Texture2D
	for i in n:
		var entity_id: int = ids[i]
		seen[entity_id] = true
		var px: float = float(SPRITE_ORIGIN_X + xs[i] * TILE_SIZE) + float(TILE_SIZE) / 2.0
		var py: float = float(SPRITE_ORIGIN_Y + ys[i] * TILE_SIZE) + float(TILE_SIZE) / 2.0
		var req: int = max(int(required[i]), 1)
		var ratio: float = clampf(float(progresses[i]) / float(req), 0.0, 1.0)
		var alpha: float = CONSTRUCTION_ALPHA_MIN + (CONSTRUCTION_ALPHA_MAX - CONSTRUCTION_ALPHA_MIN) * ratio
		var construction_sprite: Sprite2D = _construction_sprites.get(entity_id, null) as Sprite2D
		if construction_sprite == null:
			construction_sprite = Sprite2D.new()
			construction_sprite.texture = tex
			construction_sprite.z_index = Z_CONSTRUCTION
			add_child(construction_sprite)
			_construction_sprites[entity_id] = construction_sprite
		construction_sprite.position = Vector2(px, py)
		construction_sprite.modulate = Color(1.0, 1.0, 1.0, alpha)
	# Reap entries no longer present in the snapshot (despawned sites).
	for entity_id in _construction_sprites.keys():
		if not seen.has(entity_id):
			var stale: Sprite2D = _construction_sprites[entity_id]
			if stale != null:
				stale.queue_free()
			_construction_sprites.erase(entity_id)

# V7 Phase 12-γ — pull the per-frame settlement snapshot from SimBridge
# and reconcile against `_furniture_sprites`:
#   - create a hearth Sprite2D for any newly-seen Settlement entity_bits
#   - update its position to the FFI-supplied centroid
#   - reap (queue_free + erase) any keys not present this frame
# The centroid is already integer (floor of mean of member positions)
# from the Rust collector; here we only translate tile coords → pixels.
func _update_settlement_furniture() -> void:
	var snap: Dictionary = world_sim.get_settlement_snapshot()
	var ids: PackedInt64Array = snap.get("ids", PackedInt64Array())
	var xs: PackedInt32Array = snap.get("centroid_xs", PackedInt32Array())
	var ys: PackedInt32Array = snap.get("centroid_ys", PackedInt32Array())
	var n: int = ids.size()
	var seen: Dictionary = {}
	var tex: Texture2D = load(FURNITURE_SPRITE_PATH) as Texture2D
	for i in n:
		var entity_id: int = ids[i]
		seen[entity_id] = true
		var px: float = float(SPRITE_ORIGIN_X + xs[i] * TILE_SIZE) + float(TILE_SIZE) / 2.0
		var py: float = float(SPRITE_ORIGIN_Y + ys[i] * TILE_SIZE) + float(TILE_SIZE) / 2.0
		var furniture_sprite: Sprite2D = _furniture_sprites.get(entity_id, null) as Sprite2D
		if furniture_sprite == null:
			furniture_sprite = Sprite2D.new()
			furniture_sprite.texture = tex
			furniture_sprite.z_index = Z_FURNITURE
			add_child(furniture_sprite)
			_furniture_sprites[entity_id] = furniture_sprite
		furniture_sprite.position = Vector2(px, py)
	# Reap entries no longer present in the snapshot (dissolved settlements).
	for entity_id in _furniture_sprites.keys():
		if not seen.has(entity_id):
			var stale: Sprite2D = _furniture_sprites[entity_id]
			if stale != null:
				stale.queue_free()
			_furniture_sprites.erase(entity_id)

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
