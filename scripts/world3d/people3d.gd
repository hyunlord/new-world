# people3d.gd (World3D/People, made by world3d.gd): the colony's people in 3D (M3D-1 scope 6 and 7; M3D-SWITCH set B).
# Purpose: every row of get_agent_snapshot is one instance of a vertex-animated MultiMesh (vat3d.gd): no per-person
#   node. bindings3d `people.models` picks the character by life stage and sex (`child`, `female`, `male`; else
#   `people.model`). Crowds are binned by model and map chunk (`people.chunk_tiles`), so off-screen chunks are culled;
#   each bin draws the full character near the camera and its `<model>_lod` bake (same clips) beyond
#   `people.lod_distance` through Godot visibility ranges. A person who moves to another chunk or grows up changes
#   bin. A soft dark disc under every near person (`people.contact_shadow` radius/alpha), an outline pass (dark by day,
#   a light halo at night: `people.outline`) and enlarged far figures (`people.far_scale`) keep them findable.
#   A person glides from tile to tile over the step time (move_interval_ticks at the current speed, the 2D
#   AgentLayer rule), faces where they walk, and plays a clip by state: sleep (states 3 with target_kinds 3),
#   carry (carries an item, wood or food while moving), walk, work (a job while standing), idle. Children are
#   `people.child_scale`, infants smaller. Tints come from the 2D appearance data: skin and hair from the look's face
#   variant (assets/v8art/people/variant_transforms.json `target_*_rgb`, elders the 2D grey hair), clothing from
#   render_params `people.clothing_by_top` by top code, else the look's top colour; a top listed in bindings3d
#   `people.bare_torso_tops` shows skin on the torso. The person the inspector shows gets a ring at their feet; the
#   chronicle's highlighted people (Main/PeopleView) get a ring in PeopleView.HIGHLIGHT_RGBA.
#   Picking: `bodies(mode)` lists every person's feet and height (entity bits as id) for world3d.gd's pick (no mode
#   only); `body_of(bits)` is one person's (camp framing, lesson highlight).
# Bridge methods called: via SimReads get_agent_snapshot, get_world_info; signal world_reset.
extends Node3D

