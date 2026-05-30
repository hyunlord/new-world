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
# V7 Phase 13-ε — extra bootstrap building positions so the first capture
# frame reads as a clustered settlement region instead of a single isolated
# building. Two additional buildings flanking the existing centre stamp at
# (32, 32). Same radius as the centre stamp so influence overlay coverage
# is symmetric around the row.
const BOOTSTRAP_X_LEFT := 24
const BOOTSTRAP_X_RIGHT := 40
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

# V7 Phase 13-β — resource-node placeholder layer.
#
# Decorative-only resource sprites placed deterministically from
# RESOURCE_SEED so the world reads as a populated simulation map.
# No sim-core ResourceNode component exists; agents do NOT interact
# with these sprites. Substrate-driven gathering is Section 15+.
# Sprite chosen: furniture/storage_pit/1.png (32×32) — visually
# distinct from buildings (no roof, looks like a hole/pile).
const RESOURCE_SPRITE_PATH := "res://assets/sprites/furniture/storage_pit/1.png"
const Z_RESOURCE := 3
const RESOURCE_COUNT := 20
const RESOURCE_SEED := 88675123

# V7 Section 16-α0 — backend-truth resource SOURCE markers (Method 1: the
# backend tile maps are the source of truth, the screen reflects them).
# Distinct from the decorative RESOURCE_SEED scatter above (additive layer).
# Source positions are fixed for the run, so the markers are drawn once
# (idempotent guard `_resource_sources_drawn`). No user-facing text → no
# locale keys. Kind encoding matches the FFI: 0=Food, 1=Water, 2=Sleep.
const Z_RESOURCE_SOURCE := 4
const SOURCE_KIND_COLORS: Array = [
	Color(0.30, 0.85, 0.30, 1.0),  # 0 Food  — green
	Color(0.30, 0.65, 1.00, 1.0),  # 1 Water — blue
	Color(0.90, 0.75, 0.30, 1.0),  # 2 Sleep — amber
]
var _resource_sources_drawn: bool = false

# V7 Phase 14-β — 5 resource types (Wood / Stone / Berry / Water /
# Food). Each type reuses an existing sprite from the 207-asset
# inventory (P14Plan-3). RESOURCE_COUNT (20) is preserved;
# distribution becomes 4 per type via `i % 5` cycle using the
# preserved RESOURCE_SEED.
#
# Honest disclosure: V7 backend has no ResourceNode component
# (verified 2026-05-26 against rust/crates/sim-core/src/components/).
# Agents do NOT interact with these sprites. Real ResourceNode
# system is deferred to Section 16+.
#
# Index 0 = Berry deliberately reuses RESOURCE_SPRITE_PATH so the
# Phase 13-β `storage_pit/1.png` literal stays referenced and the
# `harness_p13_beta_a1` assertion remains green.
const RESOURCE_TYPE_PATHS: Array = [
	RESOURCE_SPRITE_PATH,                                         # 0: Berry (legacy alias)
	"res://assets/sprites/furniture/workbench/1.png",             # 1: Wood
	"res://assets/sprites/walls/limestone/1.png",                 # 2: Stone
	"res://assets/sprites/floors/stone_slab/1.png",               # 3: Water (modulate-tinted)
	"res://assets/sprites/furniture/hearth/2.png",                # 4: Food
]
# Water (type index 3) has no native sprite in the inventory;
# blue modulate on a floor tile is the closest visual approximation
# without DGX Spark generation.
const RESOURCE_WATER_TINT := Color(0.45, 0.65, 1.0, 1.0)

# V7 Phase 14-β — 4 village fixture sprites placed around the
# 3-campfire bootstrap row (Phase 13-ε) so the centre region
# reads as "a settled village" instead of "three isolated
# campfires". Positions form a diamond around (32, 32):
#   (28, 28) Workshop    (36, 28) Drying area
#   (28, 36) Shelter     (36, 36) Storage
#
# Honest disclosure: V7 backend has no BuildingType / Profession
# ECS substrate (verified 2026-05-26). These fixtures are
# decorative-only — agents do NOT interact with them.
# `_update_construction_sites` / `_update_settlement_furniture`
# remain the substrate-driven render paths. Real BuildingType
# system is deferred to Section 16+.
const VILLAGE_FIXTURE_PATHS: Array = [
	"res://assets/sprites/furniture/workbench/2.png",     # 0: Workshop
	"res://assets/sprites/furniture/drying_rack/1.png",   # 1: Drying area
	"res://assets/sprites/furniture/lean_to/1.png",       # 2: Shelter
	"res://assets/sprites/furniture/storage_pit/2.png",   # 3: Storage
]
const VILLAGE_FIXTURE_POSITIONS: Array = [
	Vector2i(28, 28),  # 0: Workshop  (top-left of diamond)
	Vector2i(36, 28),  # 1: Drying    (top-right)
	Vector2i(28, 36),  # 2: Shelter   (bottom-left)
	Vector2i(36, 36),  # 3: Storage   (bottom-right)
]
const Z_VILLAGE_FIXTURE := 5  # same plane as ConstructionSite layer

