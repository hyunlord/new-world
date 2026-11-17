# props3d.gd (World3D/Props, made by world3d.gd): stations, camp buildings, bedrolls, ground item stacks and fire
#   safe data-authored yard props from real template site stages, and fire light of the 3D view (M3D-1 scope 5: template buildings, workshops and item stacks).
# Purpose: a finished workshop (get_workshops) is the Blender model its blueprint's construction style names
#   (get_content_catalog `construction_style` → bindings3d `stations.by_style`, else the 2D workshop binding's structure
#   id, else `stations.fallback`), standing at the footprint centre facing south, scaled down to fit a smaller
#   footprint (models declare theirs in the manifests data3d.gd merges). Its state picks the drawn model: working
#   (an active bill) or idle → the first existing candidate of bindings3d `stations.states.<model>.<state>` (the
#   w3d_ station models, then the M3D-ASSETS asset_ ones), else the station model itself. A 2D workshop binding marked
#   `marks_hearth_firing` is not drawn (as in 2D). Legacy camp buildings (get_building_snapshot) use the 2D building
#   binding's structure (`unlit` while the nearest settlement's fire is out) → bindings3d `buildings.by_structure`;
#   ring buildings belong to structures3d.gd and rows inside a workshop footprint are skipped (the 2D rule). Bedrolls
#   lie over their 2 × 1 footprint. Ground stacks (get_colony_overlay items): the catalogue item id → bindings3d
#   `items.by_id`, the M3D-ASSETS model of the 2D count bucket (1, 2–8, 9+ items → the models depicting 1, 3, 6),
#   drawn once in its own colours; an id or bucket without a model falls back to the 2D item binding's asset for the
#   bucket → `items.by_asset`, else the catalogue `art_shape` → `items.by_shape`, else `items.fallback`, with `vein`
#   and `metal` surfaces in the item's `art_tint_material` colour and 2–8 items as two copies, 9+ three. Lit models
#   (bindings3d `stations.lit`, keyed by the station model) carry an OmniLight3D at the drawn model's `light` socket
#   and glow while lit: the hearth while its settlement's fire burns, a workshop while it works an active bill. Light
#   energy follows the night (Sky.night_factor) but never reaches zero, so a working smelter reads by day too. A
#   working station without a fire throws up chips and dust instead (`stations.work_puffs`). `drawn_counts()` counts
#   the workshops and hearths the last rebuild drew (tests).
# Bridge methods called: via SimReads get_workshops, get_building_snapshot, get_settlement_snapshot, get_space,
#   get_colony_overlay, get_content_catalog; signal world_reset.
extends Node3D

const Data3D = preload("res://scripts/world3d/data3d.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")

var _world_sim: Node = null
var _terrain: Node = null
var _sky: Node = null
var _key: String = ""
var _items_key: String = ""
var _yards_key: String = ""
var _yard_nodes: Array[MeshInstance3D] = []
var _yard_warm: Dictionary = {}
var _stations: Array[MeshInstance3D] = []
## Tiles under a drawn station (people3d draws a worker standing there in front of it).
var station_tiles: Dictionary = {}
var _lights: Array[Dictionary] = []
var _item_nodes: Dictionary = {}
var _glow_on: StandardMaterial3D
var _glow_off: StandardMaterial3D
var _tinted: Dictionary = {}
var _stamp_key: String = ""
## What the last station rebuild drew: workshops and lit camp buildings (the hearth), for tests and tools.
var _counts: Dictionary = {"workshops": 0, "hearths": 0}


func setup(world_sim: Node, terrain: Node, sky: Node) -> void:
	_world_sim = world_sim
	_terrain = terrain
	_sky = sky
	_world_sim.connect("world_reset", _on_world_reset)


func _ready() -> void:
	var fire: Color = Data3D.colour(Data3D.param("light.fire_colour", []), Color(1.0, 0.62, 0.3))
	_glow_on = StandardMaterial3D.new()
	_glow_on.albedo_color = fire
	_glow_on.emission_enabled = true
	_glow_on.emission = fire
	_glow_on.emission_energy_multiplier = 3.0
	_glow_off = StandardMaterial3D.new()
	_glow_off.albedo_color = Color(0.16, 0.13, 0.12)


