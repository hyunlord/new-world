# plants3d.gd (World3D/Plants, made by world3d.gd): crops on fields and wild tile plants in 3D (M3D-SWITCH parity
#   for the 2D PlantLayer and FieldLayer).
# Purpose: the M3D-ASSETS plant and crop models, one MultiMesh per (model, params3d `flora.chunk_tiles` chunk) so the
#   frustum culls by chunk. A wild plant (get_plants over the map; kind →
#   get_content_catalog plant id) is its bindings3d `plants.wild.<id>` model for the plant phase (dormant, growing,
#   ripe, spent), one per tile with a hashed jitter and yaw. A field tile (FieldFacts rows of get_field_tiles; crop →
#   get_crop_defs id) is its `plants.crops.<id>` model for the stage the field phase maps to (`plants.stage_of_phase`:
#   growing 3, ripe 4, harvested 5 → growing, ripe, spent), a 2 × 2 grid of clumps per tile; untilled, tilled, sown
#   and fallow show only the terrain's tilled soil. In winter every model's colours are desaturated like the 2D view
#   (render_params `season.winter_saturation`). An id or phase without a model falls back to the procedural tufts: a
#   MultiMesh of one grass tuft (blades) and one of seed heads, wild tufts in the 2D plant phase colours (render_params
#   `plants.phase_colors`, ripe heads in `ripe_accent`), field tufts as a 3 × 3 grid (growing short and green, ripe tall
#   with heads in the crop category's colour, params3d `plants.crop_ripe_by_category`, else `plants.ripe`; harvested
#   straw stubble). Read when the tick passes a `flora.slow_read_ticks` boundary and rebuilt only when the rows change.
# Bridge methods called: get_plants; via SimReads/FieldFacts get_field_tiles, get_crop_defs, get_time_snapshot,
#   get_world_info, get_content_catalog (World3D.catalog); signal world_reset.
extends Node3D

const Data3D = preload("res://scripts/world3d/data3d.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
const Facts = preload("res://scripts/ui/field_facts.gd")

const BLADES: int = 7

const SliceVegetation = preload("res://scripts/world3d/slice1/vegetation.gd")

var _s2: bool
var _world_sim: Node = null
var _terrain: Node = null
var _tufts: MultiMeshInstance3D = null
var _heads: MultiMeshInstance3D = null
## "model|chunk x|chunk y" → its MultiMeshInstance3D.
var _models: Dictionary = {}
## "model|saturation" → the model with desaturated materials.
var _variants: Dictionary = {}
var _key: String = ""
var _rows_hash: int = 0
## Tile → [jitter x, jitter y, yaw] (seeded hashes, computed once per tile); "kind|phase" → wild model; batch key →
## hash of its rows (only changed batches are refilled).
var _tile_rand: Dictionary = {}
var _wild_cache: Dictionary = {}
var _batch_hash: Dictionary = {}


func setup(world_sim: Node, terrain: Node) -> void:
	_world_sim = world_sim
	_terrain = terrain
	_world_sim.connect("world_reset", func() -> void:
		_key = ""
		_rows_hash = 0
		_tile_rand.clear()
		_batch_hash.clear())


func _ready() -> void:
	_s2 = OS.get_cmdline_user_args().has("--style=s2")
	var mat: StandardMaterial3D = StandardMaterial3D.new()
	mat.vertex_color_use_as_albedo = true
	mat.vertex_color_is_srgb = true
	mat.cull_mode = BaseMaterial3D.CULL_DISABLED
	mat.roughness = 0.9
	_tufts = _batch("Tufts", _tuft_mesh(), mat)
	_heads = _batch("Heads", _head_mesh(), mat)


func _batch(node_name: String, mesh: ArrayMesh, mat: Material) -> MultiMeshInstance3D:
	mesh.surface_set_material(0, mat)
	var mmi: MultiMeshInstance3D = MultiMeshInstance3D.new()
	mmi.name = node_name
	mmi.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	var mm: MultiMesh = MultiMesh.new()
	mm.transform_format = MultiMesh.TRANSFORM_3D
	mm.use_colors = true
	mm.mesh = mesh
	mmi.multimesh = mm
	add_child(mmi)
	return mmi


## A tuft of blades 1 m tall around the origin, darker at the base (instance colour tints it).
static func _tuft_mesh() -> ArrayMesh:
	var verts: PackedVector3Array = PackedVector3Array()
	var cols: PackedColorArray = PackedColorArray()
	for k: int in BLADES:
		var a: float = TAU * float(k) / float(BLADES) + 0.4 * float(k % 2)
		var dir: Vector3 = Vector3(cos(a), 0.0, sin(a))
		var side: Vector3 = Vector3(-dir.z, 0.0, dir.x) * 0.035
		var root: Vector3 = dir * 0.04
		var tip: Vector3 = dir * (0.16 + 0.05 * float(k % 3)) + Vector3(0, 0.8 + 0.2 * float((k * 5) % 3) / 2.0, 0)
		verts.append_array([root - side, root + side, tip])
		cols.append_array([Color(0.55, 0.55, 0.55), Color(0.55, 0.55, 0.55), Color.WHITE])
	return _mesh(verts, cols)


## Seed heads near the tips of a 1 m tuft: small upright diamonds.
static func _head_mesh() -> ArrayMesh:
	var verts: PackedVector3Array = PackedVector3Array()
	var cols: PackedColorArray = PackedColorArray()
	for k: int in 4:
		var a: float = TAU * float(k) / 4.0 + 0.3
		var c: Vector3 = Vector3(cos(a) * 0.14, 0.92 + 0.04 * float(k % 2), sin(a) * 0.14)
		var w: Vector3 = Vector3(-sin(a), 0.0, cos(a)) * 0.045
		var up: Vector3 = Vector3(0, 0.11, 0)
		verts.append_array([c - up, c + w, c + up, c - up, c + up, c - w])
		for _i: int in 6:
			cols.append(Color.WHITE)
	return _mesh(verts, cols)


static func _mesh(verts: PackedVector3Array, cols: PackedColorArray) -> ArrayMesh:
	var arrays: Array = []
	arrays.resize(Mesh.ARRAY_MAX)
	arrays[Mesh.ARRAY_VERTEX] = verts
	arrays[Mesh.ARRAY_COLOR] = cols
	var normals: PackedVector3Array = PackedVector3Array()
	normals.resize(verts.size())
	normals.fill(Vector3.UP)
	arrays[Mesh.ARRAY_NORMAL] = normals
	var mesh: ArrayMesh = ArrayMesh.new()
	mesh.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, arrays)
	return mesh


