# people3d.gd (World3D/People, made by world3d.gd): the colony's people in 3D (M3D-1 scope 6 and 7). PLACEHOLDER
#   figures: one Blender-script villager (tools/3d/blender/people.py) for everyone until M3D-2's character models.
# Purpose: every row of get_agent_snapshot is one instance of a vertex-animated MultiMesh (vat3d.gd): no per-person
#   node. A person glides from tile to tile over the 2D view's step time (move_interval_ticks at the current speed,
#   the AgentLayer rule), faces where they walk, and plays a clip by state: sleep (states 3 with target_kinds 3),
#   carry (carries an item, wood or food while moving), walk, work (a job while standing), idle. Children are
#   `people.child_scale`, infants smaller. Tints come from the 2D appearance data: skin and hair from the look's face
#   variant (assets/v8art/people/variant_transforms.json `target_*_rgb`, elders the 2D grey hair), clothing from
#   render_params `people.clothing_by_top` by top code, else the look's top colour; a top listed in bindings3d
#   `people.bare_torso_tops` shows skin on the torso. The person the inspector shows gets a ring at their feet.
#   Picking: `pick(screen, camera, mode)` finds the person whose body (feet to head) passes nearest the cursor within
#   `people.pick_radius_px` and returns the centre of that person's 2D drawn rect (AgentLayer.drawn_rect), so the
#   unchanged OrderTool click opens the same inspector as in 2D.
# Bridge methods called: via SimReads get_agent_snapshot, get_world_info; signal world_reset.
extends Node3D

