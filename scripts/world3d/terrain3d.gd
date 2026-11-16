# terrain3d.gd (World3D/Terrain, made by world3d.gd): the ground of the 3D view (M3D-1 scope 3).
# Purpose: chunked ArrayMeshes (params3d `terrain.chunk_tiles`, 32×32 tiles) built from get_terrain_snapshot: a smooth
#   base from the elevation byte (corner heights average the four tiles around a corner, `terrain.elevation_metres`
#   over 0–255), hill and mountain tiles raised (`terrain.raise` by terrain code, averaged at each corner so hills rise on
#   slopes; a dug tile of get_space is not raised and keeps sharp walls, so a dug room is cut into the hill), water tiles (`terrain.water_codes`) lowered to a bed
#   (`terrain.water_bed`, a ford half as deep) with one flat water surface per chunk at `terrain.water_level`
#   (shaders/world3d/water.gdshader). Where a tile edge stands higher than its neighbour's (dug rooms, the map edge) a
#   vertical skirt closes the step (cliff colour); steep slopes are shaded as rock in the shader. Ground colour is a vertex colour per corner from `terrain.colours` by terrain code, blended over
#   the four tiles of a corner, varied by the world-seeded value noise (View.value_noise), packed earth around the
#   camp hearths and on floors, dug floor colour and tilled soil on field tiles (get_field_tiles). The shared
#   ShaderMaterial (shaders/world3d/terrain.gdshader) takes the season tint, snow cover and the per-tile overlay image.
#   Chunks build progressively (nearest the camera first, a few per frame) and rebuild when the space version, the
#   field tiles or a reset change them.
# Bridge methods called: via SimReads get_terrain_snapshot, get_world_info, get_space, get_field_tiles,
#   get_building_snapshot; signal world_reset.
extends Node3D

const Data3D = preload("res://scripts/world3d/data3d.gd")
const View = preload("res://scripts/world/colony/view.gd")
const TERRAIN_SHADER = preload("res://shaders/world3d/terrain.gdshader")
const WATER_SHADER = preload("res://shaders/world3d/water.gdshader")
const CHUNKS_PER_FRAME: int = 3
const SIDES: Array[Vector2i] = [Vector2i(0, -1), Vector2i(1, 0), Vector2i(0, 1), Vector2i(-1, 0)]

## Every chunk has been meshed once; from then on rebuilds take World3D's heavy-work slot.
var _loaded: bool = false
var material: ShaderMaterial
var water_material: ShaderMaterial
var width: int = 0
var height: int = 0
var _world_sim: Node = null
var _chunk: int = 32
var _terrain: PackedByteArray = PackedByteArray()
var _elev: PackedByteArray = PackedByteArray()
## Height of the smooth base at each tile corner ((width + 1) × (height + 1)).
var _corner: PackedFloat32Array = PackedFloat32Array()
## Water dip of each tile corner (0 on dry land; see _compute_water_dip).
var _dip: PackedFloat32Array = PackedFloat32Array()
## Plateau raise averaged at each tile corner (see tile_corners).
var _craise: PackedFloat32Array = PackedFloat32Array()
## Raise of each tile above the base (plateaus +, water beds -).
var _raise: PackedFloat32Array = PackedFloat32Array()
var _colour: PackedColorArray = PackedColorArray()
var _dug: Dictionary = {}
var _floor: Dictionary = {}
var _ford: Dictionary = {}
var _field: Dictionary = {}
var _clearing: Dictionary = {}
var _space_version: int = -1
var _field_key: String = ""
var _dirty: Dictionary = {}
var _meshes: Dictionary = {}
var _water: Dictionary = {}
var _ready_world: bool = false
var _stamp_key: String = ""


func setup(world_sim: Node) -> void:
	_world_sim = world_sim
	_world_sim.connect("world_reset", _on_world_reset)


func _ready() -> void:
	material = ShaderMaterial.new()
	material.shader = TERRAIN_SHADER
	material.set_shader_parameter("cliff_colour", Data3D.colour(Data3D.param("terrain.cliff_colour", []), Color(0.43, 0.39, 0.34)))
	water_material = ShaderMaterial.new()
	water_material.shader = WATER_SHADER
	water_material.set_shader_parameter("water_colour", Data3D.colour(Data3D.param("terrain.water_colour", []), Color(0.16, 0.36, 0.45, 0.82)))


