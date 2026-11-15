# visitors3d.gd (World3D/Visitors, made by world3d.gd): the M5.5 visitors in 3D (M3D-SWITCH E1).
#   PLACEHOLDER figures: a capsule per person in bindings `visitors.lod_colour` (the 2D view's visitor marker colour),
#   `people.adult_height` tall, and a box for a caravan's pack.
# Purpose: every migrant wave and caravan of get_neighbours that is walking in, waiting or leaving (states up to
#   NeighbourText.LEAVING) stands on its tile like the 2D view's groups: a wave as up to six people in a fan west of the
#   tile, a caravan as two traders east of it with their pack between them. Re-read when the sim tick moves (World3D
#   stamp). Picking: `bodies(mode)` lists every figure (kind "caravan" / "wave", the group number as id) in no mode
#   for world3d.gd's pick, so a click opens the trade or migrant dialog (OrderTool).
# Bridge methods called: via SimReads get_neighbours; signal world_reset.
extends Node3D

const Data3D = preload("res://scripts/world3d/data3d.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
const NeighbourText = preload("res://scripts/ui/neighbour_text.gd")
## A wave's people: offsets from the tile centre (tiles; west of the tile).
const WAVE_FAN: Array[Vector2] = [Vector2(-0.62, -0.1), Vector2(-1.0, 0.03), Vector2(-1.38, -0.1), Vector2(-0.81, 0.28),
	Vector2(-1.19, 0.34), Vector2(-1.56, 0.22)]
## A caravan: two traders and their pack east of the tile.
const CARAVAN_FAN: Array[Vector2] = [Vector2(0.56, 0.0), Vector2(1.31, 0.06)]
const PACK_AT: Vector2 = Vector2(0.94, 0.1)

var _world_sim: Node = null
var _terrain: Node = null
var _figures: MultiMeshInstance3D = null
var _packs: MultiMeshInstance3D = null
var _tick: int = -1
## Every figure drawn: {kind, id, feet: Vector3, h, r}.
var _bodies: Array[Dictionary] = []


func setup(world_sim: Node, terrain: Node) -> void:
	_world_sim = world_sim
	_terrain = terrain
	_world_sim.connect("world_reset", func() -> void: _tick = -1)


func _ready() -> void:
	var lc: Variant = (ArtData.bindings().get("visitors", {}) as Dictionary).get("lod_colour", [196, 182, 150])
	var mat: StandardMaterial3D = StandardMaterial3D.new()
	mat.albedo_color = Color8(int(lc[0]), int(lc[1]), int(lc[2])) if lc is Array and (lc as Array).size() >= 3 else Color(0.77, 0.71, 0.59)
	var height: float = float(Data3D.param("people.adult_height", 1.7))
	var capsule: CapsuleMesh = CapsuleMesh.new()
	capsule.radius = 0.18
	capsule.height = height
	capsule.material = mat
	_figures = _batch("Figures", capsule)
	var box: BoxMesh = BoxMesh.new()
	box.size = Vector3(0.45, 0.35, 0.35)
	var pack_mat: StandardMaterial3D = mat.duplicate() as StandardMaterial3D
	pack_mat.albedo_color = mat.albedo_color.darkened(0.35)
	box.material = pack_mat
	_packs = _batch("Packs", box)


func _batch(batch_name: String, mesh: Mesh) -> MultiMeshInstance3D:
	var mmi: MultiMeshInstance3D = MultiMeshInstance3D.new()
	mmi.name = batch_name
	mmi.multimesh = MultiMesh.new()
	mmi.multimesh.transform_format = MultiMesh.TRANSFORM_3D
	mmi.multimesh.mesh = mesh
	add_child(mmi)
	return mmi


func _process(_delta: float) -> void:
	if _world_sim == null or _terrain == null or int(_terrain.get("width")) <= 0 or not _world_sim.has_method("get_neighbours"):
		return
	var stamp: Dictionary = get_parent().get("stamp") if get_parent() != null else {}
	var tick: int = int(stamp.get("tick", -1))
	if tick == _tick and tick >= 0:
		return
	_tick = tick
	var read: Dictionary = SimReads.read(_world_sim, "get_neighbours")
	var height: float = float(Data3D.param("people.adult_height", 1.7))
	var figures: Array[Vector3] = []
	var packs: Array[Vector3] = []
	_bodies.clear()
	for wave: Dictionary in NeighbourText.waves(read):
		if int(wave["state"]) <= NeighbourText.LEAVING:
			for k: int in mini(int(wave["size"]), WAVE_FAN.size()):
				figures.append(_place("wave", int(wave["id"]), wave["tile"], WAVE_FAN[k], height))
	for caravan: Dictionary in NeighbourText.caravans(read):
		if int(caravan["state"]) <= NeighbourText.LEAVING:
			for offset: Vector2 in CARAVAN_FAN:
				figures.append(_place("caravan", int(caravan["id"]), caravan["tile"], offset, height))
			packs.append(_ground(caravan["tile"], PACK_AT))
	_fill(_figures.multimesh, figures, height * 0.5)
	_fill(_packs.multimesh, packs, 0.18)


## A figure of group `id` at `offset` from its tile's centre: recorded for picking; returns its feet.
func _place(kind: String, id: int, tile: Vector2i, offset: Vector2, height: float) -> Vector3:
	var feet: Vector3 = _ground(tile, offset)
	_bodies.append({"kind": kind, "id": id, "feet": feet, "h": height, "r": 0.22})
	return feet


func _ground(tile: Vector2i, offset: Vector2) -> Vector3:
	var p: Vector2 = Vector2(tile) + Vector2(0.5, 0.5) + offset
	return Vector3(p.x, float(_terrain.call("height_at", p.x, p.y)), p.y)


static func _fill(mm: MultiMesh, at: Array[Vector3], lift: float) -> void:
	if mm.instance_count != at.size():
		mm.instance_count = at.size()
	for i: int in at.size():
		mm.set_instance_transform(i, Transform3D(Basis(), at[i] + Vector3(0, lift, 0)))


## World3D picker: every visitor figure's body ({kind "caravan" / "wave", id, feet, h, r}) with no mode.
func bodies(mode: int) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	if mode == 0:
		out.assign(_bodies)
	return out


## How many waves and caravans stand on the map (for tests and tools).
func counts() -> Dictionary:
	var seen: Dictionary = {}
	for b: Dictionary in _bodies:
		seen["%s%d" % [b["kind"], b["id"]]] = b["kind"]
	var waves: int = seen.values().count("wave")
	return {"waves": waves, "caravans": seen.size() - waves}
