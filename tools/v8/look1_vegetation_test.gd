# L1 vegetation: smooth canopy seams and snapshot-driven seasonal groundcover.
# Bridge methods: none; the fixture supplies stable world/crop metadata and a flat terrain.
# Headless checks cover CPU geometry/counts only, not GPU colour, wind, fade or visual quality.
extends SceneTree

const Flora = preload("res://scripts/world3d/flora3d.gd")
const Plants = preload("res://scripts/world3d/plants3d.gd")
const Data3D = preload("res://scripts/world3d/data3d.gd")
var _failures: int = 0


class FixtureWorld extends Node:
	@warning_ignore("unused_signal")
	signal world_reset

	func get_world_info() -> Dictionary:
		return {"seed": 0}

	func get_crop_defs() -> Dictionary:
		return {"rows": []}


class FlatTerrain extends Node:
	func height_at(_x: float, _z: float) -> float:
		return 0.0


class Host extends Node3D:
	func catalog() -> Dictionary:
		return {"plants": [{"code": 4123, "id": "look1_fixture_plant"}]}


func _init() -> void:
	call_deferred("_run")


func _check(label: String, passed: bool) -> void:
	print("LOOK1_VEGETATION_%s %s" % ["PASS" if passed else "FAIL", label])
	if not passed:
		_failures += 1


