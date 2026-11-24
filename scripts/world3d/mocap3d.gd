# S2 near-only clip selection and baked prop placement.
# Clips follow ACTION1: each villager's visual key, else its fallback key, maps through params3d
#   `mocap.clip_for_visual` to a clip of the near VAT (an unlisted key is tried as a clip name; else "idle"). The one
#   discrimination ACTION1 lacks is kept: an elder's fire seat (activity kind 6) plays sit_log instead of sit.
#   One-shot cues (pick_up, set_down) come from the cue ring, played once per (tick, sequence).
# Bridge methods called via SimReads: get_visible_action_snapshot, get_activity_snapshot; SimReads.visible_action_cues
#   (get_visible_action_cues).
extends RefCounted

const Data3D = preload("res://scripts/world3d/data3d.gd")
const Vat = preload("res://scripts/world3d/vat3d.gd")

var _meshes: Dictionary = {}
var _actors: Dictionary = {}
var _pool: Dictionary = {}
var _log_seats: Dictionary = {}
var _tree_output: int = -1
var _clip_for: Dictionary = {}
## AgentId → [visual key, fallback key] of this tick's row; of the latest new cue.
var _rows: Dictionary = {}
var _cues: Dictionary = {}
var _cue_tick: int = -1
var _cue_seen: Dictionary = {}


func setup(catalog: Dictionary) -> void:
	_tree_output = int((catalog.get("item_roles", {}) as Dictionary).get("tree_output", -1))
	_clip_for = Data3D.param("mocap.clip_for_visual", {})


## Once per tick: this tick's action rows (joined by AgentId), elder log seats and newly committed cues.
func refresh(world: Node, tick: int) -> void:
	_log_seats.clear()
	var activities: Dictionary = SimReads.read(world, "get_activity_snapshot")
	var bits: PackedInt64Array = activities.get("ids", PackedInt64Array())
	var kinds: PackedByteArray = activities.get("kinds", PackedByteArray())
	var until: PackedInt64Array = activities.get("until_tick", PackedInt64Array())
	for i: int in mini(bits.size(), mini(kinds.size(), until.size())):
		if until[i] > tick and kinds[i] == 6:
			_log_seats[bits[i]] = true
	_rows.clear()
	var rows: Dictionary = SimReads.read(world, "get_visible_action_snapshot")
	var aids: PackedInt64Array = rows.get("agent_ids", PackedInt64Array())
	var keys: PackedStringArray = rows.get("visual_keys", PackedStringArray())
	var falls: PackedStringArray = rows.get("fallback_keys", PackedStringArray())
	for i: int in mini(aids.size(), mini(keys.size(), falls.size())):
		_rows[aids[i]] = [keys[i], falls[i]]
	_refresh_cues(world, tick)


## PERF2-1 C: refresh() from the people view (`get_people_view`, no visible-action or activity read): the action
## rows and log seats of view rows `rows` (every row when `full`), departed AgentIds dropped; cues as refresh().
func refresh_view(world: Node, tick: int, view: Dictionary, rows: PackedInt32Array, full: bool) -> void:
	var aids: PackedInt64Array = view.get("agent_ids", PackedInt64Array())
	var bits: PackedInt64Array = view.get("ids", PackedInt64Array())
	var keys: PackedStringArray = view.get("visual_keys", PackedStringArray())
	var falls: PackedStringArray = view.get("fallback_keys", PackedStringArray())
	var seats: PackedByteArray = view.get("log_seats", PackedByteArray())
	var n: int = mini(mini(aids.size(), bits.size()), mini(mini(keys.size(), falls.size()), seats.size()))
	if full:
		_rows.clear()
		_log_seats.clear()
		rows = PackedInt32Array()
		rows.resize(n)
		for i: int in n:
			rows[i] = i
	for aid: int in view.get("removed", PackedInt64Array()):
		_rows.erase(aid)
	for i: int in rows:
		if i >= n:
			continue
		# A person without an ACTION1 row has no keys (get_visible_action_snapshot lists every live person).
		if keys[i].is_empty() and falls[i].is_empty():
			_rows.erase(aids[i])
		else:
			_rows[aids[i]] = [keys[i], falls[i]]
		if seats[i] == 1:
			_log_seats[bits[i]] = true
		else:
			_log_seats.erase(bits[i])
	_refresh_cues(world, tick)