func _on_world_reset() -> void:
	_key = ""
	_items_key = ""
	_yards_key = ""


func _process(_delta: float) -> void:
	if _world_sim == null or _terrain == null or not bool(_terrain.call("is_complete")):
		return
	var stamp: Dictionary = get_parent().get("stamp") if get_parent() != null else {}
	@warning_ignore("integer_division")
	var skey: String = "%s|%s" % [stamp.get("overlay", -1), int(stamp.get("tick", -1)) / 12]
	# A refresh takes World3D's heavy-work slot (one per frame, none on a simulation step); else try next frame.
	if (skey == _stamp_key and not stamp.is_empty()) or (get_parent() != null and get_parent().has_method("claim_heavy")
			and not bool(get_parent().call("claim_heavy"))):
		_flicker()
		return
	_stamp_key = skey
	var shops: Dictionary = SimReads.read(_world_sim, "get_workshops")
	var buildings: Dictionary = SimReads.read(_world_sim, "get_building_snapshot")
	_rebuild_yards(buildings)
	var settlements: Dictionary = SimReads.read(_world_sim, "get_settlement_snapshot")
	var space: Dictionary = SimReads.read(_world_sim, "get_space")
	var key: String = str(hash([shops.get("kinds"), shops.get("xs"), shops.get("ys"), shops.get("bill_active"), buildings.get("kinds"),
		buildings.get("xs"), buildings.get("ys"), settlements.get("fire_lit"), space.get("version")]))
	if key != _key:
		_key = key
		_rebuild_stations(shops, buildings, settlements, space)
	var overlay: Dictionary = SimReads.read(_world_sim, "get_colony_overlay")
	var ikey: String = str(int(overlay.get("version", 0)))
	if ikey != _items_key:
		_items_key = ikey
		_rebuild_items(overlay)
	_flicker()


func _flicker() -> void:
	var night: float = float(_sky.call("night_factor")) if _sky != null else 0.0
	var base_energy: float = float(Data3D.param("light.fire_light_energy", 2.2))
	for l: Dictionary in _lights:
		var light: OmniLight3D = l["light"]
		var flicker: float = 0.9 + 0.1 * sin(Time.get_ticks_msec() * 0.011 + float(l["phase"])) * sin(Time.get_ticks_msec() * 0.0037 + float(l["phase"]) * 2.0)
		light.light_energy = base_energy * (0.35 + 0.65 * night) * flicker


func _catalog_styles() -> Dictionary:
	var out: Dictionary = {}
	for row: Variant in (get_parent().call("catalog") as Dictionary).get("blueprints", []):
		if row is Dictionary:
			out[int((row as Dictionary).get("code", -1))] = str((row as Dictionary).get("construction_style", ""))
	return out


func _station_model(code: int, styles: Dictionary) -> String:
	var by_style: Dictionary = Data3D.binding("stations.by_style", {})
	var style: String = str(styles.get(code, ""))
	if by_style.has(style):
		return str(by_style[style])
	var structure: Variant = ArtData.binding("workshops", code).get("structure")
	if structure is String and by_style.has(structure):
		return str(by_style[structure])
	return str(Data3D.binding("stations.fallback", ""))


