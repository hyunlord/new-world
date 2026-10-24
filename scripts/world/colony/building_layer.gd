# building_layer.gd (ColonyView/BuildingLayer in scenes/main.tscn): the hearth, shelters, finished huts, finished
#   workshops and bedrolls of the colony view at 64-px tiles (M-ART2 §3.5 "things"). Floors, walls, doors and fords
#   of the built space are the ground layer's (ground_layer.gd).
# Every thing is a standing Sprite2D in the y-sorted ColonyView/World (position = its base point, offset = −anchor,
#   nearest filtering), so it depth-sorts with people, animals, trees and wall strips by base y. Sprites come from
#   bindings.json by sim code: `buildings` (hearth: `structure` lit, `unlit` when its settlement's fire is out —
#   get_settlement_snapshot `fire_lit` of the settlement whose formation tile is nearest —, `firing` while the hearth's
#   fire-pot workshop, the `workshops` row with `marks_hearth_firing`, works a bill; shelter `structure` on its tile),
#   `workshops` (`structure`, or `working` while a bill is active; base at the footprint's bottom centre:
#   x = left·64 + w·32, y = (top + h − 1)·64 + 43) and `space.bed` (bedrolls of get_space, footprint 2 × 1 from the
#   bed tile). A finished hut (building kind with a `ring` row) is not in get_space: the ring's walls are assembled
#   by wall_baker.gd through site_ghost.gd (the material resolver on the ring's `wall_style`) into one strip per row,
#   its door stands as a sprite (open while a person is on it) and its floor tiles are drawn flat by this node under
#   the world layer (z −8). A building snapshot row inside a workshop footprint is skipped (finished workshops sit in
#   the building registry without a kind and read as kind 0); the camp hearth's fire-pot workshop (a `workshops` row
#   with no structure) is not drawn.
# Rebuilt when the building snapshot, the workshop list or the settlement fire state changes, or the space version
#   moves (bedrolls); F12 (params) rebuilds everything.
# Bridge methods: via SimReads get_building_snapshot, get_workshops, get_space, get_settlement_snapshot,
#   get_agent_snapshot; signal world_reset.
extends Node2D

