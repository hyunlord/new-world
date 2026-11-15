# flora3d.gd (World3D/Flora, made by world3d.gd): trees, stumps, bushes, deadwood, boulders and outcrops of the 3D
#   view (M3D-1 scope 4).
# Purpose: one MultiMeshInstance3D per (model, params3d `flora.chunk_tiles` chunk, 16×16) so the frustum culls by chunk
#   and a tree costs one instance (32×32 batches drew ~4× the visible trees: 1.0M primitives at Z3).
#   Trees come from get_trees (species = persisted wood material) → bindings3d `flora.species` (else the habit
#   fallback); variant, yaw, scale and position jitter are integer hashes of (world seed, tile) (View.key_hash), never
#   randf(). Canopy colour per (habit or species, season) from bindings3d `flora.canopy_by_season`; a habit listed
#   in `flora.bare_in_season` drops its canopy surfaces that season. Stumps: a tree that disappears from get_trees
#   leaves a stump for the 2D binding `trees.stump_ticks` (the same rule as tree_layer.gd). Bushes and deadwood from
#   get_resource_snapshot (bindings3d `resources.codes`), boulders/flint/clay from get_colony_overlay nodes (2D node
#   binding structure → bindings3d `nodes.by_structure`), ore outcrops from get_outcrops with the vein surface
#   tinted by the outcrop's catalogue tint material and a floating marker above each known deposit; a worked-out
#   outcrop (`remaining` 0) is smaller, vein in bindings3d `outcrops.depleted_tint`, no marker.
#   shaders/world3d/flora.gdshader adds a slow sway and winter snow on top surfaces.
# Bridge methods called: via SimReads get_trees, get_time_snapshot, get_world_info, get_resource_snapshot,
#   get_colony_overlay, get_outcrops, get_content_catalog; signal world_reset.
extends Node3D

