# structures3d.gd (World3D/Structures, made by world3d.gd): built space in 3D — walls, doors, fences, floors, roofs,
#   the cutaway and construction stages (M3D-1 scope 5).
# Purpose: every wall tile of get_space is extruded from the tile centre toward each wall/door/fence neighbour
#   (params3d `structures.wall_height`, `wall_thickness`), so corners, T and X joins come out of the geometry; the
#   profile comes from the tile's construction style through the 2D material resolver (scripts/world/art/
#   material_resolver.gd) and bindings3d `walls.by_style`: `log` stacks octagonal logs that cross and overhang
#   (`structures.log_overhang`) only at free corners (a T or X join stops flush), `block` is a masonry slab with mortar lines in the shader, `palisade` a row of pointed stakes. Colour is
#   the wall material (bindings3d `materials`) with the resolver's lightness tint. Wall tiles flagged as openings
#   (`wall_is_door`) with an opening neighbour are fence runs: the M3D-ASSETS fence model of the tile's join mask
#   (bindings3d `fences.by_mask`, bit 1 north, 2 east, 4 south, 8 west toward any wall/door/fence neighbour), one
#   MultiMesh per model, else procedural posts and two rails; other openings are doors: two
#   posts and a lintel in the wall, and a door leaf (model `door`) that swings open while a person stands on it and is cut down with the walls. A
#   finished hut (2D building binding with a `ring`) gets its ring of walls and its door from that binding. Floors are
#   thin slabs in the floor material, or the model of bindings3d `floors.by_material` (the C3 paved road's flagstones). Roofs: each roofed room (get_rooms, not dug) gets one hipped roof, a height field
#   over half-metre cells that rises from the wall line at `structures.roof_pitch_deg` (capped at
#   `structures.roof_max_rise`) with a one-cell overhang that stops short of any other room, so no roof passes
#   through another; the cover (thatch, bark, plank:
#   bindings3d `roofs`) follows the era of the template site that holds the room, else the room's wall category.
#   Cutaway: at the closest zoom (`structures.cutaway_zoom`) or with the room overlay (R, "방 보기") every roof
#   dissolves and walls drop to `structures.cutaway_wall_height`; with a room selected (RoomPanel) or a person in the
#   inspector standing in a roofed room, that room's roof dissolves and the walls around it drop. Construction: a
#   blueprint site of get_colony_overlay shows its footprint outline (stakes and sills); a wall or door site is its
#   blueprint id's M3D-ASSETS stage model (bindings3d `sites.walls`: foundation, half; stage = floor(progress × count))
#   along its wall run, else a block rising with its work progress; a template site still building is its template's
#   stage model (`sites.templates`, by completed / total pieces) scaled to its footprint and turned by its rotation,
#   else its footprint outline. Snapshots are read when the
#   overlay or rooms counter moves (or every `flora.slow_read_ticks`); walls rebuild per 32×32 chunk only when the
#   wall/floor content changes, roofs when the rooms version or the template sites change. Occupied roofed
#   surface rooms have emissive windows on their exterior walls at night.
# Bridge methods called: via SimReads get_space, get_rooms, get_building_snapshot, get_colony_overlay,
#   get_agent_snapshot, get_content_catalog; signal world_reset.
extends Node3D

