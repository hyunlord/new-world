# world3d_data_test.gd — M3D-1 headless checks of the 3D view's data contract (no window needed; nothing drawn).
# Purpose: (1) every model a binding in assets/3d/bindings3d.json names exists in assets/3d/models (or is a VAT
#   character), and every family has a fallback; (2) a missing model id yields the placeholder mesh, never null
#   (data-only: a new content name without a model still draws); (3) roofs of two rooms across one wall share no cell
#   and an L room's hipped height field is finite and rises from the wall line; (4) VAT characters load with sizes matching their JSON; (5) the 3D scene
#   boots headless with a real WorldSimNode and builds its terrain, walls and roofs for the default world without a
#   script error; (6) M3D-SWITCH E1: with every 2D world layer under ColonyView freed, the runtime callers still work
#   through ViewProjection, Main/RoomIndex and Main/PeopleView: a click at a person's feet opens the inspector on that
#   person (pick_world → entity bits) with a portrait, the body has a screen rect, a wild animal is picked with its
#   species in the hunt mode, rooms index and toggle, chronicle highlights ring people in 3D, the mood flag toggles.
#   Prints `W3D_DATA <name> PASS|FAIL <detail>` and `W3D_DATA_DONE failures=<n>`; quits 1 on failure.
#   `Godot --headless --path . --script res://tools/v8/world3d_data_test.gd -- --renderer=3d`
# Bridge methods called: via SimReads get_agent_snapshot, get_rooms, get_animals (the booted main scene reads its own).
extends SceneTree

const Data3D = preload("res://scripts/world3d/data3d.gd")
const Structures = preload("res://scripts/world3d/structures3d.gd")
const Vat = preload("res://scripts/world3d/vat3d.gd")
const ViewProjection = preload("res://scripts/world/view_projection.gd")
const OrderTool = preload("res://scripts/world/colony/order_tool.gd")

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
					"bare_in_season", "habit", "clip_for_pose", "lit", "by_era", "by_wall_category", "by_style_walls"]:
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
	for family: String in ["flora", "resources", "nodes", "outcrops", "stations", "buildings", "items", "animals"]:
		_models_named(bindings.get(family, {}), names)
	var missing: Array[String] = []
	for id: String in names:
		if not Data3D.has_model(id) and not FileAccess.file_exists("res://assets/3d/vat/%s_vat.json" % id):
			missing.append(id)
	_check("bindings_models_exist", missing.is_empty(), "%d named, missing %s" % [names.size(), missing])
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
	_check("world3d_present", w3d != null, "pass --renderer=3d")
	if w3d != null:
		w3d.get_node("Terrain").call("build_all")
		for _i: int in 30:
			await process_frame
		_check("terrain_complete", bool(w3d.get_node("Terrain").call("is_complete")))
		_check("colony_hidden_2d", not (main.get_node("ColonyView") as CanvasItem).visible)
		var px: Vector2 = w3d.call("screen_to_world", Vector2(960, 540))
		_check("projection_provider", px.x > 0.0 and px.y > 0.0, str(px))
		await _without_2d(main, w3d)
	print("W3D_DATA_DONE failures=%d" % _failures)
	quit(1 if _failures > 0 else 0)


## (6) The 2D world layers freed; every runtime path goes through the renderer-neutral helpers.
func _without_2d(main: Node, w3d: Node) -> void:
	var view: Node = main.get_node("ColonyView")
	for layer_name: String in ["AgentLayer", "AnimalLayer", "VisitorLayer", "OverlayLayer", "GroundLayer", "PlantLayer",
			"TreeLayer", "BuildingLayer", "OutcropLayer", "LivestockLayer", "World"]:
		var layer: Node = view.get_node_or_null(layer_name)
		if layer != null:
			layer.queue_free()
	for _i: int in 10:
		await process_frame
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
	var hit: Dictionary = ViewProjection.pick_world(tool, at)
	_check("e1_pick_person", hit.get("kind", "") == "person" and int(hit.get("id", -1)) == bits, "%s want %d" % [hit, bits])
	tool.call("click_world", at, Vector2i(int(feet.x), int(feet.z)))
	await process_frame
	var inspector: Node = main.get_node("UI/AgentInspectorPanel")
	var got: int = int(inspector.call("get_selected_entity_bits"))
	_check("e1_click_inspector", bits >= 0 and got == bits, "want %d got %d" % [bits, got])
	for _i: int in 5:
		await process_frame
	var portrait: Dictionary = inspector.call("portrait_state")
	_check("e1_inspector_portrait", not str(portrait.get("look", "")).is_empty(), str(portrait))
	var rect: Rect2 = ViewProjection.person_rect(tool, bits)
	_check("e1_person_rect", rect.has_area() and not ViewProjection.people_lod(tool), str(rect))
	# A wild animal in the hunt mode.
	var animals: Dictionary = SimReads.read(world, "get_animals")
	var bodies: Array = w3d.get_node("Animals").call("bodies", OrderTool.Mode.HUNT)
	if bodies.is_empty():
		_check("e1_pick_animal", (animals.get("ids", PackedInt64Array()) as PackedInt64Array).is_empty(), "no animal bodies")
	else:
		var a: Dictionary = bodies[0]
		var afeet: Vector3 = a["feet"]
		var ahit: Dictionary = ViewProjection.pick_world(tool, Vector2(afeet.x, afeet.z) * px, OrderTool.Mode.HUNT)
		var kinds: PackedInt64Array = animals.get("kinds", PackedInt64Array())
		var row: int = (animals.get("ids", PackedInt64Array()) as PackedInt64Array).find(int(ahit.get("id", -1)))
		_check("e1_pick_animal", ahit.get("kind", "") == "animal" and row >= 0 and row < kinds.size() and int(ahit.get("species", -2)) == kinds[row],
			str(ahit))
		_check("e1_people_not_hunted", ViewProjection.pick_world(tool, at, OrderTool.Mode.HUNT).get("kind", "") != "person")
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