func _rebuild_stations(shops: Dictionary, buildings: Dictionary, settlements: Dictionary, space: Dictionary) -> void:
	for n: MeshInstance3D in _stations:
		n.queue_free()
	_stations.clear()
	_lights.clear()
	_counts = {"workshops": 0, "hearths": 0}
	station_tiles.clear()
	var styles: Dictionary = _catalog_styles()
	var ids: PackedInt32Array = shops.get("ids", PackedInt32Array())
	var kinds: PackedInt64Array = shops.get("kinds", PackedInt64Array())
	var xs: PackedInt32Array = shops.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = shops.get("ys", PackedInt32Array())
	var ws: PackedInt32Array = shops.get("ws", PackedInt32Array())
	var hs: PackedInt32Array = shops.get("hs", PackedInt32Array())
	var active: Dictionary = {}
	var bill_shops: PackedInt32Array = shops.get("bill_shops", PackedInt32Array())
	var bill_active: PackedInt32Array = shops.get("bill_active", PackedInt32Array())
	for i: int in mini(bill_shops.size(), bill_active.size()):
		if bill_active[i] == 1:
			active[bill_shops[i]] = true
	var footprints: Array[Rect2i] = []
	var firing: Dictionary = {}
	for i: int in mini(kinds.size(), mini(xs.size(), ys.size())):
		var rect: Rect2i = Rect2i(xs[i], ys[i], ws[i] if i < ws.size() else 1, hs[i] if i < hs.size() else 1)
		var b: Dictionary = ArtData.binding("workshops", int(kinds[i]))
		# The hearth's fire-pot workshop is not drawn and does not hide the hearth (the 2D rule); it marks firing.
		if bool(b.get("marks_hearth_firing", false)):
			if active.has(i):
				firing[rect.position] = true
			continue
		footprints.append(rect)
		for sy: int in rect.size.y:
			for sx: int in rect.size.x:
				station_tiles[rect.position + Vector2i(sx, sy)] = true
		_place(_station_model(int(kinds[i]), styles), rect, active.has(i), "shop%d" % (ids[i] if i < ids.size() else i))
		_counts["workshops"] = int(_counts["workshops"]) + 1
	# Legacy camp buildings.
	var bx: PackedInt32Array = buildings.get("xs", PackedInt32Array())
	var by: PackedInt32Array = buildings.get("ys", PackedInt32Array())
	var bk: PackedByteArray = buildings.get("kinds", PackedByteArray())
	var by_structure: Dictionary = Data3D.binding("buildings.by_structure", {})
	for i: int in mini(bk.size(), mini(bx.size(), by.size())):
		var t: Vector2i = Vector2i(bx[i], by[i])
		var inside: bool = false
		for r: Rect2i in footprints:
			if r.has_point(t):
				inside = true
				break
		if inside:
			continue
		var row: Dictionary = ArtData.binding("buildings", int(bk[i]))
		if row.has("ring") or not row.get("structure") is String:
			continue
		var structure: String = str(row["structure"])
		var lit: bool = _fire_lit(settlements, t)
		if not lit and row.get("unlit") is String:
			structure = str(row["unlit"])
		elif firing.has(t) and row.get("firing") is String:
			structure = str(row["firing"])
		var model: String = str(by_structure.get(structure, Data3D.binding("buildings.fallback", "")))
		_place(model, Rect2i(t, Vector2i.ONE), lit and bool(row.get("light", false)), "b%d" % i)
		if bool(row.get("light", false)):
			_counts["hearths"] = int(_counts["hearths"]) + 1
	# Bedrolls (2 × 1 from the bed tile).
	var bed_model: String = str(Data3D.binding("stations.bed", ""))
	var bedx: PackedInt32Array = space.get("bed_xs", PackedInt32Array())
	var bedy: PackedInt32Array = space.get("bed_ys", PackedInt32Array())
	var fp: Array = (ArtData.bindings().get("space", {}) as Dictionary).get("bed", {}).get("footprint", [2, 1])
	for i: int in mini(bedx.size(), bedy.size()):
		_place(bed_model, Rect2i(bedx[i], bedy[i], int(fp[0]), int(fp[1])), false, "bed%d" % i)


func _fire_lit(settlements: Dictionary, t: Vector2i) -> bool:
	var fx: PackedInt32Array = settlements.get("formation_xs", PackedInt32Array())
	var fy: PackedInt32Array = settlements.get("formation_ys", PackedInt32Array())
	var lit: PackedByteArray = settlements.get("fire_lit", PackedByteArray())
	var best: int = -1
	var best_d: int = 1 << 30
	for i: int in mini(lit.size(), mini(fx.size(), fy.size())):
		var d: int = (fx[i] - t.x) * (fx[i] - t.x) + (fy[i] - t.y) * (fy[i] - t.y)
		if d < best_d:
			best_d = d
			best = i
	return best < 0 or lit[best] == 1


## What the last station rebuild drew: {workshops, hearths} (camp buildings whose 2D binding gives light).
func drawn_counts() -> Dictionary:
	return _counts


