# animals3d.gd (World3D/Animals, made by world3d.gd): wild animals and livestock in 3D (M3D-1 scope 6 and 7).
#   PLACEHOLDER low-poly animals from tools/3d/blender/animals.py, one vertex-animated MultiMesh per species.
# Purpose: get_animals rows (wild; species = the row's canonical `names`) and get_livestock rows (domestic; species =
#   the catalogue animal id of `kinds`) → bindings3d `animals.by_id`, else `animals.by_category` by the row's
#   category / mechanic, else `animals.fallback`. Clips follow the 2D animal binding's pose for the state code
#   (`poses`) through bindings3d `animals.clip_for_pose`, walk/run while moving between tiles; fur colours per species
#   from bindings3d `animals.colours`. Motion glides over the 2D step time like people3d.gd. Picking: in no mode or the
#   hunt mode, the animal whose body is nearest the cursor yields its 2D tile cell centre (AnimalLayer.drawn_position),
#   so OrderTool's hunt and draft orders and clicks work unchanged.
# Bridge methods called: via SimReads get_animals, get_livestock, get_world_info, get_content_catalog; signal world_reset.
extends Node3D

const Data3D = preload("res://scripts/world3d/data3d.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
const Vat = preload("res://scripts/world3d/vat3d.gd")
const View = preload("res://scripts/world/colony/view.gd")
const OrderTool = preload("res://scripts/world/colony/order_tool.gd")

var _world_sim: Node = null
var _terrain: Node = null
var _vats: Dictionary = {}
var _crowds: Dictionary = {}
var _motion: Dictionary = {}
var _drawn: Array[Dictionary] = []
var _rows: Array[Dictionary] = []
var _rows_tick: int = -1


func setup(world_sim: Node, terrain: Node) -> void:
	_world_sim = world_sim
	_terrain = terrain
	_world_sim.connect("world_reset", func() -> void: _motion.clear())


func _vat(model: String) -> RefCounted:
	if not _vats.has(model):
		_vats[model] = Vat.load_character(model)
	return _vats[model]


func _species_model(name: String, category: String) -> String:
	var by_id: Dictionary = Data3D.binding("animals.by_id", {})
	if by_id.has(name):
		return str(by_id[name])
	var by_cat: Dictionary = Data3D.binding("animals.by_category", {})
	if by_cat.has(category):
		return str(by_cat[category])
	return str(Data3D.binding("animals.fallback", "deer"))


func _step_seconds() -> float:
	var info: Dictionary = SimReads.read(_world_sim, "get_world_info")
	var camera: Node = get_node_or_null("/root/Main/Camera2D")
	var tps: float = float(camera.call("ticks_per_second")) if camera != null and camera.has_method("ticks_per_second") else 12.0
	return maxf(0.02, float(info.get("move_interval_ticks", 3)) / maxf(1.0, tps))


func _process(delta: float) -> void:
	if _world_sim == null or _terrain == null or int(_terrain.get("width")) <= 0:
		return
	var stamp: Dictionary = get_parent().get("stamp") if get_parent() != null else {}
	if not _rows.is_empty() and not stamp.is_empty() and int(stamp.get("tick", -1)) == _rows_tick:
		_update(_rows, delta)
		return
	_rows_tick = int(stamp.get("tick", -1))
	var rows: Array[Dictionary] = []
	var wild: Dictionary = SimReads.read(_world_sim, "get_animals")
	var ids: PackedInt64Array = wild.get("ids", PackedInt64Array())
	var kinds: PackedInt64Array = wild.get("kinds", PackedInt64Array())
	var names: PackedStringArray = wild.get("names", PackedStringArray())
	var cats: PackedStringArray = wild.get("categories", PackedStringArray())
	var mechs: PackedStringArray = wild.get("mechanics", PackedStringArray())
	var xs: PackedInt32Array = wild.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = wild.get("ys", PackedInt32Array())
	var states: PackedInt32Array = wild.get("states", PackedInt32Array())
	for i: int in mini(ids.size(), mini(xs.size(), ys.size())):
		var cat: String = mechs[i] if i < mechs.size() else (cats[i] if i < cats.size() else "")
		rows.append({"key": "w%d" % ids[i], "id": int(ids[i]), "kind": int(kinds[i]) if i < kinds.size() else -1,
			"model": _species_model(names[i] if i < names.size() else "", cat), "tile": Vector2i(xs[i], ys[i]),
			"state": states[i] if i < states.size() else 0, "wild": true})
	var tame: Dictionary = SimReads.read(_world_sim, "get_livestock")
	var lids: PackedInt64Array = tame.get("ids", PackedInt64Array())
	var lkinds: PackedInt64Array = tame.get("kinds", PackedInt64Array())
	var lxs: PackedInt64Array = tame.get("xs", PackedInt64Array())
	var lys: PackedInt64Array = tame.get("ys", PackedInt64Array())
	var catalog: Dictionary = {}
	for row: Variant in (get_parent().call("catalog") as Dictionary).get("animals", []):
		if row is Dictionary:
			catalog[int((row as Dictionary).get("code", -1))] = row
	for i: int in mini(lids.size(), mini(lxs.size(), lys.size())):
		var crow: Dictionary = catalog.get(int(lkinds[i]) if i < lkinds.size() else -1, {})
		rows.append({"key": "l%d" % lids[i], "id": -1, "kind": int(lkinds[i]) if i < lkinds.size() else -1,
			"model": _species_model(str(crow.get("id", "")), str(crow.get("mechanic", crow.get("category", "")))),
			"tile": Vector2i(int(lxs[i]), int(lys[i])), "state": 2, "wild": false})
	_rows = rows
	_update(rows, delta)


func _update(rows: Array[Dictionary], delta: float) -> void:
	var running: bool = _world_sim.can_process()
	var dur: float = _step_seconds()
	var by_model: Dictionary = {}
	var seen: Dictionary = {}
	_drawn.clear()
	for r: Dictionary in rows:
		var key: String = r["key"]
		seen[key] = true
		var tile: Vector2i = r["tile"]
		var target: Vector2 = Vector2(tile) + Vector2(0.5, 0.5)
		var m: Dictionary = _motion.get(key, {})
		if m.is_empty():
			m = {"from": target, "to": target, "t": 1.0, "dur": dur, "yaw": float(hash(key) % 4) * PI * 0.5, "tile": tile}
			_motion[key] = m
		elif m["tile"] != tile:
			var now_pos: Vector2 = _pos(m)
			m["from"] = now_pos
			m["to"] = target
			m["t"] = 0.0
			m["dur"] = dur * maxf(1.0, (target - now_pos).length())
			m["tile"] = tile
			var d: Vector2 = target - now_pos
			if d.length_squared() > 0.0001:
				m["yaw"] = atan2(d.x, d.y)
		if running:
			m["t"] = minf(1.0, float(m["t"]) + delta / maxf(0.01, float(m["dur"])))
		var p: Vector2 = _pos(m)
		var model: String = r["model"]
		if not by_model.has(model):
			by_model[model] = []
		var y: float = float(_terrain.call("height_at", p.x, p.y))
		(by_model[model] as Array).append({"xf": Transform3D(Basis(Vector3.UP, float(m["yaw"])), Vector3(p.x, y, p.y)),
			"clip": _clip(r, float(m["t"]) < 1.0), "phase": float(hash(key) % 89) * 0.211})
		_drawn.append({"id": r["id"], "wild": r["wild"], "pos": Vector3(p.x, y, p.y), "h": 1.0})
	for key: String in _motion.keys():
		if not seen.has(key):
			_motion.erase(key)
	for model: String in _crowds.keys():
		if not by_model.has(model):
			(_crowds[model] as MultiMeshInstance3D).multimesh.instance_count = 0
	for model: String in by_model:
		var vat: RefCounted = _vat(model)
		if not bool(vat.get("ok")):
			continue
		var mmi: MultiMeshInstance3D = _crowds.get(model)
		if mmi == null:
			mmi = MultiMeshInstance3D.new()
			mmi.name = "Crowd_" + model
			var colours: Dictionary = (Data3D.binding("animals.colours", {}) as Dictionary).get(model, {})
			mmi.material_override = vat.call("material", Data3D.colour(colours.get("other", []), Color(0.2, 0.17, 0.14)))
			var mm0: MultiMesh = MultiMesh.new()
			mm0.transform_format = MultiMesh.TRANSFORM_3D
			mm0.use_colors = true
			mm0.use_custom_data = true
			mm0.mesh = vat.get("mesh")
			mmi.multimesh = mm0
			add_child(mmi)
			_crowds[model] = mmi
		var list: Array = by_model[model]
		var mm: MultiMesh = mmi.multimesh
		if mm.instance_count != list.size():
			mm.instance_count = list.size()
		var col: Dictionary = (Data3D.binding("animals.colours", {}) as Dictionary).get(model, {})
		var fur: Color = Data3D.colour(col.get("fur", []), Color(0.45, 0.36, 0.26))
		var light: Color = Data3D.colour(col.get("fur_light", []), Color(0.7, 0.62, 0.5))
		var horn: Color = Data3D.colour(col.get("horn", []), Color(0.85, 0.8, 0.7))
		for i: int in list.size():
			var e: Dictionary = list[i]
			mm.set_instance_transform(i, e["xf"])
			mm.set_instance_color(i, fur)
			mm.set_instance_custom_data(i, Color(float(vat.call("clip_pack", e["clip"])), float(e["phase"]), Vat.pack(light), Vat.pack(horn)))


func _clip(r: Dictionary, moving: bool) -> String:
	var pose: String = "stand"
	var b: Dictionary = ArtData.binding("animals", int(r["kind"])) if bool(r["wild"]) else {}
	var poses: Variant = b.get("poses", {})
	if poses is Dictionary:
		pose = str((poses as Dictionary).get(str(r["state"]), pose))
	var table: Dictionary = Data3D.binding("animals.clip_for_pose", {})
	var clip: String = str(table.get(pose, "idle"))
	if moving and clip not in ["walk", "run", "attack"]:
		clip = "walk"
	if not moving and clip in ["walk", "run"]:
		clip = "idle"
	return clip


static func _pos(m: Dictionary) -> Vector2:
	var t: float = float(m["t"])
	return (m["from"] as Vector2).lerp(m["to"], t * t * (3.0 - 2.0 * t))


## World3D picker: the 2D cell centre of the wild animal under `screen` (no mode or hunt), or (-1, -1).
func pick(screen: Vector2, camera: Camera3D, mode: int) -> Vector2:
	if mode != 0 and mode != OrderTool.Mode.HUNT:
		return Vector2(-1, -1)
	var radius: float = float(Data3D.param("people.pick_radius_px", 18))
	var best: int = -1
	var best_depth: float = INF
	for d: Dictionary in _drawn:
		if not bool(d["wild"]):
			continue
		var feet: Vector3 = d["pos"]
		var top: Vector3 = feet + Vector3(0, float(d["h"]), 0)
		if camera.is_position_behind(feet) or camera.is_position_behind(top):
			continue
		var a: Vector2 = camera.unproject_position(feet)
		var b: Vector2 = camera.unproject_position(top)
		if not _on_screen(a, b):
			continue
		if Geometry2D.get_closest_point_to_segment(screen, a, b).distance_to(screen) <= maxf(radius, (a - b).length() * 0.5):
			var depth: float = camera.global_position.distance_to(feet)
			if depth < best_depth:
				best_depth = depth
				best = int(d["id"])
	if best < 0:
		return Vector2(-1, -1)
	var layer: Node = get_node_or_null("/root/Main/ColonyView/AnimalLayer")
	if layer != null and layer.has_method("drawn_position"):
		var p: Vector2 = layer.call("drawn_position", best)
		if p.x >= 0.0:
			return p + Vector2(View.TILE, View.TILE) * 0.5
	return Vector2(-1, -1)


func count() -> int:
	return _drawn.size()


## A body is pickable only when both ends project inside the viewport and it spans a sane height on screen.
func _on_screen(a: Vector2, b: Vector2) -> bool:
	var view: Rect2 = get_viewport().get_visible_rect().grow(8.0)
	return view.has_point(a) and view.has_point(b) and a.distance_to(b) < view.size.y * 0.5
