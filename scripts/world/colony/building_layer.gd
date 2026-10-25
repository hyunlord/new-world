# building_layer.gd (ColonyView/BuildingLayer in scenes/main.tscn): the hearth, shelters, finished huts, finished
#   workshops and bedrolls of the colony view at 64-px tiles (M-ART2 §3.5 "things"). Floors, walls, doors and fords
#   of the built space are the ground layer's (ground_layer.gd).
# Every thing is a standing Sprite2D in the y-sorted ColonyView/World (position = its base point, offset = −anchor,
#   nearest filtering), so it depth-sorts with people, animals, trees and wall strips by base y. Sprites come from
#   bindings.json by sim code: `buildings` (hearth: `structure` lit, `unlit` when its settlement's fire is out —
#   get_settlement_snapshot `fire_lit` of the settlement whose formation tile is nearest —, `firing` while the hearth's
#   fire-pot workshop, the `workshops` row with `marks_hearth_firing`, works a bill; shelter `structure` on its tile),
#   `workshops` (`structure`, or `working` while a bill is active; base at the footprint's bottom centre:
#   x = left·64 + w·32, y = (top + h − 1)·64 + 43) and `space.bed` (bedrolls of get_space, footprint 2 × 1 from the
#   bed tile). A finished hut (building kind with a `ring` row) is not in get_space: the ring's walls are assembled
#   by wall_baker.gd through site_ghost.gd (the material resolver on the ring's `wall_style`) into one strip per row,
#   its door stands as a sprite (open while a person is on it) and its floor tiles are drawn flat by this node under
#   the world layer (z −8). A building snapshot row inside a workshop footprint is skipped (finished workshops sit in
#   the building registry without a kind and read as kind 0); the camp hearth's fire-pot workshop (a `workshops` row
#   with no structure) is not drawn.
# Rebuilt when the building snapshot, the workshop list or the settlement fire state changes, or the space version
#   moves (bedrolls); F12 (params) rebuilds everything.
# Bridge methods: via SimReads get_building_snapshot, get_workshops, get_space, get_settlement_snapshot,
#   get_agent_snapshot; signal world_reset.
extends Node2D

