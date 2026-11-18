# world3d_data_test.gd — M3D-1 headless checks of the 3D view's data contract (no window needed; nothing drawn).
# Purpose: (1) every model a binding in assets/3d/bindings3d.json names exists in assets/3d/models (or is a VAT
#   character), every station state list resolves (its last candidate exists; earlier w3d_ candidates may still be
#   pending), and every family has a fallback; the merged manifests give every bound model a row and the explicit
#   models/manifest.json row wins a duplicate id; a plant model drawn in winter has its colours desaturated;
#   (2) a missing model id yields the placeholder mesh, never null
#   (data-only: a new content name without a model still draws); (3) roofs of two rooms across one wall share no cell
#   and an L room's hipped height field is finite and rises from the wall line; (4) VAT characters load with sizes matching their JSON; (5) the 3D scene
#   boots headless with a real WorldSimNode and builds its terrain, walls and roofs for the default world without a
#   script error; (6) M3D-SWITCH E1: with every 2D world layer under ColonyView freed, the runtime callers still work
#   through ViewProjection, Main/RoomIndex and Main/PeopleView: a click at a person's feet opens the inspector on that
#   person (pick_world → entity bits) with a portrait, the body has a screen rect, a wild animal is picked with its
#   species in the hunt mode, rooms index and toggle, chronicle highlights ring people in 3D, the mood flag toggles;
#   (7) M3D-ASSETS coverage on the booted world's catalogue: every item code has its own model for each count bucket
#   (props3d own_item_model), every wild plant id each plant phase and every crop id each shown field phase a model
#   (plants3d wild_model / crop_model), every outcrop kind its own model (flora3d outcrop_model).
#   Prints `W3D_DATA <name> PASS|FAIL <detail>` and `W3D_DATA_DONE failures=<n>`; quits 1 on failure.
#   `Godot --headless --path . --script res://tools/v8/world3d_data_test.gd`
# Bridge methods called: via SimReads get_agent_snapshot, get_rooms, get_animals, get_content_catalog, get_crop_defs
#   (the booted main scene reads its own).
extends SceneTree

const Data3D = preload("res://scripts/world3d/data3d.gd")
const Structures = preload("res://scripts/world3d/structures3d.gd")
const Vat = preload("res://scripts/world3d/vat3d.gd")
const ViewProjection = preload("res://scripts/world/view_projection.gd")
const OrderTool = preload("res://scripts/world/colony/order_tool.gd")
const Props = preload("res://scripts/world3d/props3d.gd")
const Plants = preload("res://scripts/world3d/plants3d.gd")
const Flora = preload("res://scripts/world3d/flora3d.gd")

var _failures: int = 0


func _init() -> void:
	call_deferred("_run")


func _check(name: String, ok: bool, detail: String = "") -> void:
	print("W3D_DATA %s %s %s" % [name, "PASS" if ok else "FAIL", detail])
	if not ok:
		_failures += 1


func _models_named(node: Variant, out: Dictionary, key: String = "") -> void:
	if node is Dictionary:
		for k: Variant in (node as Dictionary):
			if str(k) in ["about", "colours", "materials", "patterns", "canopy_by_season", "species_canopy_by_season", "trunk",
					"bare_in_season", "habit", "clip_for_pose", "lit", "by_era", "by_wall_category", "by_style_walls", "states"]:
				continue
			_models_named((node as Dictionary)[k], out, str(k))
	elif node is Array:
		for v: Variant in (node as Array):
			_models_named(v, out, key)
	elif node is String and key not in ["models_dir", "fallback_habit", "habit", "depleted_tint"]:
		out[str(node)] = key


