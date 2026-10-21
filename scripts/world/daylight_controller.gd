# daylight_controller.gd (DaylightController in scenes/main.tscn): day/night CanvasModulate tint, hearth glows lit by
#   settlement fire state (centred on the hearth tile), and shelter doorway glows at night (asks AgentRenderer for
#   sheltered sleepers). `tint_at` / `night_factor_at` are static so the colony agent layer warms people near a fire
#   with the same night curve.
# Bridge methods: via SimReads get_time_snapshot, get_settlement_snapshot, get_building_snapshot; signal world_reset.
extends Node2D

const WOOD = preload("res://scripts/world/wood_visuals.gd")
const SPRITE_ORIGIN: Vector2 = Vector2(8, 8) # M1: the centre of tile (0, 0) in the colony view
const TILE_SIZE: int = 16
const NIGHT_TINT: Color = Color(0.42, 0.48, 0.68)
const DUSK_TINT: Color = Color(0.90, 0.72, 0.60)
const HOURS: Array[float] = [0.0, 5.0, 7.0, 17.0, 19.5, 21.0, 24.0]
const TINTS: Array[Color] = [NIGHT_TINT, NIGHT_TINT, Color.WHITE, Color.WHITE, DUSK_TINT, NIGHT_TINT, NIGHT_TINT]
var _world_sim: WorldSimNode
var _modulate: CanvasModulate
var _glows: Dictionary = {}
var _glow_texture: Texture2D
var _glow_material: CanvasItemMaterial
var _frame: int = 0
var _elapsed: float = 0.0
var _doorways: Dictionary = {}
var _doorway_texture: Texture2D

func _ready() -> void:
	z_index = 6
	_world_sim = get_node("../WorldSim") as WorldSimNode
	_modulate = get_node("CanvasModulate") as CanvasModulate
	_glow_texture = _radial_texture(160)
	_doorway_texture = _radial_texture(20)
	_glow_material = CanvasItemMaterial.new()
	_glow_material.blend_mode = CanvasItemMaterial.BLEND_MODE_ADD
	_world_sim.connect("world_reset", refresh_snapshot)
	refresh_snapshot()

func refresh_snapshot() -> void:
	_reconcile_hearths()
	_update_daylight()

func _process(delta: float) -> void:
	_elapsed += delta
	_frame += 1
	if _frame % 30 == 0 or _glows.is_empty():
		_reconcile_hearths()
	_update_daylight()
	_update_doorways()

func _update_daylight() -> void:
	var snapshot: Dictionary = SimReads.read(_world_sim, "get_time_snapshot")
	var hour: float = fposmod(float(snapshot.get("time_of_day", 0.0)), 24.0)
	_modulate.color = tint_at(hour)
	var night_factor: float = night_factor_at(hour)
	var settlements: Dictionary = SimReads.read(_world_sim, "get_settlement_snapshot")
	for id: int in _glows.keys():
		var glow: Sprite2D = _glows[id] as Sprite2D
		var state: Dictionary = WOOD.settlement(settlements, glow.get_meta("tile", Vector2i.ZERO))
		glow.modulate = Color(1.0, 0.6, 0.25, WOOD.glow_alpha(state, night_factor, _elapsed))
		glow.set_meta("fire_lit", state.get("fire_lit", null))
		glow.set_meta("glow_alpha", glow.modulate.a)

## The CanvasModulate tint at clock hour `hour` (0–24).
static func tint_at(hour: float) -> Color:
	var h: float = fposmod(hour, 24.0)
	for i: int in range(HOURS.size() - 1):
		if h >= HOURS[i] and h < HOURS[i + 1]:
			var blend: float = (h - HOURS[i]) / (HOURS[i + 1] - HOURS[i])
			return TINTS[i].lerp(TINTS[i + 1], blend)
	return NIGHT_TINT


