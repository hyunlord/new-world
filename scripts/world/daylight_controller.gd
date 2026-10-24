# daylight_controller.gd (DaylightController in scenes/main.tscn): day and night by tone (M-ART2 §3.5).
# Night lowers every value by one global CanvasModulate: the day is white; full night is `light.night_value` (0.55)
#   times the blue shift `light.night_tint` (the compositor's night transform, tone.py `night_value` and the ground's
#   blue `season_ground`); dusk (`light.dusk_hours`) and dawn (`light.dawn_hours`) blend linearly. Each hearth whose
#   settlement fire is lit carries an additive PointLight2D that restores value linearly to zero at
#   `light.hearth_radius_tiles` (5) tiles, so the camp reads at night; its energy follows the night factor, so it is
#   invisible by day. People and things keep their colours: only this global value change and the lights touch them.
#   `dark_at(p)` tells the agent layer how dark it is where a person stands (the night factor where no lit hearth's
#   light reaches), for the night halo (craft rule 5: a halo where people could be lost, at night away from a fire).
#   `tint_at` / `night_factor_at` are static (the HUD's clock word and tests use the same curve).
# Bridge methods: via SimReads get_time_snapshot, get_settlement_snapshot, get_building_snapshot; signal world_reset.
extends Node2D

const View = preload("res://scripts/world/colony/view.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
const WOOD = preload("res://scripts/world/wood_visuals.gd")

var _world_sim: WorldSimNode
var _modulate: CanvasModulate
var _lights: Dictionary = {}
var _light_texture: Texture2D = null
var _light_radius: float = 0.0
var _frame: int = 0
var _night: float = 0.0


func _ready() -> void:
	_world_sim = get_node("../WorldSim") as WorldSimNode
	_modulate = get_node("CanvasModulate") as CanvasModulate
	_world_sim.connect("world_reset", refresh_snapshot)
	refresh_snapshot()


func refresh_snapshot() -> void:
	for l: Node in _lights.values():
		l.queue_free()
	_lights.clear()
	_reconcile_hearths()
	_update_daylight()


func _process(_delta: float) -> void:
	_frame += 1
	if _frame % 30 == 0 or _lights.is_empty():
		_reconcile_hearths()
	_update_daylight()


static func _pair(path: String, fallback: Vector2) -> Vector2:
	var v: Variant = ArtData.param(path, null)
	if v is Array and (v as Array).size() == 2:
		return Vector2(float(v[0]), float(v[1]))
	return fallback


## The full-night CanvasModulate colour (value × blue shift).
static func night_colour() -> Color:
	var value: float = float(ArtData.param("light.night_value", 0.55))
	var t: Variant = ArtData.param("light.night_tint", [0.86, 0.94, 0.96])
	if t is Array and (t as Array).size() == 3:
		return Color(clampf(value * float(t[0]), 0.0, 1.0), clampf(value * float(t[1]), 0.0, 1.0), clampf(value * float(t[2]), 0.0, 1.0))
	return Color(value, value, value)


## 0 by day, 1 at full night, at clock hour `hour` (0–24): night between dusk's end and dawn's start.
static func night_factor_at(hour: float) -> float:
	var h: float = fposmod(hour, 24.0)
	var dusk: Vector2 = _pair("light.dusk_hours", Vector2(18, 21))
	var dawn: Vector2 = _pair("light.dawn_hours", Vector2(4, 7))
	if h >= dusk.y or h < dawn.x:
		return 1.0
	if h >= dawn.y and h < dusk.x:
		return 0.0
	if h < dawn.y:
		return 1.0 - (h - dawn.x) / maxf(0.01, dawn.y - dawn.x)
	return (h - dusk.x) / maxf(0.01, dusk.y - dusk.x)


## The CanvasModulate tint at clock hour `hour`.
static func tint_at(hour: float) -> Color:
	return Color.WHITE.lerp(night_colour(), night_factor_at(hour))


func _update_daylight() -> void:
	var snapshot: Dictionary = SimReads.read(_world_sim, "get_time_snapshot")
	var hour: float = fposmod(float(snapshot.get("time_of_day", 12.0)), 24.0)
	_modulate.color = tint_at(hour)
	var night: float = night_factor_at(hour)
	_night = night
	var restore: Color = Color.WHITE - night_colour()
	var settlements: Dictionary = SimReads.read(_world_sim, "get_settlement_snapshot")
	for id: int in _lights.keys():
		var light: PointLight2D = _lights[id]
		var state: Dictionary = WOOD.settlement(settlements, light.get_meta("tile", Vector2i.ZERO))
		var lit: bool = not state.has("fire_lit") or bool(state["fire_lit"])
		light.enabled = night > 0.0 and lit
		light.color = Color(restore.r, restore.g, restore.b)
		light.energy = night
		light.set_meta("fire_lit", lit)


func _texture(radius_px: int) -> Texture2D:
	if _light_texture != null and absf(_light_radius - radius_px) < 0.5:
		return _light_texture
	var size: int = radius_px * 2
	var img: Image = Image.create(size, size, false, Image.FORMAT_RGBA8)
	var centre: Vector2 = Vector2(radius_px, radius_px)
	for y: int in size:
		for x: int in size:
			var d: float = Vector2(x + 0.5, y + 0.5).distance_to(centre) / float(radius_px)
			var v: float = clampf(1.0 - d, 0.0, 1.0)
			img.set_pixel(x, y, Color(v, v, v, 1.0))
	_light_texture = ImageTexture.create_from_image(img)
	_light_radius = radius_px
	return _light_texture


func _reconcile_hearths() -> void:
	var snapshot: Dictionary = SimReads.read(_world_sim, "get_building_snapshot")
	var ids: PackedInt64Array = snapshot.get("ids", PackedInt64Array())
	var xs: PackedInt32Array = snapshot.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snapshot.get("ys", PackedInt32Array())
	var kinds: PackedByteArray = snapshot.get("kinds", PackedByteArray())
	var radius: int = int(float(ArtData.param("light.hearth_radius_tiles", 5)) * View.TILE)
	var tex: Texture2D = _texture(maxi(View.TILE, radius))
	var seen: Dictionary = {}
	for i: int in mini(ids.size(), mini(kinds.size(), mini(xs.size(), ys.size()))):
		if not bool(ArtData.binding("buildings", kinds[i]).get("light", false)):
			continue
		var light: PointLight2D = _lights.get(ids[i], null)
		if light == null:
			light = PointLight2D.new()
			light.blend_mode = Light2D.BLEND_MODE_ADD
			add_child(light)
			_lights[ids[i]] = light
		light.texture = tex
		light.set_meta("tile", Vector2i(xs[i], ys[i]))
		light.position = View.tile_centre(Vector2i(xs[i], ys[i]))
		seen[ids[i]] = true
	for id: int in _lights.keys():
		if not seen.has(id):
			(_lights[id] as Node).queue_free()
			_lights.erase(id)


## The lights (for tests): {tile, enabled, energy}.
func lights() -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for l: PointLight2D in _lights.values():
		out.append({"tile": l.get_meta("tile", Vector2i.ZERO), "enabled": l.enabled, "energy": l.energy})
	return out


## How dark it is at world position `p`: the night factor (0 by day, 1 at full night) times one minus the strongest
## lit hearth's linear restore there (0 at the hearth, 1 beyond its radius).
func dark_at(p: Vector2) -> float:
	if _night <= 0.0:
		return 0.0
	var lit: float = 0.0
	for l: PointLight2D in _lights.values():
		if l.enabled and _light_radius > 0.0:
			lit = maxf(lit, clampf(1.0 - p.distance_to(l.position) / _light_radius, 0.0, 1.0))
	return _night * (1.0 - lit)
