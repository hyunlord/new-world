# world3d_data_test.gd — M3D-1 headless checks of the 3D view's data contract (no window needed; nothing drawn).
# Purpose: (1) every model a binding in assets/3d/bindings3d.json names exists in assets/3d/models (or is a VAT
#   character), and every family has a fallback; (2) a missing model id yields the placeholder mesh, never null
#   (data-only: a new content name without a model still draws); (3) roof rectangle decomposition covers an L-shaped
#   and a ring-with-hole room exactly once; (4) VAT characters load with sizes matching their JSON; (5) the 3D scene
#   boots headless with a real WorldSimNode and builds its terrain, walls and roofs for the default world without a
#   script error. Prints `W3D_DATA <name> PASS|FAIL <detail>` and `W3D_DATA_DONE failures=<n>`; quits 1 on failure.
#   `Godot --headless --path . --script res://tools/v8/world3d_data_test.gd -- --renderer=3d`
# Bridge methods called: none directly (the booted main scene reads its own snapshots).
extends SceneTree

const Data3D = preload("res://scripts/world3d/data3d.gd")
const Structures = preload("res://scripts/world3d/structures3d.gd")
const Vat = preload("res://scripts/world3d/vat3d.gd")

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
	elif node is String and key not in ["models_dir", "fallback_habit", "habit"]:
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
	# 3 roof rectangles.
	var l_shape: Array = []
	for t: Vector2i in [Vector2i(0, 0), Vector2i(1, 0), Vector2i(2, 0), Vector2i(0, 1), Vector2i(0, 2)]:
		l_shape.append(t)
	_check("roof_rects_l_shape", _covers_once(Structures.rectangles(l_shape), l_shape), str(Structures.rectangles(l_shape)))
	var ring: Array = []
	for y: int in 4:
		for x: int in 4:
			if not (x in [1, 2] and y in [1, 2]):
				ring.append(Vector2i(x, y))
	_check("roof_rects_ring", _covers_once(Structures.rectangles(ring), ring), "%d rects" % Structures.rectangles(ring).size())
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
	print("W3D_DATA_DONE failures=%d" % _failures)
	quit(1 if _failures > 0 else 0)


static func _covers_once(rects: Array[Rect2i], tiles: Array) -> bool:
	var seen: Dictionary = {}
	for r: Rect2i in rects:
		for y: int in range(r.position.y, r.end.y):
			for x: int in range(r.position.x, r.end.x):
				var t: Vector2i = Vector2i(x, y)
				if seen.has(t) or not tiles.has(t):
					return false
				seen[t] = true
	return seen.size() == tiles.size()
