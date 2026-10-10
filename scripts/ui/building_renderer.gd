extends Node2D

const TILE_SIZE: int = 16
const SPRITE_ORIGIN: Vector2 = Vector2(448, 28)
const RECONCILE_FRAMES: int = 30
const STOCKPILE_FOOD_KEY: String = "HUD_STOCKPILE_FOOD"
const ART = preload("res://scripts/ui/art_sprite_library.gd")
const BUILDING_IDS: Array[String] = [
	"direction_01_campfire", "direction_02_lean_to", "direction_03_storage_pit",
]
const BUSH_IDS: Array[String] = [
	"direction_08_berry_bush_bare", "direction_07_berry_bush_one",
	"direction_06_berry_bush_two", "direction_05_berry_bush_full",
]
var _world_sim: WorldSimNode
var _locale: Node
var _camera: Camera2D
var _buildings: Dictionary = {}
var _sites: Dictionary = {}
var _bushes: Dictionary = {}
var _labels: Dictionary = {}
var _label_anchors: Dictionary = {}
var _objects: Node2D
var _shadows: Node2D
var _shadow_texture: Texture2D
var _object_shadows: Dictionary = {}
var _stockpile_positions: Array[Vector2i] = []
var _frame: int = 0

func _ready() -> void:
	z_index = 5
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_world_sim = get_node("../WorldSim") as WorldSimNode
	_locale = get_node_or_null("/root/Locale")
	_camera = get_node("../Camera2D") as Camera2D
	_objects = ART.shared_layer(self, "WorldObjects", 4, true)
	_shadows = ART.shared_layer(self, "Shadows", 3)
	_shadow_texture = load("res://assets/art/contact_shadow.png") as Texture2D
	_world_sim.connect("world_reset", refresh_snapshot)
	refresh_snapshot()

func _process(_delta: float) -> void:
	_frame += 1
	# Tick zero has queued camp buildings; retry until Rust stamps the registry.
	if _frame % RECONCILE_FRAMES == 0 or _buildings.is_empty():
		refresh_snapshot()
	_update_labels()

func refresh_snapshot() -> void:
	_update_buildings()
	_update_construction_sites()
	_update_bushes()
	_update_stockpiles()
	_update_labels()

func _tile_center(x: int, y: int) -> Vector2:
	return SPRITE_ORIGIN + Vector2(x, y) * TILE_SIZE + Vector2(8, 15)

func _sprite_for(nodes: Dictionary, id: int) -> Sprite2D:
	var sprite: Sprite2D = nodes.get(id) as Sprite2D
	if sprite == null:
		sprite = Sprite2D.new()
		_objects.add_child(sprite)
		nodes[id] = sprite
	return sprite

func _reap(nodes: Dictionary, seen: Dictionary) -> void:
	for id: int in nodes.keys():
		if not seen.has(id):
			var stale: Node = nodes[id] as Node
			var shadow_id: int = stale.get_instance_id()
			if _object_shadows.has(shadow_id):
				(_object_shadows[shadow_id] as Sprite2D).queue_free()
				_object_shadows.erase(shadow_id)
			stale.queue_free()
			nodes.erase(id)

func _configure(sprite: Sprite2D, sprite_id: String, kind: String, id: int, x: int, y: int) -> void:
	if not ART.apply(sprite, sprite_id):
		push_error("Missing building art: " + sprite_id)
	sprite.position = _tile_center(x, y)
	sprite.set_meta("art_kind", kind)
	sprite.set_meta("art_id", id)
	sprite.set_meta("tile_x", x)
	sprite.set_meta("tile_y", y)
	var shadow_id: int = sprite.get_instance_id()
	var shadow: Sprite2D = _object_shadows.get(shadow_id) as Sprite2D
	if shadow == null:
		shadow = Sprite2D.new()
		shadow.texture = _shadow_texture
		shadow.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
		shadow.modulate.a = 0.4
		shadow.set_meta("art_kind", "object_shadow")
		shadow.set_meta("art_id", id)
		_shadows.add_child(shadow)
		_object_shadows[shadow_id] = shadow
	shadow.position = sprite.position

func _update_buildings() -> void:
	var snapshot: Dictionary = _world_sim.get_building_snapshot()
	var ids: PackedInt64Array = snapshot.get("ids", PackedInt64Array())
	var xs: PackedInt32Array = snapshot.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snapshot.get("ys", PackedInt32Array())
	var kinds: PackedByteArray = snapshot.get("kinds", PackedByteArray())
	var seen: Dictionary = {}
	_stockpile_positions.clear()
	for i: int in range(ids.size()):
		var kind: int = kinds[i]
		var sprite: Sprite2D = _sprite_for(_buildings, ids[i])
		_configure(sprite, BUILDING_IDS[kind], "building", ids[i], xs[i], ys[i])
		sprite.set_meta("building_kind", kind)
		seen[ids[i]] = true
		if kind == 2:
			_stockpile_positions.append(Vector2i(xs[i], ys[i]))
	_reap(_buildings, seen)

