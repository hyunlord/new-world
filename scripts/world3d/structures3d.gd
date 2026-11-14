# structures3d.gd (World3D/Structures, made by world3d.gd): built space in 3D — walls, doors, fences, floors, roofs,
#   the cutaway and construction stages (M3D-1 scope 5).
# Purpose: every wall tile of get_space is extruded from the tile centre toward each wall/door/fence neighbour
#   (params3d `structures.wall_height`, `wall_thickness`), so corners, T and X joins come out of the geometry; the
#   profile comes from the tile's construction style through the 2D material resolver (scripts/world/art/
#   material_resolver.gd) and bindings3d `walls.by_style`: `log` stacks octagonal logs that cross and overhang at
#   corners, `block` is a masonry slab with mortar lines in the shader, `palisade` a row of pointed stakes. Colour is
#   the wall material (bindings3d `materials`) with the resolver's lightness tint. Wall tiles flagged as openings
#   (`wall_is_door`) with an opening neighbour are fence runs (posts and two rails); other openings are doors: two
#   posts and a lintel in the wall, and a door leaf (model `door`) that swings open while a person stands on it. A
#   finished hut (2D building binding with a `ring`) gets its ring of walls and its door from that binding. Floors are
#   thin slabs in the floor material. Roofs: each roofed room (get_rooms, not dug) is split into rectangles (greedy,
#   row-major) and each rectangle gets a gable along its long side from the wall centre lines, eaves
#   `structures.roof_overhang` beyond the walls at `structures.roof_pitch_deg`; the cover (thatch, bark, plank:
#   bindings3d `roofs`) follows the era of the template site that holds the room, else the room's wall category.
#   Cutaway: at the closest zoom (`structures.cutaway_zoom`) or with the room overlay (R, "방 보기") every roof
#   dissolves and walls drop to `structures.cutaway_wall_height`; with a room selected (RoomPanel) or a person in the
#   inspector standing in a roofed room, that room's roof dissolves and the walls around it drop. Construction: a
#   blueprint site of get_colony_overlay shows its footprint outline (stakes and sills); a wall or door site rises with
#   its work progress; a template site still building shows its whole footprint outline. Walls rebuild per 32×32 chunk
#   when the space version moves; roofs when the rooms version or the template sites change.
# Bridge methods called: via SimReads get_space, get_rooms, get_building_snapshot, get_colony_overlay,
#   get_agent_snapshot, get_content_catalog; signal world_reset.
extends Node3D

