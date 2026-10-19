extends Node2D

const ART = preload("res://scripts/ui/art_sprite_library.gd")
const ORIGIN := Vector2(448,28)
var _world: WorldSimNode
var _objects: Node2D
var _agents: Node
var _elapsed: float = 0.0
var _previous: Dictionary = {}
var _last_dust: Dictionary = {}
var _effects: Array[Dictionary] = []
var _persistent: Dictionary = {}
var _rng := RandomNumberGenerator.new()
var _next_birds: float = 15.0
var _next_fish: float = 25.0
var _next_insect: float = 8.0
var _river: Array[Vector2i] = []
var _flowers: Array[Vector2] = []

func _ready() -> void:
	_world = get_node_or_null("../WorldSim") as WorldSimNode
	_objects = get_node_or_null("../WorldObjects") as Node2D
	_agents = get_node_or_null("../AgentRenderer")
	if _world != null:
		_world.world_reset.connect(_reset)
		_reset()

func _reset() -> void:
	for effect: Dictionary in _effects:
		var sprite: Sprite2D = effect["sprite"] as Sprite2D
		if is_instance_valid(sprite):
			sprite.free()
	_effects.clear()
	for sprite: Sprite2D in _persistent.values():
		if is_instance_valid(sprite):
			sprite.free()
	_persistent.clear()
	_elapsed = 0.0
	_next_birds = 15.0
	_next_fish = 25.0
	_next_insect = 8.0
	_previous.clear()
	_last_dust.clear()
	if _world == null:
		return
	_rng.seed = int(SimReads.read(_world, "get_world_info").get("seed",1))
	var terrain: Dictionary = SimReads.read(_world, "get_terrain_snapshot")
	var width: int = int(terrain.get("width",64))
	var types: PackedByteArray = terrain.get("terrain",PackedByteArray())
	_river.clear()
	for i: int in range(types.size()):
		if types[i] == 4:
			_river.append(Vector2i(i % width, int(floor(float(i) / float(width)))))

func _spawn(ids: Array[String], at: Vector2, duration: float, velocity: Vector2 = Vector2.ZERO, agent_owned: bool = false) -> void:
	if ids.is_empty() or ART.frame(ids[0]).is_empty():
		return
	var sprite := Sprite2D.new()
	_objects.add_child(sprite)
	ART.apply(sprite,ids[0])
	sprite.position = at.round()
	sprite.set_meta("art_kind","decorative_effect")
	_effects.append({"sprite":sprite,"frames":ids,"born":_elapsed,"duration":duration,"velocity":velocity,"origin":at,"agent_owned":agent_owned})

func _fixed(key: String, id: String, position: Vector2, seen: Dictionary) -> void:
	if ART.frame(id).is_empty():
		return
	var sprite: Sprite2D = _persistent.get(key) as Sprite2D
	if sprite == null:
		sprite = Sprite2D.new()
		_objects.add_child(sprite)
		_persistent[key] = sprite
	ART.apply(sprite,id)
	sprite.position = position.round()
	sprite.set_meta("art_kind","decorative_effect")
	seen[key] = true

