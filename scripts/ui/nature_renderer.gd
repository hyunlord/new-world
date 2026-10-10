extends Node2D

const ART = preload("res://scripts/ui/art_sprite_library.gd")
const HASH = preload("res://scripts/ui/art_hash.gd")
const ORIGIN: Vector2 = Vector2(448, 28)
var _world_sim: WorldSimNode
var _objects: Node2D
var _shadows: Node2D
var _shadow_texture: Texture2D
var _sprites: Dictionary = {}
var _contacts: Dictionary = {}
var _terrain: PackedByteArray
var _elevation: PackedByteArray
var _width: int = 0
var _height: int = 0
var _fingerprint: int = -1
var _frame: int = 0

func _ready() -> void:
	_world_sim = get_node("../WorldSim") as WorldSimNode
	_objects = ART.shared_layer(self, "WorldObjects", 4, true)
	_shadows = ART.shared_layer(self, "Shadows", 3)
	_shadow_texture = load("res://assets/art/contact_shadow.png") as Texture2D
	_world_sim.connect("world_reset", refresh_snapshot)
	refresh_snapshot()

func _process(_delta: float) -> void:
	_frame += 1
	if _frame % 30 == 0:
		_reconcile()

func refresh_snapshot() -> void:
	var terrain: Dictionary = _world_sim.get_terrain_snapshot()
	_terrain = terrain.get("terrain", PackedByteArray())
	_elevation = terrain.get("elevation", PackedByteArray())
	_width = int(terrain.get("width", 0))
	_height = int(terrain.get("height", 0))
	_fingerprint = -1
	_reconcile()

func _code(x: int, y: int) -> int:
	if x < 0 or y < 0 or x >= _width or y >= _height:
		return -1
	return _terrain[y*_width+x]

func _tree_id(x: int, y: int, h: int) -> String:
	var altitude: int = _elevation[y*_width+x] if _elevation.size() > y*_width+x else 0
	if altitude > 150:
		return "N1_%02d" % (6+h%2)
	var species: int = HASH.mix(h) % 100
	if species < 60:
		return "N1_%02d" % (1+h%3)
	if species < 95:
		return "N1_%02d" % (4+h%2)
	return "N1_08" if species < 98 else "N1_16"

func _place(key: String, art_id: String, fallback: String, kind: String, tile: Vector2i, jitter: Vector2i, seen: Dictionary) -> void:
	var sprite: Sprite2D = _sprites.get(key) as Sprite2D
	if sprite == null:
		sprite = Sprite2D.new()
		_objects.add_child(sprite)
		_sprites[key] = sprite
		var shadow: Sprite2D = Sprite2D.new()
		shadow.texture = _shadow_texture
		shadow.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
		shadow.modulate.a = 0.4
		shadow.set_meta("art_kind", "object_shadow")
		shadow.set_meta("art_id", key)
		_shadows.add_child(shadow)
		_contacts[key] = shadow
	var chosen: String = fallback if ART.frame(art_id).is_empty() else art_id
	ART.apply(sprite, chosen)
	sprite.position = ORIGIN + Vector2(tile*16+Vector2i(8,15)+jitter)
	sprite.set_meta("art_kind", kind)
	sprite.set_meta("art_id", key)
	sprite.set_meta("tile_x", tile.x)
	sprite.set_meta("tile_y", tile.y)
	sprite.set_meta("jitter", jitter)
	(_contacts[key] as Sprite2D).position = sprite.position
	seen[key] = true

func _reconcile() -> void:
	var buildings: Dictionary = _world_sim.get_building_snapshot()
	var sites: Dictionary = _world_sim.get_construction_snapshot()
	var food: Dictionary = _world_sim.get_resource_snapshot()
	var fingerprint: int = hash([buildings, sites, food.get("xs"), food.get("ys"), food.get("kinds")])
	if fingerprint == _fingerprint:
		return
	_fingerprint = fingerprint
	var excluded: Dictionary = {}
	for snapshot: Dictionary in [buildings, sites]:
		var xs: PackedInt32Array = snapshot.get("xs", PackedInt32Array())
		var ys: PackedInt32Array = snapshot.get("ys", PackedInt32Array())
		for i: int in range(xs.size()):
			excluded[Vector2i(xs[i],ys[i])] = true
	var food_tiles: Dictionary = {}
	var food_xs: PackedInt32Array = food.get("xs", PackedInt32Array())
	var food_ys: PackedInt32Array = food.get("ys", PackedInt32Array())
	var food_kinds: PackedInt32Array = food.get("kinds", PackedInt32Array())
	for i: int in range(food_xs.size()):
		if food_kinds[i] == 0:
			food_tiles[Vector2i(food_xs[i],food_ys[i])] = true
	var seen: Dictionary = {}
	for y: int in range(_height):
		for x: int in range(_width):
			var tile: Vector2i = Vector2i(x,y)
			if excluded.has(tile):
				continue
			var h: int = ((x*73856093) ^ (y*19349663)) & 0x7fffffff
			var code: int = _code(x,y)
			var jitter: Vector2i = Vector2i((h>>8)%7-3,(h>>12)%5-2)
			var key: String = "%d,%d" % [x,y]
			if code == 1 and not food_tiles.has(tile):
				var count: int = 0 if h%100 < 12 else (1 if h%100 < 85 else 2)
				for n: int in range(count):
					var shift: Vector2i = jitter if n == 0 else Vector2i(5 if h%2 == 0 else -5,3 if (h>>1)%2 == 0 else -3)
					_place(key+":tree%d" % n, _tree_id(x,y,h+n), "direction_14_young_tree", "tree",tile,shift,seen)
			if (code == 3 and h%100 < 25) or code == 2:
				var index: int = (10+h%2 if h%100 < 40 else 8+h%2) if code == 2 else 5+h%5
				_place(key+":rock", "N2_%02d" % index, "direction_13_boulder", "rock",tile,jitter,seen)
			if code != 4 and h%100 < 18:
				for d: Vector2i in [Vector2i.LEFT,Vector2i.RIGHT,Vector2i.UP,Vector2i.DOWN]:
					if _code(x+d.x,y+d.y) == 4:
						_place(key+":reed", "N2_%02d" % (13+h%4), "direction_15_reed_tuft", "reed",tile,jitter,seen)
						break
	for key: String in _sprites.keys():
		if not seen.has(key):
			(_sprites[key] as Sprite2D).queue_free()
			(_contacts[key] as Sprite2D).queue_free()
			_sprites.erase(key)
			_contacts.erase(key)
