extends Node2D

const TILE_SIZE: int = 16
const SPRITE_ORIGIN: Vector2 = Vector2(448, 28)
const RECONCILE_FRAMES: int = 30
const WOOD_SHORT_TINT: Color = Color(0.96, 0.62, 0.32)
const STOCKPILE_FOOD_KEY: String = "HUD_STOCKPILE_FOOD"
const SHELTER_COLD = preload("res://scripts/world/shelter_cold.gd")
const WOOD = preload("res://scripts/world/wood_visuals.gd")
const HASH = preload("res://scripts/world/art_hash.gd")
const ART = preload("res://scripts/world/art_sprite_library.gd")
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
var _deadwood: Dictionary = {}
var _labels: Dictionary = {}
var _label_anchors: Dictionary = {}
var _objects: Node2D
var _shadows: Node2D
var _shadow_texture: Texture2D
var _object_shadows: Dictionary = {}
var _stockpile_positions: Array[Vector2i] = []
var _frame: int = 0
var _elapsed: float = 0.0
var _props: Dictionary = {}
var _death_cache: Dictionary = {}
var _last_tick: int = -1
var _snapshot_tick: int = -1
var _chill_tint: ColorRect
var _tint_strength: float = 0.0
var _tint_target: float = 0.0
var _tint_start: float = 0.0
var _tint_elapsed: float = 0.0
var _sleeper_world: Node
var _sleeper_inputs: Array = []
var _sleeper_counts: Dictionary = {}

func _ready() -> void:
	z_index = 5
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_world_sim = get_node("../WorldSim") as WorldSimNode
	_locale = get_node_or_null("/root/Locale")
	if _locale != null:
		_locale.connect("language_changed", _refresh_stockpile_language)
	_camera = get_node("../Camera2D") as Camera2D
	_objects = ART.shared_layer(self, "WorldObjects", 4, true)
	_shadows = ART.shared_layer(self, "Shadows", 3)
	_shadow_texture = load("res://assets/art/contact_shadow.png") as Texture2D
	_setup_chill_tint()
	_world_sim.connect("world_reset", _reset_cold)
	_world_sim.connect("world_reset", refresh_snapshot)
	refresh_snapshot()

func _process(delta: float) -> void:
	_elapsed += delta
	_frame += 1
	var tick: int = int(SimReads.read(_world_sim, "get_time_snapshot").get("tick", -1))
	# Resource stocks and amounts must match the same simulation tick as the actors.
	if tick != _snapshot_tick or _frame % RECONCILE_FRAMES == 0 or _buildings.is_empty():
		refresh_snapshot()
	_animate_hearths()
	_update_shelter_cold(delta)
	_update_cairns()
	_update_labels()

func refresh_snapshot() -> void:
	_snapshot_tick = int(SimReads.read(_world_sim, "get_time_snapshot").get("tick", -1))
	_update_buildings()
	_update_construction_sites()
	_update_bushes()
	_update_deadwood()
	_update_stockpiles()
	_update_camp_props()
	_update_labels()

func _refresh_stockpile_language(_language: String) -> void:
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
	if not ART.apply(sprite, _available(sprite_id)) and not ART.is_suppressed(sprite_id):
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
	shadow.visible = sprite.texture != null

func _update_buildings() -> void:
	var snapshot: Dictionary = SimReads.read(_world_sim, "get_building_snapshot")
	var settlements: Dictionary = SimReads.read(_world_sim, "get_settlement_snapshot")
	var hour: float = _hour()
	var ids: PackedInt64Array = snapshot.get("ids", PackedInt64Array())
	var xs: PackedInt32Array = snapshot.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snapshot.get("ys", PackedInt32Array())
	var kinds: PackedByteArray = snapshot.get("kinds", PackedByteArray())
	var seen: Dictionary = {}
	_stockpile_positions.clear()
	for i: int in range(ids.size()):
		var kind: int = kinds[i]
		var sprite: Sprite2D = _sprite_for(_buildings, ids[i])
		var desired: String = "S1_%02d" % (11 + posmod(HASH.mix(int(ids[i])), 3)) if kind == 1 else ("S1_08" if kind == 2 else WOOD.hearth_frame(WOOD.settlement(settlements, Vector2i(xs[i], ys[i])), hour, _elapsed))
		_configure(sprite, desired, "building", ids[i], xs[i], ys[i])
		sprite.set_meta("building_kind", kind)
		seen[ids[i]] = true
		if kind == 2:
			_stockpile_positions.append(Vector2i(xs[i], ys[i]))
	_reap(_buildings, seen)