func _run() -> void:
	# 1 bindings → models.
	var bindings: Dictionary = JSON.parse_string(FileAccess.get_file_as_string("res://assets/3d/bindings3d.json"))
	var names: Dictionary = {}
	for family: String in ["flora", "resources", "nodes", "outcrops", "stations", "buildings", "items", "animals", "plants", "sites", "fences"]:
		_models_named(bindings.get(family, {}), names)
	var missing: Array[String] = []
	var rowless: Array[String] = []
	for id: String in names:
		if not Data3D.has_model(id) and not FileAccess.file_exists("res://assets/3d/vat/%s_vat.json" % id):
			missing.append(id)
		elif Data3D.has_model(id) and Data3D.model_row(id).is_empty():
			rowless.append(id)
	_check("bindings_models_exist", missing.is_empty(), "%d named, missing %s" % [names.size(), missing])
	_check("bound_models_have_rows", rowless.is_empty(), "%d without a manifest row %s" % [rowless.size(), rowless.slice(0, 8)])
	# Station states: a state list may name pending w3d_ models first; its last candidate must exist.
	var bad_states: Array[String] = []
	var states: Dictionary = (bindings.get("stations", {}) as Dictionary).get("states", {})
	var w3d_drawn: int = 0
	for base: String in states:
		if states[base] is not Dictionary:
			continue
		for state: String in ["idle", "working"]:
			var list: Array = (states[base] as Dictionary).get(state, [])
			if list.is_empty() or not Data3D.has_model(str(list[-1])):
				bad_states.append("%s.%s" % [base, state])
			elif Props.state_model(base, state == "working") == base and not list.has(base):
				bad_states.append("%s.%s unresolved" % [base, state])
			elif Props.state_model(base, state == "working").begins_with("w3d_"):
				w3d_drawn += 1
	_check("station_states_resolve", bad_states.is_empty() and not states.is_empty(), "%s; %d of %d state models are w3d_" % [bad_states, w3d_drawn, 2 * (states.size() - 1)])
	# Manifest merge: the explicit models/manifest.json row wins a duplicate id over assets_manifest.json.
	var explicit: Dictionary = (JSON.parse_string(FileAccess.get_file_as_string("res://assets/3d/models/manifest.json")) as Dictionary).get("models", {})
	var combined: Dictionary = (JSON.parse_string(FileAccess.get_file_as_string("res://assets/3d/assets_manifest.json")) as Dictionary).get("models", {})
	var wrong_row: Array[String] = []
	var only_combined: int = 0
	for id: String in combined:
		if explicit.has(id) and Data3D.model_row(id) != explicit[id]:
			wrong_row.append(id)
		elif not explicit.has(id) and not str(id).begins_with("w3d_") and Data3D.model_row(id) != combined[id]:
			wrong_row.append(id)
		only_combined += 0 if explicit.has(id) else 1
	_check("manifest_merge_order", wrong_row.is_empty() and only_combined > 0, "%d assets-only rows, wrong %s" % [only_combined, wrong_row.slice(0, 8)])
	# Winter: a plant model's materials are desaturated (saturation × the 2D winter factor).
	var plants_node: Node3D = Plants.new()
	var wild_rows: Dictionary = (bindings.get("plants", {}) as Dictionary).get("wild", {})
	var sample: String = str((wild_rows[wild_rows.keys()[0]] as Array)[1]) if not wild_rows.is_empty() else ""
	var summer: Mesh = plants_node.call("_desaturated", sample, 1.0)
	var winter: Mesh = plants_node.call("_desaturated", sample, 0.35)
	var greyer: bool = summer != null and winter != null and summer.get_surface_count() == winter.get_surface_count() and summer.get_surface_count() > 0
	for si: int in summer.get_surface_count() if greyer else 0:
		var a: Material = summer.surface_get_material(si)
		var b: Material = winter.surface_get_material(si)
		if a is BaseMaterial3D and (a as BaseMaterial3D).albedo_color.s > 0.05:
			greyer = greyer and (b as BaseMaterial3D).albedo_color.s < (a as BaseMaterial3D).albedo_color.s * 0.5
	plants_node.free()
	_check("plants_winter_desaturated", greyer, sample)
	var no_fallback: Array[String] = []
	for family: String in ["nodes", "stations", "buildings", "items", "animals", "walls", "roofs", "materials"]:
		if not (bindings.get(family, {}) as Dictionary).has("fallback"):
			no_fallback.append(family)
	_check("families_have_fallback", no_fallback.is_empty(), str(no_fallback))
	# 2 data only: an unknown model id draws the placeholder.
	var m: Mesh = Data3D.mesh("no_such_model_m3d1")
	_check("unknown_model_placeholder", m != null and m.get_surface_count() > 0)
	# 3 roofs: two rooms either side of one wall get disjoint roof cells (no roof through another); an L room's
	# heights are 0 on the wall line and rise inward.
	var room_a: Array = [Vector2i(0, 0), Vector2i(1, 0), Vector2i(0, 1), Vector2i(1, 1)]
	var room_b: Array = [Vector2i(3, 0), Vector2i(4, 0), Vector2i(3, 1), Vector2i(4, 1)]
	var claimed: Dictionary = {}
	for c: Vector2i in Structures.footprint_cells(room_a):
		claimed[c] = 1
	for c: Vector2i in Structures.footprint_cells(room_b):
		claimed[c] = 2
	var cells_a: Dictionary = Structures.roof_cells(room_a, 1, claimed)
	var cells_b: Dictionary = Structures.roof_cells(room_b, 2, claimed)
	var shared: int = 0
	for c: Vector2i in cells_a:
		if cells_b.has(c):
			shared += 1
	_check("roofs_disjoint", shared == 0 and not cells_a.is_empty(), "%d shared cells" % shared)
	var l_shape: Array = [Vector2i(0, 0), Vector2i(1, 0), Vector2i(2, 0), Vector2i(0, 1), Vector2i(0, 2)]
	var h: Dictionary = Structures.roof_heights(Structures.roof_cells(l_shape, 1, {}))
	var finite: bool = true
	var top: float = -INF
	for v: Vector2i in h:
		finite = finite and is_finite(float(h[v]))
		top = maxf(top, float(h[v]))
	_check("roof_heightfield", finite and top >= 1.0 and float(h[Vector2i(-1, -1)]) == 0.0, "max %.1f" % top)
	# NOTE 6: at most 2 hip rectangles; a small spur (< 25% of the footprint) gets a flat cap instead.
	var l_rects: Array[Rect2i] = Structures.roof_rects(l_shape)
	var spur: Array = [Vector2i(3, 1)]
	for y: int in 3:
		for x: int in 4:
			spur.append(Vector2i(x, y + 2))
	var spur_rects: Array[Rect2i] = Structures.roof_rects(spur)
	_check("roof_rects", l_rects == [Rect2i(0, 0, 3, 1), Rect2i(0, 1, 1, 2)] and spur_rects == [Rect2i(0, 2, 4, 3)], "%s / %s" % [l_rects, spur_rects])
	# 4 VAT characters.
	var bad: Array[String] = []
	for file: String in DirAccess.get_files_at("res://assets/3d/vat"):
		if file.ends_with("_vat.json"):
			var v: RefCounted = Vat.load_character(file.trim_suffix("_vat.json"))
			if not bool(v.get("ok")):
				bad.append(file)
	_check("vat_characters_load", bad.is_empty(), str(bad))
	# 5 the scene boots and builds.
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(main)
	for _i: int in 30:
		await process_frame
	var w3d: Node = main.get_node_or_null("World3D")
	_check("world3d_present", w3d != null)
	if w3d != null:
		w3d.get_node("Terrain").call("build_all")
		for _i: int in 30:
			await process_frame
		_check("terrain_complete", bool(w3d.get_node("Terrain").call("is_complete")))
		_check("colony_hidden_2d", not (main.get_node("ColonyView") as CanvasItem).visible)
		var px: Vector2 = w3d.call("screen_to_world", Vector2(960, 540))
		_check("projection_provider", px.x > 0.0 and px.y > 0.0, str(px))
		await _without_2d(main, w3d)
		_assets_coverage(main.get_node("WorldSim"))
	print("W3D_DATA_DONE failures=%d" % _failures)
	quit(1 if _failures > 0 else 0)