func _process(delta: float) -> void:
	if _world == null or _objects == null:
		return
	_elapsed += delta
	var hour: float = float(SimReads.read(_world, "get_time_snapshot").get("time_of_day",12.0))
	var seen: Dictionary = {}
	var sleeping_tiles: Dictionary = {}
	var capture_visible: bool = _agents != null and bool(_agents.call("is_capture_visible"))
	var active: Array = _agents.call("get_active_agent_sprites") if _agents != null else []
	var live_ids: Dictionary = {}
	for sprite: Sprite2D in active:
		var id: int = int(sprite.get_meta("art_id",0))
		live_ids[id] = true
		var activity: String = str(sprite.get_meta("activity","idle"))
		var old: String = str(_previous.get(id,"idle"))
		if capture_visible and activity in ["walk","carry"] and old not in ["walk","carry"] and _elapsed-float(_last_dust.get(id,-3.0)) >= 2.0:
			_spawn(["F1_07","F1_08","F1_09"],sprite.position,0.5,Vector2.ZERO,true)
			_last_dust[id] = _elapsed
		if capture_visible and activity == "drink" and old != "drink":
			_spawn(["F1_10","F1_11","F1_12"],sprite.position+Vector2(3,0),0.65,Vector2.ZERO,true)
		if capture_visible and activity == "asleep":
			if bool(sprite.get_meta("sheltered_sleep",false)):
				sleeping_tiles[Vector2i(int(sprite.get_meta("tile_x",0)),int(sprite.get_meta("tile_y",0)))] = true
			else:
				_fixed("bed_%d" % id,"S2_08",sprite.position+Vector2(0,-1),seen)
		_previous[id] = activity
	for id: int in _previous.keys():
		if not live_ids.has(id):
			_previous.erase(id)
			_last_dust.erase(id)
	for child: Node in _objects.get_children():
		if not child is Sprite2D or child.get_meta("art_kind","") != "building":
			continue
		var sprite: Sprite2D = child as Sprite2D
		var id: int = int(sprite.get_meta("art_id",0))
		var kind: int = int(sprite.get_meta("building_kind",-1))
		if kind == 0 and bool(sprite.get_meta("fire_lit", false)):
			_fixed("smoke_%d" % id,"F1_%02d" % (1+int(_elapsed*4.0)%4),sprite.position+Vector2(0,-22),seen)
			if hour >= 19.0:
				_fixed("spark_%d" % id,"F1_%02d" % (5+int(_elapsed*6.0)%2),sprite.position+Vector2(2,-16),seen)
		elif kind == 1 and (hour>=20.0 or hour<6.0):
			var tile := Vector2i(int(sprite.get_meta("tile_x",0)),int(sprite.get_meta("tile_y",0)))
			if sleeping_tiles.has(tile) and int(_elapsed*2.0)%8<2:
				_fixed("sleep_%d" % id,"F1_%02d" % (15+int(_elapsed*4.0)%2),sprite.position+Vector2(5,-20),seen)
	for key: String in _persistent.keys():
		if not seen.has(key):
			(_persistent[key] as Sprite2D).queue_free()
			_persistent.erase(key)
	for index: int in range(_effects.size()-1,-1,-1):
		var effect: Dictionary = _effects[index]
		var age: float = _elapsed-float(effect["born"])
		var sprite: Sprite2D = effect["sprite"]
		if age >= float(effect["duration"]) or (bool(effect.get("agent_owned",false)) and not capture_visible):
			sprite.queue_free()
			_effects.remove_at(index)
			continue
		var frames: Array[String] = effect["frames"]
		ART.apply(sprite,frames[int(age*8.0)%frames.size()])
		sprite.position = (Vector2(effect["origin"])+Vector2(effect["velocity"])*age).round()
	_ambient(hour)

func _ambient(hour: float) -> void:
	if hour>=6.0 and hour<18.0 and _elapsed>=_next_birds:
		_next_birds = _elapsed+_rng.randf_range(20,40)
		var y: float = _rng.randi_range(100,850)
		for i: int in range(4):
			_spawn(["F2_01","F2_02","F2_03","F2_04"],ORIGIN+Vector2(-i*12,y+i%2*8),20,Vector2(55,0))
	if _river.is_empty():
		return
	if _elapsed>=_next_fish:
		_next_fish = _elapsed+_rng.randf_range(30,60)
		var tile: Vector2i = _river[_rng.randi_range(0,_river.size()-1)]
		_spawn(["F2_09","F2_10","F2_11","F2_12"],ORIGIN+Vector2(tile)*16+Vector2(8,8),0.6)
	if _elapsed>=_next_insect:
		_next_insect = _elapsed+_rng.randf_range(5,12)
		var tile: Vector2i = _river[_rng.randi_range(0,_river.size()-1)]
		if hour>=17.0 and hour<21.0:
			_spawn(["F2_13","F2_14"],ORIGIN+Vector2(tile)*16+Vector2(8,2),3,Vector2(3,-1))
		elif hour>=6.0 and hour<18.0:
			_spawn(["F1_13","F1_14"],ORIGIN+Vector2(tile)*16+Vector2(8,8),0.4)
			var decals: Node = get_node_or_null("../TerrainRenderer/Decals")
			if decals != null:
				var flowers: Array[Sprite2D] = []
				for child: Node in decals.get_children():
					if child is Sprite2D and str(child.get_meta("sprite_id", "")) in ["N3_05","N3_06","N3_07","N3_08"]:
						flowers.append(child as Sprite2D)
				if not flowers.is_empty():
					var flower: Sprite2D = flowers[_rng.randi_range(0,flowers.size()-1)]
					_spawn(["F2_05","F2_06","F2_07","F2_08"],flower.global_position+Vector2(0,-5),3,Vector2(2,-1))
