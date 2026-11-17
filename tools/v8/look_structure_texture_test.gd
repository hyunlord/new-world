# look_structure_texture_test.gd — real procedural wall/roof atlas bindings and batching.
# Bridge methods called: none. Flat terrain isolates material metadata from simulation state.
extends SceneTree

const Structures = preload("res://scripts/world3d/structures3d.gd")
const Textures = preload("res://scripts/world3d/textures3d.gd")
const Builder = preload("res://scripts/world3d/mesh_builder3d.gd")

class FlatTerrain extends Node3D:

	func stand_height(_tile: Vector2i) -> float:
		return 1.25


var _failures: int = 0


func _init() -> void:
	call_deferred("_run")


func _check(label: String, ok: bool) -> void:
	print("LOOK_STRUCTURE_TEXTURE %s %s" % [label, "PASS" if ok else "FAIL"])
	if not ok:
		_failures += 1


func _bound(mat: ShaderMaterial) -> bool:
	return mat.get_shader_parameter("material_atlas") is Texture2D and float(mat.get_shader_parameter("texture_strength")) > 0.0


func _run() -> void:
	var structures: Node3D = Structures.new()
	root.add_child(structures)
	var terrain: FlatTerrain = FlatTerrain.new()
	structures.set("_terrain", terrain)
	var walls: Dictionary = {}
	for fixture: Array in [[0, "log", "Wood"], [2, "drystone", "Stone"], [4, "mud_brick", "Mineral"]]:
		walls[Vector2i(fixture[0], 0)] = structures.call("_wall_entry", fixture[1], -1, fixture[2], false)
	structures.set("_walls", walls)
	structures.call("_build_wall_chunk", Vector2i.ZERO, {Vector2i(6, 0): Color(0.4, 0.3, 0.2)})
	var chunks: Dictionary = structures.get("_chunks")
	var wall_mesh: Mesh = (chunks[Vector2i.ZERO] as MeshInstance3D).mesh
	_check("mixed_styles_keep_one_batch", chunks.size() == 1 and wall_mesh.get_surface_count() == 1)
	var wall_mat: ShaderMaterial = wall_mesh.surface_get_material(0) as ShaderMaterial
	_check("real_wall_atlas_bound", _bound(wall_mat))
	var metadata: PackedVector2Array = wall_mesh.surface_get_arrays(0)[Mesh.ARRAY_TEX_UV2]
	var seen: Dictionary = {}
	var bases_kept: bool = true
	for uv: Vector2 in metadata:
		seen[int(roundf(uv.x))] = true
		bases_kept = bases_kept and is_equal_approx(uv.y, 1.25)
	_check("resolved_styles_reach_batched_vertices", seen.has(14) and seen.has(21) and seen.has(20))
	_check("unbound_floor_does_not_inherit_wall_tile", seen.has(-1))
	_check("cutaway_ground_base_preserved", bases_kept)
	var plain: Builder = Builder.new()
	plain.box(Vector3.ZERO, Vector3.ONE, Color.WHITE)
	_check("unbound_builder_defaults_neutral", plain.uv2s[0].x == -1.0)
	var means: PackedFloat32Array = wall_mat.get_shader_parameter("material_luminance")
	var configured: Array = Textures.bindings().get("linear_luminance_means", [])
	var means_match: bool = means.size() == 64 and configured.size() == 40
	for i: int in mini(configured.size(), means.size()):
		means_match = means_match and is_equal_approx(means[i], float(configured[i]))
	_check("measured_linear_means_uploaded", means_match)
	# Exercise the production roof selector and material creation for all three current covers.
	for fixture: Array in [["paleolithic", 24], ["bronze", 17], ["", 26]]:
		var sites: Array = []
		if str(fixture[0]) != "":
			sites.append({"x": 0, "y": 0, "width": 2, "height": 3, "era_id": fixture[0]})
		structures.call("_rebuild_roofs", {
			"ids": PackedInt32Array([7]), "roofed": PackedByteArray([1]),
			"tile_xs": PackedInt32Array([0, 1]), "tile_ys": PackedInt32Array([1, 1]),
			"tile_rooms": PackedInt32Array([7, 7]),
		}, sites)
		var roofs: Dictionary = structures.get("_roofs")
		var roof_mesh: Mesh = (roofs[7] as MeshInstance3D).mesh
		var roof_mat: ShaderMaterial = roof_mesh.surface_get_material(0) as ShaderMaterial
		_check("real_roof_atlas_and_cover_%d" % int(fixture[1]), _bound(roof_mat) and int(roof_mat.get_shader_parameter("texture_tile")) == int(fixture[1]))
	structures.free()
	terrain.free()
	print("LOOK_STRUCTURE_TEXTURE_DONE failures=%d" % _failures)
	quit(0 if _failures == 0 else 1)