func _on_world_reset() -> void:
	_ready_world = false
	_loaded = false
	_space_version = -1
	_field_key = ""


## True once every chunk of the current world is built.
## Chunks of the map (columns × rows).
func _chunk_count() -> int:
	return ceili(float(width) / _chunk) * ceili(float(height) / _chunk) if width > 0 and height > 0 else 1 << 30


func is_complete() -> bool:
	return _ready_world and _dirty.is_empty()


func _process(_delta: float) -> void:
	if _world_sim == null:
		return
	if not _ready_world:
		_load_world()
		if not _ready_world:
			return
	var stamp: Dictionary = get_parent().get("stamp") if get_parent() != null else {}
	@warning_ignore("integer_division")
	var key: String = "%s|%s" % [stamp.get("overlay", -1), int(stamp.get("tick", -1)) / maxi(1, int(Data3D.param("flora.slow_read_ticks", 24)))]
	# After the first full build, recomputes and chunk meshing take World3D's heavy-work slot (one per frame).
	if _meshes.size() >= _chunk_count():
		_loaded = true
	if (key != _stamp_key or stamp.is_empty()) and (not _loaded or _claim()):
		_stamp_key = key
		_poll_space()
	elif _loaded and not _dirty.is_empty() and _claim():
		_build_some(1)
	if not _loaded:
		_build_some(CHUNKS_PER_FRAME)


func _claim() -> bool:
	return get_parent() == null or not get_parent().has_method("claim_heavy") or bool(get_parent().call("claim_heavy"))


func _load_world() -> void:
	var snap: Dictionary = SimReads.read(_world_sim, "get_terrain_snapshot")
	width = int(snap.get("width", 0))
	height = int(snap.get("height", 0))
	if width <= 0 or height <= 0:
		return
	_terrain = snap.get("terrain", PackedByteArray())
	_elev = snap.get("elevation", PackedByteArray())
	if _terrain.size() < width * height:
		return
	_chunk = maxi(4, int(Data3D.param("terrain.chunk_tiles", 32)))
	material.set_shader_parameter("map_size", Vector2(width, height))
	for child: Node in get_children():
		child.queue_free()
	_meshes.clear()
	_water.clear()
	_loaded = false
	_compute_corners()
	_ready_world = true
	_space_version = -1
	_poll_space()
	_mark_all()


func _mark_all() -> void:
	_dirty.clear()
	@warning_ignore("integer_division")
	var cw: int = (width + _chunk - 1) / _chunk
	@warning_ignore("integer_division")
	var ch: int = (height + _chunk - 1) / _chunk
	for cy: int in ch:
		for cx: int in cw:
			_dirty[Vector2i(cx, cy)] = true


func _compute_corners() -> void:
	var metres: float = float(Data3D.param("terrain.elevation_metres", 3.0))
	_corner.resize((width + 1) * (height + 1))
	for cy: int in height + 1:
		for cx: int in width + 1:
			var total: float = 0.0
			var n: int = 0
			for dy: int in [-1, 0]:
				for dx: int in [-1, 0]:
					var tx: int = clampi(cx + dx, 0, width - 1)
					var ty: int = clampi(cy + dy, 0, height - 1)
					total += float(_elev[ty * width + tx]) if _elev.size() == width * height else 128.0
					n += 1
			_corner[cy * (width + 1) + cx] = (total / float(n)) / 255.0 * metres


