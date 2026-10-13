extends Node2D

const Selection = preload("res://scripts/core/settlement_selection.gd")
const ART = preload("res://scripts/ui/art_sprite_library.gd")
const ORIGIN := Vector2(448, 28)
const RAIN_COLOR := Color(0.624, 0.706, 0.784, 0.5)
const GIFT_COLOR := Color("7CC46A")
const SPOIL_COLOR := Color("8A6A3A")
const WINDFALL_DROP_SECONDS: float = 0.8
const WINDFALL_DUST_SECONDS: float = 0.6
const WINDFALL_DUST_WIDTH: float = 2.0
const WINDFALL_DUST_COLOR := Color("E8D8B8")
var _world: Node
var _objects: Node
var _buildings: Node
var _elapsed: float = 0.0
var _state: Dictionary = {}
var _rain_centers: Array[Vector2] = []
var _rain_ids: Array[int] = []
var _rain_streaks: int = 0
var _previous_log: Array[String] = []
var _previous_wood: Dictionary = {}
var _falls: Dictionary = {}
var _flash_kind: String = ""
var _flash_target: int = -1
var _notified_target: int = -1
var _flash_born: float = -2.0
var _flash_label: Label

func _ready() -> void:
	z_index = 8
	process_priority = 20
	add_to_group("art_god_hand_effects")
	set_meta("art_kind", "god_hand_effects")
	_objects = get_node_or_null("../WorldObjects")
	_buildings = get_node_or_null("../BuildingRenderer")
	configure(get_node_or_null("../WorldSim"))

func configure(world: Node) -> void:
	_world = world
	if _world != null and _world.has_signal("world_reset") and not _world.is_connected("world_reset", _reset):
		_world.connect("world_reset", _reset)
	_reset()

func _reset() -> void:
	_previous_log.clear()
	_previous_wood.clear()
	_falls.clear()
	_rain_centers.clear()
	_rain_ids.clear()
	_state.clear()
	_flash_kind = ""
	_flash_target = -1
	_notified_target = -1
	_flash_born = -2.0
	if is_instance_valid(_flash_label):
		_flash_label.self_modulate = Color.WHITE
	_flash_label = null
	_elapsed = 0.0

func _process(delta: float) -> void:
	if _world == null or not _world.has_method("get_intervention_state"):
		return
	_elapsed += delta
	_state = _world.call("get_intervention_state")
	var settlements: Dictionary = _world.call("get_settlement_snapshot")
	var resources: Dictionary = _world.call("get_resource_snapshot")
	_observe(resources)
	_rain_centers.clear()
	_rain_ids.clear()
	var ids: PackedInt32Array = settlements.get("settlement_ids", PackedInt32Array())
	var xs: PackedInt32Array = settlements.get("formation_xs", PackedInt32Array())
	var ys: PackedInt32Array = settlements.get("formation_ys", PackedInt32Array())
	var rain: PackedInt32Array = _state.get("rain_settlement_ids", PackedInt32Array())
	var left: PackedInt32Array = _state.get("rain_ticks_left", PackedInt32Array())
	for index: int in rain.size():
		var row: int = ids.find(rain[index])
		if left[index] > 0 and row >= 0:
			_rain_ids.append(rain[index])
			_rain_centers.append(ORIGIN + Vector2(xs[row], ys[row]) * 16.0 + Vector2(8, 8))
	_update_falls()
	_update_flash(settlements)
	set_meta("intervention_effects", metadata())
	queue_redraw()