func _process(_delta: float) -> void:
	if _world_sim == null or _terrain == null or not bool(_terrain.call("is_complete")):
		return
	var stamp: Dictionary = get_parent().get("stamp") if get_parent() != null else {}
	@warning_ignore("integer_division")
	var key: String = "%d" % (int(stamp.get("tick", -1)) / maxi(1, int(Data3D.param("flora.slow_read_ticks", 24))))
	if key == _key:
		return
	# The read and rebuild take World3D's heavy-work slot (one per frame); otherwise retry next frame.
	if get_parent() != null and get_parent().has_method("claim_heavy") and not bool(get_parent().call("claim_heavy")):
		return
	_key = key
	var width: int = int(_terrain.get("width"))
	var height: int = int(_terrain.get("height"))
	var plants: Dictionary = _world_sim.call("get_plants", 0, 0, width - 1, height - 1) if _world_sim.has_method("get_plants") else {}
	var fields: Array[Dictionary] = Facts.rows(_world_sim)
	var season: int = int(SimReads.read(_world_sim, "get_time_snapshot").get("season", 0))
	var rows_hash: int = hash([plants, fields, season, ArtData.params_version()])
	if rows_hash == _rows_hash:
		return
	_rows_hash = rows_hash
	_rebuild(plants, fields, season)