# V7 Phase 14-γ — click inspector probe radius.
#
# Mouse position (passed through unchanged from _unhandled_input) is
# converted to world-coord agent centres via SPRITE_ORIGIN + TILE_SIZE.
# An agent is considered "clicked" if its tile centre lies within
# CLICK_RADIUS_WORLD_PX of the mouse. Bound symbolically to TILE_SIZE
# (= 16, Phase 4-γ locked) so the radius stays coupled to the legibility
# floor at zoom 3.0× and never drifts if TILE_SIZE is ever retuned.
const CLICK_RADIUS_WORLD_PX := float(TILE_SIZE)

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
	# V7 Phase 13-ε — additional bootstrap buildings flanking the centre at ±8.
	world_sim.on_building_placed(BOOTSTRAP_X_LEFT, BOOTSTRAP_Y, BOOTSTRAP_RADIUS)
	world_sim.on_building_placed(BOOTSTRAP_X_RIGHT, BOOTSTRAP_Y, BOOTSTRAP_RADIUS)
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
		# V7 Phase 13-ε — flanking bootstrap building sprites at the left and
		# right positions. Reuse `building_tex` already loaded above; if it
		# failed to load, the parent guard handles it.
		for extra_x in [BOOTSTRAP_X_LEFT, BOOTSTRAP_X_RIGHT]:
			var extra_sprite := Sprite2D.new()
			extra_sprite.texture = building_tex
			extra_sprite.position = Vector2(
				float(SPRITE_ORIGIN_X + extra_x * TILE_SIZE) + float(TILE_SIZE) / 2.0,
				float(SPRITE_ORIGIN_Y + BOOTSTRAP_Y * TILE_SIZE) + float(TILE_SIZE) / 2.0,
			)
			extra_sprite.z_index = Z_BUILDING
			add_child(extra_sprite)
	else:
		push_error("WorldRenderer: failed to load building sprite at %s" % BUILDING_SPRITE_PATH)

	# V7 Phase 13-β — scatter resource placeholder sprites.
	# Decorative-only layer placed deterministically from RESOURCE_SEED.
	# Agents do NOT interact with these sprites; substrate-driven gathering
	# is Section 15+. Z_RESOURCE=3 puts them above terrain (z=0) and below
	# the furniture (z=4) and construction (z=5) layers so buildings remain
	# visually dominant.
	#
	# V7 Phase 14-β — pre-load all 5 resource type textures so the loop
	# doesn't hit the resource loader RESOURCE_COUNT times. Failure to
	# load any one type emits a warning and leaves the corresponding
	# entry null; the loop's null-guard then skips that index.
	var resource_textures: Array = []
	for path in RESOURCE_TYPE_PATHS:
		var tex: Texture2D = load(path) as Texture2D
		resource_textures.append(tex)
		if tex == null:
			push_warning("WorldRenderer: failed to load resource sprite at %s" % path)
	var rng_res := RandomNumberGenerator.new()
	rng_res.seed = RESOURCE_SEED
	for i in RESOURCE_COUNT:
		var rtx: int = rng_res.randi_range(0, GRID_W - 1)
		var rty: int = rng_res.randi_range(0, GRID_H - 1)
		var type_idx: int = i % RESOURCE_TYPE_PATHS.size()
		var rtex: Texture2D = resource_textures[type_idx] as Texture2D
		if rtex == null:
			continue
		var res_sprite := Sprite2D.new()
		res_sprite.texture = rtex
		res_sprite.position = Vector2(
			float(SPRITE_ORIGIN_X + rtx * TILE_SIZE) + float(TILE_SIZE) / 2.0,
			float(SPRITE_ORIGIN_Y + rty * TILE_SIZE) + float(TILE_SIZE) / 2.0,
		)
		res_sprite.z_index = Z_RESOURCE
		# V7 Phase 14-β — Water type uses a blue modulate tint because
		# no native water sprite exists in the inventory. All other
		# types render with default modulate (white).
		if type_idx == 3:
			res_sprite.modulate = RESOURCE_WATER_TINT
		add_child(res_sprite)

	# V7 Phase 14-β — place 4 village fixture sprites at the
	# pre-determined diamond positions around the bootstrap row.
	# Each fixture has its own texture; they share Z_VILLAGE_FIXTURE
	# = Z_CONSTRUCTION = 5 so they sit in the same plane as
	# substrate-driven construction sites.
	for fi in VILLAGE_FIXTURE_PATHS.size():
		var fpath: String = VILLAGE_FIXTURE_PATHS[fi]
		var ftex: Texture2D = load(fpath) as Texture2D
		if ftex == null:
			push_warning("WorldRenderer: failed to load village fixture at %s" % fpath)
			continue
		var fpos: Vector2i = VILLAGE_FIXTURE_POSITIONS[fi]
		var fixture_sprite := Sprite2D.new()
		fixture_sprite.texture = ftex
		fixture_sprite.position = Vector2(
			float(SPRITE_ORIGIN_X + fpos.x * TILE_SIZE) + float(TILE_SIZE) / 2.0,
			float(SPRITE_ORIGIN_Y + fpos.y * TILE_SIZE) + float(TILE_SIZE) / 2.0,
		)
		fixture_sprite.z_index = Z_VILLAGE_FIXTURE
		add_child(fixture_sprite)

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
	# V7 Section 16-α0 — draw backend-truth resource source markers once.
	_render_resource_sources()

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

