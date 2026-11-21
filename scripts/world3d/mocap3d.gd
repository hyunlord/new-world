# S2 near-only clip selection and baked prop placement.
# Bridge methods called via SimReads: get_activity_snapshot, get_workshops.
extends RefCounted

const Data3D = preload("res://scripts/world3d/data3d.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
const Vat = preload("res://scripts/world3d/vat3d.gd")
const JOB_CLIPS: Dictionary = {1: "chop", 4: "build", 9: "dig", 12: "dig",
	18: "harvest", 22: "harvest"}

var _meshes: Dictionary = {}
var _actors: Dictionary = {}
var _pool: Dictionary = {}
var _food_recipes: Dictionary = {}
var _cooking: Dictionary = {}
var _sitting: Dictionary = {}
var _talkers: Dictionary = {}
var _tree_output: int = -1


func setup(catalog: Dictionary) -> void:
	_tree_output = int((catalog.get("item_roles", {}) as Dictionary).get("tree_output", -1))
	var food: Dictionary = {}
	for row: Dictionary in catalog.get("items", []):
		if row.get("category") == "food":
			food[int(row["code"])] = true
	for row: Dictionary in catalog.get("recipes", []):
		if food.has(int(row["output_code"])):
			_food_recipes[int(row["code"])] = true


func refresh(world: Node, tick: int, snap: Dictionary) -> void:
	_sitting.clear()
	var activities: Dictionary = SimReads.read(world, "get_activity_snapshot")
	var bits: PackedInt64Array = activities.get("ids", PackedInt64Array())
	var kinds: PackedByteArray = activities.get("kinds", PackedByteArray())
	var until: PackedInt64Array = activities.get("until_tick", PackedInt64Array())
	for i: int in bits.size():
		if until[i] > tick:
			if kinds[i] == 3:
				_sitting[bits[i]] = "sit"
			elif kinds[i] == 6:
				_sitting[bits[i]] = "sit_log"

	var states: PackedByteArray = snap.get("states", PackedByteArray())
	var targets: PackedByteArray = snap.get("target_kinds", PackedByteArray())
	var jobs: PackedByteArray = snap.get("jobs", PackedByteArray())
	var drafted: PackedByteArray = snap.get("drafted", PackedByteArray())
	var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
	_talkers.clear()
	for i: int in states.size():
		if states[i] == 2 and targets[i] == 5 and (i >= drafted.size() or drafted[i] == 0):
			var tile: Vector2i = Vector2i(xs[i], ys[i])
			_talkers[tile] = int(_talkers.get(tile, 0)) + 1

	_cooking.clear()
	if not jobs.has(7):
		return
	var shops: Dictionary = SimReads.read(world, "get_workshops")
	var shop_x: PackedInt32Array = shops.get("xs", PackedInt32Array())
	var shop_y: PackedInt32Array = shops.get("ys", PackedInt32Array())
	var shop_kinds: PackedInt64Array = shops.get("kinds", PackedInt64Array())
	var bill_shops: PackedInt32Array = shops.get("bill_shops", PackedInt32Array())
	var recipes: PackedInt64Array = shops.get("bill_recipes", PackedInt64Array())
	var active: PackedInt32Array = shops.get("bill_active", PackedInt32Array())
	for i: int in bill_shops.size():
		var shop: int = bill_shops[i]
		if active[i] == 1 and _food_recipes.has(recipes[i]) and bool(ArtData.binding("workshops", shop_kinds[shop]).get("marks_hearth_firing", false)):
			_cooking[Vector2i(shop_x[shop], shop_y[shop])] = true


func performing(state: int, target: int, job: int, tile: Vector2i, bits: int, drafted: bool) -> String:
	if state == 4:
		return str(_sitting.get(bits, ""))
	if state == 2 and target == 5 and not drafted and int(_talkers.get(tile, 0)) > 1:
		# A mutually settled pair matches the legacy social state.
		return "talk"
	if state != 3:
		return ""
	if target == 4:
		return "build"
	if target == 6:
		return "harvest"
	if target == 9:
		if job == 7 and _cooking.has(tile):
			return "cook"
		return str(JOB_CLIPS.get(job, ""))
	return ""


func wood_load(item: int, loose_wood: int) -> bool:
	return loose_wood > 0 or (item >= 0 and item == _tree_output)


static func transfer(old_count: int, count: int, old_job: int, job: int) -> String:
	# The action branch has explicit cues. This branch observes the two stock transfer edges.
	if old_job not in [2, 3] and job not in [2, 3]:
		return ""
	if old_count <= 0 and count > 0:
		return "pick_up"
	if old_count > 0 and count <= 0:
		return "set_down"
	return ""


func fixture_offset(asset: String) -> Vector3:
	var mesh: Mesh = _mesh(asset)
	return mesh.get_aabb().get_center() if mesh != null else Vector3.ZERO


func _mesh(asset: String) -> Mesh:
	if _meshes.has(asset):
		return _meshes[asset]
	var path: String = Vat.DIR.path_join(asset + ".glb")
	var mesh: Mesh = null
	if ResourceLoader.exists(path):
		var root: Node = (load(path) as PackedScene).instantiate()
		mesh = Data3D.merge_meshes(root, asset)
		root.free()
	_meshes[asset] = mesh
	return mesh


func select(aid: int, vat: RefCounted, clip: String, phase: float, xf: Transform3D, bin: MultiMeshInstance3D) -> void:
	if _actors.has(aid) and _actors[aid]["clip"] == clip and _actors[aid]["vat"] == vat:
		var row: Dictionary = _actors[aid]
		row["phase"] = phase
		row["xf"] = xf
		row["bin"] = bin
		return
	release(aid)
	var grip: String = str(vat.call("prop_name", clip))
	var fixture: String = str(vat.call("fixture_name", clip))
	if grip.is_empty() and fixture.is_empty():
		return
	_actors[aid] = {"vat": vat, "clip": clip, "phase": phase, "xf": xf, "bin": bin,
		"grip": "grip_" + grip if not grip.is_empty() else "", "fixture": fixture,
		"grip_node": null, "fixture_node": null}


func release(aid: int) -> void:
	var row: Dictionary = _actors.get(aid, {})
	if row.is_empty():
		return
	for key: String in ["grip", "fixture"]:
		var node: MeshInstance3D = row[key + "_node"]
		if node != null:
			node.hide()
			var asset: String = row[key]
			if not _pool.has(asset):
				_pool[asset] = []
			(_pool[asset] as Array).append(node)
	_actors.erase(aid)


func reset() -> void:
	for aid: int in _actors.keys():
		release(aid)


func draw(host: Node3D, clock: float, camera: Camera3D) -> void:
	if camera == null:
		return
	var eye: Vector3 = camera.global_position
	for row: Dictionary in _actors.values():
		var bin: MultiMeshInstance3D = row["bin"]
		var centre: Vector3 = bin.to_global(bin.multimesh.custom_aabb.get_center())
		var xf: Transform3D = row["xf"]
		var near_at: float = bin.visibility_range_end
		var visible: bool = eye.distance_squared_to(centre) < near_at * near_at
		for key: String in ["grip", "fixture"]:
			var node: MeshInstance3D = row[key + "_node"]
			if not visible:
				if node != null:
					node.hide()
				continue
			var asset: String = row[key]
			if asset.is_empty():
				continue
			if node == null:
				node = _take(host, asset)
				row[key + "_node"] = node
			if node == null:
				continue
			node.transform = xf
			if key == "grip":
				var pose: Dictionary = row["vat"].call("prop_pose", row["clip"], clock + float(row["phase"]))
				node.visible = not pose.is_empty()
				if not pose.is_empty():
					node.transform = xf * (pose["transform"] as Transform3D)
			else:
				node.show()


func _take(host: Node3D, asset: String) -> MeshInstance3D:
	var free: Array = _pool.get(asset, [])
	if not free.is_empty():
		return free.pop_back() as MeshInstance3D
	var mesh: Mesh = _mesh(asset)
	if mesh == null:
		return null
	var node: MeshInstance3D = MeshInstance3D.new()
	node.name = "Mocap_" + asset
	node.mesh = mesh
	host.add_child(node, true)
	return node