func _update_construction_sites() -> void:
	var snapshot: Dictionary = SimReads.read(_world_sim, "get_construction_snapshot")
	var ids: PackedInt64Array = snapshot.get("ids", PackedInt64Array())
	var xs: PackedInt32Array = snapshot.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snapshot.get("ys", PackedInt32Array())
	var progresses: PackedInt32Array = snapshot.get("progresses", PackedInt32Array())
	var required: PackedInt32Array = snapshot.get("required_progresses", PackedInt32Array())
	var seen: Dictionary = {}
	for i: int in range(ids.size()):
		var site: Sprite2D = _sprite_for(_sites, ids[i])
		var stage: int = construction_stage(int(progresses[i]) if i < progresses.size() else 0, int(required[i]) if i < required.size() else 1)
		_configure(site, "S2_%02d" % (stage + 1), "site", ids[i], xs[i], ys[i])
		site.set_meta("building_kind", 1)
		seen[ids[i]] = true
	_reap(_sites, seen)

func _update_bushes() -> void:
	var snapshot: Dictionary = SimReads.read(_world_sim, "get_resource_snapshot")
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
		var species_b: bool = posmod(HASH.mix(id), 2) == 1
		_configure(bush, "N2_%02d" % (4-stage) if species_b and not ART.frame("N2_01").is_empty() else BUSH_IDS[stage], "bush", id, xs[i], ys[i])
		bush.set_meta("building_kind", -1)
		seen[id] = true
	_reap(_bushes, seen)

func _update_deadwood() -> void:
	var snapshot: Dictionary = SimReads.read(_world_sim, "get_resource_snapshot")
	var seen: Dictionary = {}
	var valid: bool = true
	for key: String in ["xs", "ys", "kinds", "amounts", "maxes"]:
		valid = valid and snapshot.get(key) is PackedInt32Array
	if valid:
		var xs: PackedInt32Array = snapshot["xs"]
		var ys: PackedInt32Array = snapshot["ys"]
		var kinds: PackedInt32Array = snapshot["kinds"]
		var amounts: PackedInt32Array = snapshot["amounts"]
		var maxes: PackedInt32Array = snapshot["maxes"]
		for i: int in range(mini(xs.size(), mini(ys.size(), mini(kinds.size(), mini(amounts.size(), maxes.size()))))):
			if kinds[i] != 3:
				continue
			var id: int = (xs[i] << 32) | ys[i]
			var stage: int = WOOD.deadwood_stage(amounts[i], maxes[i])
			var sprite: Sprite2D = _sprite_for(_deadwood, id)
			var frame: String = WOOD.deadwood_frame(stage)
			_configure(sprite, frame, "deadwood", id, xs[i], ys[i])
			sprite.modulate.a = 0.35 if stage == 0 and frame == "S2_15" else 1.0
			sprite.set_meta("deadwood_stage", stage)
			sprite.set_meta("resource_amount", amounts[i])
			sprite.set_meta("resource_max", maxes[i])
			var shadow: Sprite2D = _object_shadows.get(sprite.get_instance_id()) as Sprite2D
			if shadow != null:
				shadow.modulate.a = sprite.modulate.a * 0.4
			seen[id] = true
	_reap(_deadwood, seen)

