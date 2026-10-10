extends Node2D

const SPRITE_ORIGIN: Vector2 = Vector2(448, 28)
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

func _ready() -> void:
	z_index = 6
	_world_sim = get_node("../WorldSim") as WorldSimNode
	_modulate = get_node("CanvasModulate") as CanvasModulate
	_glow_texture = load("res://assets/sprites/fx/glow_radial.png") as Texture2D
	_glow_material = CanvasItemMaterial.new()
	_glow_material.blend_mode = CanvasItemMaterial.BLEND_MODE_ADD
	_world_sim.connect("world_reset", refresh_snapshot)
	refresh_snapshot()

func refresh_snapshot() -> void:
	_reconcile_hearths()
	_update_daylight()

func _process(_delta: float) -> void:
	_frame += 1
	if _frame % 30 == 0 or _glows.is_empty():
		_reconcile_hearths()
	_update_daylight()

func _update_daylight() -> void:
	var snapshot: Dictionary = _world_sim.get_time_snapshot()
	var hour: float = fposmod(float(snapshot.get("time_of_day", 0.0)), 24.0)
	var tint: Color = NIGHT_TINT
	for i: int in range(HOURS.size() - 1):
		if hour >= HOURS[i] and hour < HOURS[i + 1]:
			var blend: float = (hour - HOURS[i]) / (HOURS[i + 1] - HOURS[i])
			tint = TINTS[i].lerp(TINTS[i + 1], blend)
			break
	_modulate.color = tint
	var night_factor: float = clampf((1.0 - tint.r) / (1.0 - NIGHT_TINT.r), 0.0, 1.0)
	for id: int in _glows.keys():
		var glow: Sprite2D = _glows[id] as Sprite2D
		glow.modulate = Color(1.0, 0.6, 0.25, 0.55 * night_factor)

func _reconcile_hearths() -> void:
	var snapshot: Dictionary = _world_sim.get_building_snapshot()
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
		glow.position = SPRITE_ORIGIN + Vector2(xs[i], ys[i]) * TILE_SIZE + Vector2.ONE * (TILE_SIZE / 2.0)
		seen[ids[i]] = true
	for id: int in _glows.keys():
		if not seen.has(id):
			var stale: Sprite2D = _glows[id] as Sprite2D
			stale.queue_free()
			_glows.erase(id)
