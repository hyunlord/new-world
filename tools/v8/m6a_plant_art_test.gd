# Selected plant phase-art contract and the actual 3D plant batches (World3D/Plants, plants3d.gd; M3D-SWITCH E2 replaced
#   the 2D PlantLayer checks: native texture, anchor and UV per phase became tuft count, phase colour, seed heads and the
#   tile anchor per phase, and crop heads in their category's ripe colour).
# Synthetic data-only renderer fixture, not a simulation capture. Bridge methods: none (a stand-in world answers
#   get_world_info and get_crop_defs; the terrain stand-in is flat).
# Requires the real OpenGL renderer: dummy/headless MultiMesh transform and colour setters are no-ops.
extends SceneTree

const Art = preload("res://scripts/world/art/art_data.gd")
const Content = preload("res://scripts/ui/content_context.gd")
const Plants = preload("res://scripts/world3d/plants3d.gd")
const Data3D = preload("res://scripts/world3d/data3d.gd")
const COLOUR_EPS: float = 0.004
var _checks: Array[Dictionary] = []
var _failed: int = 0


## get_world_info and get_crop_defs for plants3d (SimReads calls a world without a SimReads sibling directly).
class FakeWorld extends Node:
	@warning_ignore("unused_signal")
	signal world_reset
	var crops: Array = []

	func get_world_info() -> Dictionary:
		return {"seed": 0}

	func get_crop_defs() -> Dictionary:
		return {"rows": crops}


## A flat map: plants3d reads only `height_at` while it fills its batches.
class FlatTerrain extends Node:
	func height_at(_x: float, _z: float) -> float:
		return 0.0


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
	# The real 3D plant batches on a stand-in world and a flat map.
	root.size = Vector2i(640, 240)
	var world: FakeWorld = FakeWorld.new()
	var species: Dictionary = fixture["plants"][0]
	world.crops = [{"code": int(species["code"]), "category": str(species["category"])}]
	var terrain: FlatTerrain = FlatTerrain.new()
	var host: Node3D = Node3D.new()
	root.add_child(host)
	host.add_child(world)
	host.add_child(terrain)
	var plants: Node3D = Plants.new()
	host.add_child(plants)
	plants.call("setup", world, terrain)
	var style: Dictionary = Art.params().get("plants", {})
	var phase_cols: Array = style.get("phase_colors", [])
	var accent: Color = Color(str(style.get("ripe_accent", "c6a26a")))
	var tile: Vector2i = Vector2i(3, 5)
	for phase: int in 4:
		plants.call("_rebuild", {"xs": PackedInt64Array([tile.x]), "ys": PackedInt64Array([tile.y]), "kinds": PackedInt64Array([species["code"]]), "phases": PackedInt64Array([phase])}, [] as Array[Dictionary], 0)
		var tufts: MultiMesh = (plants.get_node("Tufts") as MultiMeshInstance3D).multimesh
		var heads: MultiMesh = (plants.get_node("Heads") as MultiMeshInstance3D).multimesh
		_check("one actual batch phase %d" % phase, tufts.instance_count == 1)
		if tufts.instance_count != 1:
			continue
		var at: Transform3D = tufts.get_instance_transform(0)
		print("M6A_PLANT_ART_ANCHOR phase=%d origin=%s scale=%s colour=%s" % [phase, at.origin, at.basis.get_scale(), tufts.get_instance_color(0)])
		_check("phase colour phase %d" % phase, _same(tufts.get_instance_color(0), Color(str(phase_cols[phase]))))
		_check("tile anchor phase %d" % phase, Vector2(at.origin.x, at.origin.z).distance_to(Vector2(tile) + Vector2(0.5, 0.5)) <= 0.2 * sqrt(2.0) + 0.001 and is_zero_approx(at.origin.y) and at.basis.get_scale().y > 0.0)
		var want_heads: int = 1 if phase == 2 else 0
		_check("seed heads phase %d" % phase, heads.instance_count == want_heads and (want_heads == 0 or _same(heads.get_instance_color(0), accent)))
	# Winter desaturates a wild plant by `season.winter_saturation`.
	var winter: float = float((Art.params().get("season", {}) as Dictionary).get("winter_saturation", 0.35))
	plants.call("_rebuild", {"xs": PackedInt64Array([tile.x]), "ys": PackedInt64Array([tile.y]), "kinds": PackedInt64Array([species["code"]]), "phases": PackedInt64Array([1])}, [] as Array[Dictionary], 3)
	var summer: Color = Color(str(phase_cols[1]))
	_check("winter desaturates the phase colour", _same((plants.get_node("Tufts") as MultiMeshInstance3D).multimesh.get_instance_color(0), Color.from_hsv(summer.h, summer.s * winter, summer.v)))
	# Crops: 3 × 3 tufts per field tile for growing, ripe and harvested; ripe heads in the crop category's colour.
	var plant_params: Dictionary = Data3D.param("plants", {})
	var by_category: Dictionary = plant_params.get("crop_ripe_by_category", {})
	var saved: Dictionary = by_category.duplicate()
	var ripe_heads: Dictionary = {}
	for bound: bool in [false, true]:
		if bound:
			by_category[str(species["category"])] = [0.2, 0.3, 0.9]
		var ripe: Color = Data3D.colour(by_category.get(str(species["category"]), Data3D.param("plants.ripe", [])), Color(0.78, 0.64, 0.3))
		for phase: int in [3, 4, 5]:
			plants.call("_rebuild", {}, [{"x": 2, "y": 2, "phase": phase, "crop": int(species["code"])}] as Array[Dictionary], 0)
			var tufts: MultiMesh = (plants.get_node("Tufts") as MultiMeshInstance3D).multimesh
			var heads: MultiMesh = (plants.get_node("Heads") as MultiMeshInstance3D).multimesh
			var heads_ok: bool = heads.instance_count == (9 if phase == 4 else 0) and (phase != 4 or _same(heads.get_instance_color(0), ripe))
			if phase == 4:
				ripe_heads[bound] = heads.instance_count == 9 and _same(heads.get_instance_color(0), ripe)
			_check("crop tufts and heads phase %d%s" % [phase, " (category bound)" if bound else ""], tufts.instance_count == 9 and heads_ok)
	plant_params["crop_ripe_by_category"] = saved
	_check("category ripe colour wins", bool(ripe_heads.get(true, false)))
	_check("unbound category uses the ripe fallback", bool(ripe_heads.get(false, false)) and not saved.has(str(species["category"])))
	host.queue_free()
	print("M6A_PLANT_ART_DONE checks=%d failures=%d" % [_checks.size(), _failed])
	quit(1 if _failed else 0)


static func _same(a: Color, b: Color) -> bool:
	return absf(a.r - b.r) < COLOUR_EPS and absf(a.g - b.g) < COLOUR_EPS and absf(a.b - b.b) < COLOUR_EPS