func _food_totals_by_stockpile(snapshot: Dictionary, column: String = "food_stocks") -> Dictionary:
	var xs: PackedInt32Array = snapshot.get("formation_xs", PackedInt32Array())
	var ys: PackedInt32Array = snapshot.get("formation_ys", PackedInt32Array())
	var foods: PackedInt32Array = snapshot.get(column, PackedInt32Array()) if snapshot.get(column) is PackedInt32Array else PackedInt32Array()
	var totals: Dictionary = {}
	if _stockpile_positions.is_empty():
		return totals
	for i: int in range(mini(foods.size(), mini(xs.size(), ys.size()))):
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
	var settlement_snapshot: Dictionary = SimReads.read(_world_sim, "get_settlement_snapshot")
	var totals: Dictionary = _food_totals_by_stockpile(settlement_snapshot)
	var woods: Dictionary = _food_totals_by_stockpile(settlement_snapshot, "wood_stocks")
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
			var wood_label: Label = label.duplicate() as Label
			wood_label.name = "Wood"
			wood_label.set_meta("art_kind", "stockpile_wood_label")
			label.add_child(wood_label)
		label.text = (str(_locale.ltr(STOCKPILE_FOOD_KEY)) + " %d" % int(totals[nearest])) if _locale != null else str(int(totals[nearest]))
		var wood_label: Label = label.get_node("Wood") as Label
		var target: int = wood_target(settlement_snapshot, nearest)
		var stock: int = int(woods.get(nearest, 0))
		wood_label.visible = woods.has(nearest)
		wood_label.text = wood_caption(str(_locale.ltr("HUD_STOCKPILE_WOOD")) if _locale != null else "", stock, target)
		wood_label.modulate = wood_tint(stock, target)
		wood_label.set_meta("wood_stock", stock)
		wood_label.set_meta("wood_target", target)
		label.set_meta("food_stock", int(totals[nearest]))
		label.set_meta("wood_stock", int(woods.get(nearest, -1)))
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
		var wood_label: Label = label.get_node("Wood") as Label
		wood_label.position = Vector2(label.get_minimum_size().x + 8.0, 0.0)
		var label_size: Vector2 = label.get_minimum_size()
		if wood_label.visible:
			label_size.x += 8.0 + wood_label.get_minimum_size().x
		label.position = Vector2(_label_anchors[id]) - Vector2(label_size.x * 0.5, label_size.y) * label.scale

func _hour() -> float:
	return float(SimReads.read(_world_sim, "get_time_snapshot").get("time_of_day", 12.0)) if _world_sim != null else 12.0

static func construction_stage(progress: int, required: int) -> int:
	return clampi(int(floor(4.0 * float(progress) / float(maxi(required, 1)))), 0, 3)

static func hearth_frame(hour: float, elapsed: float) -> String:
	return WOOD.hearth_frame({}, hour, elapsed)

static func storage_frame(food: int) -> String:
	return "S1_08" if food <= 0 else ("S1_09" if food < 40 else "S1_10")

static func cairn_alpha(age: int, day_ticks: int) -> float:
	return clampf((1.0 - float(age) / float(maxi(day_ticks, 1))) * 4.0, 0.0, 1.0)

func _available(sprite_id: String) -> String:
	if ART.is_suppressed(sprite_id):
		return sprite_id
	if not ART.frame(sprite_id).is_empty():
		return sprite_id
	if sprite_id.begins_with("S1_"):
		var index: int = int(sprite_id.trim_prefix("S1_"))
		return BUILDING_IDS[0] if index < 8 else (BUILDING_IDS[2] if index < 11 else BUILDING_IDS[1])
	if sprite_id.begins_with("S2_"):
		return "direction_04_shelter_frame" if int(sprite_id.trim_prefix("S2_")) <= 4 else "direction_16_berry_basket"
	return sprite_id

func _animate_hearths() -> void:
	if _world_sim == null:
		return
	var settlements: Dictionary = SimReads.read(_world_sim, "get_settlement_snapshot")
	var clock: Dictionary = SimReads.read(_world_sim, "get_time_snapshot")
	var hour: float = float(clock.get("time_of_day", 12.0))
	var winter_severity: int = int(clock.get("winter_severity", -1))
	for sprite: Sprite2D in _buildings.values():
		if int(sprite.get_meta("building_kind", -1)) == 0:
			var tile := Vector2i(int(sprite.get_meta("tile_x")), int(sprite.get_meta("tile_y")))
			var state: Dictionary = WOOD.settlement(settlements, tile)
			ART.apply(sprite, _available(WOOD.hearth_frame(state, hour, _elapsed, winter_severity)))
			_update_cold_ash(sprite, WOOD.cold_ash(state, hour, winter_severity))
			sprite.set_meta("fire_lit", state.get("fire_lit", false))
			sprite.set_meta("wood_stock", state.get("wood_stock", -1))
			sprite.set_meta("settlement_id", state.get("settlement_id", -1))

