# Selected plant phase-art contract and optional windowed capture of actual PlantLayer batches.
# Synthetic data-only renderer fixture, not a simulation capture. Bridge methods: none.
# Requires the real OpenGL renderer: dummy/headless MultiMesh transform setters are no-ops.
extends SceneTree

const Art = preload("res://scripts/world/art/art_data.gd")
const Content = preload("res://scripts/ui/content_context.gd")
const Layer = preload("res://scripts/world/colony/plant_layer.gd")
const View = preload("res://scripts/world/colony/view.gd")
var _checks: Array[Dictionary] = []
var _failed: int = 0

func _init() -> void:
	call_deferred("_run")

func _check(label: String, passed: bool) -> void:
	_checks.append({"check": label, "passed": passed})
	if not passed: _failed += 1
	print("M6A_PLANT_ART_%s %s" % ["PASS" if passed else "FAIL", label])

func _run() -> void:
	print("M6A_PLANT_ART_BACKEND display=%s method=%s" % [DisplayServer.get_name(), RenderingServer.get_current_rendering_method()])
	var fixture: Dictionary = JSON.parse_string(FileAccess.get_file_as_string("res://tools/v8/fixtures/m6a_plant_art.json"))
	var dto: Dictionary = {"schema_version": 1, "registry_revision": "m6a-plant-art-fixture", "items": [], "blueprints": [], "recipes": [], "pressures": [], "animals": [], "plants": fixture["plants"]}
	var document: Dictionary = fixture["bindings"]
	var original: Dictionary = Art.bindings().duplicate(true)
	_check("data-only plant maps admitted", Art.validate_extension(document, dto).is_empty())
	for bad_plants: Variant in [[], {}, {"codes": []}, {"by_id": {"unregistered": {"0": "missing"}}}, {"by_category": {"absent_category": {"0": "missing"}}}, {"by_id": {"fixture_orchard": []}}, {"by_id": {"fixture_orchard": {"4": "berry_bush_bare"}}}, {"by_id": {"fixture_orchard": {"2": "absent_asset"}}}, {"by_id": {"fixture_orchard": {"2": 7}}}]:
		_check("malformed plant art rejected: " + JSON.stringify(bad_plants), not Art.validate_extension({"schema_version": 1, "plants": bad_plants}, dto).is_empty())
	_check("fixture content installs", Content.install(dto, {}).is_empty())
	Art.install_extension(document)
	_check("duplicate selected maps rejected", not Art.validate_extension(document, dto).is_empty())
	for family: String in original:
		if family != "plants": _check("existing family preserved: " + family, original[family] == Art.bindings()[family])
	var species: Dictionary = fixture["plants"][0]
	var category_only: Dictionary = fixture["plants"][1]
	var category: Dictionary = document["plants"]["by_category"][species["category"]]
	_check("species ripe phase wins", Art.plant_asset(species, 2) == document["plants"]["by_id"][species["id"]]["2"])
	for phase: int in 4:
		_check("unknown species uses category phase %d" % phase, Art.plant_asset(category_only, phase) == category[str(phase)])
	_check("unbound category retains placeholder", Art.plant_asset({"id": "unknown", "category": "unknown"}, 0).is_empty())
	root.size = Vector2i(640, 240)
	var display := Node2D.new()
	root.add_child(display)
	for phase: int in 4:
		var layer := Layer.new()
		layer._rebuild({"xs": PackedInt64Array([0]), "ys": PackedInt64Array([0]), "kinds": PackedInt64Array([species["code"]]), "phases": PackedInt64Array([phase])})
		_check("one actual batch phase %d" % phase, layer.get_child_count() == 1)
		if layer.get_child_count() == 1:
			var batch: MultiMeshInstance2D = layer.get_child(0)
			var asset: String = Art.plant_asset(species, phase)
			var texture: Texture2D = Art.texture(asset)
			_check("native texture phase %d" % phase, batch.texture == texture and _native_size(batch.multimesh.mesh, texture.get_size()))
			_check("upright UV phase %d" % phase, _upright_uv(batch.multimesh.mesh, texture.get_size()))
			var expected: Vector2 = View.base_point(Vector2i.ZERO) + texture.get_size() / 2.0 - Vector2(Art.anchor(asset))
			var actual: Transform2D = batch.multimesh.get_instance_transform_2d(0)
			print("M6A_PLANT_ART_ANCHOR phase=%d actual=%s expected_origin=%s manifest_anchor=%s texture_size=%s" % [phase, actual, expected, Art.anchor(asset), texture.get_size()])
			_check("native anchor phase %d" % phase, actual.origin.is_equal_approx(expected) and actual.x.is_equal_approx(Vector2.RIGHT) and actual.y.is_equal_approx(Vector2.DOWN))
			layer.remove_child(batch)
			batch.position = Vector2(110 + phase * 140, 150) - View.base_point(Vector2i.ZERO)
			display.add_child(batch)
		layer.free()
	var args: PackedStringArray = OS.get_cmdline_user_args()
	if not args.is_empty():
		var output: String = args[0]
		_check("external capture path", output.is_absolute_path() and not output.begins_with(ProjectSettings.globalize_path("res://")))
		_check("windowed renderer", DisplayServer.get_name() != "headless")
		if _failed == 0:
			_check("capture directory", DirAccess.make_dir_recursive_absolute(output) == OK)
			await process_frame
			await RenderingServer.frame_post_draw
			var capture: Image = root.get_texture().get_image()
			_check("capture saved", capture.save_png(output.path_join("plant-art-phases.png")) == OK)
			var file := FileAccess.open(output.path_join("plant-art.json"), FileAccess.WRITE)
			file.store_string(JSON.stringify({"checks": _checks, "failures": _failed, "fixture": "synthetic renderer contract", "bindings": document, "source_hashes": {"fixture": FileAccess.get_sha256("res://tools/v8/fixtures/m6a_plant_art.json"), "art_data": FileAccess.get_sha256("res://scripts/world/art/art_data.gd"), "plant_layer": FileAccess.get_sha256("res://scripts/world/colony/plant_layer.gd"), "manifest": FileAccess.get_sha256("res://assets/v8art/manifest.json")}, "capture_sha256": FileAccess.get_sha256(output.path_join("plant-art-phases.png"))}, "\t"))
	print("M6A_PLANT_ART_DONE checks=%d failures=%d" % [_checks.size(), _failed])
	quit(1 if _failed else 0)