func _observe(resources: Dictionary) -> void:
	var wood: Dictionary = {}
	var xs: PackedInt32Array = resources.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = resources.get("ys", PackedInt32Array())
	var kinds: PackedInt32Array = resources.get("kinds", PackedInt32Array())
	var amounts: PackedInt32Array = resources.get("amounts", PackedInt32Array())
	for index: int in xs.size():
		if kinds[index] == 3:
			wood[Vector2i(xs[index], ys[index])] = amounts[index]
	var records: Array[String] = []
	var log_kinds: PackedStringArray = _state.get("log_kinds", PackedStringArray())
	var ticks: PackedInt64Array = _state.get("log_ticks", PackedInt64Array())
	var requested: PackedInt64Array = _state.get("log_amounts", PackedInt64Array())
	var applied: PackedInt64Array = _state.get("log_applied", PackedInt64Array())
	for index: int in log_kinds.size():
		records.append("%d:%s:%d:%d" % [ticks[index], log_kinds[index], requested[index], applied[index]])
	var overlap: int = mini(records.size(), _previous_log.size())
	while overlap > 0 and _previous_log.slice(_previous_log.size() - overlap) != records.slice(0, overlap):
		overlap -= 1
	for index: int in range(overlap, records.size()):
		var kind: String = log_kinds[index]
		if kind == "windfall" and applied[index] > 0:
			for tile: Vector2i in wood:
				if _previous_wood.has(tile) and int(wood[tile]) > int(_previous_wood[tile]):
					_falls[tile] = _elapsed
		elif kind in ["gift_food", "spoil_food", "gift_wood", "rot_wood"]:
			_flash_kind = kind
			_flash_target = _notified_target if _notified_target >= 0 else Selection.cast_target(ticks[index], kind)
			_flash_born = _elapsed
	_notified_target = -1
	_previous_log = records
	_previous_wood = wood

static func fall_offset(age: float) -> float:
	if age < 0.0 or age >= WINDFALL_DROP_SECONDS:
		return 0.0
	if age < 0.6:
		return -6.0 * (1.0 - age / 0.6)
	return -sin((age - 0.6) / 0.2 * PI)

static func dust_alpha(age: float) -> float:
	return 0.85 * (1.0 - age / WINDFALL_DUST_SECONDS) if age >= 0.0 and age < WINDFALL_DUST_SECONDS else 0.0

func _update_falls() -> void:
	if _objects != null:
		for child: Node in _objects.get_children():
			if child.get_meta("art_kind", "") != "deadwood":
				continue
			var sprite := child as Sprite2D
			var tile := Vector2i(int(sprite.get_meta("tile_x", 0)), int(sprite.get_meta("tile_y", 0)))
			var age: float = _elapsed - float(_falls.get(tile, -1.0))
			var offset_y: float = fall_offset(age) if _falls.has(tile) else 0.0
			# Preserve the atlas pivot; only presentation position receives the fall.
			var frame: Dictionary = ART.frame(str(sprite.get_meta("sprite_id", "")))
			var anchor: Array = frame.get("anchor", [0, 0])
			sprite.offset.y = -float(anchor[1]) + offset_y
			sprite.set_meta("windfall_offset", offset_y)
			sprite.set_meta("windfall_age", age if _falls.has(tile) else -1.0)
	for tile: Vector2i in _falls.keys():
		if _elapsed - float(_falls[tile]) >= WINDFALL_DROP_SECONDS:
			_falls.erase(tile)

func _update_flash(settlements: Dictionary) -> void:
	if _buildings == null:
		return
	var age: float = _elapsed - _flash_born
	if age >= 1.2:
		if is_instance_valid(_flash_label):
			_flash_label.self_modulate = Color.WHITE
			_flash_label.set_meta("intervention_flash_alpha", 0.0)
		return
	var ids: PackedInt32Array = settlements.get("settlement_ids", PackedInt32Array())
	var xs: PackedInt32Array = settlements.get("formation_xs", PackedInt32Array())
	var ys: PackedInt32Array = settlements.get("formation_ys", PackedInt32Array())
	if ids.is_empty():
		return
	var home: int = 0
	for index: int in ids.size():
		if ids[index] < ids[home]:
			home = index
	if _flash_target >= 0:
		home = ids.find(_flash_target)
		if home < 0:
			return
	var anchor := Vector2(xs[home], ys[home])
	var distance: float = INF
	for child: Node in _buildings.get_children():
		if child.get_meta("art_kind", "") == "stockpile_label":
			var id: int = int(child.get_meta("art_id", 0))
			var tile := Vector2(id >> 32, id & 0xFFFFFFFF)
			if tile.distance_squared_to(anchor) < distance:
				distance = tile.distance_squared_to(anchor)
				_flash_label = child as Label
	if is_instance_valid(_flash_label):
		_flash_label.self_modulate = GIFT_COLOR if _flash_kind.begins_with("gift") else SPOIL_COLOR
		_flash_label.set_meta("intervention_flash_kind", _flash_kind)
		_flash_label.set_meta("intervention_flash_alpha", 1.0 - age / 1.2)