func _rebuild(plants: Dictionary, fields: Array[Dictionary], season: int) -> void:
	var style: Dictionary = ArtData.params().get("plants", {})
	var phase_cols: Array = style.get("phase_colors", ["777461", "718253", "9c9b56", "77704f"])
	var accent: Color = Color(str(style.get("ripe_accent", "c6a26a")))
	var winter: float = float((ArtData.params().get("season", {}) as Dictionary).get("winter_saturation", 0.35)) if season == 3 else 1.0
	var tufts: Array = []
	var heads: Array = []
	# "model|chunk x|chunk y" → [[position, yaw, scale], …].
	var placed: Dictionary = {}
	var seed: int = int(SimReads.read(_world_sim, "get_world_info").get("seed", 0))
	var plant_ids: Dictionary = {}
	for row: Variant in (get_parent().call("catalog") as Dictionary).get("plants", []):
		if row is Dictionary:
			plant_ids[int((row as Dictionary).get("code", -1))] = str((row as Dictionary).get("id", ""))
	# Wild plants: one model (else tuft) per tile.
	var xs: PackedInt64Array = plants.get("xs", PackedInt64Array())
	var ys: PackedInt64Array = plants.get("ys", PackedInt64Array())
	var kinds: PackedInt64Array = plants.get("kinds", PackedInt64Array())
	var phases: PackedInt64Array = plants.get("phases", PackedInt64Array())
	for i: int in mini(mini(xs.size(), ys.size()), phases.size()):
		var phase: int = clampi(phases[i], 0, 3)
		var t: Vector2i = Vector2i(xs[i], ys[i])
		var rnd: Array = _tile_rand.get(t, [])
		if rnd.is_empty():
			rnd = [float(hash([seed, t, 11]) % 1000) / 1000.0 - 0.5, float(hash([seed, t, 23]) % 1000) / 1000.0 - 0.5, float(hash([seed, t, 31]) % 360) * PI / 180.0]
			_tile_rand[t] = rnd
		var j: Vector2 = Vector2(float(rnd[0]), float(rnd[1]))
		var p: Vector2 = Vector2(t) + Vector2(0.5, 0.5) + j * 0.4
		if _s2:
			_put(placed, "slice_fern", [p, float(rnd[2]), [0.38, 0.58, 0.72, 0.45][phase]])
			continue
		var kind: int = int(kinds[i]) if i < kinds.size() else -1
		var wkey: int = kind * 8 + phase
		if not _wild_cache.has(wkey):
			_wild_cache[wkey] = wild_model(plant_ids.get(kind, ""), phase)
		var model: String = _wild_cache[wkey]
		if model != "":
			_put(placed, model, [p, float(rnd[2]), 0.9 + 0.2 * (j.x + 0.5)])
			continue
		var size: float = [0.32, 0.5, 0.55, 0.3][phase]
		var col: Color = _winter(Color(str(phase_cols[clampi(phase, 0, phase_cols.size() - 1)])), winter)
		tufts.append([p, size, col])
		if phase == 2:
			heads.append([p, size, _winter(accent, winter)])
	# Crops: a 3 × 3 grid per field tile by phase.
	var ripe_by_cat: Dictionary = Data3D.param("plants.crop_ripe_by_category", {})
	var categories: Dictionary = {}
	var crop_ids: Dictionary = {}
	for d: Variant in Facts.definitions(_world_sim):
		if d is Dictionary:
			categories[int((d as Dictionary).get("code", -1))] = str((d as Dictionary).get("category", ""))
			crop_ids[int((d as Dictionary).get("code", -1))] = str((d as Dictionary).get("id", ""))
	var green: Color = _winter(Data3D.colour(Data3D.param("plants.growing", []), Color(0.36, 0.5, 0.2)), winter)
	var straw: Color = Data3D.colour(Data3D.param("plants.straw", []), Color(0.72, 0.62, 0.38))
	for row: Dictionary in fields:
		var phase: int = int(row["phase"])
		if phase < 3 or phase > 5:
			continue
		var crop_model: String = crop_model(crop_ids.get(int(row["crop"]), ""), phase)
		if crop_model != "":
			# A crop model is one clump (~0.5 m): a 2 × 2 grid fills the tile.
			for c: int in 4:
				_put(placed, crop_model, [Vector2(float(row["x"]) + 0.25 + 0.5 * float(c % 2), float(row["y"]) + (0.75 if c >= 2 else 0.25)),
					PI * 0.5 * float(c % 2), 0.9 + 0.1 * float(c % 3)])
			continue
		var ripe: Color = _winter(Data3D.colour(ripe_by_cat.get(categories.get(int(row["crop"]), ""), Data3D.param("plants.ripe", [])), Color(0.78, 0.64, 0.3)), winter)
		var size: float = [0.38, 0.75, 0.13][phase - 3]
		var col: Color = [green, ripe.lerp(green, 0.35), straw][phase - 3]
		for gy: int in 3:
			for gx: int in 3:
				var p: Vector2 = Vector2(float(row["x"]) + 0.2 + 0.3 * gx, float(row["y"]) + 0.2 + 0.3 * gy)
				tufts.append([p, size * (0.9 + 0.2 * float((gx + gy * 2) % 3) / 2.0), col])
				if phase == 4:
					heads.append([p, size, ripe])
	for pair: Array in [["tufts", _tufts.multimesh, tufts], ["heads", _heads.multimesh, heads]]:
		var h: int = hash(pair[2])
		if int(_batch_hash.get(pair[0], 0)) != h:
			_batch_hash[pair[0]] = h
			_fill(pair[1], pair[2])
	_fill_models(placed, winter)


## The model of wild plant `id` in plant phase `phase` (bindings3d `plants.wild`), else "".
static func wild_model(id: String, phase: int) -> String:
	var models: Variant = (Data3D.binding("plants.wild", {}) as Dictionary).get(id)
	return Data3D.first_model([(models as Array)[phase]]) if models is Array and phase >= 0 and phase < (models as Array).size() else ""


