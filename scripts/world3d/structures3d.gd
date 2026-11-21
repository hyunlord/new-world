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
#   inspector standing in a roofed room, that room's roof dissolves and the walls around it drop. Style 2 (default)
#   keeps every roof closed at every zoom; it opens the selected house (walls drop), the house hovered for
#   `s2_house.hover_open_seconds` and, with the inside view (V), houses within `s2_house.inside_radius`. Construction: a
#   blueprint site of get_colony_overlay shows its footprint outline (stakes and sills); a wall or door site is its
#   blueprint id's M3D-ASSETS stage model (bindings3d `sites.walls`: foundation, half; stage = floor(progress × count))
#   along its wall run, else a block rising with its work progress; a template site still building is its template's
#   stage model (`sites.templates`, by completed / total pieces) scaled to its footprint and turned by its rotation,
#   else its footprint outline. Snapshots are read when the
#   overlay or rooms counter moves (or every `flora.slow_read_ticks`); walls rebuild per 32×32 chunk only when the
#   wall/floor content changes, roofs when the rooms version or the template sites change. Occupied roofed
#   surface rooms have emissive windows on their exterior walls at night.
# Style 2 takes building provenance from intact roofed template sites for shells, vents and work stages.
# Bridge methods called: via SimReads get_space, get_rooms, get_building_snapshot, get_colony_overlay,
#   get_agent_snapshot, get_content_catalog, get_settlement_snapshot; get_build_template_preview; signal world_reset.
extends Node3D