func _draw() -> void:
	_rain_streaks = 0
	for tile: Vector2i in _falls:
		var dust_age: float = _elapsed - float(_falls[tile])
		var alpha: float = dust_alpha(dust_age)
		if alpha > 0.0:
			var center: Vector2 = ORIGIN + Vector2(tile) * 16.0 + Vector2(8, 8)
			draw_arc(center, 4.0 + 4.0 * dust_age / WINDFALL_DUST_SECONDS, 0.0, TAU, 16, Color(WINDFALL_DUST_COLOR, alpha), WINDFALL_DUST_WIDTH, false)
	var transform: Transform2D = get_global_transform_with_canvas()
	var inverse: Transform2D = transform.affine_inverse()
	var zoom: float = transform.get_scale().x
	var viewport: Rect2 = get_viewport_rect()
	for center: Vector2 in _rain_centers:
		var screen: Vector2 = transform * center
		var radius: float = 192.0 * zoom
		var bounds := Rect2(screen - Vector2.ONE * radius, Vector2.ONE * radius * 2.0).intersection(viewport)
		for x: int in range(int(floor(bounds.position.x / 24.0)), int(ceil(bounds.end.x / 24.0))):
			for y: int in range(int(floor(bounds.position.y / 24.0)) - 1, int(ceil(bounds.end.y / 24.0))):
				var at := Vector2(x * 24 + posmod(y * 7, 24), y * 24 + fposmod(_elapsed * 120.0, 24.0))
				if at.distance_squared_to(screen) <= radius * radius and viewport.has_point(at):
					draw_rect(Rect2(inverse * at, Vector2(1, 3) / zoom), RAIN_COLOR)
					_rain_streaks += 1
	var age: float = _elapsed - _flash_born
	if age < 1.2 and is_instance_valid(_flash_label):
		var color: Color = GIFT_COLOR if _flash_kind.begins_with("gift") else SPOIL_COLOR
		color.a = 1.0 - age / 1.2
		var center: Vector2 = to_local(_flash_label.global_position)
		for index: int in 4:
			var direction := Vector2.from_angle(float(index) * PI / 2.0 - PI / 4.0)
			draw_rect(Rect2(center + direction * age * 18.0 / zoom, Vector2.ONE * 2.0 / zoom), color)

func metadata() -> Dictionary:
	var falls: Array[Dictionary] = []
	var dust: Array[Dictionary] = []
	for tile: Vector2i in _falls:
		var age: float = _elapsed - float(_falls[tile])
		falls.append({"tile": [tile.x, tile.y], "age": age, "offset_y": fall_offset(age), "duration": WINDFALL_DROP_SECONDS})
		if dust_alpha(age) > 0.0:
			dust.append({"tile": [tile.x, tile.y], "age": age, "alpha": dust_alpha(age), "duration": WINDFALL_DUST_SECONDS, "width": WINDFALL_DUST_WIDTH, "color": "E8D8B8"})
	var flashes: Array[Dictionary] = []
	if _elapsed - _flash_born < 1.2:
		flashes.append({"settlement_id": _flash_target, "kind": _flash_kind, "age": _elapsed - _flash_born, "color": "7CC46A" if _flash_kind.begins_with("gift") else "8A6A3A"})
	return {"rain_streaks": _rain_streaks, "rain_settlement_ids": _rain_ids.duplicate(), "cold_snap_tonight": bool(_state.get("cold_snap_tonight", false)), "windfall_falls": falls, "windfall_dust": dust, "stockpile_flashes": flashes}


func notify_cast(kind: String, target: int) -> void:
	if kind in ["gift_food", "spoil_food", "gift_wood", "rot_wood"]:
		_notified_target = target
