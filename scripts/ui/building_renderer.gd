extends Node2D

const TILE_SIZE: int = 16
const SPRITE_ORIGIN: Vector2 = Vector2(448, 28)
const RECONCILE_FRAMES: int = 30
const STOCKPILE_FOOD_KEY: String = "HUD_STOCKPILE_FOOD"
const BUILDING_PATHS: Array[String] = [
	"res://assets/sprites/buildings/campfire/1.png",
	"res://assets/sprites/furniture/lean_to/1.png",
	"res://assets/sprites/buildings/stockpile/1.png",
]
var _world_sim: WorldSimNode
var _locale: Node
var _camera: Camera2D
var _buildings: Dictionary = {}
var _sites: Dictionary = {}
var _bushes: Dictionary = {}
var _labels: Dictionary = {}
var _label_anchors: Dictionary = {}
var _building_textures: Array[Texture2D] = []
var _berry_textures: Array[Texture2D] = []
var _stockpile_positions: Array[Vector2i] = []
var _frame: int = 0

func _ready() -> void:
	z_index = 5
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_world_sim = get_node("../WorldSim") as WorldSimNode
	_locale = get_node_or_null("/root/Locale")
	_camera = get_node("../Camera2D") as Camera2D
	for path: String in BUILDING_PATHS:
		_building_textures.append(load(path) as Texture2D)
	for stage: int in range(4):
		_berry_textures.append(load("res://assets/sprites/resources/berry_bush_%d.png" % stage) as Texture2D)
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
	return SPRITE_ORIGIN + Vector2(x, y) * TILE_SIZE + Vector2.ONE * (TILE_SIZE / 2.0)

func _sprite_for(nodes: Dictionary, id: int) -> Sprite2D:
	var sprite: Sprite2D = nodes.get(id) as Sprite2D
	if sprite == null:
		sprite = Sprite2D.new()
		add_child(sprite)
		nodes[id] = sprite
	return sprite

func _reap(nodes: Dictionary, seen: Dictionary) -> void:
	for id: int in nodes.keys():
		if not seen.has(id):
			var stale: Node = nodes[id] as Node
			stale.queue_free()
			nodes.erase(id)

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
		sprite.texture = _building_textures[kind]
		sprite.position = _tile_center(xs[i], ys[i])
		seen[ids[i]] = true
		if kind == 2:
			_stockpile_positions.append(Vector2i(xs[i], ys[i]))
	_reap(_buildings, seen)

func _update_construction_sites() -> void:
	var snapshot: Dictionary = _world_sim.get_construction_snapshot()
	var ids: PackedInt64Array = snapshot.get("ids", PackedInt64Array())
	var xs: PackedInt32Array = snapshot.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snapshot.get("ys", PackedInt32Array())
	var progresses: PackedInt32Array = snapshot.get("progresses", PackedInt32Array())
	var required: PackedInt32Array = snapshot.get("required_progresses", PackedInt32Array())
	var seen: Dictionary = {}
	for i: int in range(ids.size()):
		var site: Node2D = _sites.get(ids[i]) as Node2D
		if site == null:
			site = Node2D.new()
			var sprite: Sprite2D = Sprite2D.new()
			sprite.texture = _building_textures[1]
			sprite.modulate.a = 0.35
			site.add_child(sprite)
			var track: ColorRect = ColorRect.new()
			track.color = Color("3a3a3a")
			track.position = Vector2(-7, 17)
			track.size = Vector2(14, 2)
			track.mouse_filter = Control.MOUSE_FILTER_IGNORE
			site.add_child(track)
			var fill: ColorRect = ColorRect.new()
			fill.name = "Progress"
			fill.color = Color("e0c060")
			fill.position = track.position
			fill.mouse_filter = Control.MOUSE_FILTER_IGNORE
			site.add_child(fill)
			add_child(site)
			_sites[ids[i]] = site
		site.position = _tile_center(xs[i], ys[i])
		var progress: ColorRect = site.get_node("Progress") as ColorRect
		var ratio: float = clampf(float(progresses[i]) / float(maxi(required[i], 1)), 0.0, 1.0)
		progress.size = Vector2(14.0 * ratio, 2)
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
		bush.texture = _berry_textures[stage]
		bush.position = _tile_center(xs[i], ys[i])
		bush.z_index = -1
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
			label.mouse_filter = Control.MOUSE_FILTER_IGNORE
			label.add_theme_font_size_override("font_size", 12)
			label.add_theme_color_override("font_outline_color", Color("141414"))
			label.add_theme_constant_override("outline_size", 3)
			add_child(label)
			_labels[id] = label
		label.text = str(_locale.ltr(STOCKPILE_FOOD_KEY)) + " %d" % int(totals[nearest])
		_label_anchors[id] = _tile_center(nearest.x, nearest.y) + Vector2(0, -18)
		seen[id] = true
	_reap(_labels, seen)

func _update_labels() -> void:
	for id: int in _labels.keys():
		var label: Label = _labels[id] as Label
		label.scale = Vector2.ONE / _camera.zoom
		label.visible = _camera.zoom.x >= 2.0
		var label_size: Vector2 = label.get_minimum_size()
		label.position = Vector2(_label_anchors[id]) - Vector2(label_size.x * 0.5, label_size.y) * label.scale