## (7) Every item code, plant/crop stage and outcrop kind of the live catalogue resolves to an M3D-ASSETS model.
func _assets_coverage(world: Node) -> void:
	var catalog: Dictionary = SimReads.read(world, "get_content_catalog")
	var items: Array = catalog.get("items", [])
	var missing_items: Array[String] = []
	for row: Variant in items:
		for bucket: int in 3:
			if Props.own_item_model(str((row as Dictionary).get("id", "")), bucket) == "":
				missing_items.append("%s/%d" % [(row as Dictionary).get("id", ""), bucket])
	_check("items_own_models", not items.is_empty() and missing_items.is_empty(), "%d codes × 3 buckets, fallback %s" % [items.size(), missing_items.slice(0, 8)])
	var plants: Array = catalog.get("plants", [])
	var missing_plants: Array[String] = []
	for row: Variant in plants:
		for phase: int in 4:
			if Plants.wild_model(str((row as Dictionary).get("id", "")), phase) == "":
				missing_plants.append("%s/%d" % [(row as Dictionary).get("id", ""), phase])
	_check("wild_plant_stages_models", not plants.is_empty() and missing_plants.is_empty(), "%d plants × 4 phases, fallback %s" % [plants.size(), missing_plants])
	var crops: Array = SimReads.read(world, "get_crop_defs").get("rows", [])
	var missing_crops: Array[String] = []
	for row: Variant in crops:
		for phase: int in [3, 4, 5]:
			if Plants.crop_model(str((row as Dictionary).get("id", "")), phase) == "":
				missing_crops.append("%s/%d" % [(row as Dictionary).get("id", ""), phase])
	_check("crop_stages_models", not crops.is_empty() and missing_crops.is_empty(), "%d crops × 3 field phases, fallback %s" % [crops.size(), missing_crops])
	var outcrops: Array = catalog.get("outcrops", [])
	var missing_ore: Array[String] = []
	for row: Variant in outcrops:
		if Flora.outcrop_model(str((row as Dictionary).get("id", ""))) == "":
			missing_ore.append(str((row as Dictionary).get("id", "")))
	_check("outcrop_kinds_models", not outcrops.is_empty() and missing_ore.is_empty(), "%d kinds, fallback %s" % [outcrops.size(), missing_ore])
	# C3-b: the iron-age stations draw their own models (not the wood-bench fallback) and the paved road its flagstones.
	var by_style: Dictionary = Data3D.binding("stations.by_style", {})
	var c3_unbound: Array[String] = []
	for style: String in ["bloomery", "forge", "anvil", "mason_bench"]:
		var model: String = str(by_style.get(style, ""))
		if Data3D.first_model([Props.state_model(model, false)]) == "" or Data3D.first_model([Props.state_model(model, true)]) == "":
			c3_unbound.append(style)
	if Data3D.first_model([Data3D.binding("floors.by_material", {}).get("cut_stone", "")]) == "":
		c3_unbound.append("paved_road")
	_check("c3_iron_age_models", c3_unbound.is_empty(), str(c3_unbound))