const Data3D = preload("res://scripts/world3d/data3d.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
const Vat = preload("res://scripts/world3d/vat3d.gd")
const BLOB_SHADER = preload("res://shaders/world3d/contact_blob.gdshader")
const PeopleView = preload("res://scripts/world/people_view.gd")

var _world_sim: Node = null
var _terrain: Node = null
## model id → {vat, mat, lod_vat, lod_mat} (lod_* null when the model has no `_lod` bake)
var _models: Dictionary = {}
## "model|cx|cy" → {model, chunk, mmi, lod: MultiMeshInstance3D or null, blob, slot: {AgentId: slot}, free}
var _crowds: Dictionary = {}
var _ring: MeshInstance3D = null
## The chronicle highlight rings (one instance per highlighted person present).
var _highlight: MultiMeshInstance3D = null
var _people_view: Node = null
## AgentId → {from: Vector2, to: Vector2, t: float, dur: float, yaw: float, tile: Vector2i}
var _motion: Dictionary = {}
var _transforms: Dictionary = {}
var _top_colours: Dictionary = {}
## Tools only (world3d_fps.gd `--replicate`): draw every person this many times, offset, to load the renderer past the
## bridge's stress cap. Never set in the game.
var replicate: int = 1
var _looks: Dictionary = {}
var _snap_tick: int = -1
## AgentId → {model, bin, look, scale, bits, flags, clip, dirty, feet, h}; the slot in its bin is stable while they
## stay in it.
var _state: Dictionary = {}
var _clip_packs: Dictionary = {}
var _blob_mesh: QuadMesh = null


func setup(world_sim: Node, terrain: Node) -> void:
	_world_sim = world_sim
	_terrain = terrain
	_world_sim.connect("world_reset", func() -> void:
		_motion.clear()
		_state.clear()
		for crowd: Dictionary in _crowds.values():
			for n: Variant in [crowd["mmi"], crowd["lod"], crowd["blob"]]:
				if n != null:
					(n as Node).queue_free()
		_crowds.clear()
		_looks.clear())


func _ready() -> void:
	_blob_mesh = QuadMesh.new()
	_blob_mesh.orientation = PlaneMesh.FACE_Y
	var radius: float = float(Data3D.param("people.contact_shadow.radius", 0.42))
	_blob_mesh.size = Vector2(radius, radius) * 2.0
	var blob_mat: ShaderMaterial = ShaderMaterial.new()
	blob_mat.shader = BLOB_SHADER
	blob_mat.set_shader_parameter("alpha", float(Data3D.param("people.contact_shadow.alpha", 0.45)))
	_blob_mesh.material = blob_mat
	for model: String in _model_ids():
		_model(model)
	var torus: TorusMesh = TorusMesh.new()
	torus.inner_radius = 0.36
	torus.outer_radius = 0.46
	torus.rings = 24
	var mat: StandardMaterial3D = StandardMaterial3D.new()
	var sel: Variant = ArtData.param("people.selection_rgba", [240, 226, 170, 255])
	mat.albedo_color = Color8(int(sel[0]), int(sel[1]), int(sel[2])) if sel is Array else Color(0.94, 0.89, 0.67)
	mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	torus.material = mat
	_ring = MeshInstance3D.new()
	_ring.mesh = torus
	_ring.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	_ring.visible = false
	add_child(_ring)
	var hl_mat: StandardMaterial3D = StandardMaterial3D.new()
	hl_mat.albedo_color = PeopleView.HIGHLIGHT_RGBA
	hl_mat.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	hl_mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	var hl_torus: TorusMesh = torus.duplicate() as TorusMesh
	hl_torus.material = hl_mat
	_highlight = MultiMeshInstance3D.new()
	_highlight.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	_highlight.multimesh = MultiMesh.new()
	_highlight.multimesh.transform_format = MultiMesh.TRANSFORM_3D
	_highlight.multimesh.mesh = hl_torus
	add_child(_highlight)
	_people_view = get_node_or_null("/root/Main/PeopleView")
	_load_appearance()


func _load_appearance() -> void:
	var parsed: Variant = JSON.parse_string(FileAccess.get_file_as_string("res://assets/v8art/people/variant_transforms.json"))
	if parsed is Dictionary:
		for t: Variant in (parsed as Dictionary).get("transforms", []):
			if t is Dictionary:
				_transforms["%s/%s" % [t.get("look", ""), t.get("variant", "")]] = t
	var tops: Variant = JSON.parse_string(FileAccess.get_file_as_string("res://assets/v8art/people/top_colours.json"))
	if tops is Dictionary:
		_top_colours = tops


func _step_seconds() -> float:
	var info: Dictionary = SimReads.read(_world_sim, "get_world_info")
	var camera: Node = get_node_or_null("/root/Main/Camera2D")
	var tps: float = float(camera.call("ticks_per_second")) if camera != null and camera.has_method("ticks_per_second") else 12.0
	return maxf(0.02, float(info.get("move_interval_ticks", 3)) / maxf(1.0, tps))


func _process(delta: float) -> void:
	if _world_sim == null or _terrain == null or int(_terrain.get("width")) <= 0 or _models.is_empty():
		return
	var stamp: Dictionary = get_parent().get("stamp") if get_parent() != null else {}
	if _state.is_empty() or stamp.is_empty() or int(stamp.get("tick", -1)) != _snap_tick:
		_snap_tick = int(stamp.get("tick", -1))
		_refresh(SimReads.read(_world_sim, "get_agent_snapshot"))
	var running: bool = _world_sim.process_mode != Node.PROCESS_MODE_DISABLED
	var copies: int = maxi(1, replicate)
	var height: float = float(Data3D.param("people.adult_height", 1.7))
	var selected: int = _selected_bits()
	_ring.visible = false
	# Only people who move, or whose clip or look changed, are written: a still crowd costs nothing per frame.
	for aid: int in _state:
		var st: Dictionary = _state[aid]
		var m: Dictionary = _motion[aid]
		var moving: bool = float(m["t"]) < 1.0
		if running and moving:
			m["t"] = minf(1.0, float(m["t"]) + delta / maxf(0.01, float(m["dur"])))
		var crowd: Dictionary = _crowds[st["bin"]]
		var mm: MultiMesh = crowd["mmi"].multimesh
		var lm: MultiMesh = crowd["lod"].multimesh if crowd["lod"] != null else null
		var bm: MultiMesh = crowd["blob"].multimesh
		var cap: int = _cap(mm, copies)
		var slot: int = int(crowd["slot"][aid])
		if moving or bool(st["dirty"]):
			var p: Vector2 = _pos(m)
			var y: float = float(_terrain.call("height_at", p.x, p.y))
			var scale: float = float(st["scale"])
			var basis: Basis = Basis(Vector3.UP, float(m["yaw"])).scaled(Vector3(scale, scale, scale))
			var feet: Vector3 = Vector3(p.x, y, p.y)
			st["feet"] = feet
			st["h"] = height * scale
			for c: int in copies:
				var off: Vector3 = Vector3(float((c * 7) % 5) - 2.0, 0.0, float((c * 3) % 5) - 2.0) * 0.6 if c > 0 else Vector3.ZERO
				mm.set_instance_transform(cap * c + slot, Transform3D(basis, feet + off))
				if lm != null:
					lm.set_instance_transform(cap * c + slot, Transform3D(basis, feet + off))
				bm.set_instance_transform(cap * c + slot, Transform3D(Basis().scaled(Vector3(scale, 1.0, scale)), feet + off + Vector3(0, 0.03, 0)))
		# Paused mid-step: stand (not walk on the spot).
		var clip: float = _clip_pack(st["model"], _clip_of(st, running and float(m["t"]) < 1.0))
		if clip != float(st["clip"]) or bool(st["dirty"]):
			st["clip"] = clip
			var look: Dictionary = st["look"]
			var custom: Color = Color(clip, float(aid % 97) * 0.173, float(look["hair_p"]), float(look["cloth_p"]))
			for c: int in copies:
				mm.set_instance_color(cap * c + slot, look["skin"])
				mm.set_instance_custom_data(cap * c + slot, custom)
				if lm != null:
					lm.set_instance_color(cap * c + slot, look["skin"])
					lm.set_instance_custom_data(cap * c + slot, custom)
		st["dirty"] = false
		if int(st["bits"]) == selected and selected != -1:
			_ring.visible = true
			_ring.position = (st["feet"] as Vector3) + Vector3(0, 0.04, 0)
	_update_highlights()


## One ring per highlighted person present, slightly wider than the selection ring.
func _update_highlights() -> void:
	var ids: PackedInt64Array = _people_view.call("highlight_ids") if _people_view != null else PackedInt64Array()
	var at: Array[Vector3] = []
	for aid: int in ids:
		if _state.has(aid):
			at.append(_state[aid]["feet"])
	var mm: MultiMesh = _highlight.multimesh
	if mm.instance_count != at.size():
		mm.instance_count = at.size()
	for i: int in at.size():
		mm.set_instance_transform(i, Transform3D(Basis().scaled(Vector3(1.25, 1.0, 1.25)), at[i] + Vector3(0, 0.05, 0)))


## Instances per copy of a crowd MultiMesh.
static func _cap(mm: MultiMesh, copies: int) -> int:
	if mm == null:
		return 0
	@warning_ignore("integer_division")
	return mm.instance_count / maxi(1, copies)


## Every model id bindings3d `people.models` / `people.model` names, in a stable order.
func _model_ids() -> Array[String]:
	var out: Array[String] = []
	var models: Dictionary = Data3D.binding("people.models", {})
	for key: String in ["male", "female", "child"]:
		var id: String = str(models.get(key, ""))
		if not id.is_empty() and not out.has(id):
			out.append(id)
	var fallback: String = str(Data3D.binding("people.model", ""))
	if not fallback.is_empty() and not out.has(fallback):
		out.append(fallback)
	return out


## The VATs and materials of `model` (and of its `<model>_lod` bake), loaded once; {} when the model is missing.
func _model(model: String) -> Dictionary:
	if _models.has(model):
		return _models[model]
	var vat: RefCounted = Vat.load_character(model, true)
	if not bool(vat.get("ok")):
		return {}
	var other: Color = Data3D.colour(Data3D.binding("people.other_colour", []), Color(0.3, 0.22, 0.15))
	var row: Dictionary = {"vat": vat, "mat": vat.call("with_findability", vat.call("material", other)), "lod_vat": null, "lod_mat": null}
	var lod: RefCounted = Vat.load_character(model + "_lod", true)
	if bool(lod.get("ok")):
		row["lod_vat"] = lod
		row["lod_mat"] = lod.call("with_findability", lod.call("material", other))
	_models[model] = row
	return row


## The bin of `model` in the chunk holding `tile`, made on first use.
func _bin(model: String, tile: Vector2i) -> String:
	var size: int = maxi(4, int(Data3D.param("people.chunk_tiles", 16)))
	var chunk: Vector2i = Vector2i(floori(float(tile.x) / size), floori(float(tile.y) / size))
	var key: String = "%s|%d|%d" % [model, chunk.x, chunk.y]
	if _crowds.has(key):
		return key
	var row: Dictionary = _models[model]
	var lod_at: float = float(Data3D.param("people.lod_distance", 55.0))
	var mmi: MultiMeshInstance3D = _bin_node("Crowd_" + key, row["mat"])
	var lod: MultiMeshInstance3D = null
	if row["lod_vat"] != null:
		mmi.visibility_range_end = lod_at
		lod = _bin_node("Lod_" + key, row["lod_mat"])
		lod.visibility_range_begin = lod_at
	var blob: MultiMeshInstance3D = _bin_node("Contact_" + key, null)
	blob.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	blob.visibility_range_end = lod_at
	_crowds[key] = {"model": model, "chunk": chunk, "mmi": mmi, "lod": lod, "blob": blob, "slot": {}, "free": []}
	return key


func _bin_node(node_name: String, mat: Material) -> MultiMeshInstance3D:
	var mmi: MultiMeshInstance3D = MultiMeshInstance3D.new()
	mmi.name = node_name.replace("|", "_")
	mmi.material_override = mat
	add_child(mmi)
	return mmi


## The model of row `i`: by life stage (1 child) and sex (1 female), falling back to `people.model`.
func _model_of(snap: Dictionary, i: int) -> String:
	var models: Dictionary = Data3D.binding("people.models", {})
	var key: String = "child" if _byte(snap, "life_stages", i) == 1 else ("female" if _byte(snap, "sexes", i) == 1 else "male")
	var id: String = str(models.get(key, ""))
	if id.is_empty() or _model(id).is_empty():
		id = str(Data3D.binding("people.model", ""))
	return id


## Grows bin `key` to fit `need` people (keeping no slots: everyone in it is rewritten). Every MultiMesh of a bin has
## the chunk's AABB, and hidden slots sit at zero scale inside it, so culling stays per chunk.
func _grow(key: String, need: int, copies: int) -> void:
	var crowd: Dictionary = _crowds[key]
	var mm: MultiMesh = crowd["mmi"].multimesh
	if mm != null and mm.instance_count >= need * copies and mm.instance_count % copies == 0:
		return
	var size: float = float(maxi(4, int(Data3D.param("people.chunk_tiles", 16))))
	var corner: Vector3 = Vector3(float(crowd["chunk"].x) * size, 0.0, float(crowd["chunk"].y) * size)
	var aabb: AABB = AABB(corner - Vector3(3.0, 8.0, 3.0), Vector3(size + 6.0, 24.0, size + 6.0))
	var count: int = maxi(8, need + (need >> 2)) * copies
	var row: Dictionary = _models[crowd["model"]]
	var hidden: Transform3D = Transform3D(Basis().scaled(Vector3.ZERO), corner)
	var parts: Array = [["mmi", row["vat"].get("mesh"), true], ["blob", _blob_mesh, false]]
	if row["lod_vat"] != null:
		parts.append(["lod", row["lod_vat"].get("mesh"), true])
	for part: Array in parts:
		var m: MultiMesh = MultiMesh.new()
		m.transform_format = MultiMesh.TRANSFORM_3D
		m.use_colors = part[2]
		m.use_custom_data = part[2]
		m.mesh = part[1]
		m.instance_count = count
		m.custom_aabb = aabb
		for k: int in count:
			m.set_instance_transform(k, hidden)
		(crowd[part[0]] as MultiMeshInstance3D).multimesh = m
	var cap: int = _cap(crowd["mmi"].multimesh, copies)
	var free: Array[int] = []
	for k: int in cap:
		free.append(cap - 1 - k)
	crowd["free"] = free
	crowd["slot"] = {}
	for aid: int in _state:
		if _state[aid].get("bin", "") == key:
			_state[aid]["dirty"] = true


func _release(aid: int, copies: int) -> void:
	var crowd: Dictionary = _crowds.get(_state[aid].get("bin", ""), {})
	if crowd.is_empty() or not (crowd["slot"] as Dictionary).has(aid):
		return
	var slot: int = int(crowd["slot"][aid])
	var cap: int = _cap(crowd["mmi"].multimesh, copies)
	var size: float = float(maxi(4, int(Data3D.param("people.chunk_tiles", 16))))
	var hidden: Transform3D = Transform3D(Basis().scaled(Vector3.ZERO), Vector3(float(crowd["chunk"].x) * size, 0.0, float(crowd["chunk"].y) * size))
	for c: int in copies:
		for n: Variant in [crowd["mmi"], crowd["lod"], crowd["blob"]]:
			if n != null:
				(n as MultiMeshInstance3D).multimesh.set_instance_transform(cap * c + slot, hidden)
	(crowd["free"] as Array).append(slot)
	(crowd["slot"] as Dictionary).erase(aid)


## A new snapshot (once per tick): motion targets, appearance and state flags per AgentId, stable instance slots.
func _refresh(snap: Dictionary) -> void:
	var ids: PackedInt64Array = snap.get("ids", PackedInt64Array())
	var aids: PackedInt64Array = snap.get("agent_ids", PackedInt64Array())
	var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
	var n: int = mini(aids.size(), mini(xs.size(), ys.size()))
	var dur: float = _step_seconds()
	var child_scale: float = float(Data3D.param("people.child_scale", 0.62))
	var copies: int = maxi(1, replicate)
	var models: PackedStringArray = PackedStringArray()
	models.resize(n)
	var bins: PackedStringArray = PackedStringArray()
	bins.resize(n)
	var need: Dictionary = {}
	for i: int in n:
		models[i] = _model_of(snap, i)
		bins[i] = _bin(models[i], Vector2i(xs[i], ys[i]))
		need[bins[i]] = int(need.get(bins[i], 0)) + 1
	for key: String in need:
		_grow(key, int(need[key]), copies)
	var seen: Dictionary = {}
	for i: int in n:
		var aid: int = aids[i]
		seen[aid] = true
		var model: String = models[i]
		if _state.has(aid) and _state[aid].get("bin", "") != bins[i]:
			_release(aid, copies)
			_state[aid]["dirty"] = true
		var crowd: Dictionary = _crowds[bins[i]]
		if not (crowd["slot"] as Dictionary).has(aid):
			crowd["slot"][aid] = (crowd["free"] as Array).pop_back()
			if _state.has(aid):
				_state[aid]["dirty"] = true
		var tile: Vector2i = Vector2i(xs[i], ys[i])
		var target: Vector2 = Vector2(tile) + Vector2(0.5, 0.5)
		var m: Dictionary = _motion.get(aid, {})
		if m.is_empty():
			m = {"from": target, "to": target, "t": 1.0, "dur": dur, "yaw": float(aid % 4) * PI * 0.5, "tile": tile}
			_motion[aid] = m
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
		var stage: int = _byte(snap, "life_stages", i)
		var akey: int = hash([aid, _int(snap, "look_ids", i), _byte(snap, "variants", i), _byte(snap, "tops", i), stage])
		var look: Dictionary = _looks.get(akey, {})
		if look.is_empty():
			look = _appearance(snap, i)
			look["hair_p"] = Vat.pack(look["hair"])
			look["cloth_p"] = Vat.pack(look["cloth"])
			_looks[akey] = look
		var st: Dictionary = _state.get(aid, {"clip": -1.0, "dirty": true, "feet": Vector3(target.x, 0.0, target.y), "h": 1.7})
		# A model made for children already has a child's height; the scale only shrinks the adult fallback.
		var own_child: bool = model == str((Data3D.binding("people.models", {}) as Dictionary).get("child", ""))
		var scale: float = (1.0 if own_child else child_scale) * (0.75 if _byte(snap, "infants", i) == 1 else 1.0) if stage == 1 else 1.0
		if st.get("look") != look or float(st.get("scale", -1.0)) != scale:
			st["dirty"] = true
		st["model"] = model
		st["bin"] = bins[i]
		st["look_id"] = _int(snap, "look_ids", i)
		st["variant"] = _byte(snap, "variants", i)
		st["look"] = look
		st["scale"] = scale
		st["bits"] = ids[i] if i < ids.size() else -1
		st["flags"] = _flags(snap, i)
		_state[aid] = st
	for aid: int in _state.keys():
		if not seen.has(aid):
			_release(aid, copies)
			_state.erase(aid)
			_motion.erase(aid)


static func _pos(m: Dictionary) -> Vector2:
	var t: float = float(m["t"])
	return (m["from"] as Vector2).lerp(m["to"], t * t * (3.0 - 2.0 * t))


func _clip_pack(model: String, clip: String) -> float:
	var key: String = model + "/" + clip
	if not _clip_packs.has(key):
		_clip_packs[key] = float(_models[model]["vat"].call("clip_pack", clip))
	return _clip_packs[key]


static func _int(snap: Dictionary, key: String, i: int) -> int:
	var col: Variant = snap.get(key)
	if col is PackedInt32Array and i < (col as PackedInt32Array).size():
		return (col as PackedInt32Array)[i]
	return 0


static func _byte(snap: Dictionary, key: String, i: int) -> int:
	var col: Variant = snap.get(key)
	if col is PackedByteArray and i < (col as PackedByteArray).size():
		return (col as PackedByteArray)[i]
	return 0


## [sleeping, carrying, working] of row `i`.
func _flags(snap: Dictionary, i: int) -> Array:
	var states: PackedByteArray = snap.get("states", PackedByteArray())
	var targets: PackedByteArray = snap.get("target_kinds", PackedByteArray())
	var carry: PackedInt64Array = snap.get("carry_kinds", PackedInt64Array())
	var woods: PackedInt32Array = snap.get("carried_woods", PackedInt32Array())
	var jobs: PackedByteArray = snap.get("jobs", PackedByteArray())
	return [i < states.size() and i < targets.size() and states[i] == 3 and targets[i] == 3,
		(i < carry.size() and carry[i] >= 0) or (i < woods.size() and woods[i] > 0), i < jobs.size() and jobs[i] != 0]


func _clip_of(st: Dictionary, moving: bool) -> String:
	var f: Array = st["flags"]
	if bool(f[0]):
		return "sleep"
	if moving:
		return "carry" if bool(f[1]) else "walk"
	return "work" if bool(f[2]) else "idle"


## Skin, hair and clothing colours of row `i` from the 2D appearance data.
func _appearance(snap: Dictionary, i: int) -> Dictionary:
	var looks: PackedInt32Array = snap.get("look_ids", PackedInt32Array())
	var variants: PackedByteArray = snap.get("variants", PackedByteArray())
	var tops: PackedByteArray = snap.get("tops", PackedByteArray())
	var look: Dictionary = ArtData.look(looks[i] if i < looks.size() else 0)
	var frames: String = str(look.get("frames", ""))
	var vlist: Array = (ArtData.manifest().get("people", {}) as Dictionary).get("variants", [])
	var v: int = variants[i] if i < variants.size() else 0
	var t: Dictionary = _transforms.get("%s/%s" % [frames, str(vlist[v]) if v < vlist.size() else ""], {})
	var skin: Color = _rgb(t.get("target_skin_rgb"), Color8(196, 150, 110))
	var hair: Color = _rgb(t.get("target_hair_rgb"), Color8(60, 45, 30))
	if _byte(snap, "life_stages", i) == 2:
		hair = _rgb((ArtData.bindings().get("people", {}) as Dictionary).get("grey_hair_rgb"), hair)
	var by_top: Dictionary = ArtData.param("people.clothing_by_top", {})
	var top_code: int = tops[i] if i < tops.size() else 0
	var cloth: Color = _rgb(by_top.get(str(top_code)), _rgb(_top_colours.get(frames), Color8(150, 100, 50)))
	if (Data3D.binding("people.bare_torso_tops", []) as Array).has(str(look.get("top", ""))):
		cloth = skin
	return {"skin": skin, "hair": hair, "cloth": cloth}


static func _rgb(v: Variant, fallback: Color) -> Color:
	if v is Array and (v as Array).size() >= 3:
		return Color8(int(v[0]), int(v[1]), int(v[2]))
	return fallback


func _selected_bits() -> int:
	var inspector: Control = get_node_or_null("/root/Main/UI/AgentInspectorPanel") as Control
	if inspector != null and inspector.is_visible_in_tree() and inspector.has_method("get_selected_entity_bits"):
		return int(inspector.call("get_selected_entity_bits"))
	return -1


## World3D picker: every person's body ({kind "person", id = entity bits, feet, h, r}); people answer with no mode only.
func bodies(mode: int) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	if mode != 0:
		return out
	for d: Dictionary in _state.values():
		out.append({"kind": "person", "id": int(d["bits"]), "feet": d["feet"], "h": float(d["h"]), "r": 0.22})
	return out


## The body ({feet, h}) of the person with entity bits `bits`, or {} when not drawn.
func body_of(bits: int) -> Dictionary:
	for d: Dictionary in _state.values():
		if int(d["bits"]) == bits:
			return {"feet": d["feet"], "h": float(d["h"])}
	return {}


## Tools and gate checks: what is drawn for entity `bits` — {tile (the tile the figure stands on or walks to),
## settled (not mid-step), feet, model, look_id, variant, clip, scale} — or {} when not drawn.
func state_of(bits: int) -> Dictionary:
	for aid: int in _state:
		var d: Dictionary = _state[aid]
		if int(d["bits"]) != bits:
			continue
		var m: Dictionary = _motion.get(aid, {})
		return {"tile": m.get("tile", Vector2i(-1, -1)), "settled": float(m.get("t", 1.0)) >= 1.0, "feet": d["feet"],
			"model": d["model"], "look_id": int(d.get("look_id", 0)), "variant": int(d.get("variant", 0)),
			"clip": _clip_of(d, _world_sim.process_mode != Node.PROCESS_MODE_DISABLED and float(m.get("t", 1.0)) < 1.0), "scale": float(d["scale"])}
	return {}


## Tools (gate step 39): every drawn figure instance of entity `bits` as [MultiMesh, index] pairs — the full and the
## LOD crowd of its bin, all replicate copies; contact shadows excluded. [] when not drawn.
func figure_instances(bits: int) -> Array:
	var out: Array = []
	for aid: int in _state:
		if int(_state[aid]["bits"]) != bits:
			continue
		var crowd: Dictionary = _crowds.get(_state[aid].get("bin", ""), {})
		if crowd.is_empty() or not (crowd["slot"] as Dictionary).has(aid):
			return out
		var copies: int = maxi(1, replicate)
		var cap: int = _cap(crowd["mmi"].multimesh, copies)
		for n: Variant in [crowd["mmi"], crowd["lod"]]:
			if n != null:
				for c: int in copies:
					out.append([(n as MultiMeshInstance3D).multimesh, cap * c + int(crowd["slot"][aid])])
	return out


## Tools: the 3D feet position of entity `bits`, or Vector3.INF.
func feet_of(bits: int) -> Vector3:
	for d: Dictionary in _state.values():
		if int(d["bits"]) == bits:
			return d["feet"]
	return Vector3.INF


func count() -> int:
	return _state.size()