const View = preload("res://scripts/world/colony/view.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
const SiteGhost = preload("res://scripts/world/art/site_ghost.gd")
const TILE: int = View.TILE

var _world_sim: Node = null
var _world: Node2D = null
var _key: int = -1
var _space_version: int = -1
var _params_version: int = -1
## [building code, tile] per drawn building (hearths, shelters, huts).
var _rows: Array[Array] = []
## [workshop code, footprint Rect2i, working] per drawn workshop.
var _shops: Array[Array] = []
## Bed tiles of the built space and the number of walls (for tests and tools).
var _beds: Array[Vector2i] = []
var _wall_count: int = 0
var _floor_pieces: int = 0
## Key → Sprite2D of every standing thing (pooled by key).
var _sprites: Dictionary = {}
## Hut door tile → [Sprite2D, door style].
var _doors: Dictionary = {}
## Hut floor tiles (world) → floor style, drawn flat by this node.
var _floor_tiles: Dictionary = {}
var _floor_tex: Dictionary = {}


func _ready() -> void:
	z_as_relative = false
	z_index = -8
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_world = get_node_or_null("../World")
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	if _world_sim != null:
		_world_sim.connect("world_reset", _on_world_reset)


func _on_world_reset() -> void:
	_key = -1
	_space_version = -1
	_clear()


func _clear() -> void:
	for s: Node in _sprites.values():
		s.queue_free()
	_sprites.clear()
	for d: Array in _doors.values():
		(d[0] as Node).queue_free()
	_doors.clear()
	_floor_tiles.clear()
	queue_redraw()


func _process(_delta: float) -> void:
	if _world_sim == null or _world == null:
		return
	if _params_version != ArtData.params_version():
		_params_version = ArtData.params_version()
		_key = -1
		_space_version = -1
		_floor_tex.clear()
		_clear()
	var snap: Dictionary = SimReads.read(_world_sim, "get_building_snapshot")
	var shops: Dictionary = SimReads.read(_world_sim, "get_workshops")
	var settlements: Dictionary = SimReads.read(_world_sim, "get_settlement_snapshot")
	var space: Dictionary = SimReads.read(_world_sim, "get_space")
	var space_version: int = int(space.get("version", 0))
	var key: int = hash([snap.get("xs"), snap.get("ys"), snap.get("kinds"), shops.get("kinds"), shops.get("xs"), shops.get("ys"),
		shops.get("bill_active"), settlements.get("fire_lit"), space_version])
	if key != _key:
		_key = key
		_space_version = space_version
		_rebuild(snap, shops, settlements, space)
	if not _doors.is_empty():
		_update_doors()


func _rebuild(snap: Dictionary, shops: Dictionary, settlements: Dictionary, space: Dictionary) -> void:
	var wanted: Dictionary = {}
	# Workshops (get_workshops): footprint, structure or its working sprite; the fire-pot row marks the hearth firing.
	_shops.clear()
	var firing_tiles: Dictionary = {}
	var footprints: Array[Rect2i] = []
	var kinds: PackedInt32Array = shops.get("kinds", PackedInt32Array())
	var sxs: PackedInt32Array = shops.get("xs", PackedInt32Array())
	var sys: PackedInt32Array = shops.get("ys", PackedInt32Array())
	var sws: PackedInt32Array = shops.get("ws", PackedInt32Array())
	var shs: PackedInt32Array = shops.get("hs", PackedInt32Array())
	var active: Dictionary = _active_shops(shops)
	for i: int in mini(mini(kinds.size(), sxs.size()), sys.size()):
		var rect: Rect2i = Rect2i(sxs[i], sys[i], sws[i] if i < sws.size() else 1, shs[i] if i < shs.size() else 1)
		var row: Dictionary = ArtData.binding("workshops", kinds[i])
		if bool(row.get("marks_hearth_firing", false)):
			if active.has(i):
				firing_tiles[rect.position] = true
			continue
		footprints.append(rect)
		var id: Variant = row.get("structure", null)
		if not (id is String):
			continue
		var working: bool = active.has(i) and row.get("working", null) is String
		var sprite_id: String = str(row["working"]) if working else str(id)
		_shops.append([kinds[i], rect, working])
		wanted["w%d_%d" % [rect.position.x, rect.position.y]] = [sprite_id, SiteGhost.footprint_base(rect.position, rect.size)]
	# Buildings (hearths, shelters, huts).
	_rows.clear()
	var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
	var bkinds: PackedByteArray = snap.get("kinds", PackedByteArray())
	var huts: Array[Array] = []
	for i: int in mini(bkinds.size(), mini(xs.size(), ys.size())):
		var tile: Vector2i = Vector2i(xs[i], ys[i])
		if _inside_any(tile, footprints):
			continue
		var row: Dictionary = ArtData.binding("buildings", bkinds[i])
		if row.has("ring"):
			huts.append([tile, row["ring"]])
			_rows.append([int(bkinds[i]), tile])
			continue
		var id: Variant = row.get("structure", null)
		if not (id is String):
			continue
		var sprite_id: String = str(id)
		if bool(row.get("light", false)):
			if firing_tiles.has(tile) and row.get("firing", null) is String:
				sprite_id = str(row["firing"])
			elif not _fire_lit(settlements, tile) and row.get("unlit", null) is String:
				sprite_id = str(row["unlit"])
		_rows.append([int(bkinds[i]), tile])
		wanted["b%d_%d" % [tile.x, tile.y]] = [sprite_id, View.base_point(tile)]
	# Bedrolls of the built space (2 × 1 from the bed tile by bindings space.bed).
	var bed: Dictionary = (ArtData.bindings().get("space", {}) as Dictionary).get("bed", {})
	var fp: Variant = bed.get("footprint", [1, 1])
	var bed_size: Vector2i = Vector2i(int(fp[0]), int(fp[1])) if fp is Array and (fp as Array).size() == 2 else Vector2i.ONE
	_beds.clear()
	var bxs: PackedInt32Array = space.get("bed_xs", PackedInt32Array())
	var bys: PackedInt32Array = space.get("bed_ys", PackedInt32Array())
	for i: int in mini(bxs.size(), bys.size()):
		var t: Vector2i = Vector2i(bxs[i], bys[i])
		_beds.append(t)
		# Lying flat: one pixel above the base line so a person on the tile draws over it.
		wanted["d%d_%d" % [t.x, t.y]] = [str(bed.get("structure", "")), SiteGhost.footprint_base(t, bed_size) - Vector2(0, 1)]
	_wall_count = (space.get("wall_xs", PackedInt32Array()) as PackedInt32Array).size()
	_floor_pieces = (space.get("floor_xs", PackedInt32Array()) as PackedInt32Array).size() + _beds.size() \
		+ (space.get("ford_xs", PackedInt32Array()) as PackedInt32Array).size()
	_sync(wanted)
	_build_huts(huts)


## Workshop row indices with an active bill.
func _active_shops(shops: Dictionary) -> Dictionary:
	var out: Dictionary = {}
	var bill_shops: PackedInt32Array = shops.get("bill_shops", PackedInt32Array())
	var bill_active: PackedInt32Array = shops.get("bill_active", PackedInt32Array())
	for i: int in mini(bill_shops.size(), bill_active.size()):
		if bill_active[i] != 0:
			out[bill_shops[i]] = true
	return out


## Whether the fire of the settlement whose formation tile is nearest `tile` burns (true without settlements).
func _fire_lit(settlements: Dictionary, tile: Vector2i) -> bool:
	var fxs: PackedInt32Array = settlements.get("formation_xs", PackedInt32Array())
	var fys: PackedInt32Array = settlements.get("formation_ys", PackedInt32Array())
	var lit: PackedByteArray = settlements.get("fire_lit", PackedByteArray())
	var best: int = -1
	var best_d: int = 1 << 30
	for i: int in mini(mini(fxs.size(), fys.size()), lit.size()):
		var d: int = absi(fxs[i] - tile.x) + absi(fys[i] - tile.y)
		if d < best_d:
			best_d = d
			best = i
	return best < 0 or lit[best] != 0


## Keep one sprite per wanted key ([manifest id, base point]); unchanged keys keep their node.
func _sync(wanted: Dictionary) -> void:
	for k: String in _sprites.keys():
		var s: Sprite2D = _sprites[k]
		var w: Variant = wanted.get(k, null)
		if w == null or str(s.get_meta("id", "")) != str(w[0]) or s.position != (w[1] as Vector2).round():
			s.queue_free()
			_sprites.erase(k)
	for k: String in wanted.keys():
		if _sprites.has(k):
			continue
		var id: String = str(wanted[k][0])
		var tex: Texture2D = ArtData.texture(id)
		if tex == null:
			continue
		var s: Sprite2D = Sprite2D.new()
		s.texture = tex
		s.centered = false
		s.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
		s.position = (wanted[k][1] as Vector2).round()
		s.offset = -Vector2(ArtData.anchor(id))
		s.set_meta("id", id)
		_world.add_child(s)
		_sprites[k] = s


## Finished huts: ring walls as row strips, the door as a sprite, the floor tiles drawn flat by this node.
func _build_huts(huts: Array[Array]) -> void:
	for k: String in _sprites.keys():
		if k.begins_with("h"):
			(_sprites[k] as Node).queue_free()
			_sprites.erase(k)
	for d: Array in _doors.values():
		(d[0] as Node).queue_free()
	_doors.clear()
	_floor_tiles.clear()
	for hut: Array in huts:
		var centre: Vector2i = hut[0]
		var ring: Dictionary = hut[1]
		var rt: Dictionary = SiteGhost.ring_tiles(ring)
		var size: Vector2i = rt["size"]
		var origin: Vector2i = centre - Vector2i(floori((size.x - 1) / 2.0), floori((size.y - 1) / 2.0))
		var walls: Dictionary = {}
		for t: Vector2i in (rt["walls"] as Dictionary).keys():
			walls[t + origin] = rt["walls"][t]
		var doors: Dictionary = {}
		for t: Vector2i in (rt["doors"] as Dictionary).keys():
			doors[t + origin] = rt["doors"][t]
		var rows: Dictionary = SiteGhost.wall_rows(walls, doors)
		for ty: int in rows.keys():
			var r: Dictionary = rows[ty]
			var strip: Sprite2D = Sprite2D.new()
			strip.texture = ImageTexture.create_from_image(r["image"])
			strip.centered = false
			strip.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
			strip.position = Vector2(int(r["x0"]) * TILE, ty * TILE + View.BASE.y)
			strip.offset = Vector2(0, -(SiteGhost.CAP + View.BASE.y))
			_world.add_child(strip)
			_sprites["h%d_%d_%d" % [centre.x, centre.y, ty]] = strip
		for t: Vector2i in doors.keys():
			var door: Sprite2D = Sprite2D.new()
			door.centered = false
			door.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
			door.position = Vector2(t.x * TILE, t.y * TILE + View.BASE.y)
			door.offset = Vector2(0, -(SiteGhost.CAP + View.BASE.y))
			door.texture = _door_texture(str(doors[t]), false)
			_world.add_child(door)
			_doors[t] = [door, str(doors[t])]
		for t: Vector2i in rt["floors"]:
			_floor_tiles[t + origin] = str(ring.get("floor_style", ""))
	queue_redraw()


func _door_texture(style: String, open: bool) -> Texture2D:
	var key: String = "door|%s|%s" % [style, str(open)]
	if not _floor_tex.has(key):
		var img: Image = SiteGhost.door_image(style, open)
		_floor_tex[key] = ImageTexture.create_from_image(img) if img != null else null
	return _floor_tex[key]


## Hut doors open while a person stands on them.
func _update_doors() -> void:
	var agents: Dictionary = SimReads.read(_world_sim, "get_agent_snapshot")
	var axs: PackedInt32Array = agents.get("xs", PackedInt32Array())
	var ays: PackedInt32Array = agents.get("ys", PackedInt32Array())
	var occupied: Dictionary = {}
	for i: int in mini(axs.size(), ays.size()):
		var t: Vector2i = Vector2i(axs[i], ays[i])
		if _doors.has(t):
			occupied[t] = true
	for t: Vector2i in _doors.keys():
		var d: Array = _doors[t]
		(d[0] as Sprite2D).texture = _door_texture(str(d[1]), occupied.has(t))


func _draw() -> void:
	for t: Vector2i in _floor_tiles.keys():
		var style: String = str(_floor_tiles[t])
		var key: String = "floor|%s|%d|%d" % [style, posmod(t.x, 4), posmod(t.y, 4)]
		if not _floor_tex.has(key):
			var img: Image = SiteGhost.floor_tile(style, t)
			_floor_tex[key] = ImageTexture.create_from_image(img) if img != null else null
		var tex: Texture2D = _floor_tex[key]
		if tex != null:
			draw_texture(tex, View.tile_to_world(t))


static func _inside_any(tile: Vector2i, rects: Array[Rect2i]) -> bool:
	for rect: Rect2i in rects:
		if rect.has_point(tile):
			return true
	return false


## Number of wall and door tiles of the built space (drawn by the ground layer; for tests and tools).
func wall_count() -> int:
	return _wall_count


## Number of floor-level pieces of the built space (floors, fords, bedrolls; for tests and tools).
func floor_piece_count() -> int:
	return _floor_pieces


## Number of huts drawn (for tests and tools).
func hut_count() -> int:
	var n: int = 0
	for row: Array in _rows:
		if ArtData.binding("buildings", int(row[0])).has("ring"):
			n += 1
	return n


## Number of hearths drawn (buildings whose binding gives light; for tests and tools).
func hearth_count() -> int:
	var n: int = 0
	for row: Array in _rows:
		if bool(ArtData.binding("buildings", int(row[0])).get("light", false)):
			n += 1
	return n


## Number of workshops drawn (for tests and tools).
func workshop_count() -> int:
	return _shops.size()


## The hearth's fire frame (the M-ART2 hearth has one lit frame; kept for tools): always 0.
func fire_frame() -> int:
	return 0