## (6) No 2D world layer exists; every runtime path goes through the renderer-neutral helpers.
func _without_2d(main: Node, w3d: Node) -> void:
	var view: Node = main.get_node("ColonyView")
	for _i: int in 10:
		await process_frame
	# M3D-SWITCH E2: the 2D layers are deleted; ColonyView only holds the order tool and the (hidden) task marker.
	var left: Array[String] = []
	for child: Node in view.get_children():
		left.append(str(child.name))
	_check("e1_2d_layers_freed", left == ["WorldMarker", "OrderTool"], str(left))
	var world: Node = main.get_node("WorldSim")
	var tool: Node = view.get_node("OrderTool")
	var px: float = float(Data3D.param("camera.px_per_tile_at_1x", 64.0))
	# A click at a person's feet opens the inspector on that person.
	var snap: Dictionary = SimReads.read(world, "get_agent_snapshot")
	var ids: PackedInt64Array = snap.get("ids", PackedInt64Array())
	var aids: PackedInt64Array = snap.get("agent_ids", PackedInt64Array())
	var bits: int = ids[0] if ids.size() > 0 else -1
	var body: Dictionary = w3d.get_node("People").call("body_of", bits)
	var feet: Vector3 = body.get("feet", Vector3.ZERO)
	var at: Vector2 = Vector2(feet.x, feet.z) * px
	var hit: Dictionary = ViewProjection.pick_world(at)
	_check("e1_pick_person", hit.get("kind", "") == "person" and int(hit.get("id", -1)) == bits, "%s want %d" % [hit, bits])
	tool.call("click_world", at, Vector2i(int(feet.x), int(feet.z)))
	await process_frame
	var inspector: Node = main.get_node("UI/AgentInspectorPanel")
	var got: int = int(inspector.call("get_selected_entity_bits"))
	_check("e1_click_inspector", bits >= 0 and got == bits, "want %d got %d" % [bits, got])
	for _i: int in 5:
		await process_frame
	var portrait: Dictionary = inspector.call("portrait_state")
	_check("e1_inspector_initial_badge", str(portrait.get("initial", "")).length() == 1 and str((inspector.get("_label_id") as Label).text).begins_with(str(portrait["initial"])), str(portrait))
	var rect: Rect2 = ViewProjection.person_rect(bits)
	_check("e1_person_rect", rect.has_area(), str(rect))
	# A wild animal in the hunt mode.
	var animals: Dictionary = SimReads.read(world, "get_animals")
	var bodies: Array = w3d.get_node("Animals").call("bodies", OrderTool.Mode.HUNT)
	if bodies.is_empty():
		_check("e1_pick_animal", (animals.get("ids", PackedInt64Array()) as PackedInt64Array).is_empty(), "no animal bodies")
	else:
		var a: Dictionary = bodies[0]
		var afeet: Vector3 = a["feet"]
		var ahit: Dictionary = ViewProjection.pick_world(Vector2(afeet.x, afeet.z) * px, OrderTool.Mode.HUNT)
		var kinds: PackedInt64Array = animals.get("kinds", PackedInt64Array())
		var row: int = (animals.get("ids", PackedInt64Array()) as PackedInt64Array).find(int(ahit.get("id", -1)))
		_check("e1_pick_animal", ahit.get("kind", "") == "animal" and row >= 0 and row < kinds.size() and int(ahit.get("species", -2)) == kinds[row],
			str(ahit))
		_check("e1_people_not_hunted", ViewProjection.pick_world(at, OrderTool.Mode.HUNT).get("kind", "") != "person")
	# Rooms: the index matches get_rooms and R's flag toggles with its signal.
	var rooms: Node = main.get_node("RoomIndex")
	var room_ids: PackedInt32Array = SimReads.read(world, "get_rooms").get("ids", PackedInt32Array())
	var heard: Array[bool] = []
	rooms.connect("toggled", func(on: bool) -> void: heard.append(on))
	rooms.call("toggle")
	_check("e1_rooms", int(rooms.call("room_count")) == room_ids.size() and bool(rooms.call("is_shown")) and heard == [true],
		"%d rooms of %d, heard %s" % [int(rooms.call("room_count")), room_ids.size(), heard])
	rooms.call("set_shown", false)
	# Chronicle highlights ring that person in 3D; the mood flag.
	var people_view: Node = main.get_node("PeopleView")
	people_view.call("set_highlights", PackedInt64Array([aids[0]]) if aids.size() > 0 else PackedInt64Array())
	await process_frame
	await process_frame
	var rings: MultiMeshInstance3D = w3d.get_node("People").get("_highlight")
	_check("e1_highlight_rings", aids.size() > 0 and rings.multimesh.instance_count == 1, "%d rings" % rings.multimesh.instance_count)
	people_view.call("clear_highlights")
	await process_frame
	_check("e1_highlight_clear", rings.multimesh.instance_count == 0, "%d rings" % rings.multimesh.instance_count)
	people_view.call("set_mood_shown", true)
	_check("e1_mood_flag", bool(people_view.call("mood_shown")))
	people_view.call("set_mood_shown", false)
	print("W3D_DATA INFO camp_framing %s visitors %s" % [main.get_node("Camera2D").call("camp_framing_report").get("status", ""),
		w3d.get_node("Visitors").call("counts")])