## The model of crop `id` in field phase `phase` (bindings3d `plants.crops`, stage by `plants.stage_of_phase`), else "".
static func crop_model(id: String, phase: int) -> String:
	var stage: int = int((Data3D.binding("plants.stage_of_phase", {}) as Dictionary).get(str(phase), -1))
	var models: Variant = (Data3D.binding("plants.crops", {}) as Dictionary).get(id)
	return Data3D.first_model([(models as Array)[stage]]) if models is Array and stage >= 0 and stage < (models as Array).size() else ""


static func _put(placed: Dictionary, model: String, row: Array) -> void:
	var chunk: int = maxi(4, int(Data3D.param("flora.chunk_tiles", 16)))
	var p: Vector2 = row[0]
	var key: String = "%s|%d|%d" % [model, floori(p.x / chunk), floori(p.y / chunk)]
	if not placed.has(key):
		placed[key] = []
	(placed[key] as Array).append(row)


## One MultiMesh per (model, chunk) key of `placed` (rows [position, yaw, scale]); keys no longer placed are freed.
func _fill_models(placed: Dictionary, saturation: float) -> void:
	for id: String in _models.keys():
		if not placed.has(id):
			(_models[id] as Node).queue_free()
			_models.erase(id)
			_batch_hash.erase(id)
	for id: String in placed:
		var rows: Array = placed[id]
		# A plant changing phase moves it between batches: refill only batches whose rows changed.
		var h: int = hash([rows, saturation])
		if _models.has(id) and int(_batch_hash.get(id, 0)) == h:
			continue
		_batch_hash[id] = h
		var mm: MultiMesh = MultiMesh.new()
		mm.transform_format = MultiMesh.TRANSFORM_3D
		var model: String = id.get_slice("|", 0)
		mm.mesh = SliceVegetation.mesh(model) if model == "slice_fern" else _desaturated(model, saturation)
		mm.instance_count = rows.size()
		for i: int in rows.size():
			var p: Vector2 = rows[i][0]
			var s: float = rows[i][2]
			mm.set_instance_transform(i, Transform3D(Basis(Vector3.UP, float(rows[i][1])).scaled(Vector3(s, s, s)),
				Vector3(p.x, float(_terrain.call("height_at", p.x, p.y)), p.y)))
		var mmi: MultiMeshInstance3D = _models.get(id)
		if mmi == null:
			mmi = MultiMeshInstance3D.new()
			mmi.name = "Model_" + id.replace("|", "_")
			if model == "slice_fern":
				mmi.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
				mmi.visibility_range_end = float(Data3D.mood("style2.fern_distance", 41.0))
			add_child(mmi)
			_models[id] = mmi
		mmi.multimesh = mm


## Model `id` with every material's albedo saturation × `saturation` (cached; 1 = the model as authored).
func _desaturated(id: String, saturation: float) -> Mesh:
	var key: String = "%s|%.3f" % [id, saturation]
	if _variants.has(key):
		return _variants[key]
	var base: Mesh = Data3D.mesh(id)
	if saturation >= 1.0 or base is not ArrayMesh:
		_variants[key] = base
		return base
	var out: ArrayMesh = (base as ArrayMesh).duplicate() as ArrayMesh
	for s: int in out.get_surface_count():
		var mat: Material = out.surface_get_material(s)
		if mat is BaseMaterial3D:
			var m: BaseMaterial3D = (mat as BaseMaterial3D).duplicate() as BaseMaterial3D
			m.albedo_color = _winter(m.albedo_color, saturation)
			out.surface_set_material(s, m)
		elif mat is ShaderMaterial and (mat as ShaderMaterial).get_shader_parameter("base_colour") is Color:
			var m: ShaderMaterial = mat.duplicate() as ShaderMaterial
			m.set_shader_parameter("base_colour", _winter(m.get_shader_parameter("base_colour"), saturation))
			out.surface_set_material(s, m)
	_variants[key] = out
	return out


func _fill(mm: MultiMesh, rows: Array) -> void:
	mm.instance_count = rows.size()
	for i: int in rows.size():
		var p: Vector2 = rows[i][0]
		var s: float = rows[i][1]
		var yaw: float = float((i * 7919) % 360) * PI / 180.0
		var y: float = float(_terrain.call("height_at", p.x, p.y))
		mm.set_instance_transform(i, Transform3D(Basis(Vector3.UP, yaw).scaled(Vector3(s, s, s)), Vector3(p.x, y, p.y)))
		mm.set_instance_color(i, rows[i][2])


static func _winter(c: Color, saturation: float) -> Color:
	return c if saturation >= 1.0 else Color.from_hsv(c.h, c.s * saturation, c.v)