## The cue actors of this tick's poll (refresh / refresh_view), not yet taken.
func cue_actors() -> Array:
	return _cues.keys()


func _refresh_cues(world: Node, tick: int) -> void:
	# Overlap the last tick (a command can commit a cue at the current tick) and drop the ones already played.
	var since: int = _cue_tick - 1 if _cue_tick >= 0 else tick - 1
	var cues: Dictionary = SimReads.visible_action_cues(world, since)
	var ticks: PackedInt64Array = cues.get("ticks", PackedInt64Array())
	var seqs: PackedInt64Array = cues.get("sequences", PackedInt64Array())
	var actors: PackedInt64Array = cues.get("agent_ids", PackedInt64Array())
	var cue_keys: PackedStringArray = cues.get("visual_keys", PackedStringArray())
	var cue_falls: PackedStringArray = cues.get("fallback_keys", PackedStringArray())
	var seen: Dictionary = {}
	_cues.clear()
	for i: int in mini(mini(ticks.size(), seqs.size()), mini(actors.size(), mini(cue_keys.size(), cue_falls.size()))):
		var key: Vector2i = Vector2i(ticks[i], seqs[i])
		seen[key] = true
		# The first poll after a load or reset only sets the cursor: retained history is not replayed.
		if _cue_tick >= 0 and not _cue_seen.has(key):
			_cues[actors[i]] = [cue_keys[i], cue_falls[i]]
	_cue_seen = seen
	_cue_tick = maxi(tick, int(cues.get("tick", tick)))


## The sustained near clip of AgentId `aid` (entity `bits`) on `vat`, or "" when ACTION1 has no row for it.
func action_clip(aid: int, bits: int, vat: RefCounted) -> String:
	var row: Array = _rows.get(aid, [])
	if row.is_empty():
		return ""
	var clip: String = _resolve(vat, row)
	if clip.is_empty():
		return "idle"
	if clip == "sit" and _log_seats.has(bits) and bool(vat.call("has_clip", "sit_log")):
		return "sit_log"
	return clip


## The one-shot clip of a cue newly committed for `aid` (consumed), or "".
func take_cue(aid: int, vat: RefCounted) -> String:
	var row: Array = _cues.get(aid, [])
	_cues.erase(aid)
	return _resolve(vat, row) if not row.is_empty() else ""


func _resolve(vat: RefCounted, keys: Array) -> String:
	for key: Variant in keys:
		var clip: String = str(_clip_for.get(key, key))
		if not clip.is_empty() and bool(vat.call("has_clip", clip)):
			return clip
	return ""


func wood_load(item: int, loose_wood: int) -> bool:
	return loose_wood > 0 or (item >= 0 and item == _tree_output)


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


func select(aid: int, vat: RefCounted, clip: String, phase: float, xf: Transform3D) -> void:
	if _actors.has(aid) and _actors[aid]["clip"] == clip and _actors[aid]["vat"] == vat:
		var row: Dictionary = _actors[aid]
		row["phase"] = phase
		row["xf"] = xf
		return
	release(aid)
	var grip: String = str(vat.call("prop_name", clip))
	var fixture: String = str(vat.call("fixture_name", clip))
	if grip.is_empty() and fixture.is_empty():
		return
	_actors[aid] = {"vat": vat, "clip": clip, "phase": phase, "xf": xf,
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
	_rows.clear()
	_cues.clear()
	_cue_seen.clear()
	_cue_tick = -1


func draw(host: Node3D, clock: float) -> void:
	for row: Dictionary in _actors.values():
		var xf: Transform3D = row["xf"]
		for key: String in ["grip", "fixture"]:
			var node: MeshInstance3D = row[key + "_node"]
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