## Space/fields/hearths changed → recompute raises and colours, mark touched chunks.
func _poll_space() -> void:
	var space: Dictionary = SimReads.read(_world_sim, "get_space")
	# The space counter is the shared overlay counter (it moves with every hauled stack): key on the content.
	var version: int = hash([space.get("dug_xs"), space.get("dug_ys"), space.get("floor_xs"), space.get("floor_ys"),
		space.get("ford_xs"), space.get("ford_ys")])
	var fields: Dictionary = SimReads.read(_world_sim, "get_field_tiles")
	var fxs: PackedInt64Array = fields.get("xs", PackedInt64Array())
	var fphases: PackedInt64Array = fields.get("phases", PackedInt64Array())
	# Only which tiles are fields colours the ground (not their phase), and only lit hearths clear a camp: key on
	# those, so growing crops and new bedrolls do not recompute the whole map.
	var field_key: String = "%d:%d:%d" % [fxs.size(), hash(fxs), hash(fields.get("ys"))] + ":" + str(hash(_lit_hearths()))
	if version == _space_version and field_key == _field_key and not _raise.is_empty():
		return
	var dug: Dictionary = _tileset(space.get("dug_xs", PackedInt32Array()), space.get("dug_ys", PackedInt32Array()))
	var floor: Dictionary = _tileset(space.get("floor_xs", PackedInt32Array()), space.get("floor_ys", PackedInt32Array()))
	var ford: Dictionary = _tileset(space.get("ford_xs", PackedInt32Array()), space.get("ford_ys", PackedInt32Array()))
	var field: Dictionary = {}
	var fys: PackedInt64Array = fields.get("ys", PackedInt64Array())
	for i: int in mini(fxs.size(), fys.size()):
		field[Vector2i(int(fxs[i]), int(fys[i]))] = int(fphases[i]) if i < fphases.size() else 0
	var hearths: String = field_key.get_slice(":", 3)
	# Dug, floor and field tiles only change their own tile and its corners: recompute just those. Fords (water dip)
	# and lit hearths (camp clearings) recompute the map.
	var incremental: bool = not _raise.is_empty() and _raise.size() == width * height and ford.hash() == _ford.hash() \
		and hearths == _field_key.get_slice(":", 3)
	var changed: Dictionary = {}
	if incremental:
		for pair: Array in [[_dug, dug], [_floor, floor], [_field, field]]:
			for t: Vector2i in pair[0]:
				if not (pair[1] as Dictionary).has(t):
					changed[t.y * width + t.x] = true
			for t: Vector2i in pair[1]:
				if not (pair[0] as Dictionary).has(t):
					changed[t.y * width + t.x] = true
	var old_raise: PackedFloat32Array = _raise.duplicate() if not incremental else PackedFloat32Array()
	var old_colour: PackedColorArray = _colour.duplicate() if not incremental else PackedColorArray()
	_space_version = version
	_field_key = field_key
	_dug = dug
	_floor = floor
	_ford = ford
	_field = field
	if incremental:
		_compute_some(changed)
	else:
		_compute_clearing()
		_compute_tiles()
		if old_raise.size() != _raise.size():
			return
		for i: int in _raise.size():
			if old_raise[i] != _raise[i] or old_colour[i] != _colour[i]:
				changed[i] = true
	for i: int in changed:
		var tile: Vector2i = Vector2i(i % width, int(float(i) / float(width)))
		# A step changes the skirts of the neighbouring chunk too.
		for d: Vector2i in [Vector2i.ZERO, Vector2i(1, 0), Vector2i(-1, 0), Vector2i(0, 1), Vector2i(0, -1)]:
			var t: Vector2i = tile + d
			if t.x >= 0 and t.y >= 0 and t.x < width and t.y < height:
				@warning_ignore("integer_division")
				_dirty[Vector2i(t.x / _chunk, t.y / _chunk)] = true


static func _tileset(xs: Variant, ys: Variant) -> Dictionary:
	var out: Dictionary = {}
	if xs is PackedInt32Array and ys is PackedInt32Array:
		for i: int in mini((xs as PackedInt32Array).size(), (ys as PackedInt32Array).size()):
			out[Vector2i(xs[i], ys[i])] = true
	return out