func _native_size(mesh: Mesh, size: Vector2) -> bool:
	var arrays: Array = mesh.surface_get_arrays(0)
	var vertices: Variant = arrays[Mesh.ARRAY_VERTEX]
	if not (vertices is PackedVector2Array or vertices is PackedVector3Array) or vertices.size() != 4:
		return false
	var bounds := Rect2(Vector2(vertices[0].x, vertices[0].y), Vector2.ZERO)
	for index: int in vertices.size():
		bounds = bounds.expand(Vector2(vertices[index].x, vertices[index].y))
	var aabb: AABB = mesh.get_aabb()
	return bounds.position.is_equal_approx(-size / 2.0) and bounds.size.is_equal_approx(size) and Vector2(aabb.size.x, aabb.size.y).is_equal_approx(size)


func _upright_uv(mesh: Mesh, size: Vector2) -> bool:
	var arrays: Array = mesh.surface_get_arrays(0)
	var vertices: Variant = arrays[Mesh.ARRAY_VERTEX]
	var uvs: PackedVector2Array = arrays[Mesh.ARRAY_TEX_UV]
	if not (vertices is PackedVector2Array or vertices is PackedVector3Array) or vertices.size() != 4 or uvs.size() != 4:
		return false
	for index: int in vertices.size():
		var position: Vector2 = Vector2(vertices[index].x, vertices[index].y)
		var expected: Vector2 = position / size + Vector2(0.5, 0.5)
		if not uvs[index].is_equal_approx(expected):
			return false
	return true