## 0 by day, 1 at full night, from the tint's red channel.
static func night_factor_at(hour: float) -> float:
	return clampf((1.0 - tint_at(hour).r) / (1.0 - NIGHT_TINT.r), 0.0, 1.0)

func _reconcile_hearths() -> void:
	var snapshot: Dictionary = SimReads.read(_world_sim, "get_building_snapshot")
	var ids: PackedInt64Array = snapshot.get("ids", PackedInt64Array())
	var xs: PackedInt32Array = snapshot.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snapshot.get("ys", PackedInt32Array())
	var kinds: PackedByteArray = snapshot.get("kinds", PackedByteArray())
	var seen: Dictionary = {}
	for i: int in range(ids.size()):
		if kinds[i] != 0:
			continue
		var glow: Sprite2D = _glows.get(ids[i]) as Sprite2D
		if glow == null:
			glow = Sprite2D.new()
			glow.texture = _glow_texture
			glow.material = _glow_material
			add_child(glow)
			_glows[ids[i]] = glow
		glow.set_meta("tile", Vector2i(xs[i], ys[i]))
		glow.set_meta("art_kind", "hearth_glow")
		glow.set_meta("art_id", ids[i])
		glow.position = SPRITE_ORIGIN + Vector2(xs[i], ys[i]) * TILE_SIZE
		seen[ids[i]] = true
	for id: int in _glows.keys():
		if not seen.has(id):
			var stale: Sprite2D = _glows[id] as Sprite2D
			stale.queue_free()
			_glows.erase(id)

func _radial_texture(size: int) -> Texture2D:
	var image := Image.create(size,size,false,Image.FORMAT_RGBA8)
	var center := Vector2(float(size-1)/2.0,float(size-1)/2.0)
	for y: int in range(size):
		for x: int in range(size):
			var distance: float = Vector2(x,y).distance_to(center)/(float(size)/2.0)
			image.set_pixel(x,y,Color(1,1,1,pow(maxf(0.0,1.0-distance),2)))
	return ImageTexture.create_from_image(image)

func _update_doorways() -> void:
	var hour: float = float(SimReads.read(_world_sim, "get_time_snapshot").get("time_of_day",12.0))
	var seen: Dictionary = {}
	var objects: Node2D = get_node_or_null("../WorldObjects") as Node2D
	if objects == null:
		return
	var sleeping: Dictionary = _sheltered_sleep_tiles()
	if hour>=20.0 or hour<6.0:
		for child: Node in objects.get_children():
			if child.get_meta("art_kind","") != "building" or int(child.get_meta("building_kind",-1)) != 1:
				continue
			var tile := Vector2i(int(child.get_meta("tile_x",0)),int(child.get_meta("tile_y",0)))
			if not sleeping.has(tile):
				continue
			var id: int = int(child.get_meta("art_id",0))
			var glow: Sprite2D = _doorways.get(id) as Sprite2D
			if glow == null:
				glow = Sprite2D.new()
				glow.texture = _doorway_texture
				glow.material = _glow_material
				add_child(glow)
				_doorways[id] = glow
			glow.position = (child as Sprite2D).position+Vector2(0,-5)
			glow.modulate = Color(1.0,0.6,0.25,0.45)
			seen[id] = true
	for id: int in _doorways.keys():
		if not seen.has(id):
			(_doorways[id] as Sprite2D).queue_free()
			_doorways.erase(id)

func _sheltered_sleep_tiles() -> Dictionary:
	var sleeping: Dictionary = {}
	var agents: Node = get_node_or_null("../AgentRenderer")
	if agents == null or not bool(agents.call("is_capture_visible")):
		return sleeping
	var active: Array = agents.call("get_active_agent_sprites")
	for sprite: Sprite2D in active:
		if bool(sprite.get_meta("sheltered_sleep",false)):
			sleeping[Vector2i(int(sprite.get_meta("tile_x",0)),int(sprite.get_meta("tile_y",0)))] = true
	return sleeping