func _compute_clearing() -> void:
	_clearing.clear()
	var buildings: Dictionary = SimReads.read(_world_sim, "get_building_snapshot")
	var bx: PackedInt32Array = buildings.get("xs", PackedInt32Array())
	var by: PackedInt32Array = buildings.get("ys", PackedInt32Array())
	var bk: PackedByteArray = buildings.get("kinds", PackedByteArray())
	var core: int = int(_clearing_param("core_tiles", 4))
	var reach: int = int(_clearing_param("reach_tiles", 8))
	var seed: int = int(SimReads.read(_world_sim, "get_world_info").get("seed", 0))
	for i: int in mini(bk.size(), mini(bx.size(), by.size())):
		# The lit hearth row of the 2D bindings marks a camp (no content name here).
		if not bool(_building_binding(int(bk[i])).get("light", false)):
			continue
		for dy: int in range(-reach, reach + 1):
			for dx: int in range(-reach, reach + 1):
				var x: int = bx[i] + dx
				var y: int = by[i] + dy
				if x < 0 or y < 0 or x >= width or y >= height:
					continue
				var d: float = sqrt(float(dx * dx + dy * dy))
				var edge: float = float(core) + float(reach - core) * View.value_noise(seed, x, y, 3, 11)
				if d <= edge:
					_clearing[Vector2i(x, y)] = maxf(float(_clearing.get(Vector2i(x, y), 0.0)), clampf((edge - d) / 2.0, 0.0, 1.0))


## Tiles of the camp-marking (lit) hearths of get_building_snapshot.
func _lit_hearths() -> Array[Vector2i]:
	var buildings: Dictionary = SimReads.read(_world_sim, "get_building_snapshot")
	var bx: PackedInt32Array = buildings.get("xs", PackedInt32Array())
	var by: PackedInt32Array = buildings.get("ys", PackedInt32Array())
	var bk: PackedByteArray = buildings.get("kinds", PackedByteArray())
	var out: Array[Vector2i] = []
	for i: int in mini(bk.size(), mini(bx.size(), by.size())):
		if bool(_building_binding(int(bk[i])).get("light", false)):
			out.append(Vector2i(bx[i], by[i]))
	return out


static func _clearing_param(key: String, fallback: Variant) -> Variant:
	var b: Dictionary = preload("res://scripts/world/art/art_data.gd").bindings()
	return ((b.get("terrain", {}) as Dictionary).get("clearing", {}) as Dictionary).get(key, fallback)


static func _building_binding(code: int) -> Dictionary:
	return preload("res://scripts/world/art/art_data.gd").binding("buildings", code)


func _compute_tiles() -> void:
	var n: int = width * height
	_raise.resize(n)
	_colour.resize(n)
	var raises: Dictionary = Data3D.param("terrain.raise", {})
	var water_codes: Array = Data3D.param("terrain.water_codes", [4])
	var bed: float = float(Data3D.param("terrain.water_bed", -0.55))
	var colours: Dictionary = Data3D.param("terrain.colours", {})
	var clearing_c: Color = Data3D.colour(Data3D.param("terrain.clearing_colour", []), Color(0.52, 0.44, 0.3))
	var dug_c: Color = Data3D.colour(Data3D.param("terrain.dug_colour", []), Color(0.4, 0.34, 0.28))
	var field_c: Color = Data3D.colour(Data3D.param("terrain.field_colour", []), Color(0.4, 0.29, 0.18))
	var ford_c: Color = Data3D.colour(Data3D.param("terrain.ford_colour", []), Color(0.55, 0.52, 0.45))
	var noise_scale: int = int(Data3D.param("terrain.noise_scale_tiles", 7))
	var noise_tint: float = float(Data3D.param("terrain.noise_tint", 0.1))
	var seed: int = int(SimReads.read(_world_sim, "get_world_info").get("seed", 0))
	_compute_water_dip(water_codes, bed)
	for i: int in n:
		_compute_tile(i, raises, water_codes, colours, clearing_c, dug_c, field_c, ford_c, noise_scale, noise_tint, seed)
	_compute_corner_raise()