const Data3D = preload("res://scripts/world3d/data3d.gd")
const Builder = preload("res://scripts/world3d/mesh_builder3d.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
const MaterialResolver = preload("res://scripts/world/art/material_resolver.gd")
const SlicePbr = preload("res://scripts/world3d/slice_pbr.gd")
const HouseShell = preload("res://scripts/world3d/house_shell3d.gd")
const Props = preload("res://scripts/world3d/props3d.gd")
const Kit = preload("res://scripts/world3d/s2kit.gd")
const WALL_SHADER = preload("res://shaders/world3d/wall.gdshader")
const ROOF_SHADER = preload("res://shaders/world3d/roof.gdshader")
const DIRS: Array[Vector2i] = [Vector2i(0, -1), Vector2i(1, 0), Vector2i(0, 1), Vector2i(-1, 0)]
const PROFILE_BLOCK: float = 0.0
const PROFILE_LOG: float = 1.0
const PROFILE_PALISADE: float = 2.0
const PROFILE_TRIM: float = 3.0
# Room and template IDs each occupy their own u32 namespace in the bridge.
const S2_ROOM_ROOF_BASE: int = 1 << 32

var wall_material: ShaderMaterial
var _slice_pbr: bool = false
var _style2: bool = SlicePbr.style2()
## Style 2 "inside view" (HUD button / V): open the houses within `s2_house.inside_radius` tiles of the view focus.
var inside_view: bool = false
var _hover_house: int = -1
var _hover_seconds: float = 0.0
var _houses: Dictionary = {}
var _house_plans: Dictionary = {}
var _house_wall: Dictionary = {}
var _house_rooms: Dictionary = {}
var _room_at_tile: Dictionary = {}
var _s2_roof_footprints: Dictionary = {}
var _room_wall_trim: Dictionary = {}
var _house_stages: Dictionary = {}
var _house_progress: Dictionary = {}
var _house_sites: Dictionary = {}
var _has_house_work: bool = false
var _roof_details: Dictionary = {}
var _hearth_vents: Dictionary = {}
var _house_hearths: Dictionary = {}
var _lit_houses: Dictionary = {}
var _s2_lights_key: int = 0
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
## Wall chunk → content signature at its last build; a wall change rebuilds only the chunks whose signature moved.
var _chunk_sigs: Dictionary = {}
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
var _window_frames: Dictionary = {}
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
	if _style2:
		Kit.prewarm()
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
	if _style2 and enabled:
		wall_material.set_shader_parameter("clean_log", true)
	for roof: MeshInstance3D in _roofs.values():
		var mat: ShaderMaterial = roof.get_active_material(0) as ShaderMaterial
		if mat != null and mat.shader != Kit.SHADER and not SlicePbr.configure(mat, ROOF_SHADER, enabled):
			return false
	return true


func _on_world_reset() -> void:
	_has_house_work = false
	_house_plans.clear()
	_house_stages.clear()
	_house_progress.clear()
	_lit_houses.clear()
	_s2_lights_key = 0
	if _style2:
		_roof_fade.clear()
		_s2_roof_footprints.clear()
	_space_version = -2
	# A new world can put identical wall content on different ground: rebuild every chunk once.
	_chunk_sigs.clear()
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
	var interval: int = 1 if _has_house_work else maxi(1, int(Data3D.param("flora.slow_read_ticks", 24)))
	@warning_ignore("integer_division")
	var stamp_key: String = "%s|%s|%s" % [stamp.get("overlay", -1), stamp.get("rooms", -1), int(stamp.get("tick", -1)) / interval]
	# A rebuild takes World3D's heavy-work slot (one per frame, none on a simulation step); else try next frame.
	var claimed: bool = stamp_key != _stamp_key and (get_parent() == null or not get_parent().has_method("claim_heavy")
		or bool(get_parent().call("claim_heavy")))
	if (stamp_key == _stamp_key and not stamp.is_empty()) or not claimed:
		_update_doors()
		_update_cutaway(delta)
		return
	_stamp_key = stamp_key
	var started: int = Time.get_ticks_usec()
	var budget: int = int(float(Data3D.param("structures.rebuild_budget_ms", 12.0)) * 1000.0)
	var space: Dictionary = SimReads.read(_world_sim, "get_space")
	var buildings: Dictionary = SimReads.read(_world_sim, "get_building_snapshot")
	var sites: Array = buildings.get("template_sites", [])
	if _style2:
		_sync_houses(sites)
		_index_house_hearths(buildings)
	# Rebuild walls on wall/floor content, not on the shared overlay counter (hauling moves it every few ticks).
	var version: int = hash([space.get("wall_xs"), space.get("wall_ys"), space.get("wall_materials"), space.get("wall_styles"),
		space.get("wall_is_door"), space.get("floor_xs"), space.get("floor_ys"), space.get("floor_materials")])
	var bkey: int = hash([buildings.get("kinds"), buildings.get("xs"), buildings.get("ys")])
	if _style2:
		bkey = hash([bkey, _house_wall])
	if version != _space_version or bkey != int(_walls.get("_bkey", -1)):
		_space_version = version
		_rebuild_walls(space, buildings)
		_walls["_bkey"] = bkey
	var rooms: Dictionary = SimReads.read(_world_sim, "get_rooms")
	var rkey: String = "%d|%d|%d" % [int(rooms.get("version", 0)), hash(str(sites)), version]
	if _style2:
		rkey = "%s|%s|%s|%s" % [rooms.get("version"), bkey, version, hash(_houses.keys())]
	if rkey != _rooms_key and Time.get_ticks_usec() - started > budget:
		# Walls used this frame's slot: rebuild the roofs on the next claimed frame (no single-frame stall).
		_stamp_key = ""
		_update_doors()
		_update_cutaway(delta)
		return
	if rkey != _rooms_key:
		_rooms_key = rkey
		var old_room_trim: Dictionary = _room_wall_trim
		_rebuild_roofs(rooms, sites)
		if _style2 and old_room_trim != _room_wall_trim:
			var trim_chunks: Dictionary = {}
			for trim_set: Dictionary in [old_room_trim, _room_wall_trim]:
				for tile: Vector2i in trim_set:
					trim_chunks[_chunk_of(tile)] = true
			var floors: Dictionary = _floor_tiles(space)
			for chunk: Vector2i in trim_chunks:
				_build_wall_chunk(chunk, floors)
	if _style2 and not _roofs.is_empty():
		_update_hearths()
	var overlay: Dictionary = SimReads.read(_world_sim, "get_colony_overlay")
	var skey: String = "%d|%d" % [int(overlay.get("version", 0)), hash(str(sites))]
	if _style2:
		skey = "%s|%s" % [hash([overlay.get("site_xs"), overlay.get("site_ys"), overlay.get("site_ws"),
			overlay.get("site_hs"), overlay.get("site_kinds"), overlay.get("site_progress"), overlay.get("site_required")]), hash(str(sites))]
	if skey != _sites_key and Time.get_ticks_usec() - started > budget:
		_stamp_key = ""
		_update_doors()
		_update_cutaway(delta)
		return
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
			if n is Dictionary and bool((n as Dictionary)["door"]) and _connects(t, d):
				w["fence"] = true
				break
	var floors: Dictionary = _floor_tiles(space)
	var sigs: Dictionary = _wall_chunk_signatures(floors)
	var dirty: Dictionary = {}
	for k: Vector2i in sigs:
		if sigs[k] != _chunk_sigs.get(k):
			dirty[k] = true
	for keys: Array in [_chunk_sigs.keys(), _chunks.keys(), _kit_walls.keys()]:
		for k: Vector2i in keys:
			if not sigs.has(k):
				dirty[k] = true
	_chunk_sigs = sigs
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


## Per-chunk hash of everything a wall chunk build reads near its tiles: wall entries and their house, room trim,
## floors and floor models, each counted in its own chunk and any chunk within one tile (joins cross chunk edges).
func _wall_chunk_signatures(floors: Dictionary) -> Dictionary:
	var parts: Dictionary = {}
	var add: Callable = func(t: Vector2i, value: Variant) -> void:
		var seen: Dictionary = {}
		for d: Vector2i in [Vector2i.ZERO, Vector2i(1, 0), Vector2i(-1, 0), Vector2i(0, 1), Vector2i(0, -1)]:
			var k: Vector2i = _chunk_of(t + d)
			if not seen.has(k):
				seen[k] = true
				if not parts.has(k):
					parts[k] = []
				(parts[k] as Array).append(value)
	for t: Variant in _walls.keys():
		if t is not Vector2i:
			continue
		var house: Variant = _house_wall.get(t)
		var id: int = int((house as Dictionary).get("id", -1)) if house is Dictionary else -1
		add.call(t, [t, _walls[t], house, _room_wall_trim.has(t), hash(_houses.get(id)), hash(_house_plans.get(id))])
	for t: Vector2i in floors:
		add.call(t, [t, floors[t]])
	for t: Vector2i in _floor_models:
		add.call(t, [t, _floor_models[t]])
	var out: Dictionary = {}
	for k: Vector2i in parts:
		out[k] = hash(parts[k])
	return out


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
	return _walls.has(t + d) and _house_wall.get(t) == _house_wall.get(t + d)


func _build_wall_chunk(k: Vector2i, floors: Dictionary) -> void:
	var b: Builder = Builder.new()
	var kit_batches: Dictionary = {}
	var kit_profiles: Dictionary = Data3D.binding("s2kit.wall_profiles", {}) if _style2 else {}
	var kit_trim: Dictionary = Data3D.binding("s2kit.house_wall_trim", {}) if _style2 else {}
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
		var house: Dictionary = _house_wall.get(t, {})
		if not house.is_empty():
			var rounded: bool = bool(house["round"])
			if rounded:
				var pit: Dictionary = Data3D.binding("s2kit.pit_house", {})
				if bool(w["door"]) and bool(house["complete"]) and Kit.mesh(str(pit.get("base", ""))) == null:
					_door_frame(b, t, g, _round_height(), thick, w)
					var id: int = int(house["id"])
					_round_threshold(b, t, g, _house_door_side(id, _house_bounds(_houses[id])))
				continue
		var trimmed: bool = not house.is_empty() or _room_wall_trim.has(t)
		if trimmed and not bool(w["door"]):
			if str(w["profile"]) == "log":
				_kit_corner(str(kit_trim.get("corner", "")), t, g, hgt, thick, kit_batches)
			if not _kit_footing(str(kit_trim.get("footing", "")), t, g, thick, kit_batches) and not house.is_empty():
				_footing(b, t, g, thick)
		if bool(w["fence"]):
			if _fence_model(t) == "":
				_fence(b, t, g)
		elif bool(w["door"]):
			var authored: bool = trimmed and _kit_house_door(str(kit_trim.get("door", "")), t, g, hgt, thick, kit_batches)
			if not authored and (house.is_empty() or not _kit_door(t, g, hgt, thick, kit_batches)):
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
	var yaw: float = -PI * 0.5 if along_z else 0.0
	var centre: Vector3 = Vector3(tile.x + 0.5, ground, tile.y + 0.5)
	_append_kit_fit(id, model, Vector3(1.02, height, thickness), yaw, centre, batches)
	return true


func _append_kit_fit(id: String, model: Mesh, size: Vector3, yaw: float, ground: Vector3, batches: Dictionary, cut_offset: float = 0.0) -> void:
	if not batches.has(id):
		batches[id] = []
	(batches[id] as Array).append({"fit": Kit.fit(model, size, yaw, ground), "cut_offset": cut_offset})


func _kit_house_door(id: String, tile: Vector2i, ground: float, height: float, thickness: float, batches: Dictionary) -> bool:
	var model: ArrayMesh = Kit.mesh(id)
	if model == null:
		return false
	var axis: Vector2i = _run_axis(tile)
	var out: Vector2i = Vector2i(axis.y, axis.x)
	var house: Dictionary = _house_wall.get(tile, {})
	var room: int = int(_room_wall_trim.get(tile, -1))
	var centre: Vector2 = Vector2(tile) + Vector2.ONE * 0.5
	if not house.is_empty():
		var bounds: Rect2i = _house_bounds(_houses[int(house["id"])])
		if (centre - Rect2(bounds).get_center()).dot(Vector2(out)) < 0.0:
			out = -out
	elif int(_room_at_tile.get(tile + out, -2)) == room:
		out = -out
	centre += Vector2(out) * (thickness * 0.30)
	var width: float = float(Data3D.param("structures.door_width", 0.9)) + 0.22
	var top: float = minf(height, float(Data3D.param("structures.door_height", 2.0)) + 0.16)
	_append_kit_fit(id, model, Vector3(width, top, thickness * 0.84), atan2(out.x, out.y), Vector3(centre.x, ground, centre.y), batches)
	return true


func _kit_footing(id: String, tile: Vector2i, ground: float, thickness: float, batches: Dictionary) -> bool:
	var model: ArrayMesh = Kit.mesh(id)
	if model == null:
		return false
	var room: int = int(_room_wall_trim.get(tile, -1))
	var house: Dictionary = _house_wall.get(tile, {})
	for along: Vector2i in DIRS:
		var neighbour: Vector2i = tile + along
		if not _connects(tile, along) or bool((_walls[neighbour] as Dictionary)["door"]):
			continue
		if house.is_empty() and int(_room_wall_trim.get(neighbour, -2)) != room:
			continue
		var centre: Vector2 = Vector2(tile) + Vector2.ONE * 0.5 + Vector2(along) * 0.25
		_append_kit_fit(id, model, Vector3(thickness * 1.43, 0.28, 0.57), atan2(along.x, along.y),
			Vector3(centre.x, ground - 0.025, centre.y), batches)
	return true


func _kit_corner(id: String, tile: Vector2i, ground: float, height: float, thickness: float, batches: Dictionary) -> void:
	var model: ArrayMesh = Kit.mesh(id)
	if model == null:
		return
	var arms: Array[Vector2i] = []
	for direction: Vector2i in DIRS:
		if _connects(tile, direction):
			arms.append(direction)
	if arms.size() != 2 or Vector2(arms[0]).dot(Vector2(arms[1])) != 0.0:
		return
	var out: Vector2 = -Vector2(arms[0] + arms[1])
	var centre: Vector2 = Vector2(tile) + Vector2.ONE * 0.5 + out * (thickness * 0.38)
	var yaw: float = atan2(out.x, out.y) - PI * 0.25
	var count: int = maxi(3, roundi(height / thickness))
	var course: float = height / float(count)
	for row: int in count:
		var lift: float = course * (float(row) + 0.10)
		_append_kit_fit(id, model, Vector3(thickness * 0.85, course * 0.78, thickness * 0.90), yaw,
			Vector3(centre.x, ground + lift, centre.y), batches, lift)


## The kit's door and frame share one mesh. Mask the baked door in its local
## coordinates, leaving the real animated door leaf in the opening.
func _kit_door(tile: Vector2i, ground: float, height: float, thickness: float, batches: Dictionary) -> bool:
	var id: String = str(Data3D.binding("s2kit.door_frame", ""))
	var model: ArrayMesh = Kit.mesh(id)
	if model == null:
		return false
	var yaw: float = -PI * 0.5 if _run_axis(tile).y != 0 else 0.0
	var centre: Vector3 = Vector3(tile.x + 0.5, ground, tile.y + 0.5)
	_append_kit_fit(id, model, Vector3(1.18, height, thickness), yaw, centre, batches)
	return true


func _build_kit_wall_batches(chunk: Vector2i, batches: Dictionary) -> void:
	for old: MultiMeshInstance3D in _kit_walls.get(chunk, []):
		old.queue_free()
	_kit_walls.erase(chunk)
	if batches.is_empty():
		return
	var nodes: Array[MultiMeshInstance3D] = []
	var kit_trim: Dictionary = Data3D.binding("s2kit.house_wall_trim", {})
	for id: String in batches:
		var has_lod: bool = Kit.mesh(id, true) != null
		for lod: bool in [false, true]:
			var model: ArrayMesh = Kit.mesh(id, lod)
			if model == null:
				continue
			var transforms: Array = batches[id]
			var raised: bool = id == str(kit_trim.get("corner", ""))
			var multimesh: MultiMesh = MultiMesh.new()
			multimesh.transform_format = MultiMesh.TRANSFORM_3D
			multimesh.use_custom_data = raised
			multimesh.mesh = model
			multimesh.instance_count = transforms.size()
			for i: int in transforms.size():
				var entry: Dictionary = transforms[i]
				multimesh.set_instance_transform(i, entry["fit"])
				if raised:
					multimesh.set_instance_custom_data(i, Color(float(entry["cut_offset"]) / 4.0, 0.0, 0.0, 0.0))
			var material_key: String = id + ("_lod" if lod else "")
			if not _kit_wall_materials.has(material_key):
				var material: ShaderMaterial = Kit.material_for(model, true, id)
				if id == str(Data3D.binding("s2kit.wall_profiles.log", "")):
					var box: AABB = model.get_aabb()
					material.set_shader_parameter("clean_log", true)
					material.set_shader_parameter("log_size", box.size.max(Vector3.ONE * 0.0001))
					material.set_shader_parameter("log_origin", Vector3(box.position.x, box.position.y, box.get_center().z))
					var log_tile: Texture2D = Kit.texture(str(Data3D.binding("s2kit.house_log_tile", "")))
					if log_tile != null:
						material.set_shader_parameter("log_tile", log_tile)
						material.set_shader_parameter("mapped_log", true)
				elif id == str(Data3D.binding("s2kit.door_frame", "")):
					var box: AABB = model.get_aabb()
					var opening: float = float(Data3D.param("structures.door_width", 0.9)) / 1.18
					var door_height: float = float(Data3D.param("structures.door_height", 2.0))
					var wall_height: float = float(Data3D.param("structures.wall_height", 2.2))
					material.set_shader_parameter("door_opening", Vector3(box.get_center().x, box.size.x * opening * 0.5, box.position.y + box.size.y * door_height / wall_height))
				elif id in kit_trim.values():
					material.set_shader_parameter("wall_base_y", model.get_aabb().position.y)
					material.set_shader_parameter("raised_wall", raised)
				_kit_wall_materials[material_key] = material
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
	if _house_wall.has(t) and hgt <= _round_height():
		door_h = hgt * 0.81
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
		if t is Vector2i and bool(_walls[t]["door"]) and (not bool(_walls[t]["fence"]) or _house_wall.has(t)):
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
		var house: Dictionary = _house_wall.get(t, {})
		if bool(house.get("round", false)):
			cut = minf(cut, float(Data3D.param("s2_house.round_door_height", 0.78)))
			_door_nodes[i].transform = _door_nodes[i].transform.scaled_local(Vector3(0.80, 1.0, 1.0))
		var keep: float = clampf(cut / door_h, 0.05, 1.0)
		_door_nodes[i].transform = _door_nodes[i].transform.scaled_local(Vector3(1.0, keep, 1.0))


# --- roofs -------------------------------------------------------------------------------------------------

## The preview exposes the rotated physical plan. Cache its geometry for the
## lifetime of a site generation; rejection reasons do not change provenance.
func _sync_houses(sites: Array) -> void:
	_houses.clear()
	_house_wall.clear()
	_has_house_work = false
	for site: Dictionary in sites:
		var status: String = str(site.get("status", ""))
		if not bool(site.get("roofed", false)) or status == "cancelled" or (status == "complete" and not bool(site.get("is_intact", false))):
			continue
		var id: int = int(site["id"])
		_houses[id] = site
		_has_house_work = _has_house_work or status == "building"
		var source: int = int(site["source_record"])
		if not _house_plans.has(id) or int(_house_plans[id]["source"]) != source:
			var preview: Dictionary = _world_sim.get_build_template_preview(site["template_id"], site["x"], site["y"], site["rotation"], site["station"])
			var catalog: Dictionary = {}
			for blueprint: Dictionary in _blueprint_rows().values():
				catalog[str(blueprint["id"])] = blueprint
			var pieces: Dictionary = {}
			var walls: Dictionary = {}
			var doors: Dictionary = {}
			for piece: Dictionary in preview.get("pieces", []):
				var row: Dictionary = catalog.get(str(piece["kind"]), {})
				var tile: Vector2i = Vector2i(int(piece["x"]), int(piece["y"]))
				var category: String = str(row.get("completion_category", ""))
				pieces[[tile, int(row.get("code", -1))]] = true
				if category == "wall" or category == "door":
					walls[tile] = true
				if category == "door":
					doors[tile] = true
			# A monolithic legacy hut reports one piece; its rendered tile ring is
			# derived from the actual building binding in _add_hut_rings.
			_house_plans[id] = {"source": source, "pieces": pieces, "walls": walls, "doors": doors,
				"footprint": HouseShell.enclosed_tiles(_house_bounds(site), walls)}
		var rounded: bool = _round_shell(id)
		if status == "complete" or rounded:
			var plan: Dictionary = _house_plans[id]
			var bounds: Rect2i = _house_bounds(site)
			var inner: Rect2i = bounds.grow(-1)
			for tile: Vector2i in HouseShell.tiles(bounds):
				if (plan["walls"] as Dictionary).has(tile) or ((plan["walls"] as Dictionary).is_empty() and not inner.has_point(tile)):
					_house_wall[tile] = {"id": id, "round": rounded, "complete": status == "complete"}


static func _house_bounds(site: Dictionary) -> Rect2i:
	return Rect2i(int(site["x"]), int(site["y"]), int(site["width"]), int(site["height"]))


func _round_shell(id: int) -> bool:
	if not _houses.has(id) or not _house_plans.has(id):
		return false
	var site: Dictionary = _houses[id]
	var size: Vector2i = _house_bounds(site).size
	var plan: Dictionary = _house_plans[id]
	var footprint: Array = plan["footprint"]
	var walls: Dictionary = plan["walls"]
	var limit: int = int(Data3D.param("s2_house.round_max_span", 3))
	return str(site.get("art_roof", "")) == "round" and mini(size.x, size.y) >= 2 and maxi(size.x, size.y) <= limit \
		and footprint.size() == size.x * size.y and footprint.size() > walls.size()


static func _round_height() -> float:
	return float(Data3D.param("s2_house.round_wall_height", 1.25))


func _window_height(id: int) -> float:
	return minf(_window_elevation, _round_height() * 0.60) if _style2 and _round_shell(id) else _window_elevation


func _footing(b: Builder, tile: Vector2i, ground: float, thick: float) -> void:
	b.profile = PROFILE_BLOCK
	var previous: float = b.texture_tile
	b.texture_tile = preload("res://scripts/world3d/textures3d.gd").index("surfaces", "stone")
	var centre: Vector3 = Vector3(tile.x + 0.5, ground, tile.y + 0.5)
	var stone: Color = Data3D.colour(Data3D.param("s2_house.footing", []), Color(0.48, 0.46, 0.41))
	for direction: Vector2i in DIRS:
		if _house_wall.get(tile + direction) != _house_wall[tile] or bool((_walls.get(tile + direction, {}) as Dictionary).get("door", false)):
			continue
		var along: Vector3 = Vector3(direction.x, 0.0, direction.y) * 0.52
		var half: Vector3 = Vector3(thick * 0.72, 0.0, thick * 0.72)
		var end: Vector3 = centre + along
		b.box(centre.min(end) - half - Vector3.UP * 0.06, centre.max(end) + half + Vector3.UP * 0.28, stone)
	b.texture_tile = previous


func _round_threshold(b: Builder, tile: Vector2i, ground: float, facing: Vector2) -> void:
	var stone: Color = Data3D.colour(Data3D.param("s2_house.footing", []))
	b.profile = PROFILE_TRIM
	b.texture_tile = preload("res://scripts/world3d/textures3d.gd").index("surfaces", "stone")
	var step: Vector3 = Vector3(tile.x + 0.5, ground, tile.y + 0.5) + Vector3(facing.x, 0.0, facing.y) * 0.08
	var half: Vector3 = Vector3(0.30, 0.0, 0.30) + Vector3(absf(facing.y), 0.0, absf(facing.x)) * 0.09
	b.box(step - half + Vector3.DOWN * 0.08, step + half + Vector3.UP * 0.13, stone)


func _rebuild_house_roofs(rooms: Dictionary) -> void:
	for node: Node3D in _roof_details.values():
		node.hide()
		node.queue_free()
	for id: int in _hearth_vents:
		for vent: CPUParticles3D in _hearth_vents[id]:
			vent.hide()
			vent.queue_free()
	_hearth_vents.clear()
	_roof_details.clear()
	_house_rooms.clear()
	_room_wall_trim = {}
	_s2_roof_footprints.clear()
	_s2_lights_key = 0
	_room_at_tile.clear()
	var tx: PackedInt32Array = rooms.get("tile_xs", PackedInt32Array())
	var ty: PackedInt32Array = rooms.get("tile_ys", PackedInt32Array())
	var ids: PackedInt32Array = rooms.get("tile_rooms", PackedInt32Array())
	var room_tiles: Dictionary = {}
	for i: int in mini(ids.size(), mini(tx.size(), ty.size())):
		var tile: Vector2i = Vector2i(tx[i], ty[i])
		_room_at_tile[tile] = ids[i]
		if not room_tiles.has(ids[i]):
			room_tiles[ids[i]] = []
		(room_tiles[ids[i]] as Array).append(tile)
	var packed_ground: Dictionary = {}
	var template_tiles: Dictionary = {}
	for id: int in _houses:
		var site: Dictionary = _houses[id]
		var footprint: Array[Vector2i] = _house_plans[id]["footprint"]
		for tile: Vector2i in footprint:
			template_tiles[tile] = true
		if str(site["status"]) != "complete":
			continue
		var bounds: Rect2i = _house_bounds(site)
		var interior: Array[Vector2i] = []
		for tile: Vector2i in footprint:
			_tile_room[tile] = id
			if _room_at_tile.has(tile):
				_house_rooms[int(_room_at_tile[tile])] = id
			if not _house_wall.has(tile):
				interior.append(tile)
				packed_ground[tile] = true
		_room_rects[id] = bounds
		var facing: Vector2 = _house_door_side(id, bounds)
		var ground: float = _roof_ground(footprint)
		var mi: MeshInstance3D = MeshInstance3D.new()
		mi.name = "HouseRoof_%d" % id
		add_child(mi)
		_make_house_roof(mi, site, bounds, footprint, ground, facing, 1.0)
		_roofs[id] = mi
		_s2_roof_footprints[id] = footprint
		_add_windows(id, interior)
	_add_room_shells(rooms, room_tiles, template_tiles, packed_ground)
	for id: int in _roof_fade.keys():
		if not _roofs.has(id):
			_roof_fade.erase(id)
	_terrain.set_house_ground(packed_ground)


func _add_room_shells(rooms: Dictionary, room_tiles: Dictionary, template_tiles: Dictionary, packed_ground: Dictionary) -> void:
	var ids: PackedInt32Array = rooms.get("ids", PackedInt32Array())
	var roles: PackedInt32Array = rooms.get("roles", PackedInt32Array())
	var roofed: PackedByteArray = rooms.get("roofed", PackedByteArray())
	var dug: PackedByteArray = rooms.get("dug", PackedByteArray())
	for i: int in mini(ids.size(), roofed.size()):
		var room: int = ids[i]
		if roofed[i] == 0 or (i < dug.size() and dug[i] != 0) or _house_rooms.has(room):
			continue
		var interior: Array = room_tiles.get(room, [])
		if interior.is_empty():
			continue
		var role: int = roles[i] if i < roles.size() else 0
		var footprint: Array = _built_room_footprint(interior, template_tiles, role == 1 or role == 2)
		if footprint.is_empty():
			continue
		var bounds: Rect2i = Rect2i(footprint[0], Vector2i.ONE)
		var id: int = S2_ROOM_ROOF_BASE + room
		for tile: Vector2i in footprint:
			bounds = bounds.expand(tile).expand(tile + Vector2i.ONE)
			# A shared wall is clickable for the first of its adjoining rooms.
			if not _tile_room.has(tile):
				_tile_room[tile] = id
			if _walls.has(tile) and not template_tiles.has(tile) and not _room_wall_trim.has(tile):
				_room_wall_trim[tile] = room
		_house_rooms[room] = id
		_room_rects[id] = bounds
		var mi: MeshInstance3D = MeshInstance3D.new()
		mi.name = "RoomRoof_%d" % room
		add_child(mi)
		_make_house_roof(mi, {"id": id}, bounds, footprint, _roof_ground(footprint), Vector2.DOWN, 1.0)
		_roofs[id] = mi
		_s2_roof_footprints[id] = footprint
		for tile: Vector2i in interior:
			packed_ground[tile] = true
		_add_windows(id, interior)


func _built_room_footprint(interior: Array, template_tiles: Dictionary, room_use: bool) -> Array:
	var cells: Dictionary = {}
	var perimeter: Dictionary = {}
	for tile: Vector2i in interior:
		if template_tiles.has(tile):
			return []
		cells[tile] = true
	var enclosed: bool = true
	var door: bool = false
	for tile: Vector2i in interior:
		for direction: Vector2i in DIRS:
			var border: Vector2i = tile + direction
			if cells.has(border) or perimeter.has(border):
				continue
			var wall: Dictionary = _walls.get(border, {})
			if wall.is_empty() or bool(wall["fence"]):
				enclosed = false
				continue
			perimeter[border] = true
			door = door or bool(wall["door"])
	if perimeter.is_empty() or not (room_use or (enclosed and door)):
		return []
	for tile: Vector2i in interior:
		for i: int in DIRS.size():
			var side: Vector2i = DIRS[i]
			var next: Vector2i = DIRS[(i + 1) % DIRS.size()]
			if perimeter.has(tile + side) and perimeter.has(tile + next):
				var corner: Vector2i = tile + side + next
				var wall: Dictionary = _walls.get(corner, {})
				if not wall.is_empty() and not bool(wall["fence"]):
					cells[corner] = true
	cells.merge(perimeter)
	return cells.keys()


func _house_door_side(id: int, bounds: Rect2i) -> Vector2:
	for tile: Vector2i in HouseShell.tiles(bounds):
		if (_house_plans[id]["doors"] as Dictionary).has(tile) or (int((_house_wall.get(tile, {}) as Dictionary).get("id", -1)) == id and bool((_walls.get(tile, {}) as Dictionary).get("door", false))):
			return (Vector2(tile) + Vector2.ONE * 0.5 - Rect2(bounds).get_center()).normalized()
	return Vector2.DOWN.rotated(float(int(_houses[id]["rotation"])) * PI * 0.5)


func _house_roof_material() -> ShaderMaterial:
	var mat: ShaderMaterial = ShaderMaterial.new()
	mat.shader = ROOF_SHADER
	preload("res://scripts/world3d/textures3d.gd").setup(mat)
	var cover: String = str(Data3D.binding("roofs.fallback", ""))
	mat.set_shader_parameter("texture_tile", float(preload("res://scripts/world3d/textures3d.gd").index("roofs", cover)))
	mat.set_shader_parameter("cover", Data3D.colour(Data3D.param("s2_house.thatch", []), Color(0.78, 0.66, 0.42)))
	return mat


func _make_house_roof(node: MeshInstance3D, site: Dictionary, bounds: Rect2i, footprint: Array, ground: float, facing: Vector2, layers: float) -> void:
	var id: int = int(site["id"])
	var rounded: bool = _round_shell(id)
	var irregular: bool = footprint.size() != bounds.get_area()
	var center: Vector2 = Rect2(bounds).get_center()
	var height: float = _round_height() if rounded else float(Data3D.param("structures.wall_height", 2.2))
	var eave: float = ground + height + 0.03
	var pitch_key: String = "s2_house.round_roof_pitch_deg" if rounded else "structures.roof_pitch_deg"
	var pitch: float = tan(deg_to_rad(float(Data3D.param(pitch_key, 38.0))))
	var rise_max: float = float(Data3D.param("structures.roof_max_rise", 2.4))
	var radius: Vector2 = Vector2(bounds.size) * 0.51
	var eave_radius: Vector2 = radius * float(Data3D.param("s2_house.round_eave_ratio", 0.88)) if rounded else radius
	var rise: float = minf(rise_max, minf(eave_radius.x, eave_radius.y) * pitch)
	if rounded and layers >= 1.0 and _install_pit_house(node, id, center, radius, ground, facing):
		return
	var mat: ShaderMaterial = _house_roof_material()
	var hearths: Dictionary = _hearths_in(footprint)
	var vents: Array[Vector3] = []
	var roof_parts: Dictionary = Data3D.binding("s2kit.house_roof_trim", {})
	var ridge_id: String = str(roof_parts.get("ridge", "")) if layers >= 1.0 and not (rounded or irregular) else ""
	var ridge: ArrayMesh = Kit.mesh(ridge_id)
	var vent_id: String = str(roof_parts.get("vent", "")) if maxi(bounds.size.x, bounds.size.y) >= 6 else ""
	var vent_part: ArrayMesh = Kit.mesh(vent_id)
	var aperture_half: float = 0.17 if vent_part != null else 0.12
	if not irregular and layers >= 1.0:
		if not rounded:
			for tile: Vector2i in hearths.values():
				vents.append(_vent_point(center, radius, eave, rise, rounded, tile))
		if vents.is_empty():
			vents.append(Vector3(center.x, eave + rise, center.y))
	if rounded:
		mat.set_shader_parameter("radial_thatch", true)
		mat.set_shader_parameter("cone_center", center)
		node.mesh = HouseShell.cone(center, eave_radius, eave, rise, not vents.is_empty(), mat)
	else:
		node.mesh = HouseShell.angular_roof(footprint, bounds, ground, eave, pitch, rise_max, mat, vents, ridge != null, aperture_half)
	if layers < 1.0:
		mat.set_shader_parameter("lay_top", eave + rise * layers)
		return
	_roof_top[id] = eave + rise * 0.42
	var trim: Builder = Builder.new()
	trim.base = ground
	var details: MeshInstance3D = MeshInstance3D.new()
	details.name = "HouseTrim_%d" % id
	add_child(details)
	if rounded:
		var berm: Builder = Builder.new()
		HouseShell.earth_ring(berm, center, radius, ground, height, facing)
		var base_node: MeshInstance3D = MeshInstance3D.new()
		base_node.mesh = berm.commit(wall_material)
		# The plinth stays when its roof is lifted.
		add_child(base_node)
		_roof_details[-id - 1] = base_node
	elif not irregular:
		_mount_gable_trim(details, trim, roof_parts, center, radius, ground, eave, rise)
		if ridge != null:
			_mount_ridge_trim(details, ridge_id, ridge, center, radius, eave + rise, vents, aperture_half)
	var ridge_x: bool = radius.x > radius.y
	for position: Vector3 in vents:
		if vent_part == null:
			HouseShell.vent(trim, position)
		else:
			var opening: MeshInstance3D = Kit.fitted_part(vent_id, vent_part, Vector3(0.44, 0.20, 0.44),
				0.0 if ridge_x else -PI * 0.5, position - Vector3.UP * 0.08)
			opening.name = "SmokeOpening"
			details.add_child(opening, true)
	if not trim.is_empty():
		details.mesh = trim.commit(_house_roof_material())
	if details.mesh != null or details.get_child_count() > 0:
		_roof_details[id] = details
	else:
		details.queue_free()


func _mount_gable_trim(group: Node3D, fallback: Builder, parts: Dictionary, center: Vector2, radius: Vector2, ground: float, eave: float, rise: float) -> void:
	var gable_id: String = str(parts.get("gable", ""))
	var model: ArrayMesh = Kit.mesh(gable_id)
	var ridge_x: bool = radius.x > radius.y
	var ridge_dir: Vector3 = Vector3.RIGHT if ridge_x else Vector3.BACK
	var span: float = maxf(radius.x, radius.y)
	var width: float = minf(radius.x, radius.y)
	var base: Vector3 = Vector3(center.x, eave, center.y)
	if model == null:
		HouseShell.gable_ends(fallback, center, radius, ground, eave, rise)
	else:
		for side: float in [-1.0, 1.0]:
			var yaw: float = (0.0 if ridge_x else -PI * 0.5) + (PI if side < 0 else 0.0)
			var pos: Vector3 = base + ridge_dir * (span - 0.20) * side - Vector3.UP * 0.08
			var part: MeshInstance3D = Kit.fitted_part(gable_id, model, Vector3(0.33, rise * 1.17, width * 1.70), yaw, pos)
			part.name = "GableTrim_%s" % ("Near" if side > 0.0 else "Far")
			group.add_child(part)
	_mount_eave_trim(group, str(parts.get("eave", "")), base, ridge_dir, span, width, ridge_x)


func _mount_eave_trim(group: Node3D, id: String, base: Vector3, along: Vector3, span: float, width: float, ridge_x: bool) -> void:
	var model: ArrayMesh = Kit.mesh(id)
	if model == null:
		return
	var count: int = maxi(1, roundi(span * 2.0 / 0.64))
	var spacing: float = span * 2.0 / count
	var across: Vector3 = Vector3.BACK if ridge_x else Vector3.RIGHT
	var batch: MultiMesh = MultiMesh.new()
	batch.transform_format = MultiMesh.TRANSFORM_3D
	batch.mesh = model
	batch.instance_count = count * 2
	for row: int in 2:
		var side: float = 1.0 if row == 0 else -1.0
		var yaw: float = (0.0 if ridge_x else PI * 0.5) + row * PI
		for i: int in count:
			var pos: Vector3 = base + across * (width + 0.018) * side + along * (-span + (i + 0.5) * spacing) - Vector3.UP * 0.11
			batch.set_instance_transform(row * count + i, Kit.fit(model, Vector3(spacing, 0.16, 0.18), yaw, pos))
	var fringe: MultiMeshInstance3D = MultiMeshInstance3D.new()
	fringe.name = "ThatchFringe"
	fringe.multimesh = batch
	fringe.material_override = Kit.material_for(model, false, id)
	group.add_child(fringe)


func _mount_ridge_trim(group: Node3D, id: String, model: ArrayMesh, center: Vector2, radius: Vector2, top: float,
		vents: Array[Vector3], aperture: float) -> void:
	var along_x: bool = radius.x > radius.y
	var along: Vector3 = Vector3.RIGHT if along_x else Vector3.BACK
	var mid: Vector3 = Vector3(center.x, top, center.y)
	var length: float = maxf(radius.x, radius.y)
	var holes: Array[float] = []
	for vent: Vector3 in vents:
		holes.append((vent - mid).dot(along))
	holes.sort()
	holes.append(length + aperture)
	var start: float = -length
	var number: int = 0
	for hole: float in holes:
		var finish: float = minf(length, hole - aperture)
		var span: float = finish - start
		if span > 0.03:
			var sections: int = maxi(1, roundi(span / 1.55))
			var step: float = span / sections
			for i: int in sections:
				var pos: Vector3 = mid + along * (start + (i + 0.5) * step) + Vector3.UP * 0.02
				var pole: MeshInstance3D = Kit.fitted_part(id, model, Vector3(step + 0.012, 0.14, 0.19), 0.0 if along_x else -PI * 0.5, pos)
				pole.name = "RidgeTrim_%d" % number
				group.add_child(pole)
				number += 1
		start = maxf(start, hole + aperture)


func _install_pit_house(node: MeshInstance3D, id: int, center: Vector2, radius: Vector2, ground: float, facing: Vector2) -> bool:
	var kit: Dictionary = Data3D.binding("s2kit.pit_house", {})
	var whole_id: String = str(kit.get("whole", ""))
	var roof_id: String = str(kit.get("roof", ""))
	var base_id: String = str(kit.get("base", ""))
	var whole: ArrayMesh = Kit.mesh(whole_id)
	var roof: ArrayMesh = Kit.mesh(roof_id)
	var base: ArrayMesh = Kit.mesh(base_id)
	if whole == null or roof == null or base == null:
		return false
	var yaw: float = atan2(facing.x, facing.y) - deg_to_rad(float(kit.get("door_yaw_deg", 0.0)))
	var fit: Transform3D = Kit.fit_proportional(whole, radius * 2.0, yaw, Vector3(center.x, ground, center.y))
	node.mesh = roof
	node.transform = fit
	node.material_override = Kit.material_for(roof, false, roof_id)
	node.set_meta("house_kit", whole_id)
	node.set_meta("roof_rest_y", fit.origin.y)
	var tip: float = (fit * roof.get_aabb().end).y
	_roof_top[id] = lerpf(ground, tip, 0.66)
	node.set_meta("house_smoke_y", tip)
	Kit.add_lod(node, roof_id)
	var plinth: MeshInstance3D = MeshInstance3D.new()
	plinth.name = "PitBase_%d" % id
	plinth.mesh = base
	plinth.transform = fit
	if not _kit_wall_materials.has(base_id):
		_kit_wall_materials[base_id] = Kit.material_for(base, true, base_id)
	plinth.material_override = _kit_wall_materials[base_id]
	Kit.add_lod(plinth, base_id)
	add_child(plinth)
	_roof_details[-id - 1] = plinth
	return true


func _index_house_hearths(buildings: Dictionary) -> void:
	_house_hearths.clear()
	var bx: PackedInt32Array = buildings.get("xs", PackedInt32Array())
	var by: PackedInt32Array = buildings.get("ys", PackedInt32Array())
	var kinds: PackedByteArray = buildings.get("kinds", PackedByteArray())
	for i: int in mini(kinds.size(), mini(bx.size(), by.size())):
		if bool(ArtData.binding("buildings", int(kinds[i])).get("light", false)):
			_house_hearths[Vector2i(bx[i], by[i])] = true


## One exhaust per room, with a house-wide fallback for huts without Room rows.
func _hearths_in(footprint: Array) -> Dictionary:
	var out: Dictionary = {}
	for tile: Vector2i in footprint:
		if _house_hearths.has(tile):
			var room: int = int(_room_at_tile.get(tile, -1))
			if not out.has(room):
				out[room] = tile
	return out


static func _vent_point(center: Vector2, radius: Vector2, eave: float, rise: float, rounded: bool, tile: Vector2i) -> Vector3:
	var offset: Vector2 = Vector2(tile) + Vector2.ONE * 0.5 - center
	var ridge: Vector2 = Vector2.ZERO
	var height: float = eave + rise
	if not rounded:
		ridge = Vector2(clampf(offset.x, -radius.x * 0.78, radius.x * 0.78), 0.0) if radius.x > radius.y else Vector2(0.0, clampf(offset.y, -radius.y * 0.78, radius.y * 0.78))
	return Vector3(center.x + ridge.x, height, center.y + ridge.y)


func _update_hearths() -> void:
	var settlements: Dictionary = SimReads.read(_world_sim, "get_settlement_snapshot")
	var key: int = hash([_house_hearths, settlements.get("formation_xs"), settlements.get("formation_ys"), settlements.get("fire_lit")])
	if _s2_lights_key == key:
		return
	_s2_lights_key = key
	_lit_houses.clear()
	for id: int in _roofs:
		for old: CPUParticles3D in _hearth_vents.get(id, []):
			old.hide()
			old.queue_free()
		var vents: Array[CPUParticles3D] = []
		var hearths: Dictionary = _hearths_in(_s2_roof_footprints[id])
		var bounds: Rect2i = _room_rects[id]
		var centre: Vector2 = Rect2(bounds).get_center()
		var radius: Vector2 = Vector2(bounds.size) * 0.51
		var rounded: bool = _round_shell(id)
		var pitch: float = tan(deg_to_rad(float(Data3D.param("structures.roof_pitch_deg", 38.0))))
		var rise: float = minf(float(Data3D.param("structures.roof_max_rise", 2.4)), minf(radius.x, radius.y) * pitch)
		var eave: float = float(_roof_top[id]) - rise * 0.42
		for tile: Vector2i in hearths.values():
			if not Props._fire_lit(settlements, tile):
				continue
			var mouth: Vector3 = _vent_point(centre, radius, eave, rise, rounded, tile)
			mouth.y = float((_roofs[id] as Node3D).get_meta("house_smoke_y", mouth.y))
			var vent: CPUParticles3D = HouseShell.smoke(mouth + Vector3.UP * 0.16)
			vent.visible = float(_roof_fade.get(id, 1.0)) > 0.55
			add_child(vent)
			vents.append(vent)
			if rounded:
				break
		_hearth_vents[id] = vents
		_lit_houses[id] = not vents.is_empty()


func _rebuild_house_sites(b: Builder, overlay: Dictionary) -> void:
	_house_stages.clear()
	_house_progress.clear()
	_house_sites.clear()
	var work: Dictionary = {}
	var xs: PackedInt32Array = overlay.get("site_xs", PackedInt32Array())
	var ys: PackedInt32Array = overlay.get("site_ys", PackedInt32Array())
	var kinds: PackedInt64Array = overlay.get("site_kinds", PackedInt64Array())
	var progress: PackedInt32Array = overlay.get("site_progress", PackedInt32Array())
	var required: PackedInt32Array = overlay.get("site_required", PackedInt32Array())
	for i: int in mini(kinds.size(), mini(xs.size(), ys.size())):
		if i < progress.size() and i < required.size():
			work[[Vector2i(xs[i], ys[i]), kinds[i]]] = clampf(float(progress[i]) / maxf(1.0, float(required[i])), 0.0, 1.0)
	for id: int in _houses:
		var site: Dictionary = _houses[id]
		if str(site["status"]) == "complete":
			_house_stages[id] = 4
			_house_progress[id] = 1.0
			continue
		var plan: Dictionary = _house_plans[id]
		var work_done: float = float(site["completed_pieces"])
		for key: Array in plan["pieces"]:
			_house_sites[key] = true
			work_done += float(work.get(key, 0.0))
		var p: float = clampf(work_done / maxf(1.0, float(site["total_pieces"])), 0.0, 0.999)
		var stage: int = 0 if p <= 0.001 else (1 if p < 0.31 else (2 if p < 0.70 else 3))
		_house_stages[id] = stage
		_house_progress[id] = p
		var bounds: Rect2i = _house_bounds(site)
		var footprint: Array[Vector2i] = plan["footprint"]
		var g: float = _roof_ground(footprint)
		var rounded: bool = _round_shell(id)
		var h: float = _round_height() if rounded else float(Data3D.param("structures.wall_height", 2.2))
		var walls: Dictionary = {}
		for tile: Vector2i in plan["walls"]:
			walls[tile] = _walls.has(tile)
		var doors: Dictionary = plan["doors"]
		if walls.is_empty():
			doors = {}
			var interior: Rect2i = bounds.grow(-1)
			for tile: Vector2i in footprint:
				if not interior.has_point(tile):
					walls[tile] = _walls.has(tile)
			var centre: Vector2 = Rect2(bounds).get_center()
			var entry: Vector2 = centre + Vector2.DOWN.rotated(float(int(site["rotation"])) * PI * 0.5) * (Vector2(bounds.size).length() * 0.31)
			doors[Vector2i(floori(entry.x), floori(entry.y))] = true
		var facing: Vector2 = _house_door_side(id, bounds)
		if rounded:
			HouseShell.round_frame(b, Rect2(bounds).get_center(), Vector2(bounds.size) * 0.51, g, h, stage, p, facing)
		else:
			HouseShell.frame(b, bounds, g, h, stage, p, walls, doors)
		if stage == 3:
			var partial: MeshInstance3D = MeshInstance3D.new()
			_site_models.add_child(partial)
			_make_house_roof(partial, site, bounds, footprint, g, facing, lerpf(0.13, 0.96, (p - 0.70) / 0.30))


func house_stage(id: int) -> int:
	return int(_house_stages.get(id, -1))


func house_progress(id: int) -> float:
	return float(_house_progress.get(id, -1.0))


func house_fade(id: int) -> float:
	return float(_roof_fade.get(id, 1.0)) if _roofs.has(id) else -1.0


func roof_key_at(tile: Vector2i) -> int:
	var id: int = int(_tile_room.get(tile, -1))
	return id if _style2 and _s2_roof_footprints.has(id) else -1


func untemplated_room_at(tile: Vector2i) -> int:
	var id: int = roof_key_at(tile)
	return id - S2_ROOM_ROOF_BASE if id >= S2_ROOM_ROOF_BASE else -1


func room_roof_fade(room: int) -> float:
	return house_fade(int(_house_rooms.get(room, -1)))


func _rebuild_roofs(rooms: Dictionary, sites: Array) -> void:
	for k: Variant in _roofs.keys():
		(_roofs[k] as Node3D).hide()
		(_roofs[k] as Node).queue_free()
	_roofs.clear()
	_clear_windows()
	_occupied_rooms.clear()
	_occupied_tick = -2
	_room_rects.clear()
	_roof_top.clear()
	_tile_room.clear()
	if _style2:
		_rebuild_house_roofs(rooms)
		return
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
	# A roofed template site whose interior the sim does not count as a room (a hut's single tile is below the room
	# size) still gets its roof: one pseudo-room per site, id -1 - site index, over the interior tiles.
	for si: int in sites.size():
		var d: Dictionary = sites[si] if sites[si] is Dictionary else {}
		if not bool(d.get("roofed", false)) or str(d.get("status", "")) != "complete":
			continue
		var inner: Array = []
		for y: int in range(int(d.get("y", 0)) + 1, int(d.get("y", 0)) + int(d.get("height", 0)) - 1):
			for x: int in range(int(d.get("x", 0)) + 1, int(d.get("x", 0)) + int(d.get("width", 0)) - 1):
				inner.append(Vector2i(x, y))
		if inner.is_empty() or inner.any(func(t: Vector2i) -> bool: return _tile_room.has(t)):
			continue
		var pid: int = -1 - si
		tiles_of[pid] = inner
		roofed_ids.append(pid)
		for t: Vector2i in inner:
			_tile_room[t] = pid
		for c: Vector2i in footprint_cells(inner):
			claimed[c] = pid
	for id: int in roofed_ids:
		var tiles: Array = tiles_of[id]
		var bounds: Rect2i = Rect2i(tiles[0], Vector2i.ONE)
		for t: Vector2i in tiles:
			bounds = bounds.expand(t).expand(t + Vector2i.ONE)
		_room_rects[id] = bounds
		var cover: String = _roof_cover(tiles, sites)
		var mi: MeshInstance3D = MeshInstance3D.new()
		mi.name = "Roof_%d" % id
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


func _clear_windows() -> void:
	for panes: Array in _windows.values():
		for pane: MeshInstance3D in panes:
			pane.hide()
			pane.queue_free()
	_windows.clear()
	for frames: Array in _window_frames.values():
		for frame: MeshInstance3D in frames:
			frame.hide()
			frame.queue_free()
	_window_frames.clear()


## One pane per façade, mounted on a wall beside a roofed room tile.
func _add_windows(room: int, tiles: Array) -> void:
	var panes: Array[MeshInstance3D] = []
	var frames: Array[MeshInstance3D] = []
	var thickness: float = float(Data3D.param("structures.wall_thickness", 0.35))
	var window_id: String = str(Data3D.binding("s2kit.house_wall_trim", {}).get("window", ""))
	var trim: ArrayMesh = Kit.mesh(window_id) if _style2 and not _round_shell(room) else null
	for outward: Vector2i in DIRS:
		for inside: Vector2i in tiles:
			var wall: Vector2i = inside + outward
			var row: Dictionary = _walls.get(wall, {})
			if row.is_empty() or bool(row["door"]) or str(row["profile"]) == "palisade":
				continue
			var outside: Vector2i = wall + outward
			if _tile_room.has(outside) or _walls.has(outside):
				continue
			var pos: Vector3 = Vector3(wall.x + 0.5, float(_terrain.call("stand_height", wall)) + _window_height(room), wall.y + 0.5)
			pos += Vector3(outward.x, 0.0, outward.y) * (thickness * 0.5 + _window_offset)
			if _style2 and _round_shell(room):
				var bounds: Rect2 = Rect2(_room_rects[room])
				var radial: Vector2 = Vector2(outward)
				var offset: Vector2 = radial * (bounds.size * 0.51 - Vector2.ONE * 0.28)
				pos.x = bounds.get_center().x + offset.x
				pos.z = bounds.get_center().y + offset.y
			var pane: MeshInstance3D = MeshInstance3D.new()
			pane.name = "Window_%d" % room
			pane.mesh = _window_mesh
			pane.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
			pane.position = pos
			pane.rotation.y = atan2(float(outward.x), float(outward.y))
			pane.hide()
			add_child(pane)
			panes.append(pane)
			if trim != null:
				var out: Vector3 = Vector3(outward.x, 0.0, outward.y)
				var frame: MeshInstance3D = Kit.fitted_part(window_id, trim, Vector3(0.64, 0.45, 0.075),
					pane.rotation.y, pos - Vector3.UP * 0.225 + out * 0.03)
				frame.name = "WindowTrim_%d_%d" % [room, frames.size()]
				add_child(frame)
				frames.append(frame)
			break
	if not panes.is_empty():
		_windows[room] = panes
	if not frames.is_empty():
		_window_frames[room] = frames


func occupied_room_at(tile: Vector2i) -> bool:
	return _occupied_rooms.has(int(_tile_room.get(tile, -1)))


## Append the occupied rooms' nearest visible exterior pane to the shared light pool.
func append_window_lights(candidates: Array[Dictionary], eye: Vector3, level: float) -> void:
	_window_material.emission_energy_multiplier = _window_emission * level
	for room: int in _windows:
		var nearest: Vector3 = Vector3.ZERO
		var best: float = INF
		var lit: bool = level > 0.0 and (_occupied_rooms.has(room) or bool(_lit_houses.get(room, false)))
		for pane: MeshInstance3D in _windows[room]:
			var cut: float = _cut_all
			if _cut_rect.has_point(Vector2(pane.position.x, pane.position.z)):
				cut = minf(cut, _cut_rect_h)
			pane.visible = lit and cut >= _window_height(room) + _window_mesh.size.y * 0.5
			if _style2:
				pane.visible = lit and float(_roof_fade.get(room, 1.0)) > 0.85
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
	if _style2:
		_rebuild_house_sites(b, overlay)
	for i: int in mini(xs.size(), ys.size()):
		site_tiles[Vector2i(xs[i], ys[i])] = true
	for i: int in mini(xs.size(), mini(ys.size(), mini(ws.size(), hs.size()))):
		var rect: Rect2i = Rect2i(xs[i], ys[i], ws[i], hs[i])
		var g: float = float(_terrain.call("stand_height", rect.position))
		if _style2 and _house_sites.has([rect.position, int(kinds[i]) if i < kinds.size() else -1]):
			continue
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
		if _style2 and _houses.has(int(d.get("id", -1))):
			continue
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
		# Style 2: the panel's house section names the house (a hut opened from the inspector has no room).
		var house_id: int = int(room_panel.call("selected_house")) if _style2 and room_panel.has_method("selected_house") else -1
		if _roofs.has(house_id):
			out[house_id] = true
		var id: Variant = room_panel.get("_room")
		if id is int and int(id) >= 0:
			var house: int = int(_house_rooms.get(id, -1)) if _style2 else int(id)
			if house >= 0 and (not _style2 or _roofs.has(house)):
				out[house] = true
	if _style2:
		var build_panel: Node = get_node_or_null("/root/Main/UI/BuildPanel")
		var site: int = int(build_panel.call("selected_site")) if build_panel != null else -1
		if _roofs.has(site):
			return {site: true}
		if not out.is_empty():
			return out
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
	# Style 2 roofs stay closed at every zoom; only the room overlay still opens them all.
	if not _style2 and camera != null and float(camera.call("get_zoom_level")) >= float(Data3D.param("structures.cutaway_zoom", 4.0)) - 0.01:
		return true
	var rooms: Node = get_node_or_null("/root/Main/RoomIndex")
	return rooms != null and rooms.has_method("is_shown") and bool(rooms.call("is_shown"))


func _update_cutaway(delta: float) -> void:
	var seconds: float = float(Data3D.param("s2_house.cutaway_seconds", 0.30)) if _style2 else float(Data3D.param("structures.cutaway_seconds", 0.25))
	var speed: float = delta / maxf(0.01, seconds)
	var selected: Dictionary = _selected_rooms()
	var all_cut: bool = global_cut()
	# Style 2 opens a roof only for the selected house (walls drop there too), the house hovered for
	# `s2_house.hover_open_seconds`, and the houses near the view focus while the inside view is on.
	var opened: Dictionary = selected.duplicate()
	if _style2:
		all_cut = all_cut and selected.is_empty()
		_update_hover(delta)
		if _hover_house >= 0 and _hover_seconds >= float(Data3D.param("s2_house.hover_open_seconds", 0.3)):
			opened[_hover_house] = true
		if inside_view:
			for id: int in _houses_near_focus():
				opened[id] = true
	var cut_height: String = "s2_house.cutaway_wall_height" if _style2 else "structures.cutaway_wall_height"
	var low: float = float(Data3D.param(cut_height, 0.9))
	for id: int in _roofs:
		var target: float = 0.0 if all_cut or opened.has(id) else 1.0
		var previous: float = float(_roof_fade.get(id, 1.0))
		var f: float = move_toward(previous, target, speed)
		if _style2 and _roof_fade.has(id) and f == previous and (_roofs[id] as Node3D).visible == (f > 0.001):
			continue
		_roof_fade[id] = f
		var mi: MeshInstance3D = _roofs[id]
		mi.visible = f > 0.001
		if _style2:
			for vent: CPUParticles3D in _hearth_vents.get(id, []):
				vent.visible = f > 0.55
		mi.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_ON if f > 0.5 else GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
		if mi.mesh != null and mi.mesh.get_surface_count() > 0:
			(mi.get_active_material(0) as ShaderMaterial).set_shader_parameter("fade", f)
		if _style2:
			var lift: float = (1.0 - f) * float(Data3D.param("s2_house.cutaway_lift", 0.9))
			mi.position.y = float(mi.get_meta("roof_rest_y", 0.0)) + lift
			var trim: MeshInstance3D = _roof_details.get(id)
			if trim != null:
				trim.position.y = lift
				trim.visible = mi.visible
				if trim.mesh != null:
					(trim.get_active_material(0) as ShaderMaterial).set_shader_parameter("fade", f)
				for part: GeometryInstance3D in trim.get_children():
					(part.material_override as ShaderMaterial).set_shader_parameter("fade", f)
			for frame: MeshInstance3D in _window_frames.get(id, []):
				frame.visible = f > 0.85
	_cut_all = move_toward(_cut_all, low if all_cut else 99.0, speed * 98.0)
	var rect: Rect2 = Rect2()
	for id: int in selected:
		if _room_rects.has(id):
			var r: Rect2i = _room_rects[id]
			var margin: Vector2 = Vector2.ZERO if _style2 else Vector2(1.6, 1.6)
			var rr: Rect2 = Rect2(Vector2(r.position) - margin, Vector2(r.size) + margin * 2.0)
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


## The Style 2 house under the mouse (a roof or a room tile), held while the pointer stays on it; none over the UI.
func _update_hover(delta: float) -> void:
	var house: int = -1
	var viewport: Viewport = get_viewport()
	var world3d: Node = get_parent()
	if viewport != null and viewport.gui_get_hovered_control() == null and world3d != null and world3d.has_method("screen_to_world"):
		var w: Vector2 = world3d.call("screen_to_world", viewport.get_mouse_position())
		if w.x >= 0.0 and w.y >= 0.0:
			var px: float = float(Data3D.param("camera.px_per_tile_at_1x", 64.0))
			var key: int = roof_key_at(Vector2i(floori(w.x / px), floori(w.y / px)))
			house = key if _roofs.has(key) else -1
	_hover_seconds = _hover_seconds + delta if house == _hover_house and house >= 0 else 0.0
	_hover_house = house


## Style 2 roofs whose house centre lies within `s2_house.inside_radius` tiles of the camera's look target.
func _houses_near_focus() -> Array[int]:
	var out: Array[int] = []
	var rig: Node = get_parent().get("rig") if get_parent() != null else null
	if rig == null:
		return out
	var focus: Vector3 = rig.call("target")
	var radius: float = float(Data3D.param("s2_house.inside_radius", 12.0))
	for id: int in _roofs:
		if _room_rects.has(id) and Rect2(_room_rects[id]).get_center().distance_to(Vector2(focus.x, focus.z)) <= radius:
			out.append(id)
	return out


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
		if _style2:
			# A loaded save clears its plans before the next scheduled roof rebuild.
			if roof_key_at(Vector2i(floori(p.x), floori(p.z))) != id:
				continue
			if _round_shell(id):
				var radial: Vector2 = (Vector2(p.x, p.z) - Rect2(r).get_center()) / (Vector2(r.size) * 0.51)
				if radial.length_squared() > 1.0:
					continue
			best = t
		elif Rect2(Vector2(r.position) - Vector2(over, over), Vector2(r.size) + Vector2(over, over) * 2.0).has_point(Vector2(p.x, p.z)):
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