## The model drawn for station model `id` in its state (bindings3d `stations.states`), else `id`.
static func state_model(id: String, working: bool) -> String:
	var states: Variant = (Data3D.binding("stations.states", {}) as Dictionary).get(id)
	if states is not Dictionary:
		return id
	var shown: String = Data3D.first_model((states as Dictionary).get("working" if working else "idle", []))
	return shown if shown != "" else id


## Place model `id` centred on footprint `rect`, facing south; `lit` (working) picks its state model and turns its
## glow and light on.
func _place(id: String, rect: Rect2i, lit: bool, node_name: String) -> void:
	var mi: MeshInstance3D = MeshInstance3D.new()
	mi.name = node_name
	var shown: String = state_model(id, lit)
	var mesh: Mesh = Data3D.mesh(shown)
	mi.mesh = mesh
	var row: Dictionary = Data3D.model_row(shown)
	if row.is_empty():
		row = Data3D.model_row(id)
	var fp: Array = row.get("footprint", [rect.size.x, rect.size.y])
	var fw: float = maxf(0.5, float(fp[0]))
	var fh: float = maxf(0.5, float(fp[1]))
	var s: float = clampf(minf(float(rect.size.x) / fw, float(rect.size.y) / fh), 0.85, 1.0)
	var centre: Vector2 = Vector2(rect.position) + Vector2(rect.size) * 0.5
	var g: float = float(_terrain.call("height_at", centre.x, centre.y))
	mi.transform = Transform3D(Basis.IDENTITY.scaled(Vector3(s, s, s)), Vector3(centre.x, g, centre.y))
	add_child(mi)
	_stations.append(mi)
	var is_lit_model: bool = bool((Data3D.binding("stations.lit", {}) as Dictionary).get(id, false))
	for surface: int in mesh.get_surface_count():
		if Data3D.surface_key(mesh, surface) == "glow":
			mi.set_surface_override_material(surface, _glow_on if lit else _glow_off)
	if lit and not is_lit_model:
		mi.add_child(_work_puffs(row))
	if is_lit_model and lit:
		var light: OmniLight3D = OmniLight3D.new()
		var sockets: Dictionary = row.get("sockets", {})
		var socket: Variant = sockets.get("light")
		var at: Vector3 = Vector3(0, 0.8, 0)
		if socket is Array and (socket as Array).size() >= 3:
			at = Vector3(float(socket[0]), float(socket[1]), float(socket[2]))
		light.position = at + Vector3(0, 0.25, 0)
		light.light_color = Data3D.colour(Data3D.param("light.fire_colour", []), Color(1.0, 0.62, 0.3))
		light.omni_range = float(Data3D.param("light.fire_light_range", 7.0))
		light.shadow_enabled = false
		mi.add_child(light)
		_lights.append({"light": light, "phase": float(rect.position.x * 7 + rect.position.y * 3)})


## Chips and dust rising off a station without a fire while it works a bill (params3d `stations.work_puffs`), so a
## working bench reads apart from an idle one.
func _work_puffs(row: Dictionary) -> CPUParticles3D:
	var p: CPUParticles3D = CPUParticles3D.new()
	p.name = "Working"
	var socket: Variant = (row.get("sockets", {}) as Dictionary).get("work")
	p.position = Vector3(float(socket[0]), float(socket[1]), float(socket[2])) if socket is Array and (socket as Array).size() >= 3 else Vector3(0, 0.85, 0)
	p.amount = int(Data3D.param("stations.work_puffs.amount", 10))
	p.lifetime = 0.9
	p.explosiveness = 0.2
	p.direction = Vector3.UP
	p.spread = 35.0
	p.initial_velocity_min = 0.4
	p.initial_velocity_max = 0.9
	p.gravity = Vector3(0, -1.2, 0)
	p.scale_amount_min = 0.5
	p.scale_amount_max = 1.0
	var quad: QuadMesh = QuadMesh.new()
	quad.size = Vector2(0.05, 0.05)
	var mat: StandardMaterial3D = StandardMaterial3D.new()
	mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	mat.billboard_mode = BaseMaterial3D.BILLBOARD_ENABLED
	mat.albedo_color = Data3D.colour(Data3D.param("stations.work_puffs.colour", []), Color(0.78, 0.7, 0.55))
	quad.material = mat
	p.mesh = quad
	p.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	return p


