# look_terrain_test.gd — CPU mesh checks for L1 smooth terrain and shoreline interpolation.
# Bridge methods called: none. A curved heightfield exercises the actual chunk builder without a window.
extends SceneTree

const Terrain = preload("res://scripts/world3d/terrain3d.gd")

var _failures: int = 0


func _init() -> void:
	call_deferred("_run")


func _check(name: String, ok: bool, detail: String) -> void:
	print("LOOK_TERRAIN %s %s %s" % [name, "PASS" if ok else "FAIL", detail])
	if not ok:
		_failures += 1


func _run() -> void:
	# Given two chunks sharing a curved heightfield and a shallow bank.
	var terrain: Node3D = Terrain.new()
	terrain.set("width", 8)
	terrain.set("height", 4)
	terrain.set("_chunk", 4)
	terrain.set("material", ShaderMaterial.new())
	terrain.set("water_material", ShaderMaterial.new())
	var corners: PackedFloat32Array = PackedFloat32Array()
	var dip: PackedFloat32Array = PackedFloat32Array()
	var raised_corners: PackedFloat32Array = PackedFloat32Array()
	for z: int in 5:
		for x: int in 9:
			corners.append(0.05 * x * x + 0.12 * z)
			dip.append(-0.2)
			raised_corners.append(0.0)
	var raised_tiles: PackedFloat32Array = PackedFloat32Array()
	raised_tiles.resize(32)
	var codes: PackedByteArray = PackedByteArray()
	codes.resize(32)
	var colours: PackedColorArray = PackedColorArray()
	colours.resize(32)
	colours.fill(Color(0.4, 0.5, 0.3))
	terrain.set("_corner", corners)
	terrain.set("_dip", dip)
	terrain.set("_craise", raised_corners)
	terrain.set("_raise", raised_tiles)
	terrain.set("_terrain", codes)
	terrain.set("_colour", colours)
	# When both real chunk meshes are built independently.
	terrain.call("_build_chunk", Vector2i(0, 0))
	terrain.call("_build_chunk", Vector2i(1, 0))
	var meshes: Dictionary = terrain.get("_meshes")
	var expected: Vector3 = Vector3(-0.4, 1.0, -0.12).normalized()
	var seam_matches: int = 0
	var smooth: bool = true
	var geometry_kept: bool = true
	var shore_continuous: bool = true
	var wall_faces: int = 0
	var height_error: float = 0.0
	var wall_normal_error: float = 0.0
	var seam_error: float = 0.0
	for key: Vector2i in meshes:
		var mesh: Mesh = (meshes[key] as MeshInstance3D).mesh
		var arrays: Array = mesh.surface_get_arrays(0)
		var vertices: PackedVector3Array = arrays[Mesh.ARRAY_VERTEX]
		var normals: PackedVector3Array = arrays[Mesh.ARRAY_NORMAL]
		var shore: PackedVector2Array = arrays[Mesh.ARRAY_TEX_UV] if arrays[Mesh.ARRAY_TEX_UV] != null else PackedVector2Array()
		shore_continuous = shore_continuous and shore.size() == vertices.size()
		for i: int in vertices.size():
			var p: Vector3 = vertices[i]
			if normals[i].y > 0.01:
				height_error = maxf(height_error, absf(p.y - (0.05 * p.x * p.x + 0.12 * p.z - 0.2)))
				geometry_kept = geometry_kept and absf(p.y - (0.05 * p.x * p.x + 0.12 * p.z - 0.2)) < 0.00001
				if is_equal_approx(p.x, 4.0) and is_equal_approx(p.z, 2.0):
					seam_matches += 1
					seam_error = maxf(seam_error, normals[i].distance_to(expected))
					smooth = smooth and normals[i].distance_to(expected) < 0.0001
			else:
				wall_faces += 1
				wall_normal_error = maxf(wall_normal_error, absf(normals[i].y))
				# ArrayMesh packs normals: an axis-aligned face decodes with 1/65535 error.
				geometry_kept = geometry_kept and absf(normals[i].y) < 2.0 / 65535.0
			if i < shore.size():
				shore_continuous = shore_continuous and absf(shore[i].x - 0.2) < 0.00001
	# Then every duplicate seam vertex has the analytic normal; skirts and positions are untouched.
	_check("smooth_shared_chunk_normals", seam_matches >= 6 and smooth, "matches=%d max_error=%.8f" % [seam_matches, seam_error])
	_check("heightfield_and_sharp_skirts_preserved", geometry_kept and wall_faces > 0, "height_error=%.8f wall_normal_error=%.8f walls=%d" % [height_error, wall_normal_error, wall_faces])
	_check("shore_weights_shared_across_chunks", shore_continuous, "expected bed dip=0.2")
	terrain.free()
	print("LOOK_TERRAIN_DONE failures=%d" % _failures)
	quit(0 if _failures == 0 else 1)
