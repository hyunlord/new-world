# plants3d.gd (World3D/Plants, made by world3d.gd): crops on fields and wild tile plants in 3D (M3D-SWITCH parity
#   for the 2D PlantLayer and FieldLayer).
# Purpose: two MultiMeshes of one procedural grass tuft (blades) and one of seed heads. Field tiles (FieldFacts rows of
#   get_field_tiles) carry a 3 × 3 grid of tufts by phase: growing (3) short and green, ripe (4) tall with heads in the
#   crop category's colour (params3d `plants.crop_ripe_by_category`, else `plants.ripe`), harvested (5) as straw
#   stubble; untilled, tilled, sown and fallow show only the terrain's tilled soil. Wild plants (get_plants over the
#   map) are one jittered tuft per tile in the 2D plant phase colours (render_params `plants.phase_colors`, ripe heads
#   in `ripe_accent`), desaturated in winter like the 2D view. Read when the tick passes a `flora.slow_read_ticks`
#   boundary and rebuilt only when the rows change.
# Bridge methods called: get_plants; via SimReads/FieldFacts get_field_tiles, get_crop_defs, get_time_snapshot;
#   signal world_reset.
extends Node3D

const Data3D = preload("res://scripts/world3d/data3d.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
const Facts = preload("res://scripts/ui/field_facts.gd")

const BLADES: int = 7

var _world_sim: Node = null
var _terrain: Node = null
var _tufts: MultiMeshInstance3D = null
var _heads: MultiMeshInstance3D = null
var _key: String = ""
var _rows_hash: int = 0


func setup(world_sim: Node, terrain: Node) -> void:
	_world_sim = world_sim
	_terrain = terrain
	_world_sim.connect("world_reset", func() -> void:
		_key = ""
		_rows_hash = 0)


func _ready() -> void:
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
	var seed: int = int(SimReads.read(_world_sim, "get_world_info").get("seed", 0))
	# Wild plants: one tuft per tile.
	var xs: PackedInt64Array = plants.get("xs", PackedInt64Array())
	var ys: PackedInt64Array = plants.get("ys", PackedInt64Array())
	var phases: PackedInt64Array = plants.get("phases", PackedInt64Array())
	for i: int in mini(mini(xs.size(), ys.size()), phases.size()):
		var phase: int = clampi(phases[i], 0, 3)
		var t: Vector2i = Vector2i(xs[i], ys[i])
		var j: Vector2 = Vector2(float(hash([seed, t, 11]) % 1000), float(hash([seed, t, 23]) % 1000)) / 1000.0 - Vector2(0.5, 0.5)
		var p: Vector2 = Vector2(t) + Vector2(0.5, 0.5) + j * 0.4
		var size: float = [0.32, 0.5, 0.55, 0.3][phase]
		var col: Color = _winter(Color(str(phase_cols[clampi(phase, 0, phase_cols.size() - 1)])), winter)
		tufts.append([p, size, col])
		if phase == 2:
			heads.append([p, size, _winter(accent, winter)])
	# Crops: a 3 × 3 grid per field tile by phase.
	var ripe_by_cat: Dictionary = Data3D.param("plants.crop_ripe_by_category", {})
	var categories: Dictionary = {}
	for d: Variant in Facts.definitions(_world_sim):
		if d is Dictionary:
			categories[int((d as Dictionary).get("code", -1))] = str((d as Dictionary).get("category", ""))
	var green: Color = _winter(Data3D.colour(Data3D.param("plants.growing", []), Color(0.36, 0.5, 0.2)), winter)
	var straw: Color = Data3D.colour(Data3D.param("plants.straw", []), Color(0.72, 0.62, 0.38))
	for row: Dictionary in fields:
		var phase: int = int(row["phase"])
		if phase < 3 or phase > 5:
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
	_fill(_tufts.multimesh, tufts)
	_fill(_heads.multimesh, heads)


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
