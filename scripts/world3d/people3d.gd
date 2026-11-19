# people3d.gd (World3D/People, made by world3d.gd): the colony's people in 3D (M3D-1 scope 6 and 7; M3D-SWITCH set B).
# Purpose: every row of get_agent_snapshot is one instance of a vertex-animated MultiMesh (vat3d.gd): no per-person
#   node. bindings3d `people.by_era[<era id>]` (else `people.models`) picks the character by life stage and sex
#   (`elder_female`, `child_male`, … then `child`/`elder`, then `female`/`male`; else `people.model`); the era is
#   get_tech's current era order resolved through the catalogue's era rows. Crowds are binned by model and map chunk (`people.chunk_tiles`), so off-screen chunks are culled;
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
# Bridge methods called: via SimReads get_agent_snapshot, get_world_info, get_tech; signal world_reset.
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
var _slice_rect: Rect2i
var _slice_model: String = ""
const STYLE2_FALLBACK: String = "s2_woman"
const STYLE2_MODELS: Array[String] = ["s2_p1", "s2_p2", "s2_p3", "s2_p4"]
const STYLE2_SCALES: Array[float] = [1.0, 0.95, 0.70, 0.97]
const STYLE2_MID_DISTANCE: float = 10.0
const STYLE2_LOD_DISTANCE: float = 18.0
var _style2: bool = OS.get_cmdline_user_args().has("--style=s2")
var _looks: Dictionary = {}
var _snap_tick: int = -1
## AgentId → {model, bin, look, scale, bits, flags, clip, dirty, feet, h}; the slot in its bin is stable while they
## stay in it.
var _state: Dictionary = {}
var _clip_packs: Dictionary = {}
var _blob_mesh: QuadMesh = null
## Entity bits → AgentId of everyone drawn (O(1) lookups for body_of / state_of / figure_instances, which panels and
## labels call per person per frame).
var _aid_of: Dictionary = {}
var _far_param: Array = []
var _lod_at: float = -1.0
## AgentIds to visit next frame (moving, dirty, or whose state flags changed); `_was_running` re-visits all on pause.
var _active: Dictionary = {}
var _was_running: bool = true
## Current era id (bindings3d `people.by_era` picks the age/sex/era variant).
var _era: String = ""
var _era_tick: int = -1000000