const Data3D = preload("res://scripts/world3d/data3d.gd")
const View = preload("res://scripts/world/colony/view.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
const FLORA_SHADER = preload("res://shaders/world3d/flora.gdshader")

var _world_sim: Node = null
var _terrain: Node = null
var _seed: int = 0
var _keys: Dictionary = {}
var _stamp_key: String = ""
var _slow_tick: int = -1000000
var _season: int = -1
var _instances: Dictionary = {}
var _materials: Dictionary = {}
var _variants: Dictionary = {}
var _previous_trees: Dictionary = {}
var _stumps: Dictionary = {}
var _chunk: int = 32
var _markers: MultiMeshInstance3D = null


func setup(world_sim: Node, terrain: Node) -> void:
	_world_sim = world_sim
	_terrain = terrain
	_world_sim.connect("world_reset", _on_world_reset)


func _on_world_reset() -> void:
	_keys.clear()
	_previous_trees.clear()
	_stumps.clear()


func _process(_delta: float) -> void:
	if _world_sim == null or _terrain == null or int(_terrain.get("width")) <= 0:
		return
	var stamp: Dictionary = get_parent().get("stamp") if get_parent() != null else {}
	var skey: String = "%s|%s|%s|%d" % [stamp.get("trees", -1), stamp.get("overlay", -1), stamp.get("tick", -1), int(_terrain.get("width"))]
	if skey == _stamp_key and not stamp.is_empty():
		return
	_stamp_key = skey
	var time: Dictionary = SimReads.read(_world_sim, "get_time_snapshot")
	var season: int = int(time.get("season", 0))
	_seed = int(SimReads.read(_world_sim, "get_world_info").get("seed", 0))
	_chunk = maxi(4, int(Data3D.param("flora.chunk_tiles", 16)))
	_season = season
	var tick: int = int(time.get("tick", 0))
	var width: int = int(_terrain.get("width"))
	# Each family reads its snapshot only when its own counter moved (an FFI read of every tree each tick cost
	# milliseconds), and rebuilds only when its content changed.
	var tkey: String = "%d|%d|%d|%d" % [int(stamp.get("trees", -1)), season, _stump_epoch(time), width]
	if str(_keys.get("trees", "")) != tkey:
		_keys["trees"] = tkey
		var batches: Dictionary = {}
		_build_trees(batches, SimReads.read(_world_sim, "get_trees"), tick)
		_apply("trees", batches)
	var cadence: int = int(Data3D.param("flora.slow_read_ticks", 24))
	if tick - _slow_tick >= cadence or tick < _slow_tick or str(_keys.get("season", "")) != "%d|%d" % [season, width]:
		_slow_tick = tick
		_keys["season"] = "%d|%d" % [season, width]
		var resources: Dictionary = SimReads.read(_world_sim, "get_resource_snapshot")
		var amounts: PackedInt32Array = resources.get("amounts", PackedInt32Array())
		var flags: PackedByteArray = PackedByteArray()
		flags.resize(amounts.size())
		for i: int in amounts.size():
			flags[i] = 1 if amounts[i] > 0 else 0
		var rkey: String = "%d|%d|%d" % [hash([flags, resources.get("xs"), resources.get("kinds")]), season, width]
		if str(_keys.get("res", "")) != rkey:
			_keys["res"] = rkey
			var rb: Dictionary = {}
			_build_resources(rb, resources)
			_apply("res", rb)
		var outcrops: Dictionary = SimReads.read(_world_sim, "get_outcrops")
		var okey: String = "%d|%d|%d" % [int(outcrops.get("version", 0)), season, width]
		if str(_keys.get("ore", "")) != okey:
			_keys["ore"] = okey
			var ob: Dictionary = {}
			_build_ore(ob, outcrops)
			_apply("ore", ob)
	var nvkey: String = "%s|%d|%d" % [stamp.get("overlay", -1), season, width]
	if str(_keys.get("nodes_v", "")) != nvkey:
		_keys["nodes_v"] = nvkey
		var overlay: Dictionary = SimReads.read(_world_sim, "get_colony_overlay")
		var nkey: String = "%d|%d|%d" % [hash([overlay.get("node_xs"), overlay.get("node_ys"), overlay.get("node_kinds"), overlay.get("node_left")]), season, width]
		if str(_keys.get("nodes", "")) != nkey:
			_keys["nodes"] = nkey
			var nb: Dictionary = {}
			_build_nodes(nb, overlay)
			_apply("nodes", nb)


## Stumps expire by tick; re-key once an hour of game time so expiry is seen without a per-frame rebuild.
func _stump_epoch(time: Dictionary) -> int:
	return int(float(time.get("tick", 0)) / 60.0) if not _stumps.is_empty() else 0


func _build_trees(batches: Dictionary, trees: Dictionary, tick: int) -> void:
	var jitter: float = float(Data3D.param("flora.jitter_tiles", 0.22))
	var smin: float = float(Data3D.param("flora.scale_min", 0.85))
	var smax: float = float(Data3D.param("flora.scale_max", 1.2))
	var large_permille: int = int((ArtData.bindings().get("trees", {}) as Dictionary).get("large_permille", 600))
	var large_scale: float = float(Data3D.param("flora.large_scale", 1.18))
	# Trees.
	var xs: PackedInt32Array = trees.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = trees.get("ys", PackedInt32Array())
	var species: PackedStringArray = trees.get("species", PackedStringArray())
	var now: Dictionary = {}
	for i: int in mini(xs.size(), ys.size()):
		var t: Vector2i = Vector2i(xs[i], ys[i])
		var sp: String = species[i] if i < species.size() else ""
		now[t] = sp
		var models: Array = _species_models(sp)
		var h: int = View.key_hash(_seed, t.x, t.y, 17)
		var model: String = str(models[h % models.size()])
		var size: float = lerpf(smin, smax, float(View.key_hash(_seed, t.x, t.y, 23) % 1000) / 1000.0)
		if View.key_hash(_seed, t.x, t.y, 29) % 1000 >= large_permille:
			size *= large_scale
		_add(batches, model + "|" + _variant_key(sp), t, h, jitter, size)
	_update_stumps(now, tick)
	var stump_model: String = str(Data3D.binding("flora.stump", ""))
	for t: Vector2i in _stumps:
		_add(batches, stump_model + "|", t, View.key_hash(_seed, t.x, t.y, 31), jitter, 1.0)


func _build_resources(batches: Dictionary, resources: Dictionary) -> void:
	var jitter: float = float(Data3D.param("flora.jitter_tiles", 0.22))
	var codes: Dictionary = Data3D.binding("resources.codes", {})
	var rx: PackedInt32Array = resources.get("xs", PackedInt32Array())
	var ry: PackedInt32Array = resources.get("ys", PackedInt32Array())
	var rk: PackedInt32Array = resources.get("kinds", PackedInt32Array())
	var ra: PackedInt32Array = resources.get("amounts", PackedInt32Array())
	for i: int in mini(rk.size(), mini(rx.size(), ry.size())):
		var row: Variant = codes.get(str(rk[i]))
		if row is not Dictionary:
			continue
		var m: String = str((row as Dictionary).get("model", ""))
		if i < ra.size() and ra[i] <= 0 and (row as Dictionary).has("depleted"):
			m = str(row["depleted"])
		var t2: Vector2i = Vector2i(rx[i], ry[i])
		_add(batches, m + "|", t2, View.key_hash(_seed, t2.x, t2.y, 37), jitter * 0.5, 1.0)


func _build_nodes(batches: Dictionary, overlay: Dictionary) -> void:
	var jitter: float = float(Data3D.param("flora.jitter_tiles", 0.22))
	# Resource nodes (boulders, flint, clay) through the 2D node bindings.
	var by_structure: Dictionary = Data3D.binding("nodes.by_structure", {})
	var node_fallback: String = str(Data3D.binding("nodes.fallback", ""))
	var nx: PackedInt32Array = overlay.get("node_xs", PackedInt32Array())
	var ny: PackedInt32Array = overlay.get("node_ys", PackedInt32Array())
	var nk: PackedInt32Array = overlay.get("node_kinds", PackedInt32Array())
	var nl: PackedInt32Array = overlay.get("node_left", PackedInt32Array())
	for i: int in mini(nk.size(), mini(nx.size(), ny.size())):
		var b: Dictionary = ArtData.binding("nodes", nk[i])
		var structure: String = str(b.get("structure", ""))
		var yields: int = maxi(1, int(b.get("yields", 1)))
		if b.has("depleted") and i < nl.size() and nl[i] * 1000 < int(b.get("depleted_below_permille", 0)) * yields:
			structure = str(b["depleted"])
		var m2: String = str(by_structure.get(structure, node_fallback))
		var t3: Vector2i = Vector2i(nx[i], ny[i])
		_add(batches, m2 + "|", t3, View.key_hash(_seed, t3.x, t3.y, 41), jitter * 0.3, 1.0)


func _build_ore(batches: Dictionary, outcrops: Dictionary) -> void:
	# Ore outcrops: vein tint by the catalogue's art tint material, plus a marker.
	var ore_model: String = str(Data3D.binding("outcrops.model", ""))
	var ox: PackedInt64Array = outcrops.get("xs", PackedInt64Array())
	var oy: PackedInt64Array = outcrops.get("ys", PackedInt64Array())
	var ok: PackedInt64Array = outcrops.get("kinds", PackedInt64Array())
	var left: PackedInt64Array = outcrops.get("remaining", PackedInt64Array())
	var depleted_tint: String = str(Data3D.binding("outcrops.depleted_tint", ""))
	var marker_points: Array[Transform3D] = []
	var marker_colours: Array[Color] = []
	for i: int in mini(ok.size(), mini(ox.size(), oy.size())):
		var tint: String = _outcrop_tint(int(ok[i]))
		var t4: Vector2i = Vector2i(int(ox[i]), int(oy[i]))
		# A worked-out outcrop keeps its shape but loses the vein colour and the marker.
		if i < left.size() and left[i] <= 0:
			_add(batches, ore_model + "|vein=" + depleted_tint, t4, View.key_hash(_seed, t4.x, t4.y, 43), 0.0, 0.85)
			continue
		_add(batches, ore_model + "|vein=" + tint, t4, View.key_hash(_seed, t4.x, t4.y, 43), 0.0, 1.0)
		var top: float = float(_terrain.call("stand_height", t4)) + float(Data3D.binding("outcrops.marker_height", 1.9))
		marker_points.append(Transform3D(Basis.IDENTITY.scaled(Vector3(0.22, 0.32, 0.22)), Vector3(t4.x + 0.5, top, t4.y + 0.5)))
		marker_colours.append(Data3D.material_colour(tint, "Mineral"))
	_apply_markers(marker_points, marker_colours)


func _species_models(sp: String) -> Array:
	var row: Variant = (Data3D.binding("flora.species", {}) as Dictionary).get(sp)
	if row is Dictionary and (row as Dictionary).get("models") is Array and not ((row as Dictionary)["models"] as Array).is_empty():
		return row["models"]
	var habit: String = str(Data3D.binding("flora.fallback_habit", ""))
	var hrow: Variant = (Data3D.binding("flora.habits", {}) as Dictionary).get(habit, {})
	if hrow is Dictionary and (hrow as Dictionary).get("models") is Array:
		return hrow["models"]
	return [""]


func _habit(sp: String) -> String:
	var row: Variant = (Data3D.binding("flora.species", {}) as Dictionary).get(sp)
	if row is Dictionary:
		return str((row as Dictionary).get("habit", Data3D.binding("flora.fallback_habit", "")))
	return str(Data3D.binding("flora.fallback_habit", ""))


## The material variant of a species this season: "species=<sp>" (canopy/trunk colours are looked up from it).
func _variant_key(sp: String) -> String:
	return "species=" + sp


func _outcrop_tint(code: int) -> String:
	var catalog: Dictionary = get_parent().call("catalog")
	for row: Variant in catalog.get("outcrops", []):
		if row is Dictionary and int((row as Dictionary).get("code", -1)) == code:
			var art: Variant = (row as Dictionary).get("art", {})
			if art is Dictionary and (art as Dictionary).has("tint_material"):
				return str(art["tint_material"])
			return str((row as Dictionary).get("art_tint_material", (row as Dictionary).get("material", "")))
	return ""


func _update_stumps(now: Dictionary, tick: int) -> void:
	var stump_ticks: int = int((ArtData.bindings().get("trees", {}) as Dictionary).get("stump_ticks", 1440))
	if not _previous_trees.is_empty():
		for t: Vector2i in _previous_trees:
			if not now.has(t):
				_stumps[t] = tick
	for t: Vector2i in _stumps.keys():
		if now.has(t) or tick - int(_stumps[t]) > stump_ticks:
			_stumps.erase(t)
	_previous_trees = now


func _add(batches: Dictionary, key: String, t: Vector2i, h: int, jitter: float, size: float) -> void:
	@warning_ignore("integer_division")
	var bkey: String = "%s|%d|%d" % [key, t.x / _chunk, t.y / _chunk]
	if not batches.has(bkey):
		batches[bkey] = [[], []]
	var jx: float = (float(h % 1000) / 1000.0 - 0.5) * 2.0 * jitter
	@warning_ignore("integer_division")
	var jz: float = (float((h / 1000) % 1000) / 1000.0 - 0.5) * 2.0 * jitter
	var x: float = t.x + 0.5 + jx
	var z: float = t.y + 0.5 + jz
	var y: float = float(_terrain.call("height_at", x, z))
	@warning_ignore("integer_division")
	var yaw: float = float((h / 7) % 360) * PI / 180.0
	var basis: Basis = Basis(Vector3.UP, yaw).scaled(Vector3(size, size, size))
	(batches[bkey][0] as Array).append(Transform3D(basis, Vector3(x, y, z)))
	@warning_ignore("integer_division")
	var v: float = 0.9 + 0.2 * float((h / 13) % 100) / 100.0
	(batches[bkey][1] as Array).append(Color(v, v, v))


func _apply(family: String, batches: Dictionary) -> void:
	for k: String in _instances.keys():
		if k.begins_with(family + ":") and not batches.has(k.trim_prefix(family + ":")):
			(_instances[k] as Node).queue_free()
			_instances.erase(k)
	for bk: String in batches:
		var k: String = family + ":" + bk
		var parts: PackedStringArray = bk.split("|")
		var mesh: Mesh = _mesh_variant(parts[0], parts[1])
		var data: Array = batches[bk]
		var xforms: Array = data[0]
		var cols: Array = data[1]
		var mm: MultiMesh = MultiMesh.new()
		mm.transform_format = MultiMesh.TRANSFORM_3D
		mm.use_colors = true
		mm.mesh = mesh
		mm.instance_count = xforms.size()
		for i: int in xforms.size():
			mm.set_instance_transform(i, xforms[i])
			mm.set_instance_color(i, cols[i])
		var mmi: MultiMeshInstance3D = _instances.get(k)
		if mmi == null:
			mmi = MultiMeshInstance3D.new()
			add_child(mmi)
			_instances[k] = mmi
		mmi.multimesh = mm


func _apply_markers(points: Array[Transform3D], colours: Array[Color]) -> void:
	if _markers == null:
		_markers = MultiMeshInstance3D.new()
		_markers.name = "OutcropMarkers"
		_markers.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
		add_child(_markers)
	var mm: MultiMesh = MultiMesh.new()
	mm.transform_format = MultiMesh.TRANSFORM_3D
	mm.use_colors = true
	var gem: PrismMesh = PrismMesh.new()
	gem.size = Vector3(1, 1, 1)
	var mat: StandardMaterial3D = StandardMaterial3D.new()
	mat.vertex_color_use_as_albedo = true
	mat.emission_enabled = true
	mat.emission = Color(0.35, 0.35, 0.35)
	mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	gem.material = mat
	mm.mesh = gem
	mm.instance_count = points.size()
	for i: int in points.size():
		mm.set_instance_transform(i, points[i])
		mm.set_instance_color(i, colours[i])
	_markers.multimesh = mm


## Model `id` recoloured for variant `variant` ("species=<sp>", "vein=<material>" or "") as ONE surface: each source
## surface's colour is baked into the vertex colour and its sway / glow / vein flags into UV, so a tree is one draw per
## pass (two surfaces per tree doubled the draw calls, the measured bottleneck of a forested view).
func _mesh_variant(id: String, variant: String) -> Mesh:
	var vkey: String = "%s|%s|%d" % [id, variant, _season]
	if _variants.has(vkey):
		return _variants[vkey]
	var base: Mesh = Data3D.mesh(id)
	if base is not ArrayMesh:
		_variants[vkey] = base
		return base
	var src: ArrayMesh = base
	var sp: String = variant.trim_prefix("species=") if variant.begins_with("species=") else ""
	var habit: String = _habit(sp)
	var bare: bool = false
	# JSON numbers load as floats: compare as ints.
	for v: Variant in (Data3D.binding("flora.bare_in_season", {}) as Dictionary).get(habit, []):
		bare = bare or (sp != "" and int(v) == _season)
	var verts: PackedVector3Array = PackedVector3Array()
	var normals: PackedVector3Array = PackedVector3Array()
	var cols: PackedColorArray = PackedColorArray()
	var uvs: PackedVector2Array = PackedVector2Array()
	var index: PackedInt32Array = PackedInt32Array()
	for s: int in src.get_surface_count():
		var name: String = Data3D.surface_key(src, s)
		if bare and name == "canopy":
			continue
		var params: Array = _surface_params(name, src.surface_get_material(s), variant, sp, habit)
		var arrays: Array = src.surface_get_arrays(s)
		var sv: PackedVector3Array = arrays[Mesh.ARRAY_VERTEX]
		var sn: Variant = arrays[Mesh.ARRAY_NORMAL]
		var si: Variant = arrays[Mesh.ARRAY_INDEX]
		var offset: int = verts.size()
		verts.append_array(sv)
		if sn is PackedVector3Array and (sn as PackedVector3Array).size() == sv.size():
			normals.append_array(sn)
		else:
			for _i: int in sv.size():
				normals.append(Vector3.UP)
		var c: Color = params[0]
		var flags: Vector2 = Vector2(float(params[2]), float(params[1]))
		for _i: int in sv.size():
			cols.append(c)
			uvs.append(flags)
		if si is PackedInt32Array and not (si as PackedInt32Array).is_empty():
			for v: int in si:
				index.append(v + offset)
		else:
			for v: int in sv.size():
				index.append(v + offset)
	var out: ArrayMesh = ArrayMesh.new()
	if verts.is_empty():
		_variants[vkey] = out
		return out
	var merged: Array = []
	merged.resize(Mesh.ARRAY_MAX)
	merged[Mesh.ARRAY_VERTEX] = verts
	merged[Mesh.ARRAY_NORMAL] = normals
	merged[Mesh.ARRAY_COLOR] = cols
	merged[Mesh.ARRAY_TEX_UV] = uvs
	merged[Mesh.ARRAY_INDEX] = index
	out.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, merged)
	out.surface_set_material(0, _shared_material())
	_variants[vkey] = out
	return out