## Recomputes only tiles `changed` (indices) and the corners around them; returns them for chunk marking.
func _compute_some(changed: Dictionary) -> void:
	var raises: Dictionary = Data3D.param("terrain.raise", {})
	var water_codes: Array = Data3D.param("terrain.water_codes", [4])
	var colours: Dictionary = Data3D.param("terrain.colours", {})
	var clearing_c: Color = Data3D.colour(Data3D.param("terrain.clearing_colour", []), Color(0.52, 0.44, 0.3))
	var dug_c: Color = Data3D.colour(Data3D.param("terrain.dug_colour", []), Color(0.4, 0.34, 0.28))
	var field_c: Color = Data3D.colour(Data3D.param("terrain.field_colour", []), Color(0.4, 0.29, 0.18))
	var ford_c: Color = Data3D.colour(Data3D.param("terrain.ford_colour", []), Color(0.55, 0.52, 0.45))
	var noise_scale: int = int(Data3D.param("terrain.noise_scale_tiles", 7))
	var noise_tint: float = float(Data3D.param("terrain.noise_tint", 0.1))
	var seed: int = int(SimReads.read(_world_sim, "get_world_info").get("seed", 0))
	var w1: int = width + 1
	for i: int in changed:
		_compute_tile(i, raises, water_codes, colours, clearing_c, dug_c, field_c, ford_c, noise_scale, noise_tint, seed)
	for i: int in changed:
		var tx: int = i % width
		@warning_ignore("integer_division")
		var ty: int = i / width
		for cy: int in [ty, ty + 1]:
			for cx: int in [tx, tx + 1]:
				var total: float = 0.0
				for dy: int in [-1, 0]:
					for dx: int in [-1, 0]:
						total += _raise[clampi(cy + dy, 0, height - 1) * width + clampi(cx + dx, 0, width - 1)]
				_craise[cy * w1 + cx] = total * 0.25


func _compute_tile(i: int, raises: Dictionary, water_codes: Array, colours: Dictionary, clearing_c: Color, dug_c: Color,
		field_c: Color, ford_c: Color, noise_scale: int, noise_tint: float, seed: int) -> void:
	var code: int = _terrain[i]
	var t: Vector2i = Vector2i(i % width, int(float(i) / float(width)))
	var r: float = 0.0
	if water_codes.has(code) or water_codes.has(float(code)):
		r = 0.0
	elif not _dug.has(t):
		r = float(raises.get(str(code), 0.0))
	_raise[i] = r
	var c: Color = Data3D.colour(colours.get(str(code), []), Color(0.4, 0.5, 0.3))
	if _ford.has(t):
		c = ford_c
	elif _dug.has(t):
		c = dug_c
	elif _field.has(t):
		c = field_c
	elif _floor.has(t):
		c = clearing_c
	elif _clearing.has(t) and r == 0.0:
		c = c.lerp(clearing_c, float(_clearing[t]))
	var v: float = 1.0 + (View.value_noise(seed, t.x, t.y, noise_scale, 5) - 0.5) * 2.0 * noise_tint
	_colour[i] = Color(c.r * v, c.g * v, c.b * v)


## Per-corner water dip: a corner sinks by the bed depth times the share of its four tiles that are water (a ford
## counts half), so river banks slope instead of stepping.
func _compute_water_dip(water_codes: Array, bed: float) -> void:
	_dip.resize((width + 1) * (height + 1))
	for cy: int in height + 1:
		for cx: int in width + 1:
			var wet: float = 0.0
			for dy: int in [-1, 0]:
				for dx: int in [-1, 0]:
					var tx: int = clampi(cx + dx, 0, width - 1)
					var ty: int = clampi(cy + dy, 0, height - 1)
					var code: int = _terrain[ty * width + tx]
					if water_codes.has(code) or water_codes.has(float(code)):
						wet += 0.5 if _ford.has(Vector2i(tx, ty)) else 1.0
			_dip[cy * (width + 1) + cx] = bed * clampf(wet / 3.0, 0.0, 1.0)


## The ground height under world point (x, z) in units (tile plateau or bed included).
func height_at(x: float, z: float) -> float:
	if _corner.is_empty() or width <= 0 or _raise.size() != width * height:
		return 0.0
	var tx: int = clampi(floori(x), 0, width - 1)
	var ty: int = clampi(floori(z), 0, height - 1)
	var fx: float = clampf(x - tx, 0.0, 1.0)
	var fz: float = clampf(z - ty, 0.0, 1.0)
	var h: PackedFloat32Array = tile_corners(tx, ty)
	return lerpf(lerpf(h[0], h[1], fx), lerpf(h[2], h[3], fx), fz)