func setup(world_sim: Node, terrain: Node) -> void:
	_world_sim = world_sim
	_terrain = terrain
	_world_sim.connect("world_reset", func() -> void:
		_motion.clear()
		_state.clear()
		_aid_of.clear()
		_active.clear()
		for crowd: Dictionary in _crowds.values():
			for n: Variant in [crowd["mmi"], crowd["mid"], crowd["lod"], crowd["blob"]]:
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
		if absi(_snap_tick - _era_tick) >= int(Data3D.param("flora.slow_read_ticks", 24)):
			_era_tick = _snap_tick
			_refresh_era()
		_refresh(SimReads.read(_world_sim, "get_agent_snapshot"))
		_simplify_far(_state.size() * maxi(1, replicate) > int(Data3D.param("people.simple_above", 2000)))
	var running: bool = _world_sim.process_mode != Node.PROCESS_MODE_DISABLED
	var copies: int = maxi(1, replicate)
	var height: float = float(Data3D.param("people.adult_height", 1.7))
	var selected: int = _selected_bits()
	_ring.visible = false
	# Only people who move, or whose clip or look changed, are visited: a still crowd costs nothing per frame.
	if running != _was_running:
		_was_running = running
		for aid: int in _state:
			_active[aid] = true
	for aid: int in _active.keys():
		if not _state.has(aid):
			_active.erase(aid)
			continue
		var st: Dictionary = _state[aid]
		var m: Dictionary = _motion[aid]
		var moving: bool = float(m["t"]) < 1.0
		if running and moving:
			m["t"] = minf(1.0, float(m["t"]) + delta / maxf(0.01, float(m["dur"])))
		var crowd: Dictionary = _crowds[st["bin"]]
		var mm: MultiMesh = crowd["mmi"].multimesh
		var lm: MultiMesh = crowd["lod"].multimesh if crowd["lod"] != null else null
		var mid: MultiMesh = crowd["mid"].multimesh if crowd["mid"] != null else null
		var bm: MultiMesh = crowd["blob"].multimesh
		var cap: int = _cap(mm, copies)
		var slot: int = int(crowd["slot"][aid])
		if moving or bool(st["dirty"]):
			var p: Vector2 = _pos(m)
			var yaw: float = float(m["yaw"])
			# Someone standing on a station's tile is drawn just in front of it, facing it, so the station stays
			# visible (presentation only; the tile is unchanged).
			var props: Node = get_parent().get("props") if get_parent() != null else null
			if not moving and props != null and (props.get("station_tiles") as Dictionary).has(m["tile"]):
				p += Vector2(0.0, 0.6)
				yaw = PI
			elif not moving:
				var spread: Vector2 = m.get("spread", Vector2.ZERO)
				p += spread
				if _style2 and bool((st["flags"] as Array)[0]) and not spread.is_zero_approx():
					yaw = atan2(spread.x, spread.y)
			var y: float = float(_terrain.call("height_at", p.x, p.y))
			var scale: float = float(st["scale"])
			var basis: Basis = Basis(Vector3.UP, yaw).scaled(Vector3(scale, scale, scale))
			var feet: Vector3 = Vector3(p.x, y, p.y)
			st["feet"] = feet
			st["draw_yaw"] = yaw
			st["h"] = height * scale
			for c: int in copies:
				var off: Vector3 = Vector3(float((c * 7) % 5) - 2.0, 0.0, float((c * 3) % 5) - 2.0) * 0.6 if c > 0 else Vector3.ZERO
				mm.set_instance_transform(cap * c + slot, Transform3D(basis, feet + off))
				if mid != null:
					mid.set_instance_transform(cap * c + slot, Transform3D(basis, feet + off))
				if lm != null:
					lm.set_instance_transform(cap * c + slot, Transform3D(basis, feet + off))
				bm.set_instance_transform(cap * c + slot, Transform3D(Basis().scaled(Vector3(scale, 1.0, scale)), feet + off + Vector3(0, 0.03, 0)))
		# Paused mid-step: stand (not walk on the spot).
		var clip_name: String = _clip_of(st, running and float(m["t"]) < 1.0)
		var clip: float = _clip_pack(st["model"], clip_name)
		var painted: bool = bool(crowd["painted"])
		var gait: bool = painted and clip_name == "walk"
		if clip != float(st["clip"]) or bool(st["dirty"]) or gait:
			st["clip"] = clip
			var look: Dictionary = st["look"]
			var phase: float = float(aid % 97) * 0.173
			if gait:
				var distance: float = float(m.get("gait_distance", 0.0)) + (m["from"] as Vector2).distance_to(_pos(m))
				phase = distance / float(st["scale"]) - float(get_parent().get("anim_time"))
			var custom: Color = Color(clip, phase, float(look["hair_p"]), float(look["cloth_p"]))
			var skin: Color = look["skin"]
			if painted:
				skin.a = fposmod(float(aid) * 0.618034, 1.0)
			for c: int in copies:
				mm.set_instance_color(cap * c + slot, skin)
				mm.set_instance_custom_data(cap * c + slot, custom)
				if mid != null:
					mid.set_instance_color(cap * c + slot, skin)
					mid.set_instance_custom_data(cap * c + slot, custom)
				if lm != null:
					lm.set_instance_color(cap * c + slot, skin)
					lm.set_instance_custom_data(cap * c + slot, custom)
		st["dirty"] = false
		# Done once it stands still, or the clock is stopped (a paused mid-step figure is drawn as it is).
		if float(m["t"]) >= 1.0 or not running:
			_active.erase(aid)
	var chosen: Dictionary = _state.get(_aid_of.get(selected, -1), {}) if selected != -1 else {}
	if not chosen.is_empty():
		_ring.visible = true
		_ring.position = (chosen["feet"] as Vector3) + Vector3(0, 0.04, 0)
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


## The VATs and materials of `model`, loaded once; {} when the model is missing.
func _model(model: String) -> Dictionary:
	if _models.has(model):
		return _models[model]
	var painted: bool = model == STYLE2_FALLBACK or STYLE2_MODELS.has(model)
	var vat: RefCounted = Vat.load_character(model, not painted)
	if not bool(vat.get("ok")):
		return {}
	var other: Color = Data3D.colour(Data3D.binding("people.other_colour", []), Color(0.3, 0.22, 0.15))
	var kit: bool = STYLE2_MODELS.has(model)
	var row: Dictionary = {"vat": vat, "mat": vat.call("with_findability", vat.call("material", other, painted), painted, kit), "lod_vat": null, "lod_mat": null, "mid_vat": null, "mid_mat": null, "painted": painted}
	if kit:
		var mid: RefCounted = Vat.load_character(model + "_mid", false)
		if bool(mid.get("ok")):
			row["mid_vat"] = mid
			row["mid_mat"] = mid.call("with_findability", mid.call("material", other, true), true, kit)
	var lod: RefCounted = Vat.load_character(model + "_lod", not painted)
	if bool(lod.get("ok")):
		row["lod_vat"] = lod
		row["lod_mat"] = lod.call("with_findability", lod.call("material", other, painted), painted, kit)
		if painted:
			var pass_material: ShaderMaterial = row["lod_mat"]
			while pass_material != null:
				pass_material.set_shader_parameter("vat_step_frames", true)
				pass_material = pass_material.next_pass as ShaderMaterial
	_models[model] = row
	return row


## Very large crowds: far (LOD) figures drop their outline and x-ray passes (they cost most of the frame at 10 000
## people); near figures keep them.
func _simplify_far(on: bool) -> void:
	# Large crowds also switch to LOD closer (`people.simple_lod_distance`).
	var lod_at: float = float(Data3D.param("people.simple_lod_distance" if on else "people.lod_distance", 55.0))
	if _style2:
		lod_at = minf(lod_at, STYLE2_LOD_DISTANCE)
	if lod_at != _lod_at:
		_lod_at = lod_at
		for crowd: Dictionary in _crowds.values():
			if crowd["lod"] != null:
				if crowd["mid"] != null:
					var near_at: float = minf(STYLE2_MID_DISTANCE, lod_at)
					crowd["mmi"].visibility_range_end = near_at
					crowd["mid"].visibility_range_begin = near_at
					crowd["mid"].visibility_range_end = lod_at
					crowd["mid"].visible = near_at < lod_at
				else:
					crowd["mmi"].visibility_range_end = lod_at
				crowd["lod"].visibility_range_begin = lod_at
			crowd["blob"].visibility_range_end = lod_at
	for row: Dictionary in _models.values():
		var mat: ShaderMaterial = row.get("lod_mat")
		if mat == null:
			continue
		if on and mat.next_pass != null:
			row["lod_pass"] = mat.next_pass
			mat.next_pass = null
		elif not on and mat.next_pass == null and row.has("lod_pass"):
			mat.next_pass = row["lod_pass"]


## The bin of `model` in the chunk holding `tile`, made on first use.
func _bin(model: String, tile: Vector2i) -> String:
	var size: int = maxi(4, int(Data3D.param("people.chunk_tiles", 16)))
	var chunk: Vector2i = Vector2i(floori(float(tile.x) / size), floori(float(tile.y) / size))
	var key: String = "%s|%d|%d" % [model, chunk.x, chunk.y]
	if _crowds.has(key):
		return key
	var row: Dictionary = _models[model]
	var lod_at: float = _lod_at if _lod_at > 0.0 else float(Data3D.param("people.lod_distance", 55.0))
	if _style2:
		lod_at = minf(lod_at, STYLE2_LOD_DISTANCE)
	var mmi: MultiMeshInstance3D = _bin_node("Crowd_" + key, row["mat"])
	var lod: MultiMeshInstance3D = null
	var mid: MultiMeshInstance3D = null
	if row["lod_vat"] != null:
		mmi.visibility_range_end = lod_at
		lod = _bin_node("Lod_" + key, row["lod_mat"])
		lod.visibility_range_begin = lod_at
		if row["mid_vat"] != null:
			var near_at: float = minf(STYLE2_MID_DISTANCE, lod_at)
			mmi.visibility_range_end = near_at
			mid = _bin_node("Mid_" + key, row["mid_mat"])
			mid.visibility_range_begin = near_at
			mid.visibility_range_end = lod_at
			mid.visible = near_at < lod_at
	var blob: MultiMeshInstance3D = _bin_node("Contact_" + key, null)
	blob.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	blob.visibility_range_end = lod_at
	_crowds[key] = {"model": model, "chunk": chunk, "mmi": mmi, "mid": mid, "lod": lod, "blob": blob, "painted": row["painted"], "slot": {}, "free": []}
	return key


func _bin_node(node_name: String, mat: Material) -> MultiMeshInstance3D:
	var mmi: MultiMeshInstance3D = MultiMeshInstance3D.new()
	mmi.name = node_name.replace("|", "_")
	mmi.material_override = mat
	add_child(mmi)
	return mmi


## Debug slice presentation only. An empty rectangle keeps the complete crowd.
func configure_slice(rect: Rect2i, model: String) -> bool:
	if model != "" and _model(model).is_empty():
		return false
	_slice_rect = rect
	_slice_model = model
	_snap_tick = -1
	return true


## The model of row `i`: by life stage (1 child) and sex (1 female), falling back to `people.model`.
func _model_of(snap: Dictionary, i: int) -> String:
	if _slice_model != "":
		return _slice_model
	if _style2:
		var id: String = style2_model(snap, i)
		if not _model(id).is_empty():
			return id
		if not _model(STYLE2_FALLBACK).is_empty():
			return STYLE2_FALLBACK
	var stage: int = _byte(snap, "life_stages", i)
	var sex: String = "female" if _byte(snap, "sexes", i) == 1 else "male"
	var age: String = "child" if stage == 1 else ("elder" if stage == 2 else "")
	var table: Dictionary = (Data3D.binding("people.by_era", {}) as Dictionary).get(_era, Data3D.binding("people.models", {}))
	# Most specific first: "elder_female", then "elder", then "female"; else the default table, else people.model.
	for t: Dictionary in [table, Data3D.binding("people.models", {})]:
		for key: String in ([age + "_" + sex, age] if not age.is_empty() else []) + [sex]:
			var id: String = str(t.get(key, ""))
			if not id.is_empty() and not _model(id).is_empty():
				return id
	return str(Data3D.binding("people.model", ""))


## Same row order as the people transforms. Life stage is only a fallback for rows without a recorded age.
static func style2_model(snap: Dictionary, i: int) -> String:
	var ages: PackedInt32Array = snap.get("ages_years", PackedInt32Array())
	var age: int = ages[i] if i < ages.size() else -1
	var stage: int = _byte(snap, "life_stages", i)
	if (age >= 0 and age < 13) or (age < 0 and stage == 1):
		return STYLE2_MODELS[2]
	if age >= 55 or (age < 0 and stage == 2):
		return STYLE2_MODELS[3]
	return STYLE2_MODELS[1 if _byte(snap, "sexes", i) == 1 else 0]


## The colony's current era id (get_tech current_era_order → the catalogue era row), read on the slow cadence.
func _refresh_era() -> void:
	var tech: Dictionary = SimReads.read(_world_sim, "get_tech")
	var order: int = int(tech.get("current_era_order", 0))
	var catalog: Dictionary = get_parent().call("catalog") if get_parent() != null and get_parent().has_method("catalog") else {}
	for row: Variant in catalog.get("eras", []):
		if row is Dictionary and int((row as Dictionary).get("order", -1)) == order:
			_era = str((row as Dictionary).get("id", ""))
			return


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
	# The kit splits each chunk into up to four bins, so avoid reserving eight copies per sparse cohort.
	var count: int = maxi(2 if crowd["mid"] != null else 8, need + (need >> 2)) * copies
	var row: Dictionary = _models[crowd["model"]]
	var hidden: Transform3D = Transform3D(Basis().scaled(Vector3.ZERO), corner)
	var parts: Array = [["mmi", row["vat"].get("mesh"), true], ["blob", _blob_mesh, false]]
	if row["mid_vat"] != null and row["lod_vat"] != null:
		parts.append(["mid", row["mid_vat"].get("mesh"), true])
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
		for n: Variant in [crowd["mmi"], crowd["mid"], crowd["lod"], crowd["blob"]]:
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
	var seen: Dictionary = {}
	for i: int in n:
		var tile: Vector2i = Vector2i(xs[i], ys[i])
		if _slice_rect.has_area() and not _slice_rect.has_point(tile):
			continue
		models[i] = _model_of(snap, i)
		bins[i] = _bin(models[i], tile)
		need[bins[i]] = int(need.get(bins[i], 0)) + 1
		if _style2:
			var aid: int = aids[i]
			seen[aid] = true
			if _state.has(aid) and _state[aid].get("bin", "") != bins[i]:
				_release(aid, copies)
				_state[aid]["dirty"] = true
	# Release departing slots before assigning arrivals. A tight bin can swap all its occupants in one snapshot.
	if _style2:
		for aid: int in _state:
			if not seen.has(aid):
				_release(aid, copies)
	for key: String in need:
		_grow(key, int(need[key]), copies)
	for i: int in n:
		if bins[i].is_empty():
			continue
		var aid: int = aids[i]
		seen[aid] = true
		var model: String = models[i]
		if not _style2 and _state.has(aid) and _state[aid].get("bin", "") != bins[i]:
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
			if bool(crowd["painted"]):
				m["gait_distance"] = float(m.get("gait_distance", 0.0)) + (m["from"] as Vector2).distance_to(now_pos)
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
		var kit_index: int = STYLE2_MODELS.find(model) if _style2 else -1
		if kit_index >= 0:
			scale = STYLE2_SCALES[kit_index]
		if st.get("look") != look or float(st.get("scale", -1.0)) != scale:
			st["dirty"] = true
		st["model"] = model
		st["bin"] = bins[i]
		st["look_id"] = _int(snap, "look_ids", i)
		st["variant"] = _byte(snap, "variants", i)
		st["look"] = look
		st["scale"] = scale
		st["bits"] = ids[i] if i < ids.size() else -1
		_aid_of[st["bits"]] = aid
		var flags: Array = _flags(snap, i)
		var flags_changed: bool = flags != st.get("flags")
		if _style2 and flags_changed and m.has("spread"):
			st["dirty"] = true
		if bool(st["dirty"]) or flags_changed or m["tile"] != tile or float(m["t"]) < 1.0:
			_active[aid] = true
		st["flags"] = flags
		_state[aid] = st
	for aid: int in _state.keys():
		if not seen.has(aid):
			_release(aid, copies)
			_aid_of.erase(_state[aid]["bits"])
			_state.erase(aid)
			_motion.erase(aid)
	_spread_sharers()


## People settled on one tile (several sleepers on a camp sleep tile) are drawn spread around its centre, each in
## its own slot, instead of exactly on top of each other (presentation only; the figure stays inside its tile).
func _spread_sharers() -> void:
	var sharers: Dictionary = {}
	for aid: int in _motion:
		var m: Dictionary = _motion[aid]
		if float(m["t"]) >= 1.0:
			if not sharers.has(m["tile"]):
				sharers[m["tile"]] = []
			(sharers[m["tile"]] as Array).append(aid)
	for tile: Vector2i in sharers:
		var group: Array = sharers[tile]
		group.sort()
		for k: int in group.size():
			var off: Vector2 = Vector2.ZERO if group.size() < 2 else Vector2.from_angle(TAU * float(k) / float(group.size())) * (0.44 if _style2 else 0.28)
			var m: Dictionary = _motion[group[k]]
			if m.get("spread", Vector2.ZERO) != off:
				m["spread"] = off
				_state[group[k]]["dirty"] = true
				_active[group[k]] = true


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
		var k: float = _far_scale(d["feet"])
		out.append({"kind": "person", "id": int(d["bits"]), "feet": d["feet"], "h": float(d["h"]) * k, "r": 0.22 * k})
	return out


## The shader's far-figure enlargement at `feet` (vat_anim.gdshaderinc far_scale_at), so picking and screen boxes
## match what is drawn.
func _far_scale(feet: Vector3) -> float:
	var cam: Camera3D = get_viewport().get_camera_3d() if is_inside_tree() else null
	if cam == null:
		return 1.0
	if _far_param.is_empty():
		_far_param = Data3D.param("people.style2_far_scale" if _style2 else "people.far_scale", [55.0, 2.2])
	return clampf(cam.global_position.distance_to(feet) / float(_far_param[0]), 1.0, float(_far_param[1]))


## The body ({feet, h, far: the far-figure enlargement included in h}) of the person with entity bits `bits`, or {}
## when not drawn.
func body_of(bits: int) -> Dictionary:
	var d: Dictionary = _state.get(_aid_of.get(bits, -1), {})
	if d.is_empty():
		return {}
	var k: float = _far_scale(d["feet"])
	return {"feet": d["feet"], "h": float(d["h"]) * k, "far": k}


## Tools and gate checks: what is drawn for entity `bits` — {tile (the tile the figure stands on or walks to),
## settled (not mid-step), feet, model, look_id, variant, clip, scale} — or {} when not drawn.
func state_of(bits: int) -> Dictionary:
	var aid: int = int(_aid_of.get(bits, -1))
	if not _state.has(aid):
		return {}
	var d: Dictionary = _state[aid]
	var m: Dictionary = _motion.get(aid, {})
	return {"tile": m.get("tile", Vector2i(-1, -1)), "settled": float(m.get("t", 1.0)) >= 1.0, "feet": d["feet"], "yaw": float(d.get("draw_yaw", 0.0)),
		"model": d["model"], "look_id": int(d.get("look_id", 0)), "variant": int(d.get("variant", 0)),
		"clip": _clip_of(d, _world_sim.process_mode != Node.PROCESS_MODE_DISABLED and float(m.get("t", 1.0)) < 1.0), "scale": float(d["scale"])}


## Tools (gate step 39): every drawn figure instance of entity `bits` as [MultiMesh, index] pairs across all LODs
## and replicate copies. Contact shadows are excluded.
func figure_instances(bits: int) -> Array:
	var out: Array = []
	var aid: int = int(_aid_of.get(bits, -1))
	if not _state.has(aid):
		return out
	var crowd: Dictionary = _crowds.get(_state[aid].get("bin", ""), {})
	if crowd.is_empty() or not (crowd["slot"] as Dictionary).has(aid):
		return out
	var copies: int = maxi(1, replicate)
	var cap: int = _cap(crowd["mmi"].multimesh, copies)
	for n: Variant in [crowd["mmi"], crowd["mid"], crowd["lod"]]:
		if n != null:
			for c: int in copies:
				out.append([(n as MultiMeshInstance3D).multimesh, cap * c + int(crowd["slot"][aid])])
	return out


## Tools: the 3D feet position of entity `bits`, or Vector3.INF.
func feet_of(bits: int) -> Vector3:
	var d: Dictionary = _state.get(_aid_of.get(bits, -1), {})
	return d["feet"] if not d.is_empty() else Vector3.INF


func count() -> int:
	return _state.size()
