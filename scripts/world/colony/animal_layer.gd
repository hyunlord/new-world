# animal_layer.gd (ColonyView/AnimalLayer in scenes/main.tscn): the M5 animals and hunt zones (goal 3.2, 3.5).
# Each animal of get_animals is a pooled Sprite2D (`animal_wolf` for kind 0, `animal_deer` for kind 1; M5 temporary
#   sprites from tools/v8/gen_tiles.py), facing the way it last stepped east or west and easing from tile to tile like
#   the people (AgentLayer); a deer that may be hunted (`huntable`: its own mark or inside a hunt zone) carries the
#   yellow ring `mark_hunt` at its top-right (full for its own mark, dimmer when only a zone covers it). The hunt zones
#   (`zones`, x0 y0 x1 y1 quadruples) are drawn by the child "Zones" under the people as an amber frame with a faint
#   fill. Under the mouse an animal shows "사슴 · 돌아다님 · 30" (kind, state, hit points) on a dark backing box. A
#   carcass is not an animal: the kill's meat and hide lie as ground stacks (OverlayLayer).
# The drawn rect is the click target (`animal_at`): the order tool's hunt click (F7-3) and the draft's right-click attack.
# Bridge methods: via SimReads get_animals, get_world_info; signal world_reset.
extends Node2D

const View = preload("res://scripts/world/colony/view.gd")
const StoryText = preload("res://scripts/ui/story_text.gd")
const SPRITES: Array[String] = ["animal_wolf", "animal_deer"]
const FONT_PATH: String = "res://assets/fonts/galmuri/Galmuri11.ttf"
const FONT_SIZE: int = 12
const BACK: Color = Color(0.094, 0.082, 0.075, 0.92)
const BACK_LINE: Color = Color(0.275, 0.243, 0.212, 1.0)
const TEXT: Color = Color(0.96, 0.92, 0.84)
const ZONE: Color = Color(0.95, 0.72, 0.3, 0.9)
const ZONE_ALPHA: float = 0.12
const ZONE_MARK_ALPHA: float = 0.55

var _world_sim: Node = null
var _font: Font = null
var _zones: Node2D = null
var _animals: Dictionary = {}
var _read: Dictionary = {}
var _zone_rects: Array[Rect2i] = []
var _zones_key: int = 0
var _hover: int = -1
var _frame: int = 0


class Animal:
	extends Node2D
	var id: int = 0
	var kind: int = 0
	var tile: Vector2i = Vector2i.ZERO
	var from_pos: Vector2 = Vector2.ZERO
	var to_pos: Vector2 = Vector2.ZERO
	var elapsed: float = 1.0
	var duration: float = 0.25
	var seen: int = 0
	var body: Sprite2D
	var mark: Sprite2D


func _ready() -> void:
	z_index = 5
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_font = load(FONT_PATH) as Font
	_zones = Node2D.new()
	_zones.name = "Zones"
	_zones.z_index = -1
	_zones.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_zones.draw.connect(_draw_zones)
	add_child(_zones)
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	if _world_sim != null:
		_world_sim.connect("world_reset", _on_world_reset)


func _on_world_reset() -> void:
	for animal: Animal in _animals.values():
		animal.queue_free()
	_animals.clear()
	_zone_rects.clear()
	_zones_key = 0
	_zones.queue_redraw()


func _sprite(parent: Node2D, sprite_name: String) -> Sprite2D:
	var sprite: Sprite2D = Sprite2D.new()
	sprite.texture = View.texture()
	sprite.centered = false
	sprite.region_enabled = true
	sprite.region_rect = View.rect(sprite_name)
	parent.add_child(sprite)
	return sprite


func _process(delta: float) -> void:
	if _world_sim == null:
		return
	_frame += 1
	_read = SimReads.read(_world_sim, "get_animals")
	var ids: PackedInt64Array = _read.get("ids", PackedInt64Array())
	var kinds: PackedInt32Array = _read.get("kinds", PackedInt32Array())
	var xs: PackedInt32Array = _read.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = _read.get("ys", PackedInt32Array())
	var huntable: PackedByteArray = _read.get("huntable", PackedByteArray())
	var marked: PackedByteArray = _read.get("marked", PackedByteArray())
	var n: int = mini(mini(ids.size(), kinds.size()), mini(xs.size(), ys.size()))
	var duration: float = _step_seconds()
	for i: int in n:
		var id: int = ids[i]
		var tile: Vector2i = Vector2i(xs[i], ys[i])
		var animal: Animal = _animals.get(id)
		if animal == null:
			animal = Animal.new()
			animal.id = id
			animal.tile = tile
			animal.position = View.tile_to_world(tile)
			animal.from_pos = animal.position
			animal.to_pos = animal.position
			animal.body = _sprite(animal, SPRITES[clampi(kinds[i], 0, SPRITES.size() - 1)])
			animal.mark = _sprite(animal, "mark_hunt")
			add_child(animal)
			_animals[id] = animal
		elif tile != animal.tile:
			animal.from_pos = animal.position
			animal.to_pos = View.tile_to_world(tile)
			if tile.x != animal.tile.x:
				animal.body.flip_h = tile.x < animal.tile.x
			animal.tile = tile
			animal.elapsed = 0.0
			animal.duration = duration
		animal.kind = kinds[i]
		animal.seen = _frame
		var can_hunt: bool = i < huntable.size() and huntable[i] != 0
		animal.mark.visible = can_hunt
		animal.mark.modulate = Color(1, 1, 1, 1.0 if i < marked.size() and marked[i] != 0 else ZONE_MARK_ALPHA)
	for key: int in _animals.keys():
		var animal: Animal = _animals[key]
		if animal.seen != _frame:
			animal.queue_free()
			_animals.erase(key)
			continue
		if animal.elapsed < animal.duration:
			animal.elapsed = minf(animal.duration, animal.elapsed + delta)
			animal.position = animal.from_pos.lerp(animal.to_pos, animal.elapsed / animal.duration).round()
		else:
			animal.position = animal.to_pos
	_refresh_zones()
	var hover: int = animal_at(get_global_mouse_position())
	if hover != _hover:
		_hover = hover
		queue_redraw()
	elif _hover >= 0:
		queue_redraw()