const Data3D = preload("res://scripts/world3d/data3d.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
const Vat = preload("res://scripts/world3d/vat3d.gd")

var _world_sim: Node = null
var _terrain: Node = null
var _vat: RefCounted = null
var _mmi: MultiMeshInstance3D = null
var _ring: MeshInstance3D = null
## AgentId → {from: Vector2, to: Vector2, t: float, dur: float, yaw: float, tile: Vector2i}
var _motion: Dictionary = {}
var _transforms: Dictionary = {}
var _top_colours: Dictionary = {}
## Tools only (world3d_fps.gd `--replicate`): draw every person this many times, offset, to load the renderer past the
## bridge's stress cap. Never set in the game.
var replicate: int = 1
var _looks: Dictionary = {}
var _snap_tick: int = -1
## AgentId → {look, scale, bits, flags, clip, dirty, feet, h}; AgentId → MultiMesh slot (stable while they live).
var _state: Dictionary = {}
var _slot: Dictionary = {}
var _free: Array[int] = []
var _clip_packs: Dictionary = {}


func setup(world_sim: Node, terrain: Node) -> void:
	_world_sim = world_sim
	_terrain = terrain
	_world_sim.connect("world_reset", func() -> void:
		_motion.clear()
		_state.clear()
		_slot.clear()
		_free.clear()
		_mmi.multimesh = null
		_looks.clear())


func _ready() -> void:
	_vat = Vat.load_character(str(Data3D.binding("people.model", "")))
	_mmi = MultiMeshInstance3D.new()
	_mmi.name = "Crowd"
	add_child(_mmi)
	if bool(_vat.get("ok")):
		_mmi.material_override = _vat.call("material", Data3D.colour(Data3D.binding("people.other_colour", []), Color(0.3, 0.22, 0.15)))
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
	if _world_sim == null or _terrain == null or int(_terrain.get("width")) <= 0 or not bool(_vat.get("ok")):
		return
	var stamp: Dictionary = get_parent().get("stamp") if get_parent() != null else {}
	if _state.is_empty() or stamp.is_empty() or int(stamp.get("tick", -1)) != _snap_tick:
		_snap_tick = int(stamp.get("tick", -1))
		_refresh(SimReads.read(_world_sim, "get_agent_snapshot"))
	var running: bool = _world_sim.can_process()
	var mm: MultiMesh = _mmi.multimesh
	var copies: int = maxi(1, replicate)
	var cap: int = _cap(mm, copies)
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
		var slot: int = int(_slot[aid])
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
		var clip: float = _clip_pack(_clip_of(st, float(m["t"]) < 1.0))
		if clip != float(st["clip"]) or bool(st["dirty"]):
			st["clip"] = clip
			var look: Dictionary = st["look"]
			var custom: Color = Color(clip, float(aid % 97) * 0.173, float(look["hair_p"]), float(look["cloth_p"]))
			for c: int in copies:
				mm.set_instance_color(cap * c + slot, look["skin"])
				mm.set_instance_custom_data(cap * c + slot, custom)
		st["dirty"] = false
		if int(st["bits"]) == selected and selected != -1:
			_ring.visible = true
			_ring.position = (st["feet"] as Vector3) + Vector3(0, 0.04, 0)


## Instances per copy of the crowd MultiMesh.
static func _cap(mm: MultiMesh, copies: int) -> int:
	if mm == null:
		return 0
	@warning_ignore("integer_division")
	return mm.instance_count / maxi(1, copies)


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
	var mm: MultiMesh = _mmi.multimesh
	var seen: Dictionary = {}
	var grow: bool = mm == null or mm.instance_count < n * copies or mm.instance_count % copies != 0
	if grow:
		mm = MultiMesh.new()
		mm.transform_format = MultiMesh.TRANSFORM_3D
		mm.use_colors = true
		mm.use_custom_data = true
		mm.mesh = _vat.get("mesh")
		mm.instance_count = maxi(16, n + (n >> 2)) * copies
		_mmi.multimesh = mm
		_slot.clear()
		_free.clear()
		for k: int in _cap(mm, copies):
			_free.append(_cap(mm, copies) - 1 - k)
			for c: int in copies:
				mm.set_instance_transform(_cap(mm, copies) * c + k, Transform3D(Basis().scaled(Vector3.ZERO), Vector3.ZERO))
		for st0: Dictionary in _state.values():
			st0["dirty"] = true
	for i: int in n:
		var aid: int = aids[i]
		seen[aid] = true
		if not _slot.has(aid):
			_slot[aid] = _free.pop_back()
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
		var scale: float = (child_scale * (0.75 if _byte(snap, "infants", i) == 1 else 1.0)) if stage == 1 else 1.0
		if st.get("look") != look or float(st.get("scale", -1.0)) != scale:
			st["dirty"] = true
		st["look"] = look
		st["scale"] = scale
		st["bits"] = ids[i] if i < ids.size() else -1
		st["flags"] = _flags(snap, i)
		_state[aid] = st
	for aid: int in _state.keys():
		if not seen.has(aid):
			var slot: int = int(_slot[aid])
			for c: int in copies:
				mm.set_instance_transform(_cap(mm, copies) * c + slot, Transform3D(Basis().scaled(Vector3.ZERO), Vector3.ZERO))
			_free.append(slot)
			_slot.erase(aid)
			_state.erase(aid)
			_motion.erase(aid)


static func _pos(m: Dictionary) -> Vector2:
	var t: float = float(m["t"])
	return (m["from"] as Vector2).lerp(m["to"], t * t * (3.0 - 2.0 * t))


func _clip_pack(clip: String) -> float:
	if not _clip_packs.has(clip):
		_clip_packs[clip] = float(_vat.call("clip_pack", clip))
	return _clip_packs[clip]


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


## World3D picker: the 2D drawn-rect centre of the person under `screen`, or (-1, -1).
func pick(screen: Vector2, camera: Camera3D, mode: int) -> Vector2:
	if mode != 0:
		return Vector2(-1, -1)
	var radius: float = float(Data3D.param("people.pick_radius_px", 18))
	var best: int = -1
	var best_depth: float = INF
	for d: Dictionary in _state.values():
		var feet: Vector3 = d["feet"]
		var head: Vector3 = feet + Vector3(0, float(d["h"]), 0)
		if camera.is_position_behind(feet) or camera.is_position_behind(head):
			continue
		var a: Vector2 = camera.unproject_position(feet)
		var b: Vector2 = camera.unproject_position(head)
		if not _on_screen(a, b):
			continue
		var px_radius: float = maxf(radius, (a - b).length() * 0.22)
		if Geometry2D.get_closest_point_to_segment(screen, a, b).distance_to(screen) <= px_radius:
			var depth: float = camera.global_position.distance_to(feet)
			if depth < best_depth:
				best_depth = depth
				best = int(d["bits"])
	if best == -1:
		return Vector2(-1, -1)
	var layer: Node = get_node_or_null("/root/Main/ColonyView/AgentLayer")
	if layer != null and layer.has_method("drawn_rect"):
		var r: Rect2 = layer.call("drawn_rect", best)
		if r.size != Vector2.ZERO:
			return r.get_center()
	return Vector2(-1, -1)


## Tools: the 3D feet position of entity `bits`, or Vector3.INF.
func feet_of(bits: int) -> Vector3:
	for d: Dictionary in _state.values():
		if int(d["bits"]) == bits:
			return d["feet"]
	return Vector3.INF


func count() -> int:
	return _state.size()


## A body is pickable only when both ends project inside the viewport and it spans a sane height on screen.
func _on_screen(a: Vector2, b: Vector2) -> bool:
	var view: Rect2 = get_viewport().get_visible_rect().grow(8.0)
	return view.has_point(a) and view.has_point(b) and a.distance_to(b) < view.size.y * 0.5