## Heights of tile (tx, ty)'s corners [00, 10, 01, 11]: the smooth base plus the plateau raise averaged over the four
## tiles at each corner, so hills meet the plain on slopes; a dug tile and the corners it touches keep their own
## tile's raise, so dug rooms keep sharp walls.
func tile_corners(tx: int, ty: int) -> PackedFloat32Array:
	var w1: int = width + 1
	var i: int = ty * width + tx
	var out: PackedFloat32Array = PackedFloat32Array([0.0, 0.0, 0.0, 0.0])
	var k: int = 0
	for cy: int in [ty, ty + 1]:
		for cx: int in [tx, tx + 1]:
			var r: float = _raise[i] if _dug.has(Vector2i(tx, ty)) or _corner_touches_dug(cx, cy) else _craise[cy * w1 + cx]
			out[k] = _ch(cy * w1 + cx) + r
			k += 1
	return out


## Tile `n`'s own surface height at the lattice corner under point `p` (a shared edge corner).
func _edge_height(n: Vector2i, p: Vector3) -> float:
	var h: PackedFloat32Array = tile_corners(n.x, n.y)
	var k: int = (1 if roundi(p.x) > n.x else 0) + (2 if roundi(p.z) > n.y else 0)
	return h[k]


func _corner_touches_dug(cx: int, cy: int) -> bool:
	if _dug.is_empty():
		return false
	for t: Vector2i in [Vector2i(cx - 1, cy - 1), Vector2i(cx, cy - 1), Vector2i(cx - 1, cy), Vector2i(cx, cy)]:
		if _dug.has(t):
			return true
	return false


## Average plateau raise at each corner (skipping water beds, which dip through `_dip`).
func _compute_corner_raise() -> void:
	_craise.resize((width + 1) * (height + 1))
	for cy: int in height + 1:
		for cx: int in width + 1:
			var total: float = 0.0
			for dy: int in [-1, 0]:
				for dx: int in [-1, 0]:
					total += _raise[clampi(cy + dy, 0, height - 1) * width + clampi(cx + dx, 0, width - 1)]
			_craise[cy * (width + 1) + cx] = total * 0.25


## The ground height of tile `t` at its centre, ignoring water beds (where things stand).
func stand_height(t: Vector2i) -> float:
	return height_at(t.x + 0.5, t.y + 0.5)


## The smooth base (no plateau) at the centre of tile `t`.
func base_height(t: Vector2i) -> float:
	var r: float = _raise[t.y * width + t.x] if t.x >= 0 and t.y >= 0 and t.x < width and t.y < height and _raise.size() == width * height else 0.0
	return height_at(t.x + 0.5, t.y + 0.5) - r


func water_level_at(t: Vector2i) -> float:
	return height_at(t.x + 0.5, t.y + 0.5) - _dip_at(t) + float(Data3D.param("terrain.water_level", -0.18))


func _dip_at(t: Vector2i) -> float:
	var w1: int = width + 1
	return (_ch_dip(t.y * w1 + t.x) + _ch_dip(t.y * w1 + t.x + 1) + _ch_dip((t.y + 1) * w1 + t.x) + _ch_dip((t.y + 1) * w1 + t.x + 1)) * 0.25


## Corner height including the water dip.
func _ch(i: int) -> float:
	return _corner[i] + (_dip[i] if i < _dip.size() else 0.0)


func _ch_dip(i: int) -> float:
	return _dip[i] if i < _dip.size() else 0.0


func terrain_code(t: Vector2i) -> int:
	if t.x < 0 or t.y < 0 or t.x >= width or t.y >= height or _terrain.size() < width * height:
		return -1
	return _terrain[t.y * width + t.x]


func _build_some(count: int) -> void:
	if _dirty.is_empty():
		return
	var cam: Camera3D = get_viewport().get_camera_3d()
	var focus: Vector2 = Vector2(cam.global_position.x, cam.global_position.z) if cam != null else Vector2.ZERO
	var keys: Array = _dirty.keys()
	keys.sort_custom(func(a: Vector2i, b: Vector2i) -> bool:
		return focus.distance_squared_to((Vector2(a) + Vector2(0.5, 0.5)) * _chunk) < focus.distance_squared_to((Vector2(b) + Vector2(0.5, 0.5)) * _chunk))
	for k: int in mini(count, keys.size()):
		var key: Vector2i = keys[k]
		_dirty.erase(key)
		_build_chunk(key)


