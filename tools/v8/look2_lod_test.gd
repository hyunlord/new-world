# Headless merge contract: imported LODs survive flattening and texture adaptation.
# Godot --headless --path . --script res://tools/v8/look2_lod_test.gd
extends SceneTree

const Data3D = preload("res://scripts/world3d/data3d.gd")
var _failures: int = 0


func _init() -> void:
	call_deferred("_run")


func _check(label: String, ok: bool) -> void:
	print("LOOK2_LOD %s %s" % [label, "PASS" if ok else "FAIL"])
	_failures += 0 if ok else 1


func _same_levels(actual: Array, expected: Array) -> bool:
	if actual.size() != expected.size():
		return false
	for i: int in actual.size():
		if actual[i]["index_data"] != expected[i]["index_data"] or not is_equal_approx(float(actual[i]["edge_length"]), float(expected[i]["edge_length"])):
			return false
	return true


func _synthetic(vertex_count: int, with_lods: bool) -> void:
	# Given: boundary-sized indexed meshes and a nested, transformed instance.
	var vertices: PackedVector3Array = PackedVector3Array()
	vertices.resize(vertex_count)
	vertices[0] = Vector3.RIGHT
	vertices[vertex_count - 2] = Vector3.UP
	vertices[vertex_count - 1] = Vector3.FORWARD
	var indices: PackedInt32Array = PackedInt32Array([0, vertex_count - 2, vertex_count - 1])
	var arrays: Array = []
	arrays.resize(Mesh.ARRAY_MAX)
	arrays[Mesh.ARRAY_VERTEX] = vertices
	arrays[Mesh.ARRAY_INDEX] = indices + indices
	var source: ArrayMesh = ArrayMesh.new()
	var levels: Dictionary = {2.0: indices, 4.0: indices} if with_lods else {}
	source.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, arrays, [], levels)
	var mat: StandardMaterial3D = StandardMaterial3D.new()
	mat.resource_name = "lod_contract"
	source.surface_set_material(0, mat)
	var root_node: Node3D = Node3D.new()
	var parent: Node3D = Node3D.new()
	parent.position = Vector3(3, 4, 5)
	root_node.add_child(parent)
	var instance: MeshInstance3D = MeshInstance3D.new()
	instance.mesh = source
	instance.transform = Transform3D(Basis(Vector3.UP, 0.5).scaled(Vector3(2, 2, 2)), Vector3(1, 0, 0))
	parent.add_child(instance)
	var xf: Transform3D = parent.transform * instance.transform
	var before: Dictionary = RenderingServer.mesh_get_surface(source.get_rid(), 0)
	# When: the actual runtime merger flattens the scene.
	var merged: ArrayMesh = Data3D.merge_meshes(root_node)
	# Then: base geometry, LOD indices/errors, and material identity remain meaningful.
	var label: String = "%d_%s" % [vertex_count, "lod" if with_lods else "plain"]
	var after: Dictionary = RenderingServer.mesh_get_surface(merged.get_rid(), 0)
	var expected: Array = before.get("lods", []).duplicate(true)
	for level: Dictionary in expected:
		level["edge_length"] = float(level["edge_length"]) * 2.0
	_check(label + "_levels", _same_levels(after.get("lods", []), expected) and expected.size() == levels.size())
	var merged_arrays: Array = merged.surface_get_arrays(0)
	var output_vertices: PackedVector3Array = merged_arrays[Mesh.ARRAY_VERTEX]
	_check(label + "_vertices", output_vertices[0].is_equal_approx(xf * vertices[0]) and output_vertices[-1].is_equal_approx(xf * vertices[-1]))
	_check(label + "_base_indices", merged_arrays[Mesh.ARRAY_INDEX] == indices + indices)
	_check(label + "_material", Data3D.surface_key(merged, 0) == "lod_contract")
	_check(label + "_source_unchanged", RenderingServer.mesh_get_surface(source.get_rid(), 0) == before)
	root_node.free()


func _imported() -> void:
	# Given: a real imported remodelled building, including the importer-generated LODs.
	var packed: PackedScene = load("res://assets/3d/models/building_smokehouse_complete.glb")
	var root_node: Node = packed.instantiate()
	var expected: Array = []
	var stack: Array = [[root_node, Transform3D.IDENTITY]]
	var level_count: int = 0
	while not stack.is_empty():
		var entry: Array = stack.pop_back()
		var node: Node = entry[0]
		var xf: Transform3D = entry[1]
		if node is Node3D and node != root_node:
			xf = xf * (node as Node3D).transform
		if node is MeshInstance3D:
			var mesh: Mesh = (node as MeshInstance3D).mesh
			var scale: Vector3 = xf.basis.get_scale().abs()
			for s: int in mesh.get_surface_count():
				var raw: Dictionary = RenderingServer.mesh_get_surface(mesh.get_rid(), s)
				var levels: Array = raw.get("lods", []).duplicate(true)
				for level: Dictionary in levels:
					level["edge_length"] = float(level["edge_length"]) * maxf(scale.x, maxf(scale.y, scale.z))
				level_count += levels.size()
				expected.append(levels)
		for child: Node in node.get_children():
			stack.append([child, xf])
	# When: runtime loads the real building through its material adapter.
	var merged: ArrayMesh = Data3D.merge_meshes(root_node, "building_smokehouse_complete")
	# Then: every imported level survives on its matching surface.
	var equal: bool = merged.get_surface_count() == expected.size()
	for s: int in mini(merged.get_surface_count(), expected.size()):
		var raw: Dictionary = RenderingServer.mesh_get_surface(merged.get_rid(), s)
		equal = equal and _same_levels(raw.get("lods", []), expected[s])
	_check("imported_has_lods_%d" % level_count, level_count > 0)
	_check("imported_preserves_all_levels", equal)
	root_node.free()


func _run() -> void:
	_synthetic(4, true)
	_synthetic(65536, true)
	_synthetic(65537, true)
	_synthetic(4, false)
	_imported()
	print("LOOK2_LOD_DONE failures=%d" % _failures)
	quit(0 if _failures == 0 else 1)