## Hunt zones from the read (redrawn only when they change).
func _refresh_zones() -> void:
	var zones: PackedInt32Array = _read.get("zones", PackedInt32Array())
	var key: int = hash(zones)
	if key == _zones_key:
		return
	_zones_key = key
	_zone_rects.clear()
	for k: int in range(0, zones.size() - 3, 4):
		var a: Vector2i = Vector2i(mini(zones[k], zones[k + 2]), mini(zones[k + 1], zones[k + 3]))
		var b: Vector2i = Vector2i(maxi(zones[k], zones[k + 2]), maxi(zones[k + 1], zones[k + 3]))
		_zone_rects.append(Rect2i(a, b - a + Vector2i.ONE))
	_zones.queue_redraw()


func _draw_zones() -> void:
	for r: Rect2i in _zone_rects:
		var rect: Rect2 = Rect2(View.tile_to_world(r.position), Vector2(r.size) * View.TILE)
		_zones.draw_rect(rect, Color(ZONE, ZONE_ALPHA))
		_zones.draw_rect(rect, ZONE, false, 1.0)


func _draw() -> void:
	var animal: Animal = _animals.get(_hover)
	if animal == null or _font == null:
		return
	var i: int = (_read.get("ids", PackedInt64Array()) as PackedInt64Array).find(_hover)
	if i < 0:
		return
	var locale: Node = get_node_or_null("/root/Locale")
	var states: PackedInt32Array = _read.get("states", PackedInt32Array())
	var hps: PackedInt32Array = _read.get("hps", PackedInt32Array())
	var text: String = StoryText.trf(locale, "M5_ANIMAL_HOVER_FMT", {"animal": StoryText.animal(locale, animal.kind),
		"state": StoryText.animal_state(locale, states[i] if i < states.size() else -1), "hp": hps[i] if i < hps.size() else 0})
	var size: Vector2 = _font.get_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, -1, FONT_SIZE)
	var anchor: Vector2 = animal.position + Vector2(View.TILE * 0.5, -1)
	var box: Rect2 = Rect2(Vector2(anchor.x - size.x * 0.5 - 3, anchor.y - FONT_SIZE - 3), Vector2(size.x + 6, FONT_SIZE + 3))
	box.position = box.position.round()
	draw_rect(box, BACK)
	draw_rect(box, BACK_LINE, false, 1.0)
	draw_string(_font, box.position + Vector2(3, FONT_SIZE - 1), text, HORIZONTAL_ALIGNMENT_LEFT, -1, FONT_SIZE, TEXT)


## Seconds one step lasts at the current speed (as AgentLayer): move_interval_ticks / (12 × speed).
func _step_seconds() -> float:
	var info: Dictionary = SimReads.read(_world_sim, "get_world_info")
	var interval: float = float(info.get("move_interval_ticks", 3))
	var camera: Node = get_node_or_null("/root/Main/Camera2D")
	var speed: float = float(camera.call("get_speed")) if camera != null and camera.has_method("get_speed") else 1.0
	return maxf(0.02, interval / (12.0 * maxf(1.0, speed)))


## The AnimalId of the animal drawn at world position `p` (the lowest on screen wins), or -1.
func animal_at(p: Vector2) -> int:
	var best: int = -1
	var best_y: float = -INF
	for animal: Animal in _animals.values():
		if Rect2(animal.position, Vector2(View.TILE, View.TILE)).has_point(p) and animal.position.y > best_y:
			best = animal.id
			best_y = animal.position.y
	return best


## The kind of drawn animal `id` (0 wolf, 1 deer), -1 when not drawn.
func kind_of(id: int) -> int:
	var animal: Animal = _animals.get(id)
	return animal.kind if animal != null else -1


## The drawn top-left of animal `id` (for tools), or (-1, -1).
func drawn_position(id: int) -> Vector2:
	var animal: Animal = _animals.get(id)
	return animal.position if animal != null else Vector2(-1, -1)


## Whether animal `id` shows its hunt mark (for tools).
func marked_shown(id: int) -> bool:
	var animal: Animal = _animals.get(id)
	return animal != null and animal.mark.visible


## How many animals are drawn and how many hunt zones (for tests and tools).
func counts() -> Dictionary:
	return {"animals": _animals.size(), "zones": _zone_rects.size()}