# V7 Section 16-α0 — draw backend-truth resource SOURCE markers from the
# SimBridge resource snapshot (Method 1: backend is the source of truth).
# Source positions are fixed for the run, so this draws once and then
# guards on `_resource_sources_drawn`. Each marker reuses an existing
# loaded texture (RESOURCE_SPRITE_PATH) modulated per kind; the decorative
# RESOURCE_SEED scatter layer (drawn in `_ready`) is left untouched.
func _render_resource_sources() -> void:
	if world_sim == null or _resource_sources_drawn:
		return
	var snap: Dictionary = world_sim.get_resource_snapshot()
	var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
	var kinds: PackedInt32Array = snap.get("kinds", PackedInt32Array())
	var n: int = min(xs.size(), min(ys.size(), kinds.size()))
	if n == 0:
		return
	var tex: Texture2D = load(RESOURCE_SPRITE_PATH) as Texture2D
	if tex == null:
		push_warning("WorldRenderer: failed to load resource source marker at %s" % RESOURCE_SPRITE_PATH)
		return
	for i in n:
		var k: int = kinds[i]
		if k < 0 or k >= SOURCE_KIND_COLORS.size():
			continue
		var px: float = float(SPRITE_ORIGIN_X + xs[i] * TILE_SIZE) + float(TILE_SIZE) / 2.0
		var py: float = float(SPRITE_ORIGIN_Y + ys[i] * TILE_SIZE) + float(TILE_SIZE) / 2.0
		var marker := Sprite2D.new()
		marker.texture = tex
		marker.modulate = SOURCE_KIND_COLORS[k]
		marker.z_index = Z_RESOURCE_SOURCE
		marker.position = Vector2(px, py)
		add_child(marker)
	_resource_sources_drawn = true

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
	# V7 Phase 14-γ — agent probe takes priority over the tile dispatch.
	# When an agent is within CLICK_RADIUS_WORLD_PX of the click, the
	# inspector panel is shown and the tile causal-history dispatch is
	# skipped (mutually exclusive — see Assertion 22).
	if _try_agent_click(pos):
		return
	# Phase 12-β.2 tile causal-history fallback — preserved bounds check
	# and dispatch when no agent is within radius.
	var tile_x := int(floor((pos.x - SPRITE_ORIGIN_X) / float(TILE_SIZE)))
	var tile_y := int(floor((pos.y - SPRITE_ORIGIN_Y) / float(TILE_SIZE)))
	if tile_x < 0 or tile_x >= GRID_W or tile_y < 0 or tile_y >= GRID_H:
		return
	_fetch_causal_history(tile_x, tile_y)


# V7 Phase 14-γ — agent click probe.
#
# Iterates the most recent agent snapshot (parallel arrays of
# entity_bits / xs / ys) and finds the agent whose tile centre is closest
# to the mouse world coords, within CLICK_RADIUS_WORLD_PX. On a hit,
# queries `get_agent_detail` for the 8-field row and forwards it to the
# AgentInspectorPanel via `display_agent`.
#
# Returns true iff an agent was found AND the inspector accepted the
# call (i.e. `found == true` in the detail dict). On a miss, the caller
# falls through to the tile causal-history dispatch.
#
# Ties are resolved by lower snapshot iteration index because the
# comparison is strict less-than: the first candidate to set `best_idx`
# at a given distance wins (Assertion 21).
func _try_agent_click(world_pos: Vector2) -> bool:
	if world_sim == null:
		return false
	var snap: Dictionary = world_sim.get_agent_snapshot()
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
	var panel := get_node_or_null("/root/Main/UI/AgentInspectorPanel")
	if panel != null and panel.has_method("display_agent"):
		panel.call("display_agent", detail)
		return true
	return false

func _fetch_causal_history(tx: int, ty: int) -> void:
	if world_sim == null:
		return
	var history: Array = world_sim.get_tile_causal_history(tx, ty)
	var panel := get_node_or_null("/root/Main/UI/CausalPanel")
	if panel != null and panel.has_method("display_history"):
		panel.call("display_history", history, tx, ty)
		panel.visible = true