# --- items -------------------------------------------------------------------------------------------------

func _rebuild_items(overlay: Dictionary) -> void:
	var xs: PackedInt32Array = overlay.get("item_xs", PackedInt32Array())
	var ys: PackedInt32Array = overlay.get("item_ys", PackedInt32Array())
	var kinds: PackedInt64Array = overlay.get("item_kinds", PackedInt64Array())
	var counts: PackedInt32Array = overlay.get("item_counts", PackedInt32Array())
	var catalog: Dictionary = {}
	for row: Variant in (get_parent().call("catalog") as Dictionary).get("items", []):
		if row is Dictionary:
			catalog[int((row as Dictionary).get("code", -1))] = row
	var limits: Array = (ArtData.bindings().get("items", {}) as Dictionary).get("bucket_limits", [1, 8])
	var batches: Dictionary = {}
	for i: int in mini(kinds.size(), mini(xs.size(), ys.size())):
		var code: int = int(kinds[i])
		var n: int = counts[i] if i < counts.size() else 1
		var bucket: int = 0 if n <= int(limits[0]) else (1 if n <= int(limits[1]) else 2)
		var row: Dictionary = catalog.get(code, {})
		var model: String = own_item_model(str(row.get("id", "")), bucket)
		# An own model already depicts the stack: one copy in its own colours.
		var copies: int = 1 if model != "" else bucket + 1
		var tint: String = "" if model != "" else str(row.get("art_tint_material", ""))
		if model == "":
			model = _item_model(code, bucket, row)
		var bkey: String = model + "|" + tint
		if not batches.has(bkey):
			batches[bkey] = []
		var t: Vector2i = Vector2i(xs[i], ys[i])
		for copy: int in copies:
			var off: Vector2 = [Vector2(0, 0), Vector2(0.22, -0.18), Vector2(-0.2, 0.2)][copy]
			var x: float = t.x + 0.5 + off.x
			var z: float = t.y + 0.5 + off.y
			var yaw: float = float((t.x * 37 + t.y * 91 + copy * 53) % 360) * PI / 180.0
			(batches[bkey] as Array).append(Transform3D(Basis(Vector3.UP, yaw), Vector3(x, float(_terrain.call("height_at", x, z)), z)))
	for k: String in _item_nodes.keys():
		if not batches.has(k):
			(_item_nodes[k] as Node).queue_free()
			_item_nodes.erase(k)
	for k: String in batches:
		var parts: PackedStringArray = k.split("|")
		var mm: MultiMesh = MultiMesh.new()
		mm.transform_format = MultiMesh.TRANSFORM_3D
		mm.mesh = _tinted_mesh(parts[0], parts[1])
		var xf: Array = batches[k]
		mm.instance_count = xf.size()
		for i: int in xf.size():
			mm.set_instance_transform(i, xf[i])
		var mmi: MultiMeshInstance3D = _item_nodes.get(k)
		if mmi == null:
			mmi = MultiMeshInstance3D.new()
			add_child(mmi)
			_item_nodes[k] = mmi
		mmi.multimesh = mm


## The M3D-ASSETS model of catalogue item `id` for count bucket `bucket` (bindings3d `items.by_id`), else "".
static func own_item_model(id: String, bucket: int) -> String:
	var models: Variant = (Data3D.binding("items.by_id", {}) as Dictionary).get(id)
	if models is Array and bucket < (models as Array).size():
		return Data3D.first_model([(models as Array)[bucket]])
	return ""


