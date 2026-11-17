# look2_structures_test.gd — CPU geometry checks for L2's live building details.
# Purpose: actual log-window apertures, door/corner exclusions, one-surface finite roof geometry,
# unchanged roof ownership/picking and cutaway ground metadata. No window or simulation is needed.
# Bridge methods called: none.
extends SceneTree

const Structures = preload("res://scripts/world3d/structures3d.gd")
const Detail = preload("res://scripts/world3d/look2_structure_detail.gd")
const Builder = preload("res://scripts/world3d/mesh_builder3d.gd")
var _failures: int = 0
var _checks: int = 0


class Ground extends Node:
	func stand_height(_tile: Vector2i) -> float:
		return 0.7


func _init() -> void:
	call_deferred("_run")


func _check(name: String, passed: bool) -> void:
	_checks += 1
	_failures += 0 if passed else 1
	print("LOOK2_STRUCTURES %s %s" % ["PASS" if passed else "FAIL", name])


func _ray_hits(vertices: PackedVector3Array, origin: Vector3, direction: Vector3) -> bool:
	for i: int in range(0, vertices.size(), 3):
		if Geometry3D.ray_intersects_triangle(origin, direction, vertices[i], vertices[i + 1], vertices[i + 2]) != null:
			return true
	return false


func _run() -> void:
	var walls: Dictionary = {Vector2i.LEFT: {"door": false}, Vector2i.RIGHT: {"door": false}}
	_check("straight_run_window", Detail.window_tile(Vector2i.ZERO, walls))
	walls[Vector2i.UP] = {"door": false}
	_check("junction_has_no_window", not Detail.window_tile(Vector2i.ZERO, walls))
	walls.erase(Vector2i.UP)
	walls[Vector2i.RIGHT] = {"door": true}
	_check("door_neighbour_has_no_window", not Detail.window_tile(Vector2i.ZERO, walls))
	var structure: Node3D = Structures.new()
	var terrain: Ground = Ground.new()
	structure.set("_terrain", terrain)
	var style: Dictionary = {"profile": "log", "colour": Color(0.6, 0.4, 0.2), "door": false}
	for axis: Vector2i in [Vector2i.RIGHT, Vector2i.DOWN]:
		structure.set("_walls", {-axis: style, axis: style, Vector2i.ZERO: style})
		var builder: RefCounted = Builder.new()
		builder.set("base", 0.7)
		structure.call("_wall", builder, Vector2i.ZERO, 0.7, 2.2, 0.35, style)
		var vertices: PackedVector3Array = builder.get("verts")
		var origin: Vector3 = Vector3(0.62, 2.1, 2.0) if axis.x == 1 else Vector3(2.0, 2.1, 0.62)
		var direction: Vector3 = Vector3.FORWARD if axis.x == 1 else Vector3.LEFT
		_check("real_aperture_%s" % axis, not _ray_hits(vertices, origin, direction))
		_check("wall_below_window_%s" % axis, _ray_hits(vertices, origin - Vector3.UP * 0.8, direction))
		var grounded: bool = true
		for uv: Vector2 in builder.get("uv2s"):
			grounded = grounded and is_equal_approx(uv.y, 0.7)
		_check("wall_cutaway_ground_%s" % axis, grounded)
	var room: Array = [Vector2i(0, 0), Vector2i(1, 0), Vector2i(2, 0), Vector2i(0, 1), Vector2i(1, 1), Vector2i(2, 1)]
	var roof: ArrayMesh = structure.call("_roof_mesh", room, 7, {}, StandardMaterial3D.new())
	var repeat: ArrayMesh = structure.call("_roof_mesh", room, 7, {}, StandardMaterial3D.new())
	_check("roof_one_draw_surface", roof.get_surface_count() == 1)
	var arrays: Array = roof.surface_get_arrays(0)
	var vertices: PackedVector3Array = arrays[Mesh.ARRAY_VERTEX]
	var normals: PackedVector3Array = arrays[Mesh.ARRAY_NORMAL]
	var finite: bool = true
	var inside: bool = true
	var grounded: bool = true
	for i: int in vertices.size():
		finite = finite and vertices[i].is_finite() and normals[i].is_finite() and normals[i].length() > 0.99
		inside = inside and vertices[i].x >= -1.001 and vertices[i].x <= 4.001 and vertices[i].z >= -1.001 and vertices[i].z <= 3.001
	for uv: Vector2 in arrays[Mesh.ARRAY_TEX_UV2]:
		grounded = grounded and is_equal_approx(uv.y, 0.7)
	_check("roof_finite_normals", finite)
	_check("roof_keeps_ownership_bounds", inside)
	_check("roof_cutaway_ground", grounded)
	_check("roof_deterministic", vertices == repeat.surface_get_arrays(0)[Mesh.ARRAY_VERTEX])
	_check("roof_bounded_geometry", vertices.size() < 24000)
	_check("picking_plane_unchanged", is_equal_approx(float((structure.get("_roof_top") as Dictionary)[7]), 0.7 + 2.2 + 0.175 * tan(deg_to_rad(38.0)) + 0.03))
	structure.free()
	terrain.free()
	print("LOOK2_STRUCTURES_DONE failures=%d checks=%d" % [_failures, _checks])
	quit(0 if _failures == 0 else 1)