const View = preload("res://scripts/world/colony/view.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
const LitWarm = preload("res://scripts/world/colony/lit_warm_representatives.gd")
const SiteGhost = preload("res://scripts/world/art/site_ghost.gd")
const TILE: int = View.TILE

var _world_sim: Node = null
var _world: Node2D = null
var _key: int = -1
var _space_version: int = -1
var _mux2_profile: bool = OS.get_environment("WORLDSIM_MUX2_PROFILE") == "1"
var _mux2_frame: int = -1
var _mux2_usec: Dictionary = {}
var _mux2_counts: Dictionary = {}
var _mux2_texture_ids: Array[String] = []

var _params_version: int = -1
var _texture_warm_pending: bool = true
var _texture_warm_report: Dictionary = {"complete": false, "reason": "not started"}
var _render_warm_report: Dictionary = {}
var _render_warm_viewport: SubViewport
var _render_warm_texture: ViewportTexture
var _render_warm_generation: int = 0
var _render_warm_pending: bool = false
var _render_warm_wait_started_usec: int = 0
var _render_warm_wait_checks: int = 0
var _render_warm_wait_history: Array[Dictionary] = []
## [building code, tile] per drawn building (hearths, shelters, huts).
var _rows: Array[Array] = []
## [workshop code, footprint Rect2i, working] per drawn workshop.
var _shops: Array[Array] = []
## Bed tiles of the built space and the number of walls (for tests and tools).
var _beds: Array[Vector2i] = []
var _wall_count: int = 0
var _floor_pieces: int = 0
## Key → Sprite2D of every standing thing (pooled by key).
var _sprites: Dictionary = {}
## Hut door tile → [Sprite2D, door style].
var _doors: Dictionary = {}
## Hut floor tiles (world) → floor style, drawn flat by this node.
var _floor_tiles: Dictionary = {}
var _floor_tex: Dictionary = {}


func _mux2_sync_frame() -> void:
	var frame: int = Engine.get_process_frames()
	if frame != _mux2_frame:
		_mux2_frame = frame
		_mux2_counts = {"sprite_creates": 0, "sprite_frees": 0, "texture_requests": 0}
		_mux2_texture_ids.clear()
		_mux2_usec = {"process": 0, "draw": 0, "rebuild": 0, "huts": 0, "floor_prepare": 0, "door_prepare": 0, "resource_warmup": 0}

func _mux2_record(metric: String, elapsed: int) -> void:
	_mux2_sync_frame()
	_mux2_usec[metric] = int(_mux2_usec.get(metric, 0)) + elapsed

func mux2_frame_timings() -> Dictionary:
	if not _mux2_profile: return {"status": "DISABLED"}
	_mux2_sync_frame()
	return {"status": "MEASURED", "frame": _mux2_frame, "usec": _mux2_usec.duplicate(), "counts": _mux2_counts.duplicate(), "texture_ids": _mux2_texture_ids.duplicate(), "resource_warmup": _texture_warm_report.duplicate() if int(_texture_warm_report.get("frame", -1)) == _mux2_frame else {}, "render_warmup": _render_warm_report.duplicate() if _mux2_frame <= int(_render_warm_report.get("completed_frame", _render_warm_report.get("frame", -2))) + 1 else {}}

func _ready() -> void:
	z_as_relative = false
	z_index = -8
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_world = get_node_or_null("../World")
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	if _world_sim != null:
		_world_sim.connect("world_reset", _on_world_reset)


func _on_world_reset() -> void:
	_cancel_render_warmup()
	_texture_warm_pending = true
	_key = -1
	_space_version = -1
	_clear()


func _clear() -> void:
	for s: Node in _sprites.values():
		s.queue_free()
	_sprites.clear()
	for d: Array in _doors.values():
		(d[0] as Node).queue_free()
	_doors.clear()
	_floor_tiles.clear()
	queue_redraw()


func _process(_delta: float) -> void:
	var started: int = Time.get_ticks_usec() if _mux2_profile else 0
	_process_body(_delta)
	if _mux2_profile: _mux2_record("process", Time.get_ticks_usec() - started)


func _process_body(_delta: float) -> void:
	if _world_sim == null or _world == null:
		return
	if _params_version != ArtData.params_version():
		_params_version = ArtData.params_version()
		_cancel_render_warmup()
		_texture_warm_pending = true
		_key = -1
		_space_version = -1
		_floor_tex.clear()
		_clear()
	# Native Node processing state keeps preload inside active gameplay and its measured startup frames.
	var simulation_node: Node = _world_sim
	if _texture_warm_pending and simulation_node.process_mode == Node.PROCESS_MODE_INHERIT and simulation_node.is_processing():
		_texture_warm_pending = false
		_texture_warm_report = warm_texture_variants(ArtData.bindings())
		_cancel_render_warmup()
		_render_warm_pending = true
		_render_warm_wait_started_usec = Time.get_ticks_usec()
		_render_warm_wait_checks = 0
		_render_warm_wait_history.clear()
		_render_warm_report = {"kind": "OFFSCREEN_DRAW_ONCE", "status": "WAITING_FOR_RUNTIME_LIGHT", "complete": false,
			"wait_started_usec": _render_warm_wait_started_usec, "resource_frame": Engine.get_process_frames(), "frame": Engine.get_process_frames()}
		if _mux2_profile: _mux2_record("resource_warmup", int(_texture_warm_report["elapsed_usec"]))
	if _render_warm_pending and simulation_node.process_mode == Node.PROCESS_MODE_INHERIT and simulation_node.is_processing():
		_try_render_warmup()
	var snap: Dictionary = SimReads.read(_world_sim, "get_building_snapshot")
	var shops: Dictionary = SimReads.read(_world_sim, "get_workshops")
	var settlements: Dictionary = SimReads.read(_world_sim, "get_settlement_snapshot")
	var space: Dictionary = SimReads.read(_world_sim, "get_space")
	var space_version: int = int(space.get("version", 0))
	var key: int = hash([snap.get("xs"), snap.get("ys"), snap.get("kinds"), shops.get("kinds"), shops.get("xs"), shops.get("ys"),
		shops.get("bill_active"), settlements.get("fire_lit"), space_version])
	if key != _key:
		_key = key
		_space_version = space_version
		_rebuild(snap, shops, settlements, space)
	if not _doors.is_empty():
		_update_doors()


## One real offscreen draw uses the same cached texture objects and ordinary sprite material/filter.
## The private 2D world never contributes pixels or input to the main viewport.
static func make_warm_viewport(ids: Array[String]) -> SubViewport:
	if ids.is_empty() or ids.size() > 64: return null
	var cell: Vector2i = Vector2i.ONE
	var textures: Array[Texture2D] = []
	for id: String in ids:
		var texture: Texture2D = ArtData.texture(id)
		if texture == null: return null
		textures.append(texture)
		cell.x = maxi(cell.x, texture.get_width())
		cell.y = maxi(cell.y, texture.get_height())
	var columns: int = ceili(sqrt(float(ids.size())))
	var rows: int = ceili(float(ids.size()) / columns)
	var size: Vector2i = Vector2i(maxi(2, cell.x * columns), maxi(2, cell.y * rows))
	if size.x > 2048 or size.y > 2048 or size.x * size.y > 4194304: return null
	var viewport: SubViewport = SubViewport.new()
	viewport.name = "Mux2BuildingWarmup"
	viewport.size = size
	viewport.world_2d = World2D.new()
	viewport.disable_3d = true
	viewport.gui_disable_input = true
	viewport.transparent_bg = true
	viewport.render_target_update_mode = SubViewport.UPDATE_ONCE
	viewport.add_to_group("mux2_render_warmup")
	for index: int in textures.size():
		var sprite: Sprite2D = Sprite2D.new()
		sprite.texture = textures[index]
		sprite.centered = false
		sprite.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
		sprite.position = Vector2((index % columns) * cell.x, floori(float(index) / columns) * cell.y)
		viewport.add_child(sprite)
	return viewport

## Copy actual runtime light resources/settings into a fresh private node; never mutate the source.
const WARM_LIGHT_FIELDS: Array[String] = ["texture", "texture_scale", "offset", "height", "blend_mode", "color", "texture_filter", "texture_repeat", "range_item_cull_mask", "range_layer_min", "range_layer_max", "range_z_min", "range_z_max", "shadow_enabled", "shadow_filter", "shadow_filter_smooth", "shadow_item_cull_mask", "shadow_color"]

static func clone_warm_light(source: PointLight2D) -> PointLight2D:
	if source == null or source.texture == null or source.shadow_enabled: return null
	var names: Dictionary = {}
	for property: Dictionary in source.get_property_list(): names[str(property["name"])] = true
	for field: String in WARM_LIGHT_FIELDS:
		if not names.has(field): return null
	var light: PointLight2D = PointLight2D.new()
	for field: String in WARM_LIGHT_FIELDS: light.set(field, source.get(field))
	light.name = "Mux2WarmLight"
	# Only this isolated clone uses a full legal night intensity while the actual source stays daytime-disabled.
	light.enabled = true
	light.energy = 1.0
	return light

static func make_lit_warm_viewport(ids: Array[String], source: PointLight2D) -> SubViewport:
	var light: PointLight2D = clone_warm_light(source)
	if light == null: return null
	var viewport: SubViewport = make_warm_viewport(ids)
	if viewport == null:
		light.free()
		return null
	var radius: float = minf(light.texture.get_width(), light.texture.get_height()) * absf(light.texture_scale) * 0.5
	var covered: bool = Vector2(viewport.size).length() * 0.5 + light.offset.length() < radius
	covered = covered and light.range_layer_min <= 0 and light.range_layer_max >= 0 and light.range_z_min <= 0 and light.range_z_max >= 0
	covered = covered and (light.range_item_cull_mask & 1) != 0
	if not covered:
		light.free()
		viewport.free()
		return null
	light.position = Vector2(viewport.size) * 0.5
	viewport.add_child(light)
	var tone: CanvasModulate = CanvasModulate.new()
	tone.name = "Mux2WarmTone"
	tone.color = Color(1.0 - light.color.r, 1.0 - light.color.g, 1.0 - light.color.b, 1.0)
	viewport.add_child(tone)
	var copied_fields: Dictionary = {}
	for field: String in WARM_LIGHT_FIELDS: copied_fields[field] = str(light.get(field))
	viewport.set_meta("lit_warm_receipt", {"copied_fields": copied_fields, "coverage": "default_canvas_material_only", "light_count": 1,
		"source_texture_rid": source.texture.get_rid().get_id(), "clone_texture_rid": light.texture.get_rid().get_id(),
		"source_enabled": source.enabled, "source_energy": source.energy, "isolated_energy_override": light.energy,
		"shadow_support": "unshadowed_only", "viewport_fully_inside_radial_light": covered})
	return viewport

func _try_render_warmup() -> void:
	var elapsed: int = Time.get_ticks_usec() - _render_warm_wait_started_usec
	_render_warm_report["frame"] = Engine.get_process_frames()
	_render_warm_report["wait_checks"] = _render_warm_wait_checks
	_render_warm_report["wait_elapsed_usec"] = elapsed
	if elapsed >= 2000000 or _render_warm_wait_checks >= 120:
		_render_warm_pending = false
		_render_warm_report["status"] = "FAILED"
		_render_warm_report["reason"] = "runtime light unavailable within 2 seconds / 120 active checks"
		_render_warm_report["completed_frame"] = Engine.get_process_frames()
		return
	_render_warm_wait_checks += 1
	_render_warm_report["wait_checks"] = _render_warm_wait_checks
	var queued_lights: int = 0
	var source_light: PointLight2D = null
	var daylight: Node = get_node_or_null("../../DaylightController")
	if daylight != null:
		for child: Node in daylight.get_children():
			if child is PointLight2D and child.is_queued_for_deletion(): queued_lights += 1
			if child is PointLight2D and not child.is_queued_for_deletion():
				source_light = child as PointLight2D
				break
	if source_light == null:
		var reason: String = "daylight node absent" if daylight == null else ("only queued-for-deletion lights" if queued_lights > 0 else "runtime light not reconciled yet")
		_render_warm_wait_history.append({"frame": Engine.get_process_frames(), "elapsed_usec": elapsed, "reason": reason, "queued_lights": queued_lights})
		_render_warm_report["reason"] = reason
		_render_warm_report["wait_history"] = _render_warm_wait_history.duplicate(true)
		return
	_render_warm_pending = false
	_begin_render_warmup(source_light)

func _begin_render_warmup(source_light: PointLight2D) -> void:
	_cancel_render_warmup()
	_render_warm_report = {"kind": "OFFSCREEN_DRAW_ONCE", "status": "RENDER_REQUESTED", "complete": false,
		"frame": Engine.get_process_frames(), "start_usec": Time.get_ticks_usec(), "wait_started_usec": _render_warm_wait_started_usec,
		"wait_checks": _render_warm_wait_checks, "wait_elapsed_usec": Time.get_ticks_usec() - _render_warm_wait_started_usec,
		"resource_frame": _texture_warm_report.get("frame", -1), "wait_history": _render_warm_wait_history.duplicate(true)}
	if DisplayServer.get_name() == "headless" or not bool(_texture_warm_report.get("complete", false)):
		_render_warm_report["status"] = "FAILED"
		_render_warm_report["reason"] = "headless or incomplete resource preload"
		return
	var ids: Array[String] = []
	ids.assign(_texture_warm_report.get("requested_ids", []))
	_render_warm_viewport = make_lit_warm_viewport(ids, source_light)
	if _render_warm_viewport == null:
		_render_warm_report["status"] = "FAILED"
		_render_warm_report["reason"] = "missing actual light/texture, unsupported shadow/range, or bounded light/viewport coverage exceeded"
		return
	var custom: Dictionary = LitWarm.collect(get_parent(), source_light)
	var custom_report: Dictionary = LitWarm.append_to(_render_warm_viewport, custom)
	_render_warm_report["custom_materials"] = custom_report
	if not bool(custom_report.get("complete", false)):
		_render_warm_viewport.free()
		_render_warm_viewport = null
		_render_warm_report["status"] = "FAILED"
		_render_warm_report["reason"] = "custom material representative coverage incomplete"
		return
	_render_warm_report["lit_warmup"] = _render_warm_viewport.get_meta("lit_warm_receipt", {})
	_render_warm_report["requested_ids"] = ids
	_render_warm_report["sprite_count"] = ids.size() + (custom.get("states", []) as Array).size()
	_render_warm_report["viewport_size"] = [_render_warm_viewport.size.x, _render_warm_viewport.size.y]
	_render_warm_report["requested_update_mode"] = _render_warm_viewport.render_target_update_mode
	add_child(_render_warm_viewport)
	_render_warm_texture = _render_warm_viewport.get_texture()
	_finish_render_warmup(_render_warm_viewport, _render_warm_generation)

func _finish_render_warmup(viewport: SubViewport, generation: int) -> void:
	await RenderingServer.frame_post_draw
	if generation != _render_warm_generation or not is_instance_valid(viewport): return
	var draws: int = viewport.get_render_info(Viewport.RENDER_INFO_TYPE_CANVAS, Viewport.RENDER_INFO_DRAW_CALLS_IN_FRAME)
	_render_warm_report["draw_calls"] = draws
	_render_warm_report["completed_frame"] = Engine.get_process_frames()
	_render_warm_report["completed_usec"] = Time.get_ticks_usec()
	# Godot's node property retains UPDATE_ONCE even after the renderer consumes the one-shot request.
	_render_warm_report["observed_update_mode"] = viewport.render_target_update_mode
	_render_warm_report["complete"] = draws > 0 and int(_render_warm_report.get("requested_update_mode", -1)) == SubViewport.UPDATE_ONCE
	_render_warm_report["status"] = "COMPLETED" if bool(_render_warm_report["complete"]) else "FAILED"
	viewport.render_target_update_mode = SubViewport.UPDATE_DISABLED
	_render_warm_report["explicitly_disabled_mode"] = viewport.render_target_update_mode
	_render_warm_report["queued_for_deletion"] = true
	viewport.queue_free()
	_render_warm_viewport = null
	_render_warm_texture = null

func _cancel_render_warmup() -> void:
	_render_warm_pending = false
	_render_warm_generation += 1
	if is_instance_valid(_render_warm_viewport): _render_warm_viewport.queue_free()
	_render_warm_viewport = null
	_render_warm_texture = null

## Enumerate the same variant fields consumed by _rebuild, with aliases deduplicated by manifest identity.
static func texture_variant_ids(bindings: Dictionary) -> Array[String]:
	var ids: Array[String] = []
	for family: String in ["buildings", "workshops"]:
		for row: Dictionary in (bindings.get(family, {}) as Dictionary).get("codes", []):
			for field: String in ["structure", "unlit", "firing", "working"]:
				var id: Variant = row.get(field)
				if id is String and not str(id).is_empty() and not ids.has(str(id)):
					ids.append(str(id))
	ids.sort()
	return ids

## Load resources through the ordinary cache; this deliberately does not claim to issue a GPU draw.
static func warm_texture_variants(bindings: Dictionary, max_textures: int = 64, max_pixels: int = 4194304) -> Dictionary:
	var started: int = Time.get_ticks_usec()
	var ids: Array[String] = texture_variant_ids(bindings)
	var requested: Array[String] = []
	var missing: Array[String] = []
	var pixels: int = 0
	var truncated: bool = false
	for id: String in ids:
		if requested.size() >= max_textures:
			truncated = true
			break
		# Declared dimensions let larger future content respect the pixel budget before resource loading.
		var size: Variant = ArtData.row(id).get("size", [])
		var declared_pixels: int = int(size[0]) * int(size[1]) if size is Array and size.size() == 2 else 0
		if pixels + declared_pixels > max_pixels:
			truncated = true
			break
		requested.append(id)
		var texture: Texture2D = ArtData.texture(id)
		if texture == null:
			missing.append(id)
			continue
		pixels += texture.get_width() * texture.get_height()
		if pixels > max_pixels:
			truncated = true
			break
	return {"kind": "RESOURCE_LOAD_ONLY", "complete": not truncated and missing.is_empty() and requested.size() == ids.size(),
		"frame": Engine.get_process_frames(), "start_usec": started, "elapsed_usec": Time.get_ticks_usec() - started,
		"candidate_count": ids.size(), "requested_count": requested.size(), "requested_ids": requested,
		"pixels": pixels, "missing": missing, "truncated": truncated, "max_textures": max_textures, "max_pixels": max_pixels}


func _rebuild(snap: Dictionary, shops: Dictionary, settlements: Dictionary, space: Dictionary) -> void:
	var started: int = Time.get_ticks_usec() if _mux2_profile else 0
	_rebuild_body(snap, shops, settlements, space)
	if _mux2_profile: _mux2_record("rebuild", Time.get_ticks_usec() - started)


func _rebuild_body(snap: Dictionary, shops: Dictionary, settlements: Dictionary, space: Dictionary) -> void:
	var wanted: Dictionary = {}
	# Workshops (get_workshops): footprint, structure or its working sprite; the fire-pot row marks the hearth firing.
	_shops.clear()
	var firing_tiles: Dictionary = {}
	var footprints: Array[Rect2i] = []
	var kinds: PackedInt64Array = shops.get("kinds", PackedInt64Array())
	var sxs: PackedInt32Array = shops.get("xs", PackedInt32Array())
	var sys: PackedInt32Array = shops.get("ys", PackedInt32Array())
	var sws: PackedInt32Array = shops.get("ws", PackedInt32Array())
	var shs: PackedInt32Array = shops.get("hs", PackedInt32Array())
	var active: Dictionary = _active_shops(shops)
	for i: int in mini(mini(kinds.size(), sxs.size()), sys.size()):
		var rect: Rect2i = Rect2i(sxs[i], sys[i], sws[i] if i < sws.size() else 1, shs[i] if i < shs.size() else 1)
		var row: Dictionary = ArtData.binding("workshops", kinds[i])
		if bool(row.get("marks_hearth_firing", false)):
			if active.has(i):
				firing_tiles[rect.position] = true
			continue
		footprints.append(rect)
		var id: Variant = row.get("structure", null)
		if not (id is String):
			continue
		var working: bool = active.has(i) and row.get("working", null) is String
		var sprite_id: String = str(row["working"]) if working else str(id)
		_shops.append([kinds[i], rect, working])
		wanted["w%d_%d" % [rect.position.x, rect.position.y]] = [sprite_id, SiteGhost.footprint_base(rect.position, rect.size)]
	# Buildings (hearths, shelters, huts).
	_rows.clear()
	var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
	var bkinds: PackedByteArray = snap.get("kinds", PackedByteArray())
	var huts: Array[Array] = []
	for i: int in mini(bkinds.size(), mini(xs.size(), ys.size())):
		var tile: Vector2i = Vector2i(xs[i], ys[i])
		if _inside_any(tile, footprints):
			continue
		var row: Dictionary = ArtData.binding("buildings", bkinds[i])
		if row.has("ring"):
			huts.append([tile, row["ring"]])
			_rows.append([int(bkinds[i]), tile])
			continue
		var id: Variant = row.get("structure", null)
		if not (id is String):
			continue
		var sprite_id: String = str(id)
		if bool(row.get("light", false)):
			if firing_tiles.has(tile) and row.get("firing", null) is String:
				sprite_id = str(row["firing"])
			elif not _fire_lit(settlements, tile) and row.get("unlit", null) is String:
				sprite_id = str(row["unlit"])
		_rows.append([int(bkinds[i]), tile])
		wanted["b%d_%d" % [tile.x, tile.y]] = [sprite_id, View.base_point(tile)]
	# Bedrolls of the built space (2 × 1 from the bed tile by bindings space.bed).
	var bed: Dictionary = (ArtData.bindings().get("space", {}) as Dictionary).get("bed", {})
	var fp: Variant = bed.get("footprint", [1, 1])
	var bed_size: Vector2i = Vector2i(int(fp[0]), int(fp[1])) if fp is Array and (fp as Array).size() == 2 else Vector2i.ONE
	_beds.clear()
	var bxs: PackedInt32Array = space.get("bed_xs", PackedInt32Array())
	var bys: PackedInt32Array = space.get("bed_ys", PackedInt32Array())
	for i: int in mini(bxs.size(), bys.size()):
		var t: Vector2i = Vector2i(bxs[i], bys[i])
		_beds.append(t)
		# Lying flat: one pixel above the base line so a person on the tile draws over it.
		wanted["d%d_%d" % [t.x, t.y]] = [str(bed.get("structure", "")), SiteGhost.footprint_base(t, bed_size) - Vector2(0, 1)]
	_wall_count = (space.get("wall_xs", PackedInt32Array()) as PackedInt32Array).size()
	_floor_pieces = (space.get("floor_xs", PackedInt32Array()) as PackedInt32Array).size() + _beds.size() \
		+ (space.get("ford_xs", PackedInt32Array()) as PackedInt32Array).size()
	_sync(wanted)
	_build_huts(huts)


## Workshop row indices with an active bill.
func _active_shops(shops: Dictionary) -> Dictionary:
	var out: Dictionary = {}
	var bill_shops: PackedInt32Array = shops.get("bill_shops", PackedInt32Array())
	var bill_active: PackedInt32Array = shops.get("bill_active", PackedInt32Array())
	for i: int in mini(bill_shops.size(), bill_active.size()):
		if bill_active[i] != 0:
			out[bill_shops[i]] = true
	return out


## Whether the fire of the settlement whose formation tile is nearest `tile` burns (true without settlements).
func _fire_lit(settlements: Dictionary, tile: Vector2i) -> bool:
	var fxs: PackedInt32Array = settlements.get("formation_xs", PackedInt32Array())
	var fys: PackedInt32Array = settlements.get("formation_ys", PackedInt32Array())
	var lit: PackedByteArray = settlements.get("fire_lit", PackedByteArray())
	var best: int = -1
	var best_d: int = 1 << 30
	for i: int in mini(mini(fxs.size(), fys.size()), lit.size()):
		var d: int = absi(fxs[i] - tile.x) + absi(fys[i] - tile.y)
		if d < best_d:
			best_d = d
			best = i
	return best < 0 or lit[best] != 0


## Keep one sprite per wanted key ([manifest id, base point]); unchanged keys keep their node.
func _sync(wanted: Dictionary) -> void:
	for k: String in _sprites.keys():
		var s: Sprite2D = _sprites[k]
		var w: Variant = wanted.get(k, null)
		if w == null or str(s.get_meta("id", "")) != str(w[0]) or s.position != (w[1] as Vector2).round():
			if _mux2_profile:
				_mux2_sync_frame()
				_mux2_counts["sprite_frees"] += 1
			s.queue_free()
			_sprites.erase(k)
	for k: String in wanted.keys():
		if _sprites.has(k):
			continue
		var id: String = str(wanted[k][0])
		if _mux2_profile:
			_mux2_sync_frame()
			_mux2_counts["texture_requests"] += 1
			_mux2_texture_ids.append(id)
		var tex: Texture2D = ArtData.texture(id)
		if tex == null:
			continue
		if _mux2_profile: _mux2_counts["sprite_creates"] += 1
		var s: Sprite2D = Sprite2D.new()
		s.texture = tex
		s.centered = false
		s.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
		s.position = (wanted[k][1] as Vector2).round()
		s.offset = -Vector2(ArtData.anchor(id))
		s.set_meta("id", id)
		_world.add_child(s)
		_sprites[k] = s


## Finished huts: ring walls as row strips, the door as a sprite, the floor tiles drawn flat by this node.
func _build_huts(huts: Array[Array]) -> void:
	var started: int = Time.get_ticks_usec() if _mux2_profile else 0
	_build_huts_body(huts)
	if _mux2_profile: _mux2_record("huts", Time.get_ticks_usec() - started)


func _build_huts_body(huts: Array[Array]) -> void:
	for k: String in _sprites.keys():
		if k.begins_with("h"):
			(_sprites[k] as Node).queue_free()
			_sprites.erase(k)
	for d: Array in _doors.values():
		(d[0] as Node).queue_free()
	_doors.clear()
	_floor_tiles.clear()
	for hut: Array in huts:
		var centre: Vector2i = hut[0]
		var ring: Dictionary = hut[1]
		var rt: Dictionary = SiteGhost.ring_tiles(ring)
		var size: Vector2i = rt["size"]
		var origin: Vector2i = centre - Vector2i(floori((size.x - 1) / 2.0), floori((size.y - 1) / 2.0))
		var walls: Dictionary = {}
		for t: Vector2i in (rt["walls"] as Dictionary).keys():
			walls[t + origin] = rt["walls"][t]
		var doors: Dictionary = {}
		for t: Vector2i in (rt["doors"] as Dictionary).keys():
			doors[t + origin] = rt["doors"][t]
		var rows: Dictionary = SiteGhost.wall_rows(walls, doors)
		for ty: int in rows.keys():
			var r: Dictionary = rows[ty]
			var strip: Sprite2D = Sprite2D.new()
			strip.texture = ImageTexture.create_from_image(r["image"])
			strip.centered = false
			strip.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
			strip.position = Vector2(int(r["x0"]) * TILE, ty * TILE + View.BASE.y)
			strip.offset = Vector2(0, -(SiteGhost.CAP + View.BASE.y))
			_world.add_child(strip)
			_sprites["h%d_%d_%d" % [centre.x, centre.y, ty]] = strip
		for t: Vector2i in doors.keys():
			var door: Sprite2D = Sprite2D.new()
			door.centered = false
			door.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
			door.position = Vector2(t.x * TILE, t.y * TILE + View.BASE.y)
			door.offset = Vector2(0, -(SiteGhost.CAP + View.BASE.y))
			door.texture = _door_texture(str(doors[t]), false)
			_world.add_child(door)
			_doors[t] = [door, str(doors[t])]
		for t: Vector2i in rt["floors"]:
			_floor_tiles[t + origin] = str(ring.get("floor_style", ""))
	queue_redraw()


func _door_texture(style: String, open: bool) -> Texture2D:
	var key: String = "door|%s|%s" % [style, str(open)]
	if not _floor_tex.has(key):
		var started: int = Time.get_ticks_usec() if _mux2_profile else 0
		var img: Image = SiteGhost.door_image(style, open)
		_floor_tex[key] = ImageTexture.create_from_image(img) if img != null else null
		if _mux2_profile: _mux2_record("door_prepare", Time.get_ticks_usec() - started)
	return _floor_tex[key]


## Hut doors open while a person stands on them.
func _update_doors() -> void:
	var agents: Dictionary = SimReads.read(_world_sim, "get_agent_snapshot")
	var axs: PackedInt32Array = agents.get("xs", PackedInt32Array())
	var ays: PackedInt32Array = agents.get("ys", PackedInt32Array())
	var occupied: Dictionary = {}
	for i: int in mini(axs.size(), ays.size()):
		var t: Vector2i = Vector2i(axs[i], ays[i])
		if _doors.has(t):
			occupied[t] = true
	for t: Vector2i in _doors.keys():
		var d: Array = _doors[t]
		(d[0] as Sprite2D).texture = _door_texture(str(d[1]), occupied.has(t))


func _draw() -> void:
	var started: int = Time.get_ticks_usec() if _mux2_profile else 0
	_draw_body()
	if _mux2_profile: _mux2_record("draw", Time.get_ticks_usec() - started)


func _draw_body() -> void:
	for t: Vector2i in _floor_tiles.keys():
		var style: String = str(_floor_tiles[t])
		var key: String = "floor|%s|%d|%d" % [style, posmod(t.x, 4), posmod(t.y, 4)]
		if not _floor_tex.has(key):
			var started: int = Time.get_ticks_usec() if _mux2_profile else 0
			var img: Image = SiteGhost.floor_tile(style, t)
			_floor_tex[key] = ImageTexture.create_from_image(img) if img != null else null
			if _mux2_profile: _mux2_record("floor_prepare", Time.get_ticks_usec() - started)
		var tex: Texture2D = _floor_tex[key]
		if tex != null:
			draw_texture(tex, View.tile_to_world(t))


static func _inside_any(tile: Vector2i, rects: Array[Rect2i]) -> bool:
	for rect: Rect2i in rects:
		if rect.has_point(tile):
			return true
	return false


## Number of wall and door tiles of the built space (drawn by the ground layer; for tests and tools).
func wall_count() -> int:
	return _wall_count


## Number of floor-level pieces of the built space (floors, fords, bedrolls; for tests and tools).
func floor_piece_count() -> int:
	return _floor_pieces


## Number of huts drawn (for tests and tools).
func hut_count() -> int:
	var n: int = 0
	for row: Array in _rows:
		if ArtData.binding("buildings", int(row[0])).has("ring"):
			n += 1
	return n


## Number of hearths drawn (buildings whose binding gives light; for tests and tools).
func hearth_count() -> int:
	var n: int = 0
	for row: Array in _rows:
		if bool(ArtData.binding("buildings", int(row[0])).get("light", false)):
			n += 1
	return n


## Number of workshops drawn (for tests and tools).
func workshop_count() -> int:
	return _shops.size()


## The hearth's fire frame (the M-ART2 hearth has one lit frame; kept for tools): always 0.
func fire_frame() -> int:
	return 0