func _update_construction_sites() -> void:
	var snapshot: Dictionary = _world_sim.get_construction_snapshot()
	var ids: PackedInt64Array = snapshot.get("ids", PackedInt64Array())
	var xs: PackedInt32Array = snapshot.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snapshot.get("ys", PackedInt32Array())
	var seen: Dictionary = {}
	for i: int in range(ids.size()):
		var site: Sprite2D = _sprite_for(_sites, ids[i])
		_configure(site, "direction_04_shelter_frame", "site", ids[i], xs[i], ys[i])
		site.set_meta("building_kind", 1)
		seen[ids[i]] = true
	_reap(_sites, seen)

func _update_bushes() -> void:
	var snapshot: Dictionary = _world_sim.get_resource_snapshot()
	var xs: PackedInt32Array = snapshot.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snapshot.get("ys", PackedInt32Array())
	var kinds: PackedInt32Array = snapshot.get("kinds", PackedInt32Array())
	var amounts: PackedInt32Array = snapshot.get("amounts", PackedInt32Array())
	var maxes: PackedInt32Array = snapshot.get("maxes", PackedInt32Array())
	var seen: Dictionary = {}
	for i: int in range(xs.size()):
		if kinds[i] != 0:
			continue
		var id: int = (xs[i] << 32) | ys[i]
		var stage: int = clampi(int(ceil(3.0 * float(amounts[i]) / float(maxi(maxes[i], 1)))), 0, 3)
		var bush: Sprite2D = _sprite_for(_bushes, id)
		_configure(bush, BUSH_IDS[stage], "bush", id, xs[i], ys[i])
		bush.set_meta("building_kind", -1)
		seen[id] = true
	_reap(_bushes, seen)

func _food_totals_by_stockpile(snapshot: Dictionary) -> Dictionary:
	var xs: PackedInt32Array = snapshot.get("formation_xs", PackedInt32Array())
	var ys: PackedInt32Array = snapshot.get("formation_ys", PackedInt32Array())
	var foods: PackedInt32Array = snapshot.get("food_stocks", PackedInt32Array())
	var totals: Dictionary = {}
	if _stockpile_positions.is_empty():
		return totals
	for i: int in range(foods.size()):
		# Building-id snapshot order keeps equal-distance choices stable.
		var nearest: Vector2i = _stockpile_positions[0]
		var best_distance: int = maxi(absi(nearest.x - xs[i]), absi(nearest.y - ys[i]))
		for candidate: Vector2i in _stockpile_positions:
			var distance: int = maxi(absi(candidate.x - xs[i]), absi(candidate.y - ys[i]))
			if distance < best_distance:
				nearest = candidate
				best_distance = distance
		# Shared anchors display one sum; every settlement contributes exactly once.
		totals[nearest] = int(totals.get(nearest, 0)) + foods[i]
	return totals

func _update_stockpiles() -> void:
	var totals: Dictionary = _food_totals_by_stockpile(_world_sim.get_settlement_snapshot())
	var seen: Dictionary = {}
	_label_anchors.clear()
	for nearest: Vector2i in totals:
		var id: int = (nearest.x << 32) | nearest.y
		var label: Label = _labels.get(id) as Label
		if label == null:
			label = Label.new()
			label.z_index = 6
			label.set_meta("art_kind", "stockpile_label")
			label.set_meta("art_id", id)
			label.add_theme_font_override("font", load("res://assets/fonts/galmuri/Galmuri11.ttf") as Font)
			label.mouse_filter = Control.MOUSE_FILTER_IGNORE
			label.add_theme_font_size_override("font_size", 12)
			label.add_theme_color_override("font_outline_color", Color("141414"))
			label.add_theme_constant_override("outline_size", 3)
			add_child(label)
			_labels[id] = label
		label.text = (str(_locale.ltr(STOCKPILE_FOOD_KEY)) + " %d" % int(totals[nearest])) if _locale != null else str(int(totals[nearest]))
		_label_anchors[id] = _tile_center(nearest.x, nearest.y) + Vector2(0, -18)
		seen[id] = true
	_reap(_labels, seen)

func _update_labels() -> void:
	if _camera == null:
		return
	for id: int in _labels.keys():
		var label: Label = _labels[id] as Label
		label.scale = Vector2.ONE / _camera.zoom
		label.visible = _camera.zoom.x >= 2.0
		var label_size: Vector2 = label.get_minimum_size()
		label.position = Vector2(_label_anchors[id]) - Vector2(label_size.x * 0.5, label_size.y) * label.scale