func _prop(id: int, art_id: String, tile: Vector2i, offset: Vector2, seen: Dictionary, occupied: Dictionary = {}) -> void:
	var entry: Dictionary = ART.frame(art_id)
	if entry.is_empty():
		return
	var rect: Array = entry["rect"]
	var anchor: Array = entry["anchor"]
	if not prop_footprint_allowed(tile, offset, int(rect[2]), int(anchor[0]), occupied):
		return
	var prop: Sprite2D = _sprite_for(_props, id)
	_configure(prop, art_id, "camp_prop", id, tile.x, tile.y)
	prop.position += offset
	var shadow: Sprite2D = _object_shadows.get(prop.get_instance_id()) as Sprite2D
	if shadow != null:
		shadow.position = prop.position
	seen[id] = true

func _update_camp_props() -> void:
	var seen: Dictionary = {}
	var shelters: Array[Sprite2D] = []
	var occupied: Dictionary = {}
	for sprite: Sprite2D in _buildings.values():
		occupied[Vector2i(int(sprite.get_meta("tile_x")), int(sprite.get_meta("tile_y")))] = true
		if int(sprite.get_meta("building_kind")) == 1:
			shelters.append(sprite)
	for site: Sprite2D in _sites.values():
		occupied[Vector2i(int(site.get_meta("tile_x")), int(site.get_meta("tile_y")))] = true
	var settlements: Dictionary = SimReads.read(_world_sim, "get_settlement_snapshot")
	var totals: Dictionary = _food_totals_by_stockpile(settlements)
	for sprite: Sprite2D in _buildings.values():
		var id: int = int(sprite.get_meta("art_id")) * 100
		var tile := Vector2i(int(sprite.get_meta("tile_x")), int(sprite.get_meta("tile_y")))
		var kind: int = int(sprite.get_meta("building_kind"))
		if kind == 0:
			var state: Dictionary = WOOD.settlement(settlements, tile)
			var stage: int = WOOD.pile_stage(int(state["wood_stock"])) if state.has("wood_stock") else -1
			if stage != 0:
				if stage > 0:
					_place_woodpile(id+1, WOOD.pile_frame(stage), tile, occupied, seen)
				else:
					_place_adjacent(id+1, "S1_14", tile, Vector2i(-1,0), occupied, seen)
				if seen.has(id+1):
					(_props[id+1] as Sprite2D).set_meta("woodpile_stage", stage)
			sprite.set_meta("woodpile_stage", stage)
			var offsets: Array[Vector2i] = [Vector2i(0,1), Vector2i(1,0), Vector2i(0,-1), Vector2i(-1,1)]
			for j: int in range(mini(shelters.size(),4)):
				_place_adjacent(id+2+j, "S1_15", tile, offsets[j], occupied, seen)
		elif kind == 2:
			var food: int = int(totals.get(tile,0))
			ART.apply(sprite, _available(storage_frame(food)))
			for j: int in range(clampi(int(floor(float(food) / 25.0)), 0, 6)):
				_prop(id+10+j,"S2_11",tile,Vector2(-15+j*6,9),seen,occupied)
			_place_adjacent(id+20,"S2_07",tile,Vector2i(1,0),occupied,seen)
	if not shelters.is_empty():
		var first: Sprite2D = shelters[0]
		var tile := Vector2i(int(first.get_meta("tile_x")),int(first.get_meta("tile_y")))
		var id: int = int(first.get_meta("art_id"))*100
		if shelters.size() >= 2:
			_place_adjacent(id+30,"S2_05",tile,Vector2i(-1,0),occupied,seen)
		if shelters.size() >= 4:
			_place_adjacent(id+31,"S2_06",tile,Vector2i(1,0),occupied,seen)
	for site: Sprite2D in _sites.values():
		var tile := Vector2i(int(site.get_meta("tile_x")),int(site.get_meta("tile_y")))
		var id: int = -100000-int(site.get_meta("art_id"))*100
		_prop(id,"S2_15",tile,Vector2(-18,4),seen,occupied)
		_prop(id-1,"S2_16",tile,Vector2(18,4),seen,occupied)
	_reap(_props,seen)

