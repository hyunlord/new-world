# mesh_builder3d.gd — a small triangle accumulator for the 3D view's procedural meshes (walls, roofs, fences, sites).
# Purpose: flat-shaded triangles with a vertex colour, UV (x: profile code for the wall shader, y: along-surface
#   metres) and UV2 (y: the ground height the piece stands on, which the cutaway clamps against). Godot front faces are
#   clockwise; `quad(a, b, c, d)` takes the corners clockwise as seen from the front.
# Bridge methods called: none.
extends RefCounted

var verts: PackedVector3Array = PackedVector3Array()
var normals: PackedVector3Array = PackedVector3Array()
var colours: PackedColorArray = PackedColorArray()
var uvs: PackedVector2Array = PackedVector2Array()
var uv2s: PackedVector2Array = PackedVector2Array()
## Current profile code (UV.x) and ground base (UV2.y) for the next triangles.
var profile: float = 0.0
var base: float = 0.0
## Texture atlas tile; UV2.x leaves profile and cutaway channels intact.
var texture_tile: float = -1.0


func is_empty() -> bool:
	return verts.is_empty()


func tri(a: Vector3, b: Vector3, c: Vector3, col: Color, ua: float = 0.0, ub: float = 0.0, uc: float = 0.0) -> void:
	var n: Vector3 = (c - a).cross(b - a)
	if n.length_squared() < 1e-12:
		return
	n = n.normalized()
	verts.append_array([a, b, c])
	normals.append_array([n, n, n])
	colours.append_array([col, col, col])
	uvs.append_array([Vector2(profile, ua), Vector2(profile, ub), Vector2(profile, uc)])
	uv2s.append_array([Vector2(texture_tile, base), Vector2(texture_tile, base), Vector2(texture_tile, base)])


## Quad a-b-c-d, clockwise seen from its front.
func quad(a: Vector3, b: Vector3, c: Vector3, d: Vector3, col: Color) -> void:
	tri(a, b, c, col, a.y, b.y, c.y)
	tri(a, c, d, col, a.y, c.y, d.y)


## Triangle facing `outward` (the winding is chosen so its normal points that way).
func tri_out(a: Vector3, b: Vector3, c: Vector3, col: Color, outward: Vector3) -> void:
	if (c - a).cross(b - a).dot(outward) >= 0.0:
		tri(a, b, c, col, a.y, b.y, c.y)
	else:
		tri(a, c, b, col, a.y, c.y, b.y)


## Quad a-b-c-d (in order around its edge) facing `outward`.
func quad_out(a: Vector3, b: Vector3, c: Vector3, d: Vector3, col: Color, outward: Vector3) -> void:
	if (c - a).cross(b - a).dot(outward) >= 0.0:
		quad(a, b, c, d, col)
	else:
		quad(a, d, c, b, col)


## Axis-aligned box from `lo` to `hi`; `top_col` for the top face (null-like: same as col when alpha 0).
func box(lo: Vector3, hi: Vector3, col: Color, top_col: Color = Color(0, 0, 0, 0), skip_bottom: bool = true) -> void:
	var tc: Color = col if top_col.a == 0.0 else top_col
	var p000: Vector3 = Vector3(lo.x, lo.y, lo.z)
	var p100: Vector3 = Vector3(hi.x, lo.y, lo.z)
	var p010: Vector3 = Vector3(lo.x, hi.y, lo.z)
	var p110: Vector3 = Vector3(hi.x, hi.y, lo.z)
	var p001: Vector3 = Vector3(lo.x, lo.y, hi.z)
	var p101: Vector3 = Vector3(hi.x, lo.y, hi.z)
	var p011: Vector3 = Vector3(lo.x, hi.y, hi.z)
	var p111: Vector3 = Vector3(hi.x, hi.y, hi.z)
	quad(p010, p110, p111, p011, tc)
	quad(p011, p111, p101, p001, col.darkened(0.04))
	quad(p110, p010, p000, p100, col.darkened(0.08))
	quad(p111, p110, p100, p101, col.darkened(0.02))
	quad(p010, p011, p001, p000, col.darkened(0.1))
	if not skip_bottom:
		quad(p000, p001, p101, p100, col.darkened(0.3))


## A horizontal log (an n-sided prism) from `a` to `b` with radius `r`.
func log_prism(a: Vector3, b: Vector3, r: float, col: Color, sides: int = 6) -> void:
	var axis: Vector3 = (b - a).normalized()
	var side: Vector3 = axis.cross(Vector3.UP).normalized()
	if side.length_squared() < 0.5:
		side = Vector3.RIGHT
	var up: Vector3 = side.cross(axis).normalized()
	var ring_a: Array[Vector3] = []
	var ring_b: Array[Vector3] = []
	for k: int in sides:
		var ang: float = TAU * (float(k) + 0.5) / float(sides)
		var off: Vector3 = (side * cos(ang) + up * sin(ang)) * r
		ring_a.append(a + off)
		ring_b.append(b + off)
	for k: int in sides:
		var k2: int = (k + 1) % sides
		var shade: float = 0.82 + 0.18 * clampf(((ring_a[k] - a) + (ring_a[k2] - a)).normalized().y * 0.5 + 0.5, 0.0, 1.0)
		quad(ring_a[k2], ring_b[k2], ring_b[k], ring_a[k], Color(col.r * shade, col.g * shade, col.b * shade))
	var end_col: Color = col.lightened(0.18)
	var side_tile: float = texture_tile
	texture_tile = float(preload("res://scripts/world3d/textures3d.gd").index("parts", "log_end", int(side_tile))) if side_tile >= 0.0 else side_tile
	for k: int in range(1, sides - 1):
		tri(ring_a[0], ring_a[k + 1], ring_a[k], end_col)
		tri(ring_b[0], ring_b[k], ring_b[k + 1], end_col)
	texture_tile = side_tile


## A vertical pointed stake at (x, z) from ground `y0` to `y1` (a square post with a pyramid tip).
func stake(x: float, z: float, y0: float, y1: float, half: float, col: Color) -> void:
	var tip: float = y1
	var top: float = y1 - half * 2.2
	box(Vector3(x - half, y0, z - half), Vector3(x + half, top, z + half), col)
	var c: Vector3 = Vector3(x, tip, z)
	var p0: Vector3 = Vector3(x - half, top, z - half)
	var p1: Vector3 = Vector3(x + half, top, z - half)
	var p2: Vector3 = Vector3(x + half, top, z + half)
	var p3: Vector3 = Vector3(x - half, top, z + half)
	var lc: Color = col.lightened(0.1)
	tri(p1, c, p0, lc)
	tri(p2, c, p1, lc)
	tri(p3, c, p2, lc)
	tri(p0, c, p3, lc)


func commit(material: Material) -> ArrayMesh:
	var mesh: ArrayMesh = ArrayMesh.new()
	if verts.is_empty():
		return mesh
	var arrays: Array = []
	arrays.resize(Mesh.ARRAY_MAX)
	arrays[Mesh.ARRAY_VERTEX] = verts
	arrays[Mesh.ARRAY_NORMAL] = normals
	arrays[Mesh.ARRAY_COLOR] = colours
	arrays[Mesh.ARRAY_TEX_UV] = uvs
	arrays[Mesh.ARRAY_TEX_UV2] = uv2s
	mesh.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, arrays)
	mesh.surface_set_material(0, material)
	return mesh