const Data3D = preload("res://scripts/world3d/data3d.gd")
const Builder = preload("res://scripts/world3d/mesh_builder3d.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
const MaterialResolver = preload("res://scripts/world/art/material_resolver.gd")
const SlicePbr = preload("res://scripts/world3d/slice_pbr.gd")
const Kit = preload("res://scripts/world3d/s2kit.gd")
const WALL_SHADER = preload("res://shaders/world3d/wall.gdshader")
const ROOF_SHADER = preload("res://shaders/world3d/roof.gdshader")
const DIRS: Array[Vector2i] = [Vector2i(0, -1), Vector2i(1, 0), Vector2i(0, 1), Vector2i(-1, 0)]
const PROFILE_BLOCK: float = 0.0
const PROFILE_LOG: float = 1.0
const PROFILE_PALISADE: float = 2.0
const PROFILE_TRIM: float = 3.0

var wall_material: ShaderMaterial
var _slice_pbr: bool = false
var _style2: bool = SlicePbr.style2()
var _kit_walls: Dictionary = {}
var _kit_wall_materials: Dictionary = {}
var _world_sim: Node = null
var _terrain: Node = null
var _resolver: MaterialResolver = MaterialResolver.new()
var _space_version: int = -2
var _rooms_key: String = ""
var _sites_key: String = ""
var _walls: Dictionary = {}
var _chunks: Dictionary = {}
var _roofs: Dictionary = {}
var _roof_fade: Dictionary = {}
var _room_rects: Dictionary = {}
## Room id → wall-top height (the plane a click on that roof resolves at).
var _roof_top: Dictionary = {}
var _tile_room: Dictionary = {}
var _doors: Array[Dictionary] = []
var _door_nodes: Array[MeshInstance3D] = []
var _site_node: MeshInstance3D = null
var _site_models: Node3D = null
var _fence_models: Node3D = null
## Floor tiles drawn with a model (bindings3d `floors.by_material`): tile -> model id.
var _floor_models: Dictionary = {}
var _cut_all: float = 99.0
var _cut_rect_h: float = 99.0
var _cut_rect: Rect2 = Rect2()
var _chunk: int = 32
var _stamp_key: String = ""
var _occupied: Dictionary = {}
var _occupied_tick: int = -2
var _occupied_rooms: Dictionary = {}
var _windows: Dictionary = {}
var _window_mesh: QuadMesh
var _window_material: StandardMaterial3D
var _window_elevation: float
var _window_offset: float
var _window_emission: float


func setup(world_sim: Node, terrain: Node) -> void:
	_world_sim = world_sim
	_terrain = terrain
	_world_sim.connect("world_reset", _on_world_reset)


func _ready() -> void:
	_resolver.load_imported()
	wall_material = ShaderMaterial.new()
	wall_material.shader = WALL_SHADER
	preload("res://scripts/world3d/textures3d.gd").setup(wall_material)
	_window_mesh = QuadMesh.new()
	_window_mesh.size = Vector2(float(Data3D.mood("room_lights.window_width", 0.44)), float(Data3D.mood("room_lights.window_height", 0.32)))
	_window_elevation = float(Data3D.mood("room_lights.window_elevation", 1.35))
	_window_offset = float(Data3D.mood("room_lights.window_offset", 0.025))
	_window_emission = float(Data3D.mood("room_lights.window_emission", 2.1))
	var window_colour: Color = Data3D.colour(Data3D.mood("room_lights.window_colour", []), Color(1.0, 0.56, 0.20))
	_window_material = StandardMaterial3D.new()
	_window_material.albedo_color = window_colour
	_window_material.emission_enabled = true
	_window_material.emission = window_colour
	_window_mesh.material = _window_material


func set_slice_pbr(enabled: bool) -> bool:
	if not SlicePbr.configure(wall_material, WALL_SHADER, enabled):
		return false
	_slice_pbr = enabled
	for roof: MeshInstance3D in _roofs.values():
		var mat: ShaderMaterial = roof.get_active_material(0) as ShaderMaterial
		if mat.shader != Kit.SHADER and not SlicePbr.configure(mat, ROOF_SHADER, enabled):
			return false
	return true


func _on_world_reset() -> void:
	_space_version = -2
	_rooms_key = ""
	_sites_key = ""
	_stamp_key = ""
	_occupied_tick = -2
	_occupied_rooms.clear()
	_clear_windows()


func _process(delta: float) -> void:
	if _world_sim == null or _terrain == null or not bool(_terrain.call("is_complete")):
		return
	_chunk = maxi(4, int(Data3D.param("terrain.chunk_tiles", 32)))
	var stamp: Dictionary = get_parent().get("stamp") if get_parent() != null else {}
	@warning_ignore("integer_division")
	var stamp_key: String = "%s|%s|%s" % [stamp.get("overlay", -1), stamp.get("rooms", -1), int(stamp.get("tick", -1)) / maxi(1, int(Data3D.param("flora.slow_read_ticks", 24)))]
	# A rebuild takes World3D's heavy-work slot (one per frame, none on a simulation step); else try next frame.
	var claimed: bool = stamp_key != _stamp_key and (get_parent() == null or not get_parent().has_method("claim_heavy")
		or bool(get_parent().call("claim_heavy")))
	if (stamp_key == _stamp_key and not stamp.is_empty()) or not claimed:
		_update_doors()
		_update_cutaway(delta)
		return
	_stamp_key = stamp_key
	var space: Dictionary = SimReads.read(_world_sim, "get_space")
	var buildings: Dictionary = SimReads.read(_world_sim, "get_building_snapshot")
	# Rebuild walls on wall/floor content, not on the shared overlay counter (hauling moves it every few ticks).
	var version: int = hash([space.get("wall_xs"), space.get("wall_ys"), space.get("wall_materials"), space.get("wall_styles"),
		space.get("wall_is_door"), space.get("floor_xs"), space.get("floor_ys"), space.get("floor_materials")])
	var bkey: int = hash([buildings.get("kinds"), buildings.get("xs"), buildings.get("ys")])
	if version != _space_version or bkey != int(_walls.get("_bkey", -1)):
		_space_version = version
		_rebuild_walls(space, buildings)
		_walls["_bkey"] = bkey
	var rooms: Dictionary = SimReads.read(_world_sim, "get_rooms")
	var sites: Array = buildings.get("template_sites", [])
	var rkey: String = "%d|%d|%d" % [int(rooms.get("version", 0)), hash(str(sites)), version]
	if rkey != _rooms_key:
		_rooms_key = rkey
		_rebuild_roofs(rooms, sites)
	var overlay: Dictionary = SimReads.read(_world_sim, "get_colony_overlay")
	var skey: String = "%d|%d" % [int(overlay.get("version", 0)), hash(str(sites))]
	if skey != _sites_key:
		_sites_key = skey
		_rebuild_sites(overlay, sites)
	_update_doors()
	_update_cutaway(delta)


# --- walls -------------------------------------------------------------------------------------------------

func _rebuild_walls(space: Dictionary, buildings: Dictionary) -> void:
	_walls.clear()
	var xs: PackedInt32Array = space.get("wall_xs", PackedInt32Array())
	var ys: PackedInt32Array = space.get("wall_ys", PackedInt32Array())
	var doors: PackedByteArray = space.get("wall_is_door", PackedByteArray())
	var kinds: PackedInt32Array = space.get("wall_kinds", PackedInt32Array())
	var mats: PackedInt32Array = space.get("wall_materials", PackedInt32Array())
	var styles: PackedStringArray = space.get("wall_styles", PackedStringArray())
	var cats: PackedStringArray = space.get("wall_categories", PackedStringArray())
	for i: int in mini(xs.size(), ys.size()):
		var is_door: bool = (doors[i] == 1) if i < doors.size() else (i < kinds.size() and kinds[i] == 2)
		var raw: int = (mats[i] & 0xFFFFFFFF) if i < mats.size() and mats[i] != -1 else -1
		_walls[Vector2i(xs[i], ys[i])] = _wall_entry(styles[i] if i < styles.size() else "", raw, cats[i] if i < cats.size() else "", is_door)
	_add_hut_rings(buildings)
	# Openings with an opening neighbour are fence runs, the rest are doors.
	for t: Variant in _walls.keys():
		if t is not Vector2i:
			continue
		var w: Dictionary = _walls[t]
		if not bool(w["door"]):
			continue
		for d: Vector2i in DIRS:
			var n: Variant = _walls.get((t as Vector2i) + d)
			if n is Dictionary and bool((n as Dictionary)["door"]):
				w["fence"] = true
				break
	var floors: Dictionary = _floor_tiles(space)
	var dirty: Dictionary = {}
	for t: Vector2i in _floor_models.keys():
		dirty[_chunk_of(t)] = true
	for t: Variant in _walls.keys():
		if t is Vector2i:
			dirty[_chunk_of(t)] = true
	for t: Vector2i in floors.keys():
		dirty[_chunk_of(t)] = true
	for k: Vector2i in _chunks.keys():
		dirty[k] = true
	for k: Vector2i in _kit_walls.keys():
		dirty[k] = true
	for k: Vector2i in dirty:
		_build_wall_chunk(k, floors)
	_place_fence_models()
	_collect_doors()


## The fence model of tile `t`'s join mask (bindings3d `fences.by_mask`), else "".
func _fence_model(t: Vector2i) -> String:
	var by_mask: Variant = Data3D.binding("fences.by_mask", [])
	var mask: int = 0
	for i: int in DIRS.size():
		if _connects(t, DIRS[i]):
			mask |= 1 << i
	return Data3D.first_model([(by_mask as Array)[mask]]) if by_mask is Array and mask < (by_mask as Array).size() else ""


## Fence runs drawn with models: one MultiMesh per fence model.
func _place_fence_models() -> void:
	if _fence_models == null:
		_fence_models = Node3D.new()
		_fence_models.name = "FenceModels"
		add_child(_fence_models)
	for n: Node in _fence_models.get_children():
		n.queue_free()
	var batches: Dictionary = {}
	for t: Variant in _walls.keys():
		if t is not Vector2i or not bool(_walls[t]["fence"]):
			continue
		var id: String = _fence_model(t)
		if id == "":
			continue
		if not batches.has(id):
			batches[id] = []
		(batches[id] as Array).append(Vector3(t.x + 0.5, float(_terrain.call("stand_height", t)), t.y + 0.5))
	for t: Vector2i in _floor_models.keys():
		var fid: String = str(_floor_models[t])
		if not batches.has(fid):
			batches[fid] = []
		(batches[fid] as Array).append(Vector3(t.x + 0.5, float(_terrain.call("stand_height", t)), t.y + 0.5))
	for id: String in batches:
		var mm: MultiMesh = MultiMesh.new()
		mm.transform_format = MultiMesh.TRANSFORM_3D
		mm.mesh = Data3D.mesh(id)
		mm.instance_count = (batches[id] as Array).size()
		for i: int in mm.instance_count:
			mm.set_instance_transform(i, Transform3D(Basis.IDENTITY, batches[id][i]))
		var mmi: MultiMeshInstance3D = MultiMeshInstance3D.new()
		mmi.multimesh = mm
		_fence_models.add_child(mmi)


func _chunk_of(t: Vector2i) -> Vector2i:
	return Vector2i(floori(float(t.x) / _chunk), floori(float(t.y) / _chunk))


func _wall_entry(style: String, raw_material: int, category: String, is_door: bool) -> Dictionary:
	var material_id: Variant = raw_material
	if raw_material < 0:
		material_id = ""
	var r: Dictionary = _resolver.resolve(style, material_id, category, "door" if is_door else "wall")
	var key: String = str(r.get("material_key", "")) if r.get("material_key") != null else ""
	var cat: String = str(r.get("category", category)) if r.get("category") != null else category
	var c: Color = Data3D.material_colour(key, cat)
	var tint: Dictionary = r.get("tint", {})
	c = c * (1.0 + float(tint.get("dL", 0)) / 70.0)
	c.a = 1.0
	var by_style: Dictionary = Data3D.binding("walls.by_style", {})
	var wall_style: String = str(r.get("style_id", style))
	var profile: String = str(by_style.get(wall_style, by_style.get(style, Data3D.binding("walls.fallback", ""))))
	return {"door": is_door, "fence": false, "profile": profile, "colour": c, "category": cat, "material": key, "style": wall_style}


## Finished huts are not in get_space: their ring, door and floor come from the 2D building binding.
func _add_hut_rings(buildings: Dictionary) -> void:
	var bx: PackedInt32Array = buildings.get("xs", PackedInt32Array())
	var by: PackedInt32Array = buildings.get("ys", PackedInt32Array())
	var bk: PackedByteArray = buildings.get("kinds", PackedByteArray())
	for i: int in mini(bk.size(), mini(bx.size(), by.size())):
		var ring: Variant = ArtData.binding("buildings", int(bk[i])).get("ring")
		if ring is not Dictionary:
			continue
		var size: Array = (ring as Dictionary).get("size", [3, 3])
		var w: int = int(size[0])
		var h: int = int(size[1])
		@warning_ignore("integer_division")
		var x0: int = bx[i] - w / 2
		@warning_ignore("integer_division")
		var y0: int = by[i] - h / 2
		var door: Array = (ring as Dictionary).get("door", [1, h - 1])
		for y: int in range(y0, y0 + h):
			for x: int in range(x0, x0 + w):
				if x != x0 and y != y0 and x != x0 + w - 1 and y != y0 + h - 1:
					continue
				var is_door: bool = x - x0 == int(door[0]) and y - y0 == int(door[1])
				var style: String = str((ring as Dictionary).get("door_style" if is_door else "wall_style", ""))
				_walls[Vector2i(x, y)] = _wall_entry(style, -1, "Wood", is_door)


func _floor_tiles(space: Dictionary) -> Dictionary:
	var out: Dictionary = {}
	var xs: PackedInt32Array = space.get("floor_xs", PackedInt32Array())
	var ys: PackedInt32Array = space.get("floor_ys", PackedInt32Array())
	var kinds: PackedInt32Array = space.get("floor_kinds", PackedInt32Array())
	var mats: PackedInt32Array = space.get("floor_materials", PackedInt32Array())
	var styles: PackedStringArray = space.get("floor_styles", PackedStringArray())
	var by_material: Dictionary = Data3D.binding("floors.by_material", {})
	_floor_models.clear()
	for i: int in mini(xs.size(), ys.size()):
		if i < kinds.size() and kinds[i] == 1:
			continue
		var raw: int = (mats[i] & 0xFFFFFFFF) if i < mats.size() and mats[i] != -1 else -1
		var floor_id: Variant = raw
		if raw < 0:
			floor_id = ""
		var r: Dictionary = _resolver.resolve(styles[i] if i < styles.size() else "", floor_id, "", "floor")
		var key: String = str(r.get("material_key", "")) if r.get("material_key") != null else ""
		var model: String = Data3D.first_model([by_material.get(key, "")]) if by_material.has(key) else ""
		if model != "":
			_floor_models[Vector2i(xs[i], ys[i])] = model
			continue
		var category: String = str(r.get("category", "")) if r.get("category") != null else ""
		var c: Color = Data3D.material_colour(key, category)
		var textures = preload("res://scripts/world3d/textures3d.gd")
		out[Vector2i(xs[i], ys[i])] = {"colour": c, "texture": textures.index("materials", key, textures.index("categories", category))}
	return out


func _connects(t: Vector2i, d: Vector2i) -> bool:
	return _walls.has(t + d)


func _build_wall_chunk(k: Vector2i, floors: Dictionary) -> void:
	var b: Builder = Builder.new()
	var kit_batches: Dictionary = {}
	var kit_profiles: Dictionary = Data3D.binding("s2kit.wall_profiles", {}) if _style2 else {}
	var hgt: float = float(Data3D.param("structures.wall_height", 2.2))
	var thick: float = float(Data3D.param("structures.wall_thickness", 0.35))
	var floor_h: float = float(Data3D.param("structures.floor_height", 0.06))
	for t: Variant in _walls.keys():
		if t is not Vector2i or _chunk_of(t) != k:
			continue
		var w: Dictionary = _walls[t]
		var g: float = float(_terrain.call("stand_height", t))
		b.base = g
		var textures = preload("res://scripts/world3d/textures3d.gd")
		b.texture_tile = textures.index("styles", str(w.get("style", "")), textures.index("materials", str(w.get("material", "")), textures.index("categories", str(w.get("category", "")))))
		if posmod(t.x * 17 + t.y * 31, 3) == 0:
			b.texture_tile = textures.index("alternates", str(int(b.texture_tile)), int(b.texture_tile))
		if bool(w["fence"]):
			if _fence_model(t) == "":
				_fence(b, t, g)
		elif bool(w["door"]):
			_door_frame(b, t, g, hgt, thick, w)
		else:
			var kit_id: String = str(kit_profiles.get(w["profile"], ""))
			if kit_id.is_empty() or not _kit_wall(kit_id, t, g, hgt, thick, kit_batches):
				_wall(b, t, g, hgt, thick, w)
	if _style2:
		_build_kit_wall_batches(k, kit_batches)
	for t: Variant in floors.keys():
		if _chunk_of(t) != k:
			continue
		var g2: float = float(_terrain.call("stand_height", t))
		b.base = g2
		b.profile = PROFILE_TRIM
		var fc: Color = (floors[t] as Dictionary)["colour"]
		b.texture_tile = float((floors[t] as Dictionary)["texture"])
		b.box(Vector3(t.x + 0.02, g2 - 0.05, t.y + 0.02), Vector3(t.x + 0.98, g2 + floor_h, t.y + 0.98), fc.darkened(0.05), fc)
	var mi: MeshInstance3D = _chunks.get(k)
	if b.is_empty():
		if mi != null:
			mi.queue_free()
			_chunks.erase(k)
		return
	if mi == null:
		mi = MeshInstance3D.new()
		mi.name = "Walls_%d_%d" % [k.x, k.y]
		add_child(mi)
		_chunks[k] = mi
	mi.mesh = b.commit(wall_material)


## The straight kit piece has no corner, T, or cross joint.
func _kit_wall(id: String, tile: Vector2i, ground: float, height: float, thickness: float, batches: Dictionary) -> bool:
	var along_x: bool = _connects(tile, Vector2i.LEFT) or _connects(tile, Vector2i.RIGHT)
	var along_z: bool = _connects(tile, Vector2i.UP) or _connects(tile, Vector2i.DOWN)
	if along_x and along_z:
		return false
	var model: ArrayMesh = Kit.mesh(id)
	if model == null:
		return false
	if not batches.has(id):
		batches[id] = []
	var yaw: float = -PI * 0.5 if along_z else 0.0
	var centre: Vector3 = Vector3(tile.x + 0.5, ground, tile.y + 0.5)
	(batches[id] as Array).append(Kit.fit(model, Vector3(1.02, height, thickness), yaw, centre))
	return true


func _build_kit_wall_batches(chunk: Vector2i, batches: Dictionary) -> void:
	for old: MultiMeshInstance3D in _kit_walls.get(chunk, []):
		old.queue_free()
	_kit_walls.erase(chunk)
	if batches.is_empty():
		return
	var nodes: Array[MultiMeshInstance3D] = []
	for id: String in batches:
		var has_lod: bool = Kit.mesh(id, true) != null
		for lod: bool in [false, true]:
			var model: ArrayMesh = Kit.mesh(id, lod)
			if model == null:
				continue
			var transforms: Array = batches[id]
			var multimesh: MultiMesh = MultiMesh.new()
			multimesh.transform_format = MultiMesh.TRANSFORM_3D
			multimesh.mesh = model
			multimesh.instance_count = transforms.size()
			for i: int in transforms.size():
				multimesh.set_instance_transform(i, transforms[i])
			var material_key: String = id + ("_lod" if lod else "")
			if not _kit_wall_materials.has(material_key):
				_kit_wall_materials[material_key] = Kit.material_for(model, true)
			var part: MultiMeshInstance3D = MultiMeshInstance3D.new()
			part.multimesh = multimesh
			part.material_override = _kit_wall_materials[material_key]
			if has_lod:
				if lod:
					part.visibility_range_begin = Kit.distance()
					part.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
				else:
					part.visibility_range_end = Kit.distance()
			add_child(part)
			nodes.append(part)
	_kit_walls[chunk] = nodes


func _wall(b: Builder, t: Vector2i, g: float, hgt: float, thick: float, w: Dictionary) -> void:
	var c: Color = w["colour"]
	var cx: float = t.x + 0.5
	var cz: float = t.y + 0.5
	var half: float = thick * 0.5
	var arms: Array[Vector2i] = []
	for d: Vector2i in DIRS:
		if _connects(t, d):
			arms.append(d)
	match str(w["profile"]):
		"log":
			b.profile = PROFILE_LOG
			var r: float = half
			var count: int = maxi(3, roundi(hgt / (2.0 * r)))
			var step: float = hgt / float(count)
			# Log ends: an L corner lets both runs cross and stick out `structures.log_overhang` past the outer face; at a
			# T or X join only the through run is continuous and the side runs butt into its face (no plus sign).
			var over: float = half + float(Data3D.param("structures.log_overhang", 0.15))
			var axes: Array[Vector2i] = []
			for d: Vector2i in arms:
				var ax: Vector2i = Vector2i(absi(d.x), absi(d.y))
				if not axes.has(ax):
					axes.append(ax)
			if axes.is_empty():
				axes = [Vector2i(1, 0)]
			for ax: Vector2i in axes:
				# Logs along x sit on the even courses, logs along z half a course higher (corner interlock).
				var lift: float = 0.0 if ax.x == 1 else step * 0.5
				var neg: bool = arms.has(-ax) or arms.size() == 0
				var pos: bool = arms.has(ax) or arms.size() == 0
				var other: Vector2i = Vector2i(ax.y, ax.x)
				var through_other: bool = arms.has(other) and arms.has(-other)
				var stub: float = half if through_other else over
				var from: float = -0.5 if neg else -stub
				var to: float = 0.5 if pos else stub
				if arms.size() == 0:
					from = -0.3
					to = 0.3
				for i: int in count:
					var y: float = g + step * (float(i) + 0.5) + lift - (step * 0.5 if ax.y == 1 and i == count - 1 else 0.0)
					if y + r > g + hgt + 0.01:
						continue
					var a: Vector3 = Vector3(cx + ax.x * from, y, cz + ax.y * from)
					var e: Vector3 = Vector3(cx + ax.x * to, y, cz + ax.y * to)
					var shade: float = 0.92 + 0.08 * float((t.x * 7 + t.y * 13 + i * 5) % 3) / 2.0
					b.log_prism(a, e, r * 1.02, Color(c.r * shade, c.g * shade, c.b * shade))
		"palisade":
			b.profile = PROFILE_PALISADE
			b.stake(cx, cz, g - 0.1, g + hgt + 0.25, half * 0.55, c)
			for d: Vector2i in arms:
				for s: int in 2:
					var f: float = 0.25 + 0.25 * s
					b.stake(cx + d.x * f, cz + d.y * f, g - 0.1, g + hgt + (0.1 if s == 0 else 0.3), half * 0.55, c.darkened(0.06 * s))
		_:
			b.profile = PROFILE_BLOCK
			b.box(Vector3(cx - half, g - 0.05, cz - half), Vector3(cx + half, g + hgt, cz + half), c, c.lightened(0.12))
			for d: Vector2i in arms:
				var lo: Vector3 = Vector3(cx - half + (half if d.x > 0 else 0.0) - (0.5 - half if d.x < 0 else 0.0),
					g - 0.05, cz - half + (half if d.y > 0 else 0.0) - (0.5 - half if d.y < 0 else 0.0))
				var hi: Vector3 = Vector3(cx + half + (0.5 - half if d.x > 0 else 0.0) - (half if d.x < 0 else 0.0),
					g + hgt, cz + half + (0.5 - half if d.y > 0 else 0.0) - (half if d.y < 0 else 0.0))
				if d.x != 0:
					lo.z = cz - half
					hi.z = cz + half
				else:
					lo.x = cx - half
					hi.x = cx + half
				b.box(lo, hi, c, c.lightened(0.12))


## The axis a wall run takes through tile `t` (x when walls lie east/west, else z).
func _run_axis(t: Vector2i) -> Vector2i:
	if _connects(t, Vector2i(1, 0)) or _connects(t, Vector2i(-1, 0)):
		return Vector2i(1, 0)
	return Vector2i(0, 1)


func _door_frame(b: Builder, t: Vector2i, g: float, hgt: float, thick: float, w: Dictionary) -> void:
	var ax: Vector2i = _run_axis(t)
	var door_w: float = float(Data3D.param("structures.door_width", 0.9))
	var door_h: float = float(Data3D.param("structures.door_height", 2.0))
	var half: float = thick * 0.5
	var cx: float = t.x + 0.5
	var cz: float = t.y + 0.5
	var trim: Color = (w["colour"] as Color).darkened(0.25)
	b.profile = PROFILE_TRIM
	var post: float = 0.09
	for side: int in [-1, 1]:
		var off: float = side * (door_w * 0.5 + post)
		var px: float = cx + ax.x * off
		var pz: float = cz + ax.y * off
		b.box(Vector3(px - (post if ax.x == 1 else half), g - 0.05, pz - (post if ax.y == 1 else half)),
			Vector3(px + (post if ax.x == 1 else half), g + door_h + 0.12, pz + (post if ax.y == 1 else half)), trim)
		# Wall filler between the post and the tile edge (so the opening sits in a continuous wall).
		var edge_a: float = side * (door_w * 0.5 + post * 2.0)
		var edge_b: float = side * 0.5
		var lo_o: float = minf(edge_a, edge_b)
		var hi_o: float = maxf(edge_a, edge_b)
		b.profile = PROFILE_BLOCK
		b.box(Vector3(cx + (lo_o if ax.x == 1 else -half), g - 0.05, cz + (lo_o if ax.y == 1 else -half)),
			Vector3(cx + (hi_o if ax.x == 1 else half), g + hgt, cz + (hi_o if ax.y == 1 else half)), w["colour"], (w["colour"] as Color).lightened(0.12))
		b.profile = PROFILE_TRIM
	# Lintel and the wall above it.
	var span: float = door_w * 0.5 + post * 2.0
	b.box(Vector3(cx - (span if ax.x == 1 else half), g + door_h + 0.02, cz - (span if ax.y == 1 else half)),
		Vector3(cx + (span if ax.x == 1 else half), g + door_h + 0.16, cz + (span if ax.y == 1 else half)), trim)
	b.profile = PROFILE_BLOCK
	if hgt > door_h + 0.16:
		b.box(Vector3(cx - (span if ax.x == 1 else half), g + door_h + 0.16, cz - (span if ax.y == 1 else half)),
			Vector3(cx + (span if ax.x == 1 else half), g + hgt, cz + (span if ax.y == 1 else half)), w["colour"], (w["colour"] as Color).lightened(0.12))


func _fence(b: Builder, t: Vector2i, g: float) -> void:
	var fh: float = float(Data3D.param("structures.fence_height", 1.1))
	var w: Dictionary = _walls[t]
	var c: Color = w["colour"]
	b.profile = PROFILE_TRIM
	var cx: float = t.x + 0.5
	var cz: float = t.y + 0.5
	b.stake(cx, cz, g - 0.1, g + fh + 0.12, 0.07, c.darkened(0.1))
	for d: Vector2i in DIRS:
		if not _connects(t, d):
			continue
		for rail_y: float in [fh * 0.42, fh * 0.85]:
			b.log_prism(Vector3(cx, g + rail_y, cz), Vector3(cx + d.x * 0.5, g + rail_y, cz + d.y * 0.5), 0.045, c, 5)


# --- doors -------------------------------------------------------------------------------------------------

func _collect_doors() -> void:
	_doors.clear()
	for t: Variant in _walls.keys():
		if t is Vector2i and bool(_walls[t]["door"]) and not bool(_walls[t]["fence"]):
			_doors.append({"tile": t, "axis": _run_axis(t), "colour": _walls[t]["colour"], "open": 0.0})
	for n: MeshInstance3D in _door_nodes:
		n.queue_free()
	_door_nodes.clear()
	var leaf: Mesh = Data3D.mesh("door") if Data3D.has_model("door") else null
	for d: Dictionary in _doors:
		var mi: MeshInstance3D = MeshInstance3D.new()
		if leaf != null:
			mi.mesh = leaf
		else:
			var box: BoxMesh = BoxMesh.new()
			box.size = Vector3(float(Data3D.param("structures.door_width", 0.9)), float(Data3D.param("structures.door_height", 2.0)), 0.08)
			var mat: StandardMaterial3D = StandardMaterial3D.new()
			mat.albedo_color = (d["colour"] as Color).darkened(0.35)
			box.material = mat
			mi.mesh = box
		add_child(mi)
		_door_nodes.append(mi)


func _update_occupancy() -> void:
	if _doors.is_empty() and _roofs.is_empty():
		return
	var tick: int = int((get_parent().get("stamp") as Dictionary).get("tick", -1)) if get_parent() != null else -1
	if tick == _occupied_tick:
		return
	_occupied_tick = tick
	_occupied.clear()
	_occupied_rooms.clear()
	var agents: Dictionary = SimReads.read(_world_sim, "get_agent_snapshot")
	var xs: PackedInt32Array = agents.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = agents.get("ys", PackedInt32Array())
	for i: int in mini(xs.size(), ys.size()):
		var tile: Vector2i = Vector2i(xs[i], ys[i])
		_occupied[tile] = true
		var room: int = int(_tile_room.get(tile, -1))
		if _roofs.has(room):
			_occupied_rooms[room] = true


func _update_doors() -> void:
	_update_occupancy()
	if _doors.is_empty():
		return
	var door_w: float = float(Data3D.param("structures.door_width", 0.9))
	var door_h: float = float(Data3D.param("structures.door_height", 2.0))
	var has_model: bool = Data3D.has_model("door")
	for i: int in mini(_doors.size(), _door_nodes.size()):
		var d: Dictionary = _doors[i]
		var t: Vector2i = d["tile"]
		var target: float = 1.0 if _occupied.has(t) else 0.0
		d["open"] = move_toward(float(d["open"]), target, get_process_delta_time() * 3.0)
		var ax: Vector2i = d["axis"]
		var g: float = float(_terrain.call("stand_height", t))
		var hinge: Vector3 = Vector3(t.x + 0.5 - ax.x * door_w * 0.5, g, t.y + 0.5 - ax.y * door_w * 0.5)
		var yaw: float = 0.0 if ax.x == 1 else -PI * 0.5
		var swing: float = -float(d["open"]) * deg_to_rad(80.0)
		var basis: Basis = Basis(Vector3.UP, yaw + swing)
		# The model's pivot is its left edge; the placeholder box is centred.
		var offset: Vector3 = Vector3.ZERO if has_model else Vector3(door_w * 0.5, door_h * 0.5, 0.0)
		_door_nodes[i].transform = Transform3D(basis, hinge + basis * offset)
		var cut: float = _cut_all
		if _cut_rect.has_point(Vector2(t.x + 0.5, t.y + 0.5)):
			cut = minf(cut, _cut_rect_h)
		# The cutaway lowers the leaf with the walls instead of leaving an empty gap.
		var keep: float = clampf(cut / door_h, 0.05, 1.0)
		_door_nodes[i].transform = _door_nodes[i].transform.scaled_local(Vector3(1.0, keep, 1.0))


# --- roofs -------------------------------------------------------------------------------------------------

func _rebuild_roofs(rooms: Dictionary, sites: Array) -> void:
	for k: Variant in _roofs.keys():
		(_roofs[k] as Node).queue_free()
	_roofs.clear()
	_clear_windows()
	_occupied_rooms.clear()
	_occupied_tick = -2
	_room_rects.clear()
	_roof_top.clear()
	_tile_room.clear()
	var ids: PackedInt32Array = rooms.get("ids", PackedInt32Array())
	var roofed: PackedByteArray = rooms.get("roofed", PackedByteArray())
	var dug: PackedByteArray = rooms.get("dug", PackedByteArray())
	var txs: PackedInt32Array = rooms.get("tile_xs", PackedInt32Array())
	var tys: PackedInt32Array = rooms.get("tile_ys", PackedInt32Array())
	var trs: PackedInt32Array = rooms.get("tile_rooms", PackedInt32Array())
	var tiles_of: Dictionary = {}
	for i: int in mini(trs.size(), mini(txs.size(), tys.size())):
		var t: Vector2i = Vector2i(txs[i], tys[i])
		_tile_room[t] = trs[i]
		if not tiles_of.has(trs[i]):
			tiles_of[trs[i]] = []
		(tiles_of[trs[i]] as Array).append(t)
	var roofed_ids: Array[int] = []
	var claimed: Dictionary = {}
	for i: int in ids.size():
		var rid: int = ids[i]
		if i >= roofed.size() or roofed[i] != 1 or (i < dug.size() and dug[i] == 1) or not tiles_of.has(rid):
			continue
		roofed_ids.append(rid)
		for c: Vector2i in footprint_cells(tiles_of[rid]):
			claimed[c] = rid
	for id: int in roofed_ids:
		var tiles: Array = tiles_of[id]
		var bounds: Rect2i = Rect2i(tiles[0], Vector2i.ONE)
		for t: Vector2i in tiles:
			bounds = bounds.expand(t).expand(t + Vector2i.ONE)
		_room_rects[id] = bounds
		var cover: String = _roof_cover(tiles, sites)
		var mi: MeshInstance3D = MeshInstance3D.new()
		mi.name = "Roof_%d" % id
		if not _style2 or not _kit_roof(mi, tiles, bounds, id, claimed, cover):
			var mat: ShaderMaterial = ShaderMaterial.new()
			SlicePbr.configure(mat, ROOF_SHADER, _slice_pbr)
			preload("res://scripts/world3d/textures3d.gd").setup(mat)
			mat.set_shader_parameter("texture_tile", float(preload("res://scripts/world3d/textures3d.gd").index("roofs", cover)))
			mat.set_shader_parameter("cover", Data3D.colour((Data3D.binding("roofs.colours", {}) as Dictionary).get(cover, []), Color(0.7, 0.6, 0.35)))
			mat.set_shader_parameter("pattern", int((Data3D.binding("roofs.patterns", {}) as Dictionary).get(cover, 0)))
			mat.set_shader_parameter("fade", float(_roof_fade.get(id, 1.0)))
			mi.mesh = _roof_mesh(tiles, id, claimed, mat)
		add_child(mi)
		_roofs[id] = mi
		_add_windows(id, tiles)


## Stretch the gable over rectangular footprints whose full eaves fit beside neighboring roofs.
func _kit_roof(node: MeshInstance3D, tiles: Array, bounds: Rect2i, room_id: int, claimed: Dictionary, cover: String) -> bool:
	var covers: Dictionary = Data3D.binding("s2kit.roof_covers", {})
	var id: String = str(covers.get(cover, ""))
	if id.is_empty() or tiles.size() != bounds.get_area():
		return false
	var full: Vector2i = bounds.size * 2 + Vector2i(4, 4)
	if roof_cells(tiles, room_id, claimed).size() != full.x * full.y:
		return false
	var model: ArrayMesh = Kit.mesh(id)
	if model == null:
		return false
	var height: float = float(Data3D.param("structures.wall_height", 2.2))
	var pitch: float = tan(deg_to_rad(float(Data3D.param("structures.roof_pitch_deg", 38.0))))
	var half_wall: float = float(Data3D.param("structures.wall_thickness", 0.35)) * 0.5
	var span: float = float(mini(full.x, full.y)) * 0.5
	var ridge: float = float(maxi(full.x, full.y)) * 0.5
	var rise: float = minf(float(Data3D.param("structures.roof_max_rise", 2.4)), span * 0.5 * pitch)
	var eave_height: float = _roof_ground(tiles) + height + 0.03
	var center: Vector2 = Vector2(bounds.position) + Vector2(bounds.size) * 0.5
	var yaw: float = PI * 0.5 if full.x > full.y else 0.0
	_roof_top[room_id] = eave_height + half_wall * pitch
	node.mesh = model
	node.transform = Kit.fit(model, Vector3(span, rise, ridge), yaw, Vector3(center.x, eave_height, center.y))
	var mat: ShaderMaterial = Kit.material_for(model, false)
	mat.set_shader_parameter("fade", float(_roof_fade.get(room_id, 1.0)))
	node.material_override = mat
	Kit.add_lod(node, id)
	return true


func _clear_windows() -> void:
	for panes: Array in _windows.values():
		for pane: MeshInstance3D in panes:
			pane.hide()
			pane.queue_free()
	_windows.clear()


## One pane per façade, mounted on a wall beside a roofed room tile.
func _add_windows(room: int, tiles: Array) -> void:
	var panes: Array[MeshInstance3D] = []
	var thickness: float = float(Data3D.param("structures.wall_thickness", 0.35))
	for outward: Vector2i in DIRS:
		for inside: Vector2i in tiles:
			var wall: Vector2i = inside + outward
			var row: Dictionary = _walls.get(wall, {})
			if row.is_empty() or bool(row["door"]) or str(row["profile"]) == "palisade":
				continue
			var outside: Vector2i = wall + outward
			if _tile_room.has(outside) or _walls.has(outside):
				continue
			var pos: Vector3 = Vector3(wall.x + 0.5, float(_terrain.call("stand_height", wall)) + _window_elevation, wall.y + 0.5)
			pos += Vector3(outward.x, 0.0, outward.y) * (thickness * 0.5 + _window_offset)
			var pane: MeshInstance3D = MeshInstance3D.new()
			pane.name = "Window_%d" % room
			pane.mesh = _window_mesh
			pane.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
			pane.position = pos
			pane.rotation.y = atan2(float(outward.x), float(outward.y))
			pane.hide()
			add_child(pane)
			panes.append(pane)
			break
	if not panes.is_empty():
		_windows[room] = panes


func occupied_room_at(tile: Vector2i) -> bool:
	return _occupied_rooms.has(int(_tile_room.get(tile, -1)))


## Append the occupied rooms' nearest visible exterior pane to the shared light pool.
func append_window_lights(candidates: Array[Dictionary], eye: Vector3, level: float) -> void:
	_window_material.emission_energy_multiplier = _window_emission * level
	for room: int in _windows:
		var nearest: Vector3 = Vector3.ZERO
		var best: float = INF
		var lit: bool = level > 0.0 and _occupied_rooms.has(room)
		for pane: MeshInstance3D in _windows[room]:
			var cut: float = _cut_all
			if _cut_rect.has_point(Vector2(pane.position.x, pane.position.z)):
				cut = minf(cut, _cut_rect_h)
			pane.visible = lit and cut >= _window_elevation + _window_mesh.size.y * 0.5
			if pane.visible:
				var pos: Vector3 = pane.global_position
				var distance: float = eye.distance_squared_to(pos)
				if distance < best:
					nearest = pos
					best = distance
		if best < INF:
			candidates.append({"position": nearest, "distance": best, "fire": false, "phase": 0.0})


## Roof cover by the template site holding the room (its era), else the walls' material category.
func _roof_cover(tiles: Array, sites: Array) -> String:
	var t0: Vector2i = tiles[0]
	for s: Variant in sites:
		if s is not Dictionary or str((s as Dictionary).get("status", "")) == "cancelled":
			continue
		var d: Dictionary = s
		if Rect2i(int(d.get("x", 0)), int(d.get("y", 0)), int(d.get("width", 0)), int(d.get("height", 0))).has_point(t0):
			var by_era: Dictionary = Data3D.binding("roofs.by_era", {})
			if by_era.has(str(d.get("era_id", ""))):
				return str(by_era[str(d["era_id"])])
	var by_cat: Dictionary = Data3D.binding("roofs.by_wall_category", {})
	for t: Vector2i in tiles:
		for dd: Vector2i in DIRS:
			var w: Variant = _walls.get(t + dd)
			if w is Dictionary and by_cat.has(str((w as Dictionary)["category"])):
				return str(by_cat[str(w["category"])])
	return str(Data3D.binding("roofs.fallback", ""))


## The half-metre cells a room's roof covers out to the wall centre lines (room tiles grown by half a tile).
static func footprint_cells(tiles: Array) -> Dictionary:
	var out: Dictionary = {}
	for t: Vector2i in tiles:
		for y: int in range(2 * t.y - 1, 2 * t.y + 3):
			for x: int in range(2 * t.x - 1, 2 * t.x + 3):
				out[Vector2i(x, y)] = true
	return out


## The hip rectangles of a roof (planner NOTE 6): the largest rectangle inside the footprint, plus the largest
## rectangle of the rest when it covers at least 25% of the footprint. Tiles outside both get a flat cap at wall top.
static func roof_rects(tiles: Array) -> Array[Rect2i]:
	var left: Dictionary = {}
	for t: Vector2i in tiles:
		left[t] = true
	var out: Array[Rect2i] = []
	var first: Rect2i = largest_rect(left)
	if first.get_area() == 0:
		return out
	out.append(first)
	for y: int in range(first.position.y, first.end.y):
		for x: int in range(first.position.x, first.end.x):
			left.erase(Vector2i(x, y))
	var second: Rect2i = largest_rect(left)
	if second.get_area() * 4 >= tiles.size():
		out.append(second)
	return out


## The largest axis-aligned rectangle of whole tiles inside `cells` (tile → any); ties keep the first found in
## row-major order, so the result is deterministic.
static func largest_rect(cells: Dictionary) -> Rect2i:
	var keys: Array = cells.keys()
	keys.sort_custom(func(a: Vector2i, b: Vector2i) -> bool: return a.y < b.y or (a.y == b.y and a.x < b.x))
	var best: Rect2i = Rect2i()
	for t: Vector2i in keys:
		var w_max: int = 1 << 30
		var h: int = 0
		while cells.has(t + Vector2i(0, h)):
			var w: int = 0
			while w < w_max and cells.has(t + Vector2i(w, h)):
				w += 1
			w_max = w
			h += 1
			if w_max * h > best.get_area():
				best = Rect2i(t, Vector2i(w_max, h))
	return best


## The roof cells of a room: its footprint plus a one-cell (0.5 m) overhang ring, except where the ring would enter
## another roofed room's footprint (`claimed`: cell → room id). Two rooms' roofs therefore never share a cell.
static func roof_cells(tiles: Array, room_id: int, claimed: Dictionary) -> Dictionary:
	var foot: Dictionary = footprint_cells(tiles)
	var out: Dictionary = foot.duplicate()
	for c: Vector2i in foot:
		for dy: int in [-1, 0, 1]:
			for dx: int in [-1, 0, 1]:
				var n: Vector2i = c + Vector2i(dx, dy)
				if not foot.has(n) and not _near_other(n, room_id, claimed):
					out[n] = false
	return out


## Whether cell `n` is, or touches, another room's footprint (an overhang there would cross that room's roof).
static func _near_other(n: Vector2i, room_id: int, claimed: Dictionary) -> bool:
	for dy: int in [-1, 0, 1]:
		for dx: int in [-1, 0, 1]:
			if int(claimed.get(n + Vector2i(dx, dy), room_id)) != room_id:
				return true
	return false


func _roof_ground(tiles: Array) -> float:
	var ground: float = -INF
	for t: Vector2i in tiles:
		ground = maxf(ground, float(_terrain.call("stand_height", t)))
		for dd: Vector2i in DIRS:
			if _walls.has(t + dd):
				ground = maxf(ground, float(_terrain.call("stand_height", t + dd)))
	return ground


## A hipped roof as one height field over the roof cells (lattice every 0.5 m): height rises with the Chebyshev
## distance from the wall line, so L shapes get hips and valleys and the surface is single-valued (it cannot pass
## through itself); the overhang ring slopes on down past the walls. Lifted so the wall tops stay under it.
func _roof_mesh(tiles: Array, room_id: int, claimed: Dictionary, mat: Material) -> ArrayMesh:
	var b: Builder = Builder.new()
	var hgt: float = float(Data3D.param("structures.wall_height", 2.2))
	var half_wall: float = float(Data3D.param("structures.wall_thickness", 0.35)) * 0.5
	var pitch: float = tan(deg_to_rad(float(Data3D.param("structures.roof_pitch_deg", 38.0))))
	var thick: float = float(Data3D.param("structures.roof_thickness", 0.12))
	var rise_max: float = float(Data3D.param("structures.roof_max_rise", 2.4))
	var ground: float = _roof_ground(tiles)
	var top: float = ground + hgt + half_wall * pitch + 0.03
	_roof_top[room_id] = top
	b.base = ground
	var cells: Dictionary = roof_cells(tiles, room_id, claimed)
	var height: Dictionary = roof_heights(cells)
	var rects: Array[Rect2i] = roof_rects(tiles)
	# Planar hips: over the footprint, the height is the highest of the room's rectangles' own hip roofs (clean
	# planes with valleys where rectangles meet), not a distance field that follows a stepped outline.
	for v: Vector2i in height.keys():
		if float(height[v]) <= 0.0:
			continue
		var best: float = 0.0
		for r: Rect2i in rects:
			# The rectangle's footprint in half-metre lattice vertices (tiles grown by half a tile).
			var x0: int = 2 * r.position.x - 1
			var y0: int = 2 * r.position.y - 1
			var x1: int = 2 * r.end.x + 1
			var y1: int = 2 * r.end.y + 1
			if v.x >= x0 and v.x <= x1 and v.y >= y0 and v.y <= y1:
				best = maxf(best, float(mini(mini(v.x - x0, x1 - v.x), mini(v.y - y0, y1 - v.y))))
		height[v] = best
	var white: Color = Color(1, 1, 1)
	# One height step per 0.5 m lattice cell at the full pitch: the roof reaches roof_max_rise quickly and tops out flat.
	var p: Callable = func(v: Vector2i) -> Vector3:
		return Vector3(v.x * 0.5, top + minf(rise_max, float(height[v]) * pitch), v.y * 0.5)
	var down: Vector3 = Vector3(0, thick, 0)
	for c: Vector2i in cells:
		var v00: Vector3 = p.call(c)
		var v10: Vector3 = p.call(c + Vector2i(1, 0))
		var v01: Vector3 = p.call(c + Vector2i(0, 1))
		var v11: Vector3 = p.call(c + Vector2i(1, 1))
		var tris: Array = [[v00, v10, v11], [v00, v11, v01]] if absf(v00.y - v11.y) <= absf(v10.y - v01.y) else [[v00, v10, v01], [v10, v11, v01]]
		for tr: Array in tris:
			var n: Vector3 = ((tr[2] as Vector3) - (tr[0] as Vector3)).cross((tr[1] as Vector3) - (tr[0] as Vector3))
			# Planks and shingles run down the slope: along x for a north/south slope, along z otherwise.
			b.profile = 0.0 if absf(n.z) >= absf(n.x) else 1.0
			b.tri_out(tr[0], tr[1], tr[2], white, Vector3.UP)
			b.tri_out((tr[0] as Vector3) - down, (tr[1] as Vector3) - down, (tr[2] as Vector3) - down, white.darkened(0.45), Vector3.DOWN)
		b.profile = 0.0
		# Fascia on the outer edge of the roof.
		for side: int in 4:
			var d: Vector2i = DIRS[side]
			if cells.has(c + d):
				continue
			var a: Vector3
			var e: Vector3
			match side:
				0:
					a = v00
					e = v10
				1:
					a = v10
					e = v11
				2:
					a = v11
					e = v01
				_:
					a = v01
					e = v00
			b.quad_out(a, e, e - down, a - down, white.darkened(0.25), Vector3(d.x, 0, d.y))
	return b.commit(mat)


## Lattice-vertex roof heights (in cells): 0 on the footprint's wall line, Chebyshev steps inward, -1 on the outer
## edge of the overhang. `cells` maps footprint cells to true and overhang cells to false.
static func roof_heights(cells: Dictionary) -> Dictionary:
	var out: Dictionary = {}
	var frontier: Array[Vector2i] = []
	for c: Vector2i in cells:
		for k: Vector2i in [c, c + Vector2i(1, 0), c + Vector2i(0, 1), c + Vector2i(1, 1)]:
			if out.has(k):
				continue
			var foot: int = 0
			var any: int = 0
			for q: Vector2i in [k, k - Vector2i(1, 0), k - Vector2i(0, 1), k - Vector2i(1, 1)]:
				if cells.has(q):
					any += 1
					if bool(cells[q]):
						foot += 1
			if foot == 4:
				out[k] = INF
			elif foot > 0:
				out[k] = 0.0
				frontier.append(k)
			elif any > 0:
				out[k] = -1.0
	# Multi-source BFS with 8-neighbour steps from the wall line inward.
	var head: int = 0
	while head < frontier.size():
		var v: Vector2i = frontier[head]
		head += 1
		var dv: float = float(out[v])
		for dy: int in [-1, 0, 1]:
			for dx: int in [-1, 0, 1]:
				var n: Vector2i = v + Vector2i(dx, dy)
				if out.has(n) and float(out[n]) > dv + 1.0:
					out[n] = dv + 1.0
					frontier.append(n)
	return out


# --- construction sites ------------------------------------------------------------------------------------

func _rebuild_sites(overlay: Dictionary, sites: Array) -> void:
	var b: Builder = Builder.new()
	b.profile = PROFILE_TRIM
	var sill: Color = Color(0.62, 0.52, 0.36)
	var xs: PackedInt32Array = overlay.get("site_xs", PackedInt32Array())
	var ys: PackedInt32Array = overlay.get("site_ys", PackedInt32Array())
	var ws: PackedInt32Array = overlay.get("site_ws", PackedInt32Array())
	var hs: PackedInt32Array = overlay.get("site_hs", PackedInt32Array())
	var kinds: PackedInt64Array = overlay.get("site_kinds", PackedInt64Array())
	var prog: PackedInt32Array = overlay.get("site_progress", PackedInt32Array())
	var req: PackedInt32Array = overlay.get("site_required", PackedInt32Array())
	var blueprints: Dictionary = _blueprint_rows()
	var hgt: float = float(Data3D.param("structures.wall_height", 2.2))
	var thick: float = float(Data3D.param("structures.wall_thickness", 0.35))
	if _site_models == null:
		_site_models = Node3D.new()
		_site_models.name = "SiteModels"
		add_child(_site_models)
	for n: Node in _site_models.get_children():
		n.queue_free()
	var site_tiles: Dictionary = {}
	for i: int in mini(xs.size(), ys.size()):
		site_tiles[Vector2i(xs[i], ys[i])] = true
	for i: int in mini(xs.size(), mini(ys.size(), mini(ws.size(), hs.size()))):
		var rect: Rect2i = Rect2i(xs[i], ys[i], ws[i], hs[i])
		var g: float = float(_terrain.call("stand_height", rect.position))
		b.base = g
		var p: float = clampf(float(prog[i]) / float(maxi(1, req[i])), 0.0, 1.0) if i < prog.size() and i < req.size() else 0.0
		var bp: Dictionary = blueprints.get(int(kinds[i]) if i < kinds.size() else -1, {})
		var cat: String = str(bp.get("completion_category", ""))
		var stage: String = stage_model((Data3D.binding("sites.walls", {}) as Dictionary).get(str(bp.get("id", ""))), p)
		if (cat == "wall" or cat == "door") and rect.size == Vector2i.ONE and stage != "":
			# The stage models run along x: turn a piece whose run goes north–south.
			var t: Vector2i = rect.position
			var ns: bool = false
			var ew: bool = false
			for d: Vector2i in DIRS:
				if _walls.has(t + d) or site_tiles.has(t + d):
					ns = ns or d.x == 0
					ew = ew or d.y == 0
			_site_model(stage, Rect2(Vector2(t), Vector2.ONE), g, PI * 0.5 if ns and not ew else 0.0, false)
		elif (cat == "wall" or cat == "door") and rect.size == Vector2i.ONE:
			# A wall piece rises with its work; the first course shows as soon as work starts.
			var risen: float = hgt * maxf(0.08, p)
			var c: Color = Color(0.55, 0.42, 0.28)
			b.profile = PROFILE_BLOCK
			b.box(Vector3(rect.position.x + 0.5 - thick * 0.5, g - 0.05, rect.position.y + 0.5 - thick * 0.5),
				Vector3(rect.position.x + 0.5 + thick * 0.5, g + risen, rect.position.y + 0.5 + thick * 0.5), c)
			b.profile = PROFILE_TRIM
		else:
			_outline(b, rect, g, sill, 0.1 + 0.5 * p)
	for s: Variant in sites:
		if s is not Dictionary or str((s as Dictionary).get("status", "")) != "building":
			continue
		var d: Dictionary = s
		var r2: Rect2i = Rect2i(int(d.get("x", 0)), int(d.get("y", 0)), int(d.get("width", 1)), int(d.get("height", 1)))
		var g2: float = float(_terrain.call("stand_height", r2.position))
		var built: float = float(d.get("completed_pieces", 0)) / float(maxi(1, int(d.get("total_pieces", 1))))
		var model: String = stage_model((Data3D.binding("sites.templates", {}) as Dictionary).get(str(d.get("template_id", ""))), built)
		if model != "":
			_site_model(model, Rect2(r2), g2, float(int(d.get("rotation", 0))) * PI * 0.5, true)
			continue
		b.base = g2
		_outline(b, r2, g2, sill.darkened(0.15), 0.35)
	if _site_node == null:
		_site_node = MeshInstance3D.new()
		_site_node.name = "Sites"
		add_child(_site_node)
	_site_node.mesh = b.commit(wall_material)


## The stage model of `stages` (a list foundation, half, … in bindings3d `sites`) at work progress `p` (0–1), else "".
static func stage_model(stages: Variant, p: float) -> String:
	if stages is not Array or (stages as Array).is_empty():
		return ""
	var n: int = (stages as Array).size()
	return Data3D.first_model([(stages as Array)[clampi(int(p * n), 0, n - 1)]])


## Model `id` centred on `rect` at ground `g`, turned by `yaw`; `fit` scales its manifest footprint into the rect.
func _site_model(id: String, rect: Rect2, g: float, yaw: float, fit: bool) -> void:
	var mi: MeshInstance3D = MeshInstance3D.new()
	mi.mesh = Data3D.mesh(id)
	var s: float = 1.0
	var fp: Array = Data3D.model_row(id).get("footprint", [])
	if fit and fp.size() >= 2:
		var turned: bool = absf(sin(yaw)) > 0.5
		s = minf(rect.size.x / maxf(0.5, float(fp[1] if turned else fp[0])), rect.size.y / maxf(0.5, float(fp[0] if turned else fp[1])))
	var c: Vector2 = rect.get_center()
	mi.transform = Transform3D(Basis(Vector3.UP, yaw).scaled(Vector3(s, s, s)), Vector3(c.x, g, c.y))
	_site_models.add_child(mi)


## A footprint outline: low sills along the edges and taller stakes at the corners.
func _outline(b: Builder, r: Rect2i, g: float, c: Color, stake_h: float) -> void:
	var x0: float = r.position.x + 0.08
	var z0: float = r.position.y + 0.08
	var x1: float = r.end.x - 0.08
	var z1: float = r.end.y - 0.08
	var w: float = 0.07
	b.box(Vector3(x0, g - 0.02, z0), Vector3(x1, g + 0.07, z0 + w), c)
	b.box(Vector3(x0, g - 0.02, z1 - w), Vector3(x1, g + 0.07, z1), c)
	b.box(Vector3(x0, g - 0.02, z0), Vector3(x0 + w, g + 0.07, z1), c)
	b.box(Vector3(x1 - w, g - 0.02, z0), Vector3(x1, g + 0.07, z1), c)
	for p: Vector2 in [Vector2(x0, z0), Vector2(x1, z0), Vector2(x0, z1), Vector2(x1, z1)]:
		b.stake(p.x, p.y, g - 0.05, g + stake_h + 0.15, 0.05, c.darkened(0.1))


func _blueprint_rows() -> Dictionary:
	var out: Dictionary = {}
	for row: Variant in (get_parent().call("catalog") as Dictionary).get("blueprints", []):
		if row is Dictionary:
			out[int((row as Dictionary).get("code", -1))] = row
	return out


# --- cutaway -----------------------------------------------------------------------------------------------

## Room ids whose roof is cut away now (the selected room, the room of the inspected person).
func _selected_rooms() -> Dictionary:
	var out: Dictionary = {}
	var room_panel: Control = get_node_or_null("/root/Main/UI/RoomPanel") as Control
	if room_panel != null and room_panel.is_visible_in_tree():
		var id: Variant = room_panel.get("_room")
		if id is int and int(id) >= 0:
			out[int(id)] = true
	var inspector: Control = get_node_or_null("/root/Main/UI/AgentInspectorPanel") as Control
	if inspector != null and inspector.is_visible_in_tree() and inspector.has_method("get_selected_entity_bits"):
		var bits: int = int(inspector.call("get_selected_entity_bits"))
		if bits != -1:
			var agents: Dictionary = SimReads.read(_world_sim, "get_agent_snapshot")
			var ids: PackedInt64Array = agents.get("ids", PackedInt64Array())
			var i: int = ids.find(bits)
			if i >= 0:
				var t: Vector2i = Vector2i((agents["xs"] as PackedInt32Array)[i], (agents["ys"] as PackedInt32Array)[i])
				if _tile_room.has(t):
					out[int(_tile_room[t])] = true
	return out


func global_cut() -> bool:
	var camera: Node = get_node_or_null("/root/Main/Camera2D")
	if camera != null and float(camera.call("get_zoom_level")) >= float(Data3D.param("structures.cutaway_zoom", 4.0)) - 0.01:
		return true
	var rooms: Node = get_node_or_null("/root/Main/RoomIndex")
	return rooms != null and rooms.has_method("is_shown") and bool(rooms.call("is_shown"))


func _update_cutaway(delta: float) -> void:
	var speed: float = delta / maxf(0.01, float(Data3D.param("structures.cutaway_seconds", 0.25)))
	var all_cut: bool = global_cut()
	var selected: Dictionary = _selected_rooms()
	var low: float = float(Data3D.param("structures.cutaway_wall_height", 0.9))
	for id: int in _roofs:
		var target: float = 0.0 if all_cut or selected.has(id) else 1.0
		var f: float = move_toward(float(_roof_fade.get(id, 1.0)), target, speed)
		_roof_fade[id] = f
		var mi: MeshInstance3D = _roofs[id]
		mi.visible = f > 0.001
		mi.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_ON if f > 0.5 else GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
		if mi.mesh != null and mi.mesh.get_surface_count() > 0:
			(mi.get_active_material(0) as ShaderMaterial).set_shader_parameter("fade", f)
	_cut_all = move_toward(_cut_all, low if all_cut else 99.0, speed * 98.0)
	var rect: Rect2 = Rect2()
	for id: int in selected:
		if _room_rects.has(id):
			var r: Rect2i = _room_rects[id]
			var rr: Rect2 = Rect2(Vector2(r.position) - Vector2(1.6, 1.6), Vector2(r.size) + Vector2(3.2, 3.2))
			rect = rr if rect.size == Vector2.ZERO else rect.merge(rr)
	if rect.size != Vector2.ZERO:
		_cut_rect = rect
	_cut_rect_h = move_toward(_cut_rect_h, low if rect.size != Vector2.ZERO else 99.0, speed * 98.0)
	wall_material.set_shader_parameter("cut_all", _cut_all)
	wall_material.set_shader_parameter("cut_rect", Vector4(_cut_rect.position.x, _cut_rect.position.y, _cut_rect.end.x, _cut_rect.end.y))
	wall_material.set_shader_parameter("cut_rect_height", _cut_rect_h)
	for material: ShaderMaterial in _kit_wall_materials.values():
		material.set_shader_parameter("cut_all", _cut_all)
		material.set_shader_parameter("cut_rect", Vector4(_cut_rect.position.x, _cut_rect.position.y, _cut_rect.end.x, _cut_rect.end.y))
		material.set_shader_parameter("cut_rect_height", _cut_rect_h)


## The ray parameter where `origin + dir · t` meets a visible roof (its wall-top plane over the roofed rectangle
## plus the eaves), or INF. A click on a roof then lands on the room's tiles, not on the ground behind the house.
func roof_hit(origin: Vector3, dir: Vector3) -> float:
	if absf(dir.y) < 1e-5:
		return INF
	var over: float = float(Data3D.param("structures.roof_overhang", 0.3)) + 0.5
	var best: float = INF
	for id: int in _roofs:
		if float(_roof_fade.get(id, 1.0)) < 0.5 or not _room_rects.has(id) or not _roof_top.has(id):
			continue
		var t: float = (float(_roof_top[id]) - origin.y) / dir.y
		if t <= 0.0 or t >= best:
			continue
		var p: Vector3 = origin + dir * t
		var r: Rect2i = _room_rects[id]
		if Rect2(Vector2(r.position) - Vector2(over, over), Vector2(r.size) + Vector2(over, over) * 2.0).has_point(Vector2(p.x, p.z)):
			best = t
	return best


## Test/tool hooks.
func roof_count() -> int:
	return _roofs.size()


func roof_fade(id: int) -> float:
	return float(_roof_fade.get(id, 1.0))


func wall_tile_count() -> int:
	var n: int = 0
	for t: Variant in _walls.keys():
		if t is Vector2i:
			n += 1
	return n