const Data3D = preload("res://scripts/world3d/data3d.gd")
const Builder = preload("res://scripts/world3d/mesh_builder3d.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
const MaterialResolver = preload("res://scripts/world/art/material_resolver.gd")
const WALL_SHADER = preload("res://shaders/world3d/wall.gdshader")
const ROOF_SHADER = preload("res://shaders/world3d/roof.gdshader")
const DIRS: Array[Vector2i] = [Vector2i(0, -1), Vector2i(1, 0), Vector2i(0, 1), Vector2i(-1, 0)]
const PROFILE_BLOCK: float = 0.0
const PROFILE_LOG: float = 1.0
const PROFILE_PALISADE: float = 2.0
const PROFILE_TRIM: float = 3.0

var wall_material: ShaderMaterial
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
var _cut_all: float = 99.0
var _cut_rect_h: float = 99.0
var _cut_rect: Rect2 = Rect2()
var _chunk: int = 32
var _stamp_key: String = ""
var _occupied: Dictionary = {}
var _occupied_tick: int = -2


func setup(world_sim: Node, terrain: Node) -> void:
	_world_sim = world_sim
	_terrain = terrain
	_world_sim.connect("world_reset", _on_world_reset)


func _ready() -> void:
	_resolver.load_imported()
	wall_material = ShaderMaterial.new()
	wall_material.shader = WALL_SHADER


func _on_world_reset() -> void:
	_space_version = -2
	_rooms_key = ""
	_sites_key = ""


func _process(delta: float) -> void:
	if _world_sim == null or _terrain == null or not bool(_terrain.call("is_complete")):
		return
	_chunk = maxi(4, int(Data3D.param("terrain.chunk_tiles", 32)))
	var stamp: Dictionary = get_parent().get("stamp") if get_parent() != null else {}
	var stamp_key: String = "%s|%s|%s" % [stamp.get("overlay", -1), stamp.get("rooms", -1), stamp.get("tick", -1)]
	if stamp_key == _stamp_key and not stamp.is_empty():
		_update_doors()
		_update_cutaway(delta)
		return
	_stamp_key = stamp_key
	var space: Dictionary = SimReads.read(_world_sim, "get_space")
	var buildings: Dictionary = SimReads.read(_world_sim, "get_building_snapshot")
	var version: int = int(space.get("version", 0))
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
	for t: Variant in _walls.keys():
		if t is Vector2i:
			dirty[_chunk_of(t)] = true
	for t: Vector2i in floors.keys():
		dirty[_chunk_of(t)] = true
	for k: Vector2i in _chunks.keys():
		dirty[k] = true
	for k: Vector2i in dirty:
		_build_wall_chunk(k, floors)
	_collect_doors()


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
	var profile: String = str(by_style.get(wall_style, by_style.get(style, Data3D.binding("walls.fallback", "block"))))
	return {"door": is_door, "fence": false, "profile": profile, "colour": c, "category": cat, "material": key}


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
	for i: int in mini(xs.size(), ys.size()):
		if i < kinds.size() and kinds[i] == 1:
			continue
		var raw: int = (mats[i] & 0xFFFFFFFF) if i < mats.size() and mats[i] != -1 else -1
		var floor_id: Variant = raw
		if raw < 0:
			floor_id = ""
		var r: Dictionary = _resolver.resolve(styles[i] if i < styles.size() else "", floor_id, "", "floor")
		var key: String = str(r.get("material_key", "")) if r.get("material_key") != null else ""
		out[Vector2i(xs[i], ys[i])] = Data3D.material_colour(key, str(r.get("category", "")) if r.get("category") != null else "")
	return out


func _connects(t: Vector2i, d: Vector2i) -> bool:
	return _walls.has(t + d)


func _build_wall_chunk(k: Vector2i, floors: Dictionary) -> void:
	var b: Builder = Builder.new()
	var hgt: float = float(Data3D.param("structures.wall_height", 2.2))
	var thick: float = float(Data3D.param("structures.wall_thickness", 0.35))
	var floor_h: float = float(Data3D.param("structures.floor_height", 0.06))
	for t: Variant in _walls.keys():
		if t is not Vector2i or _chunk_of(t) != k:
			continue
		var w: Dictionary = _walls[t]
		var g: float = float(_terrain.call("stand_height", t))
		b.base = g
		if bool(w["fence"]):
			_fence(b, t, g)
		elif bool(w["door"]):
			_door_frame(b, t, g, hgt, thick, w)
		else:
			_wall(b, t, g, hgt, thick, w)
	for t: Variant in floors.keys():
		if _chunk_of(t) != k:
			continue
		var g2: float = float(_terrain.call("stand_height", t))
		b.base = g2
		b.profile = PROFILE_TRIM
		var fc: Color = floors[t]
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
			var corner: bool = arms.size() != 2 or arms[0] + arms[1] != Vector2i.ZERO
			var over: float = r * 1.7 if corner else 0.0
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
				var from: float = -0.5 if neg else -over
				var to: float = 0.5 if pos else over
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


func _update_doors() -> void:
	if _doors.is_empty():
		return
	var tick: int = int((get_parent().get("stamp") as Dictionary).get("tick", -1)) if get_parent() != null else -1
	if tick != _occupied_tick:
		_occupied_tick = tick
		_occupied.clear()
		var agents: Dictionary = SimReads.read(_world_sim, "get_agent_snapshot")
		var xs: PackedInt32Array = agents.get("xs", PackedInt32Array())
		var ys: PackedInt32Array = agents.get("ys", PackedInt32Array())
		for i: int in mini(xs.size(), ys.size()):
			_occupied[Vector2i(xs[i], ys[i])] = true
	var occupied: Dictionary = _occupied
	var door_w: float = float(Data3D.param("structures.door_width", 0.9))
	var door_h: float = float(Data3D.param("structures.door_height", 2.0))
	var has_model: bool = Data3D.has_model("door")
	for i: int in mini(_doors.size(), _door_nodes.size()):
		var d: Dictionary = _doors[i]
		var t: Vector2i = d["tile"]
		var target: float = 1.0 if occupied.has(t) else 0.0
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
		_door_nodes[i].visible = cut > door_h


# --- roofs -------------------------------------------------------------------------------------------------

func _rebuild_roofs(rooms: Dictionary, sites: Array) -> void:
	for k: Variant in _roofs.keys():
		(_roofs[k] as Node).queue_free()
	_roofs.clear()
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
	for i: int in ids.size():
		var id: int = ids[i]
		if i >= roofed.size() or roofed[i] != 1 or (i < dug.size() and dug[i] == 1) or not tiles_of.has(id):
			continue
		var tiles: Array = tiles_of[id]
		var bounds: Rect2i = Rect2i(tiles[0], Vector2i.ONE)
		for t: Vector2i in tiles:
			bounds = bounds.expand(t).expand(t + Vector2i.ONE)
		_room_rects[id] = bounds
		var cover: String = _roof_cover(tiles, sites)
		var mi: MeshInstance3D = MeshInstance3D.new()
		mi.name = "Roof_%d" % id
		var mat: ShaderMaterial = ShaderMaterial.new()
		mat.shader = ROOF_SHADER
		mat.set_shader_parameter("cover", Data3D.colour((Data3D.binding("roofs.colours", {}) as Dictionary).get(cover, []), Color(0.7, 0.6, 0.35)))
		mat.set_shader_parameter("pattern", ["thatch", "bark", "plank"].find(cover) if cover in ["thatch", "bark", "plank"] else 0)
		mat.set_shader_parameter("fade", float(_roof_fade.get(id, 1.0)))
		mi.mesh = _roof_mesh(tiles, _gable_colour(tiles), mat)
		add_child(mi)
		_roofs[id] = mi


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
	return str(Data3D.binding("roofs.fallback", "thatch"))


func _gable_colour(tiles: Array) -> Color:
	for t: Vector2i in tiles:
		for dd: Vector2i in DIRS:
			var w: Variant = _walls.get(t + dd)
			if w is Dictionary and not bool((w as Dictionary)["door"]):
				return w["colour"]
	return Color(0.5, 0.4, 0.3)


## Greedy row-major split of a tile set into rectangles.
static func rectangles(tiles: Array) -> Array[Rect2i]:
	var left: Dictionary = {}
	for t: Vector2i in tiles:
		left[t] = true
	var ordered: Array = tiles.duplicate()
	ordered.sort_custom(func(a: Vector2i, b: Vector2i) -> bool: return a.y < b.y or (a.y == b.y and a.x < b.x))
	var out: Array[Rect2i] = []
	for t: Vector2i in ordered:
		if not left.has(t):
			continue
		var w: int = 1
		while left.has(t + Vector2i(w, 0)):
			w += 1
		var h: int = 1
		var grow: bool = true
		while grow:
			for x: int in w:
				if not left.has(t + Vector2i(x, h)):
					grow = false
					break
			if grow:
				h += 1
		for y: int in h:
			for x: int in w:
				left.erase(t + Vector2i(x, y))
		out.append(Rect2i(t, Vector2i(w, h)))
	return out


func _roof_mesh(tiles: Array, gable_colour: Color, mat: Material) -> ArrayMesh:
	var b: Builder = Builder.new()
	var hgt: float = float(Data3D.param("structures.wall_height", 2.2))
	var over: float = float(Data3D.param("structures.roof_overhang", 0.3)) + float(Data3D.param("structures.wall_thickness", 0.35)) * 0.5
	var pitch: float = tan(deg_to_rad(float(Data3D.param("structures.roof_pitch_deg", 38.0))))
	var thick: float = float(Data3D.param("structures.roof_thickness", 0.12))
	var ground: float = -INF
	for t: Vector2i in tiles:
		ground = maxf(ground, float(_terrain.call("stand_height", t)))
		for dd: Vector2i in DIRS:
			if _walls.has(t + dd):
				ground = maxf(ground, float(_terrain.call("stand_height", t + dd)))
	var top: float = ground + hgt
	_roof_top[_tile_room.get(tiles[0], -1)] = top
	b.base = ground
	# One ridge direction per room (its bounding box's long side), so the parts of an L meet in valleys.
	var bounds: Rect2i = Rect2i(tiles[0], Vector2i.ONE)
	for t: Vector2i in tiles:
		bounds = bounds.expand(t).expand(t + Vector2i.ONE)
	var along_x: bool = bounds.size.x >= bounds.size.y
	for r: Rect2i in rectangles(tiles):
		# Wall centre lines lie half a tile outside the room tiles.
		var x0: float = r.position.x - 0.5
		var x1: float = r.end.x + 0.5
		var z0: float = r.position.y - 0.5
		var z1: float = r.end.y + 0.5
		b.profile = 0.0 if along_x else 1.0
		_gable(b, x0, x1, z0, z1, along_x, top, over, pitch, thick, gable_colour)
	return b.commit(mat)


## One gable roof over the wall-line rectangle [x0, x1] × [z0, z1], ridge along x when `along_x`.
func _gable(b: Builder, x0: float, x1: float, z0: float, z1: float, along_x: bool, top: float, over: float, pitch: float, thick: float, gable: Color) -> void:
	# Local frame: a runs along the ridge, c across it; p(a, c, y) maps back to the world.
	var a0: float = x0 if along_x else z0
	var a1: float = x1 if along_x else z1
	var c0: float = z0 if along_x else x0
	var c1: float = z1 if along_x else x1
	var a_dir: Vector3 = Vector3(1, 0, 0) if along_x else Vector3(0, 0, 1)
	var c_dir: Vector3 = Vector3(0, 0, 1) if along_x else Vector3(1, 0, 0)
	var p: Callable = func(a: float, c: float, y: float) -> Vector3:
		return Vector3(a, y, c) if along_x else Vector3(c, y, a)
	var cm: float = (c0 + c1) * 0.5
	var ridge: float = top + (cm - c0) * pitch
	var eave: float = top - over * pitch
	var ea0: float = a0 - over
	var ea1: float = a1 + over
	var down: Vector3 = Vector3(0, thick, 0)
	var cover: Color = Color(1, 1, 1)
	var under: Color = cover.darkened(0.45)
	var edge: Color = cover.darkened(0.25)
	for side: float in [-1.0, 1.0]:
		var ec: float = (c0 - over) if side < 0.0 else (c1 + over)
		var out_top: Vector3 = Vector3.UP + c_dir * side
		var r0: Vector3 = p.call(ea0, cm, ridge)
		var r1: Vector3 = p.call(ea1, cm, ridge)
		var e1: Vector3 = p.call(ea1, ec, eave)
		var e0: Vector3 = p.call(ea0, ec, eave)
		b.quad_out(r0, r1, e1, e0, cover, out_top)
		b.quad_out(r0 - down, r1 - down, e1 - down, e0 - down, under, -out_top)
		b.quad_out(e0, e1, e1 - down, e0 - down, edge, c_dir * side)
		for end: float in [-1.0, 1.0]:
			var ea: float = ea0 if end < 0.0 else ea1
			var rr: Vector3 = p.call(ea, cm, ridge)
			var ee: Vector3 = p.call(ea, ec, eave)
			b.quad_out(rr, ee, ee - down, rr - down, edge, a_dir * end)
	# Gable triangles on the wall lines, in the wall colour (alpha 0: the roof shader leaves it untinted).
	var gc: Color = Color(gable.r, gable.g, gable.b, 0.0)
	for end2: float in [-1.0, 1.0]:
		var aw: float = a0 if end2 < 0.0 else a1
		b.tri_out(p.call(aw, c0, top), p.call(aw, cm, ridge - thick), p.call(aw, c1, top), gc, a_dir * end2)


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
	var categories: Dictionary = _blueprint_categories()
	var hgt: float = float(Data3D.param("structures.wall_height", 2.2))
	var thick: float = float(Data3D.param("structures.wall_thickness", 0.35))
	for i: int in mini(xs.size(), mini(ys.size(), mini(ws.size(), hs.size()))):
		var rect: Rect2i = Rect2i(xs[i], ys[i], ws[i], hs[i])
		var g: float = float(_terrain.call("stand_height", rect.position))
		b.base = g
		var p: float = clampf(float(prog[i]) / float(maxi(1, req[i])), 0.0, 1.0) if i < prog.size() and i < req.size() else 0.0
		var cat: String = str(categories.get(int(kinds[i]) if i < kinds.size() else -1, ""))
		if (cat == "wall" or cat == "door") and rect.size == Vector2i.ONE:
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
		b.base = g2
		_outline(b, r2, g2, sill.darkened(0.15), 0.35)
	if _site_node == null:
		_site_node = MeshInstance3D.new()
		_site_node.name = "Sites"
		add_child(_site_node)
	_site_node.mesh = b.commit(wall_material)


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


func _blueprint_categories() -> Dictionary:
	var out: Dictionary = {}
	for row: Variant in (get_parent().call("catalog") as Dictionary).get("blueprints", []):
		if row is Dictionary:
			out[int((row as Dictionary).get("code", -1))] = str((row as Dictionary).get("completion_category", ""))
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
	var rooms: Node = get_node_or_null("/root/Main/ColonyView/OverlayLayer/Rooms")
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
			(mi.mesh.surface_get_material(0) as ShaderMaterial).set_shader_parameter("fade", f)
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