func _place_woodpile(id: int, art_id: String, tile: Vector2i, occupied: Dictionary, seen: Dictionary) -> void:
	var directions: Array[Vector2i] = [Vector2i(-1,0), Vector2i(-1,1), Vector2i(0,1), Vector2i(1,1), Vector2i(1,0), Vector2i(1,-1), Vector2i(0,-1), Vector2i(-1,-1)]
	for radius: int in [1,2]:
		for direction: Vector2i in directions:
			var target: Vector2i = tile + direction * radius
			if occupied.has(target):
				continue
			_prop(id, art_id, target, Vector2.ZERO, seen, occupied)
			if seen.has(id):
				occupied[target] = true
				return

func _place_adjacent(id: int, art_id: String, tile: Vector2i, delta: Vector2i, occupied: Dictionary, seen: Dictionary) -> void:
	var target: Vector2i = tile+delta
	if occupied.has(target):
		return
	_prop(id,art_id,target,Vector2.ZERO,seen,occupied)
	occupied[target] = true

func _update_cairns() -> void:
	if _world_sim == null:
		return
	var deaths: Dictionary = SimReads.read(_world_sim, "get_recent_deaths")
	var tick: int = int(deaths.get("current_tick",0))
	if tick < _last_tick:
		for record: Dictionary in _death_cache.values():
			_remove_cairn(record["sprite"] as Sprite2D)
		_death_cache.clear()
	_last_tick = tick
	var xs: PackedInt32Array = deaths.get("xs",PackedInt32Array())
	var ys: PackedInt32Array = deaths.get("ys",PackedInt32Array())
	var ticks: PackedInt64Array = deaths.get("ticks",PackedInt64Array())
	for i: int in range(xs.size()):
		var key: String = "%d:%d:%d" % [xs[i],ys[i],ticks[i]]
		if _death_cache.has(key) or ART.frame("S2_12").is_empty():
			continue
		var cairn := Sprite2D.new()
		_objects.add_child(cairn)
		_configure(cairn,"S2_%02d" % (12+posmod(HASH.mix(xs[i]*65536+ys[i]),2)),"cairn",HASH.mix(xs[i]*65536+ys[i]),xs[i],ys[i])
		cairn.set_meta("art_kind","cairn")
		_death_cache[key] = {"sprite":cairn,"tick":int(ticks[i])}
	var day_ticks: int = int(SimReads.read(_world_sim, "get_world_info").get("ticks_per_day",7200))
	for key: String in _death_cache.keys():
		var record: Dictionary = _death_cache[key]
		var age: int = tick-int(record["tick"])
		var sprite: Sprite2D = record["sprite"]
		sprite.modulate.a = cairn_alpha(age,day_ticks)
		var shadow: Sprite2D = _object_shadows.get(sprite.get_instance_id()) as Sprite2D
		if shadow != null:
			shadow.modulate.a = sprite.modulate.a*0.4
		if age >= day_ticks:
			_remove_cairn(sprite)
			_death_cache.erase(key)

func _remove_cairn(sprite: Sprite2D) -> void:
	var key: int = sprite.get_instance_id()
	if _object_shadows.has(key):
		(_object_shadows[key] as Sprite2D).queue_free()
		_object_shadows.erase(key)
	sprite.queue_free()

static func prop_footprint_allowed(host: Vector2i, offset: Vector2, width: int, anchor_x: int, occupied: Dictionary) -> bool:
	var base: Vector2 = Vector2(8,15)+offset
	var left: int = int(floor((base.x-float(anchor_x))/16.0))
	var right: int = int(floor((base.x-float(anchor_x)+float(width)-1.0)/16.0))
	var top: int = int(floor((base.y-3.0)/16.0))
	var bottom: int = int(floor(base.y/16.0))
	for y: int in range(top,bottom+1):
		for x: int in range(left,right+1):
			var tile: Vector2i = host+Vector2i(x,y)
			if tile != host and occupied.has(tile):
				return false
	return true

static func wood_caption(caption: String, stock: int, target: int) -> String:
	return caption + (" %d/%d" % [stock, target] if target > 0 else " %d" % stock)

static func wood_tint(stock: int, target: int) -> Color:
	return WOOD_SHORT_TINT if target > 0 and stock < target else Color.WHITE