## Build every dirty chunk now (tools and tests).
func build_all() -> void:
	if not _ready_world and _world_sim != null:
		_load_world()
	_build_some(1 << 20)


func _corner_colour(cx: int, cy: int) -> Color:
	var total: Color = Color(0, 0, 0, 0)
	var n: float = 0.0
	for dy: int in [-1, 0]:
		for dx: int in [-1, 0]:
			var tx: int = cx + dx
			var ty: int = cy + dy
			if tx < 0 or ty < 0 or tx >= width or ty >= height:
				continue
			total += _colour[ty * width + tx]
			n += 1.0
	return total / maxf(1.0, n)


func _build_chunk(key: Vector2i) -> void:
	var x0: int = key.x * _chunk
	var y0: int = key.y * _chunk
	var x1: int = mini(width, x0 + _chunk)
	var y1: int = mini(height, y0 + _chunk)
	var verts: PackedVector3Array = PackedVector3Array()
	var cols: PackedColorArray = PackedColorArray()
	var water_verts: PackedVector3Array = PackedVector3Array()
	var cliff: Color = Data3D.colour(Data3D.param("terrain.cliff_colour", []), Color(0.43, 0.39, 0.34))
	var water_codes: Array = Data3D.param("terrain.water_codes", [4])
	var level: float = float(Data3D.param("terrain.water_level", -0.18))
	var w1: int = width + 1
	# Corner lattice of the chunk, computed once: shared corner heights (plateau raise averaged) and blended colours.
	# Tiles at a dug room use tile_corners (their corners keep their own raise).
	var lw: int = x1 - x0 + 1
	var lat_h: PackedFloat32Array = PackedFloat32Array()
	var lat_c: PackedColorArray = PackedColorArray()
	lat_h.resize(lw * (y1 - y0 + 1))
	lat_c.resize(lw * (y1 - y0 + 1))
	for cy: int in range(y0, y1 + 1):
		for cx: int in range(x0, x1 + 1):
			var li: int = (cy - y0) * lw + (cx - x0)
			lat_h[li] = _ch(cy * w1 + cx) + _craise[cy * w1 + cx]
			lat_c[li] = _corner_colour(cx, cy)
	for ty: int in range(y0, y1):
		for tx: int in range(x0, x1):
			var i: int = ty * width + tx
			var li: int = (ty - y0) * lw + (tx - x0)
			var hc: PackedFloat32Array
			if _dug.is_empty() or not (_dug.has(Vector2i(tx, ty)) or _corner_touches_dug(tx, ty) or _corner_touches_dug(tx + 1, ty)
					or _corner_touches_dug(tx, ty + 1) or _corner_touches_dug(tx + 1, ty + 1)):
				hc = PackedFloat32Array([lat_h[li], lat_h[li + 1], lat_h[li + lw], lat_h[li + lw + 1]])
			else:
				hc = tile_corners(tx, ty)
			var p00: Vector3 = Vector3(tx, hc[0], ty)
			var p10: Vector3 = Vector3(tx + 1, hc[1], ty)
			var p01: Vector3 = Vector3(tx, hc[2], ty + 1)
			var p11: Vector3 = Vector3(tx + 1, hc[3], ty + 1)
			var c00: Color = lat_c[li]
			var c10: Color = lat_c[li + 1]
			var c01: Color = lat_c[li + lw]
			var c11: Color = lat_c[li + lw + 1]
			# Clockwise seen from above (Godot front faces are clockwise); split along the flatter diagonal.
			if absf(hc[0] - hc[3]) <= absf(hc[1] - hc[2]):
				verts.append_array([p00, p10, p11, p00, p11, p01])
				cols.append_array([c00, c10, c11, c00, c11, c01])
			else:
				verts.append_array([p00, p10, p01, p10, p11, p01])
				cols.append_array([c00, c10, c01, c10, c11, c01])
			var wet_tile: bool = water_codes.has(_terrain[i]) or water_codes.has(float(_terrain[i]))
			if wet_tile or minf(minf(_ch_dip(ty * w1 + tx), _ch_dip(ty * w1 + tx + 1)), minf(_ch_dip((ty + 1) * w1 + tx), _ch_dip((ty + 1) * w1 + tx + 1))) < level:
				var l00: float = _corner[ty * w1 + tx] + level
				var l10: float = _corner[ty * w1 + tx + 1] + level
				var l01: float = _corner[(ty + 1) * w1 + tx] + level
				var l11: float = _corner[(ty + 1) * w1 + tx + 1] + level
				water_verts.append_array([Vector3(tx, l00, ty), Vector3(tx + 1, l10, ty), Vector3(tx + 1, l11, ty + 1),
					Vector3(tx, l00, ty), Vector3(tx + 1, l11, ty + 1), Vector3(tx, l01, ty + 1)])
			# Skirts only where this tile's edge stands above its neighbour's same edge (dug-room walls, the map edge).
			for side: int in 4:
				var n: Vector2i = Vector2i(tx, ty) + SIDES[side]
				var a: Vector3
				var b: Vector3
				match side:
					0:
						a = p10
						b = p00
					1:
						a = p11
						b = p10
					2:
						a = p01
						b = p11
					_:
						a = p00
						b = p01
				var na: float = a.y - 1.5
				var nb: float = b.y - 1.5
				if n.x >= 0 and n.y >= 0 and n.x < width and n.y < height:
					# Corners away from dug tiles share one height on both sides: no step there.
					if not _corner_touches_dug(roundi(a.x), roundi(a.z)) and not _corner_touches_dug(roundi(b.x), roundi(b.z)):
						continue
					na = _edge_height(n, a)
					nb = _edge_height(n, b)
				if a.y <= na + 0.01 and b.y <= nb + 0.01:
					continue
				var a2: Vector3 = Vector3(a.x, minf(a.y, na), a.z)
				var b2: Vector3 = Vector3(b.x, minf(b.y, nb), b.z)
				verts.append_array([a, b, b2, a, b2, a2])
				var shade: Color = cliff * (0.82 + 0.06 * side)
				cols.append_array([shade, shade, shade.darkened(0.15), shade, shade.darkened(0.15), shade.darkened(0.15)])
	var mi: MeshInstance3D = _meshes.get(key)
	if mi == null:
		mi = MeshInstance3D.new()
		mi.name = "Chunk_%d_%d" % [key.x, key.y]
		add_child(mi)
		_meshes[key] = mi
	mi.mesh = _surface(verts, cols, material)
	var wm: MeshInstance3D = _water.get(key)
	if water_verts.is_empty():
		if wm != null:
			wm.queue_free()
			_water.erase(key)
		return
	if wm == null:
		wm = MeshInstance3D.new()
		wm.name = "Water_%d_%d" % [key.x, key.y]
		wm.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
		add_child(wm)
		_water[key] = wm
	wm.mesh = _surface(water_verts, PackedColorArray(), water_material)


static func _surface(verts: PackedVector3Array, cols: PackedColorArray, mat: Material) -> ArrayMesh:
	var arrays: Array = []
	arrays.resize(Mesh.ARRAY_MAX)
	arrays[Mesh.ARRAY_VERTEX] = verts
	var normals: PackedVector3Array = PackedVector3Array()
	normals.resize(verts.size())
	for t: int in range(0, verts.size(), 3):
		var nrm: Vector3 = (verts[t + 2] - verts[t]).cross(verts[t + 1] - verts[t]).normalized()
		normals[t] = nrm
		normals[t + 1] = nrm
		normals[t + 2] = nrm
	arrays[Mesh.ARRAY_NORMAL] = normals
	if not cols.is_empty():
		arrays[Mesh.ARRAY_COLOR] = cols
	var mesh: ArrayMesh = ArrayMesh.new()
	if verts.is_empty():
		return mesh
	mesh.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, arrays)
	mesh.surface_set_material(0, mat)
	return mesh
