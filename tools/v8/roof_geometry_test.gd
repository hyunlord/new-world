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

# The roof sheet is white, upward-facing and on the half-metre lattice. Crafted trim is off-lattice
# or separately tinted; keep it in season-parity checks, but not in the underlying roof-plane checks.
func _sheet_vertices(mesh: ArrayMesh) -> PackedVector3Array:
	var arrays: Array = mesh.surface_get_arrays(0)
	var vertices: PackedVector3Array = arrays[Mesh.ARRAY_VERTEX]
	var normals: PackedVector3Array = arrays[Mesh.ARRAY_NORMAL]
	var colours: PackedColorArray = arrays[Mesh.ARRAY_COLOR]
	var sheet: PackedVector3Array = PackedVector3Array()
	for i: int in range(0, vertices.size(), 3):
		var selected: bool = normals[i].y > 0.01
		for j: int in 3:
			var v: Vector3 = vertices[i + j]
			selected = selected and colours[i + j].is_equal_approx(Color.WHITE) and is_equal_approx(v.x * 2.0, roundf(v.x * 2.0)) and is_equal_approx(v.z * 2.0, roundf(v.z * 2.0))
		if selected:
			sheet.append_array([vertices[i], vertices[i + 1], vertices[i + 2]])
	return sheet


func _check_sheet(mesh: ArrayMesh, tiles: Array, label: String) -> void:
	var sheet: PackedVector3Array = _sheet_vertices(mesh)
	var cells: Dictionary = Structures.roof_cells(tiles, 1, {})
	var triangles: Dictionary = {}
	var complete: bool = sheet.size() > 0 and sheet.size() == cells.size() * 6
	for i: int in range(0, sheet.size(), 3):
		var a: Vector3 = sheet[i]
		var b: Vector3 = sheet[i + 1]
		var c: Vector3 = sheet[i + 2]
		var cell: Vector2i = Vector2i(roundi(minf(a.x, minf(b.x, c.x)) * 2.0), roundi(minf(a.z, minf(b.z, c.z)) * 2.0))
		complete = complete and cells.has(cell) and is_equal_approx(absf((b - a).cross(c - a).y), 0.25)
		if not triangles.has(cell):
			triangles[cell] = []
		(triangles[cell] as Array).append([a, b, c])
	var high_diagonal: bool = true
	for cell: Vector2i in cells:
		var pair: Array = triangles.get(cell, [])
		complete = complete and pair.size() == 2
		if pair.size() != 2:
			continue
		var corners: Dictionary = {}
		for tri: Array in pair:
			for v: Vector3 in tri:
				corners[Vector2i(roundi(v.x * 2.0), roundi(v.z * 2.0)) - cell] = v.y
		complete = complete and corners.size() == 4
		if corners.size() != 4:
			continue
		var difference: float = float(corners[Vector2i.ZERO]) + float(corners[Vector2i.ONE]) - float(corners[Vector2i.RIGHT]) - float(corners[Vector2i.DOWN])
		if not is_zero_approx(difference):
			var raised: Array = [Vector2i.ZERO, Vector2i.ONE] if difference > 0.0 else [Vector2i.RIGHT, Vector2i.DOWN]
			for tri: Array in pair:
				var hits: int = 0
				for v: Vector3 in tri:
					hits += 1 if Vector2i(roundi(v.x * 2.0), roundi(v.z * 2.0)) - cell in raised else 0
				high_diagonal = high_diagonal and hits == 2
	_check(label + "_sheet_complete", complete)
	_check(label + "_raised_diagonal", high_diagonal)


# Count connected raised local-maxima plateaus from the actual emitted roof sheet vertices.
func _peaks(mesh: ArrayMesh) -> int:
	var heights: Dictionary = {}
	for v: Vector3 in _sheet_vertices(mesh):
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
	var sheet: PackedVector3Array = _sheet_vertices(summer)
	_check_sheet(summer, tiles, "long")
	# The centre ridge of a long rectangle must be one plateau, with no descending/rising teeth.
	var heights: Dictionary = {}
	for v: Vector3 in sheet:
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
	var planar: bool = true
	for i: int in range(0, sheet.size(), 3):
		var n: Vector3 = (sheet[i + 2] - sheet[i]).cross(sheet[i + 1] - sheet[i]).normalized()
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
		_check_sheet(dry, shape, "irregular_%d" % shape.size())
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
	var joined_vertices: PackedVector3Array = _sheet_vertices(joined_roof.mesh as ArrayMesh)
	var ridge: Dictionary = {}
	for v: Vector3 in joined_vertices:
		if is_equal_approx(v.z, 1.0):
			ridge[v.x] = maxf(float(ridge.get(v.x, -INF)), v.y)
	_check("partition_tiles_join_footprint", is_equal_approx(float(ridge[2.5]), float(ridge[1.5])))
	_check("selected_room_resolves_shared_roof", (structures.get("_room_roof") as Dictionary).get(2) == 1)
	structures.set("_roof_fade", {1: 0.25})
	_check("shared_room_reads_owner_fade", is_equal_approx(float(structures.call("roof_fade", 2)), 0.25))
	_check("shared_hidden_roof_not_pickable", is_inf(float(structures.call("roof_hit", Vector3(3.5, 10.0, 1.0), Vector3.DOWN))))
	structures.set("_roof_fade", {1: 1.0})
	_check("shared_visible_roof_pickable", is_finite(float(structures.call("roof_hit", Vector3(3.5, 10.0, 1.0), Vector3.DOWN))))
	structures.call("_rebuild_roofs", rooms, [{"x": -1, "y": -1, "width": 7, "height": 4, "status": "cancelled"}])
	_check("cancelled_site_keeps_separate_roofs", structures.call("roof_count") == 2)
	structures.call("_rebuild_roofs", rooms, [])
	_check("unrelated_rooms_keep_separate_roofs", structures.call("roof_count") == 2)
	structures.free()
	terrain.free()
	print("W3D_ROOF_DONE failures=%d " % failures)
	quit(1 if failures else 0)