func _item_model(code: int, bucket: int, row: Dictionary) -> String:
	var by_asset: Dictionary = Data3D.binding("items.by_asset", {})
	var b: Dictionary = ArtData.binding("items", code)
	var asset: String = ""
	if b.get("assets") is Array and not (b["assets"] as Array).is_empty():
		asset = str((b["assets"] as Array)[mini(bucket, (b["assets"] as Array).size() - 1)])
	elif b.get("many") is String and bucket > 0:
		asset = str(b["many"])
	elif b.get("asset") is String:
		asset = str(b["asset"])
	if by_asset.has(asset):
		return str(by_asset[asset])
	var by_shape: Dictionary = Data3D.binding("items.by_shape", {})
	var shape: String = str(row.get("art_shape", ""))
	if by_shape.has(shape):
		return str(by_shape[shape])
	return str(Data3D.binding("items.fallback", ""))


## Model `id` with its `vein` / `metal` surfaces in material `tint`'s colour (cached).
func _tinted_mesh(id: String, tint: String) -> Mesh:
	var key: String = id + "|" + tint
	if _tinted.has(key):
		return _tinted[key]
	var base: Mesh = Data3D.mesh(id)
	if tint == "" or base is not ArrayMesh:
		_tinted[key] = base
		return base
	var src: ArrayMesh = base
	var out: ArrayMesh = ArrayMesh.new()
	for s: int in src.get_surface_count():
		out.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, src.surface_get_arrays(s))
		var name: String = Data3D.surface_key(src, s)
		var mat: Material = src.surface_get_material(s)
		if name in ["vein", "metal"]:
			var m: StandardMaterial3D = StandardMaterial3D.new()
			m.albedo_color = Data3D.material_colour(tint, "Mineral")
			m.metallic = 0.5 if name == "metal" else 0.1
			m.roughness = 0.4
			m.resource_name = name
			mat = preload("res://scripts/world3d/textures3d.gd").imported(m)
			if mat is ShaderMaterial:
				(mat as ShaderMaterial).set_shader_parameter("texture_tile", float(preload("res://scripts/world3d/textures3d.gd").index("materials", tint, preload("res://scripts/world3d/textures3d.gd").index("surfaces", name))))
		out.surface_set_material(out.get_surface_count() - 1, mat)
	_tinted[key] = out
	return out


# The bridge emits only free footprint-adjacent tiles; props never claim or block simulation space.
func _rebuild_yards(buildings: Dictionary) -> void:
	for model: String in buildings.get("yard_models", PackedStringArray()):
		if not _yard_warm.has(model):
			Data3D.mesh(model)
			_yard_warm[model] = true
	var sites: Array = buildings.get("template_sites", [])
	var key: String = str(hash(sites.map(func(row: Dictionary) -> Variant: return [row.get("id"), row.get("yard_props", [])])))
	if key == _yards_key:
		return
	_yards_key = key
	for node: MeshInstance3D in _yard_nodes:
		node.queue_free()
	_yard_nodes.clear()
	var claimed: Dictionary = {}
	for value: Variant in sites:
		if value is not Dictionary:
			continue
		for prop: Dictionary in (value as Dictionary).get("yard_props", []):
			var tile: Vector2i = Vector2i(int(prop.get("x", 0)), int(prop.get("y", 0)))
			if claimed.has(tile):
				continue
			claimed[tile] = true
			var node: MeshInstance3D = MeshInstance3D.new()
			node.name = "Yard_%s_%s" % [(value as Dictionary).get("id", -1), prop.get("id", "")]
			node.mesh = Data3D.mesh(str(prop.get("model", "")))
			var x: float = float(prop.get("x", 0)) + 0.5
			var y: float = float(prop.get("y", 0)) + 0.5
			# A decorative model fits wholly within its admitted free tile, including wide fence GLBs.
			var bounds: AABB = node.mesh.get_aabb()
			var size_limit: float = float(Data3D.param("yards.tile_fill", 0.85))
			var fit: float = minf(1.0, size_limit / maxf(0.01, maxf(bounds.size.x, bounds.size.z)))
			var yard_basis: Basis = Basis(Vector3.UP, float(prop.get("rotation", 0)) * PI * 0.5).scaled(Vector3.ONE * fit)
			var centre: Vector3 = Vector3(bounds.get_center().x, bounds.position.y, bounds.get_center().z)
			node.transform = Transform3D(yard_basis, Vector3(x, float(_terrain.call("height_at", x, y)), y) - yard_basis * centre)
			add_child(node)
			_yard_nodes.append(node)