func _run() -> void:
	# Given two flat-shaded faces sharing a position, smoothing joins only that seam.
	var vertices: PackedVector3Array = PackedVector3Array([Vector3.ZERO, Vector3.RIGHT, Vector3.ZERO, Vector3.UP])
	var normals: PackedVector3Array = PackedVector3Array([Vector3.UP, Vector3.UP, Vector3.RIGHT, Vector3.RIGHT])
	var smooth: PackedVector3Array = Flora.smooth_canopy_normals(vertices, normals)
	_check("coincident leaf normals agree", smooth[0].is_equal_approx(smooth[2]) and smooth[0].is_equal_approx(Vector3(1, 1, 0).normalized()))
	_check("unshared normals retained", smooth[1].is_equal_approx(normals[1]) and smooth[3].is_equal_approx(normals[3]))
	_check("source arrays unchanged", normals[0] == Vector3.UP and normals[2] == Vector3.RIGHT)
	_check("empty canopy accepted", Flora.smooth_canopy_normals(PackedVector3Array(), PackedVector3Array()).is_empty())
	var source: StandardMaterial3D = StandardMaterial3D.new()
	source.cull_mode = BaseMaterial3D.CULL_DISABLED
	source.albedo_color = Color(0.2, 0.4, 0.1)
	source.roughness = 0.85
	var painted: ShaderMaterial = Plants._plant_material(source) as ShaderMaterial
	_check("opaque plant colour and roughness preserved", painted != null and painted.get_shader_parameter("albedo") == source.albedo_color and is_equal_approx(float(painted.get_shader_parameter("roughness")), source.roughness))
	source.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	_check("transparent source retained", Plants._plant_material(source) == source)
	source.transparency = BaseMaterial3D.TRANSPARENCY_DISABLED
	source.emission_enabled = true
	_check("emissive source retained", Plants._plant_material(source) == source)
	source.emission_enabled = false
	source.cull_mode = BaseMaterial3D.CULL_FRONT
	_check("authored culling retained", Plants._plant_material(source) == source)
	var host: Host = Host.new()
	root.add_child(host)
	var world: FixtureWorld = FixtureWorld.new()
	var terrain: FlatTerrain = FlatTerrain.new()
	host.add_child(world)
	host.add_child(terrain)
	var plants: Node3D = Plants.new()
	host.add_child(plants)
	plants.set_process(false)
	var primary_arrays: Array = (plants.get_node("Tufts") as MultiMeshInstance3D).multimesh.mesh.surface_get_arrays(0)
	var meadow_arrays: Array = (plants.get_node("MeadowGrass") as MultiMeshInstance3D).multimesh.mesh.surface_get_arrays(0)
	var primary_colours: PackedColorArray = primary_arrays[Mesh.ARRAY_COLOR]
	var meadow_colours: PackedColorArray = meadow_arrays[Mesh.ARRAY_COLOR]
	_check("meadow roots lifted without changing primary tufts", meadow_colours[0].r > primary_colours[0].r and absf(primary_colours[0].r - 0.55) < 0.005 and meadow_colours[2] == Color.WHITE)
	var primary_vertices: PackedVector3Array = primary_arrays[Mesh.ARRAY_VERTEX]
	var meadow_vertices: PackedVector3Array = meadow_arrays[Mesh.ARRAY_VERTEX]
	_check("meadow retains seven blade triangles", primary_vertices.size() == 21 and meadow_vertices.size() == primary_vertices.size())
	var widened: bool = meadow_vertices.size() == primary_vertices.size()
	var primary_radius: float = 0.0
	var meadow_radius: float = 0.0
	for i: int in mini(primary_vertices.size(), meadow_vertices.size()):
		widened = widened and meadow_vertices[i].is_equal_approx(primary_vertices[i] * Vector3(2.0, 1.0, 2.0))
		primary_radius = maxf(primary_radius, Vector2(primary_vertices[i].x, primary_vertices[i].z).length())
		meadow_radius = maxf(meadow_radius, Vector2(meadow_vertices[i].x, meadow_vertices[i].z).length())
	_check("detail widens only horizontally; primary width retained", widened and is_equal_approx(primary_radius, 0.26))
	_check("wider clumps remain inside their snapshot tile", 0.32 + meadow_radius * 0.26 < 0.5)
	var source_mesh: ArrayMesh = ArrayMesh.new()
	var source_arrays: Array = []
	source_arrays.resize(Mesh.ARRAY_MAX)
	source_arrays[Mesh.ARRAY_VERTEX] = PackedVector3Array([Vector3.ZERO, Vector3.RIGHT, Vector3.FORWARD, Vector3.ZERO, Vector3.UP, Vector3.FORWARD])
	source_arrays[Mesh.ARRAY_NORMAL] = PackedVector3Array([Vector3.UP, Vector3.UP, Vector3.UP, Vector3.RIGHT, Vector3.RIGHT, Vector3.RIGHT])
	for surface_name: String in ["canopy", "trunk"]:
		source_mesh.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, source_arrays)
		source_mesh.surface_set_name(source_mesh.get_surface_count() - 1, surface_name)
	var original_normals: PackedVector3Array = source_mesh.surface_get_arrays(0)[Mesh.ARRAY_NORMAL]
	var variants: Dictionary = plants.get("_variants")
	variants["look1_leaf_fixture|1.000"] = source_mesh
	var smooth_mesh: ArrayMesh = plants.call("_painted", "look1_leaf_fixture", 1.0) as ArrayMesh
	var leaf_normals: PackedVector3Array = smooth_mesh.surface_get_arrays(0)[Mesh.ARRAY_NORMAL]
	_check("plant leaf seams smooth", leaf_normals[0].is_equal_approx(leaf_normals[3]) and leaf_normals[0].distance_to(Vector3(1, 1, 0).normalized()) < 0.001)
	var trunk_normals: PackedVector3Array = smooth_mesh.surface_get_arrays(1)[Mesh.ARRAY_NORMAL]
	var trunk_retained: bool = true
	for i: int in original_normals.size():
		trunk_retained = trunk_retained and trunk_normals[i].distance_to(original_normals[i]) < 0.001
	_check("plant source and nonleaf normals retained", source_mesh.surface_get_arrays(0)[Mesh.ARRAY_NORMAL] == original_normals and trunk_retained)
	_check("plant smoothing retains geometry", smooth_mesh.get_surface_count() == source_mesh.get_surface_count() and smooth_mesh.surface_get_arrays(0)[Mesh.ARRAY_VERTEX] == source_arrays[Mesh.ARRAY_VERTEX])
	plants.call("setup", world, terrain)
	var xs: PackedInt64Array = PackedInt64Array()
	var ys: PackedInt64Array = PackedInt64Array()
	var kinds: PackedInt64Array = PackedInt64Array()
	var phases: PackedInt64Array = PackedInt64Array()
	for i: int in 50:
		xs.append(i)
		ys.append(2)
		kinds.append(4123)
		phases.append(2)
	var snapshot: Dictionary = {"xs": xs, "ys": ys, "kinds": kinds, "phases": phases}
	plants.call("_rebuild", snapshot, [] as Array[Dictionary], 1)
	var grass: MultiMeshInstance3D = plants.get_node("MeadowGrass") as MultiMeshInstance3D
	var flowers: MultiMeshInstance3D = plants.get_node("MeadowFlowers") as MultiMeshInstance3D
	var expected: int = 50 * maxi(0, int(Data3D.param("plants.detail_grass_per_plant", 3)))
	_check("grass bounded to snapshot tiles", _detail_count(grass) == expected)
	_check("detail batches cull by chunk", grass.multimesh.instance_count == 0 and grass.get_child_count() > 1)
	var detail_material: ShaderMaterial = grass.multimesh.mesh.surface_get_material(0) as ShaderMaterial
	var fade_start: float = float(detail_material.get_shader_parameter("fade_start"))
	var fade_end: float = float(detail_material.get_shader_parameter("fade_end"))
	var wide_distance: float = 1080.0 / (2.0 * 64.0 * 0.5 * tan(deg_to_rad(30.0) * 0.5))
	var wide_coverage: float = 1.0 - smoothstep(fade_start, fade_end, wide_distance)
	_check("detail fade retains limited wide-view coverage", is_equal_approx(fade_start, 48.0) and is_equal_approx(fade_end, 70.0) and wide_coverage > 0.20 and wide_coverage < 0.30)
	var ranges: bool = true
	for group: Node in [grass, flowers]:
		for child: Node in group.get_children():
			ranges = ranges and is_equal_approx((child as MultiMeshInstance3D).visibility_range_end, fade_end)
	_check("geometry culls at detail fade end", ranges)
	var flower_count: int = _detail_count(flowers)
	_check("sparse flowers on ripe wild plants", flower_count > 0 and flower_count < 50)
	plants.call("_rebuild", snapshot, [] as Array[Dictionary], 1)
	_check("repeat snapshot stable", _detail_count(grass) == expected and _detail_count(flowers) == flower_count)
	plants.call("_rebuild", snapshot, [] as Array[Dictionary], 3)
	_check("winter removes meadow detail", _detail_count(grass) == 0 and _detail_count(flowers) == 0)
	_check("winter retains primary plants", (plants.get_node("Tufts") as MultiMeshInstance3D).multimesh.instance_count == 50)
	plants.call("_rebuild", {}, [] as Array[Dictionary], 1)
	_check("empty snapshot clears all detail", _detail_count(grass) == 0 and _detail_count(flowers) == 0 and (plants.get_node("Tufts") as MultiMeshInstance3D).multimesh.instance_count == 0)
	await process_frame
	host.free()
	print("LOOK1_VEGETATION_DONE failures=%d" % _failures)
	quit(0 if _failures == 0 else 1)


func _detail_count(group: Node) -> int:
	var count: int = 0
	for child: Node in group.get_children():
		count += (child as MultiMeshInstance3D).multimesh.instance_count
	return count