## [colour, sway, kind (0 plain, 1 glow, 2 vein)] of a source surface for this variant and season.
func _surface_params(name: String, original: Material, variant: String, sp: String, habit: String) -> Array:
	var c: Color = _original_colour(original)
	var sway: float = 0.0
	if sp != "" and name == "canopy":
		var by_species: Dictionary = Data3D.binding("flora.species_canopy_by_season", {})
		var table: Variant = by_species.get(sp, (Data3D.binding("flora.canopy_by_season", {}) as Dictionary).get(habit, []))
		if table is Array and (table as Array).size() > _season:
			c = Data3D.colour((table as Array)[_season], c)
		sway = 1.0
	elif sp != "" and name == "trunk":
		var trunks: Dictionary = Data3D.binding("flora.trunk", {})
		c = Data3D.colour(trunks.get(sp, trunks.get("default", [])), c)
		sway = 0.3
	elif variant.begins_with("vein=") and name == "vein":
		c = Data3D.material_colour(variant.trim_prefix("vein="), "Mineral")
	return [c, sway, 1.0 if name == "glow" else (2.0 if name == "vein" else 0.0)]


func _shared_material() -> Material:
	var mkey: String = "shared|%d" % _season
	if _materials.has(mkey):
		return _materials[mkey]
	var mat: ShaderMaterial = ShaderMaterial.new()
	mat.shader = FLORA_SHADER
	_materials[mkey] = mat
	return mat


static func _original_colour(m: Material) -> Color:
	if m is BaseMaterial3D:
		return (m as BaseMaterial3D).albedo_color
	return Color(0.6, 0.6, 0.6)