static func wood_target(snapshot: Dictionary, anchor: Vector2i) -> int:
	var targets: Variant = snapshot.get("wood_targets")
	var xs: Variant = snapshot.get("formation_xs")
	var ys: Variant = snapshot.get("formation_ys")
	if not (targets is PackedInt32Array and xs is PackedInt32Array and ys is PackedInt32Array):
		return 0
	var best_distance: int = 2147483647
	var target: int = 0
	for index: int in mini(targets.size(), mini(xs.size(), ys.size())):
		var distance: int = maxi(absi(anchor.x - xs[index]), absi(anchor.y - ys[index]))
		if distance < best_distance:
			best_distance = distance
			target = int(targets[index])
	return target

func _setup_chill_tint() -> void:
	var layer := CanvasLayer.new()
	layer.name = "NightChillLayer"
	layer.layer = 0
	add_child(layer)
	_chill_tint = ColorRect.new()
	_chill_tint.name = "NightChillTint"
	_chill_tint.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_chill_tint.color = Color(0.80, 0.88, 1.0, 0.0)
	_chill_tint.set_meta("art_kind", "night_chill_tint")
	layer.add_child(_chill_tint)
	_chill_tint.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)

func _reset_cold() -> void:
	_sleeper_world = null
	_sleeper_inputs.clear()
	_sleeper_counts.clear()
	_tint_strength = 0.0
	_tint_target = 0.0
	_tint_start = 0.0
	_tint_elapsed = 0.0
	if _chill_tint != null:
		_chill_tint.color.a = 0.0
	for sprite: Sprite2D in _buildings.values():
		if int(sprite.get_meta("building_kind", -1)) == 1:
			SHELTER_COLD.apply(sprite, 0, _elapsed, 0.4)

func _update_shelter_cold(delta: float) -> void:
	var agents: Dictionary = SimReads.read(_world_sim, "get_agent_snapshot") if _world_sim.has_method("get_agent_snapshot") else {}
	var buildings: Dictionary = SimReads.read(_world_sim, "get_building_snapshot")
	var inputs: Array = []
	for key: String in ["xs", "ys", "states", "target_kinds", "colds"]:
		inputs.append([agents.has(key), typeof(agents.get(key)), agents.get(key)])
	for key: String in ["xs", "ys", "kinds"]:
		inputs.append([buildings.has(key), typeof(buildings.get(key)), buildings.get(key)])
	if _sleeper_world != _world_sim or inputs != _sleeper_inputs:
		_sleeper_world = _world_sim
		_sleeper_inputs = inputs.duplicate(true)
		_sleeper_counts = SHELTER_COLD.sleeper_counts(agents, buildings)
	var counts: Dictionary = _sleeper_counts
	for sprite: Sprite2D in _buildings.values():
		if int(sprite.get_meta("building_kind", -1)) == 1:
			var tile := Vector2i(int(sprite.get_meta("tile_x")), int(sprite.get_meta("tile_y")))
			SHELTER_COLD.apply(sprite, int(counts.get(tile, 0)), _elapsed, delta)
	var target: float = 0.12 if not counts.is_empty() else 0.0
	if target != _tint_target:
		_tint_start = _tint_strength
		_tint_target = target
		_tint_elapsed = 0.0
	_tint_elapsed += maxf(delta, 0.0)
	_tint_strength = SHELTER_COLD.tint_strength(_tint_start, _tint_target, _tint_elapsed)
	set_meta("night_chill_strength", _tint_strength)
	if _chill_tint != null:
		_chill_tint.color.a = _tint_strength
		_chill_tint.set_meta("night_chill_strength", _tint_strength)

func _update_cold_ash(sprite: Sprite2D, cold: bool) -> void:
	sprite.set_meta("cold_ash", cold)
	var ash: Sprite2D = sprite.get_node_or_null("ColdAsh") as Sprite2D
	if ash == null and cold:
		ash = Sprite2D.new()
		ash.name = "ColdAsh"
		var pixels: Image = Image.create(3, 1, false, Image.FORMAT_RGBA8)
		pixels.fill(Color("8A8A8A"))
		ash.texture = ImageTexture.create_from_image(pixels)
		ash.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
		ash.set_meta("art_kind", "cold_ash")
		ash.set_meta("art_id", sprite.get_meta("art_id", 0))
		sprite.add_child(ash)
	if ash != null:
		ash.visible = cold
		ash.position = Vector2(0, -2)
