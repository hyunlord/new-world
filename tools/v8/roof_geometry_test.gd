# roof_geometry_test.gd — actual roof meshes: clean ridges, shared building roofs and season parity.
# Bridge methods called: none; synthetic room facts exercise the presentation builder.
extends SceneTree

const Structures = preload("res://scripts/world3d/structures3d.gd")

class FlatTerrain extends Node:
	func stand_height(_tile: Vector2i) -> float:
		return 0.0

var failures: int = 0

func _init() -> void:
	call_deferred("_run")

func _check(label: String, ok: bool) -> void:
	print("W3D_ROOF %s %s" % [label, "PASS" if ok else "FAIL"])
	if not ok:
		failures += 1

# Count connected raised local-maxima plateaus from the actual emitted upper surface vertices.
func _peaks(mesh: ArrayMesh) -> int:
	var heights: Dictionary = {}
	for v: Vector3 in mesh.surface_get_arrays(0)[Mesh.ARRAY_VERTEX]:
		var tile: Vector2i = Vector2i(roundi(v.x * 2), roundi(v.z * 2))
		heights[tile] = maxf(float(heights.get(tile, -INF)), v.y)
	var floor_height: float = float(heights.values().min()) + 1.0
	var peaks: Dictionary = {}
	for tile: Vector2i in heights:
		var peak: bool = float(heights[tile]) > floor_height
		for dy: int in range(-1, 2):
			for dx: int in range(-1, 2):
				peak = peak and float(heights.get(tile + Vector2i(dx, dy), -INF)) <= float(heights[tile]) + 0.0001
		if peak:
			peaks[tile] = true
	var count: int = 0
	while not peaks.is_empty():
		count += 1
		var queue: Array = [peaks.keys()[0]]
		peaks.erase(queue[0])
		while not queue.is_empty():
			var tile: Vector2i = queue.pop_back()
			for dy: int in range(-1, 2):
				for dx: int in range(-1, 2):
					var near: Vector2i = tile + Vector2i(dx, dy)
					if peaks.has(near):
						peaks.erase(near)
						queue.append(near)
	return count

func _run() -> void:
	var terrain: FlatTerrain = FlatTerrain.new()
	root.add_child(terrain)
	var structures: Node3D = Structures.new()
	root.add_child(structures)
	structures.set("_terrain", terrain)
	var tiles: Array = []
	for y: int in 4:
		for x: int in 12:
			tiles.append(Vector2i(x, y))
	var summer: ArrayMesh = structures.call("_roof_mesh", tiles, 1, {}, ShaderMaterial.new())
	var vertices: PackedVector3Array = summer.surface_get_arrays(0)[Mesh.ARRAY_VERTEX]
	# The centre ridge of a long rectangle must be one plateau, with no descending/rising teeth.
	var heights: Dictionary = {}
	for v: Vector3 in vertices:
		if is_equal_approx(v.z, 2.0):
			heights[v.x] = maxf(float(heights.get(v.x, -INF)), v.y)
	var xs: Array = heights.keys()
	xs.sort()
	var falling: bool = false
	var clean: bool = true
	for i: int in range(1, xs.size()):
		var delta: float = float(heights[xs[i]]) - float(heights[xs[i - 1]])
		if delta < -0.0001:
			falling = true
		if falling and delta > 0.0001:
			clean = false
	_check("long_actual_mesh_single_ridge", clean and xs.size() > 12)
	var normals: PackedVector3Array = summer.surface_get_arrays(0)[Mesh.ARRAY_NORMAL]
	var planar: bool = true
	for n: Vector3 in normals:
		if n.y > 0.01 and absf(n.x) > 0.01 and absf(n.z) > 0.01:
			planar = false
	_check("actual_hip_faces_have_one_slope_axis", planar)
	RenderingServer.global_shader_parameter_set("w3d_snow", 1.0)
	var winter: ArrayMesh = structures.call("_roof_mesh", tiles, 1, {}, ShaderMaterial.new())
	_check("winter_actual_vertices_identical", vertices == winter.surface_get_arrays(0)[Mesh.ARRAY_VERTEX])
	RenderingServer.global_shader_parameter_set("w3d_snow", 0.0)
	var l_shape: Array = []
	for y: int in 8:
		for x: int in 3 if y >= 3 else 8:
			l_shape.append(Vector2i(x, y))
	var real_room: Array = []
	for x: int in range(96, 100):
		for y: int in range(101, 105):
			if x != 96 or y != 104:
				real_room.append(Vector2i(x, y))
	var spur: Array = tiles.duplicate()
	spur.append(Vector2i(11, -1))
	for shape: Array in [l_shape, real_room, spur]:
		var dry: ArrayMesh = structures.call("_roof_mesh", shape, 1, {}, ShaderMaterial.new())
		RenderingServer.global_shader_parameter_set("w3d_snow", 1.0)
		var snow: ArrayMesh = structures.call("_roof_mesh", shape, 1, {}, ShaderMaterial.new())
		var count: int = _peaks(dry)
		_check("irregular_actual_mesh_peak_budget_%d" % shape.size(), count > 0 and count <= 2)
		_check("irregular_season_parity_%d" % shape.size(), dry.surface_get_arrays(0)[Mesh.ARRAY_VERTEX] == snow.surface_get_arrays(0)[Mesh.ARRAY_VERTEX])
		RenderingServer.global_shader_parameter_set("w3d_snow", 0.0)
	var rooms: Dictionary = {"ids": PackedInt32Array([1, 2]), "roofed": PackedByteArray([1, 1]), "dug": PackedByteArray([0, 0]), "tile_xs": PackedInt32Array([0, 1, 3, 4, 0, 1, 3, 4]), "tile_ys": PackedInt32Array([0, 0, 0, 0, 1, 1, 1, 1]), "tile_rooms": PackedInt32Array([1, 1, 2, 2, 1, 1, 2, 2])}
	structures.set("_walls", {Vector2i(2, 0): {"category": "Wood"}, Vector2i(2, 1): {"category": "Wood"}, "_bkey": 0})
	structures.call("_rebuild_roofs", rooms, [{"x": -1, "y": -1, "width": 7, "height": 4, "status": "complete"}])
	_check("adjacent_rooms_share_building_roof", structures.call("roof_count") == 1)
	var joined_roof: MeshInstance3D = (structures.get("_roofs") as Dictionary)[1]
	var joined_vertices: PackedVector3Array = joined_roof.mesh.surface_get_arrays(0)[Mesh.ARRAY_VERTEX]
	var ridge: Dictionary = {}
	for v: Vector3 in joined_vertices:
		if is_equal_approx(v.z, 1.0):
			ridge[v.x] = maxf(float(ridge.get(v.x, -INF)), v.y)
	_check("partition_tiles_join_footprint", is_equal_approx(float(ridge[2.5]), float(ridge[1.5])))
	_check("selected_room_resolves_shared_roof", (structures.get("_room_roof") as Dictionary).get(2) == 1)
	structures.call("_rebuild_roofs", rooms, [{"x": -1, "y": -1, "width": 7, "height": 4, "status": "cancelled"}])
	_check("cancelled_site_keeps_separate_roofs", structures.call("roof_count") == 2)
	structures.call("_rebuild_roofs", rooms, [])
	_check("unrelated_rooms_keep_separate_roofs", structures.call("roof_count") == 2)
	structures.free()
	terrain.free()
	print("W3D_ROOF_DONE failures=%d " % failures)
	quit(1 if failures else 0)
