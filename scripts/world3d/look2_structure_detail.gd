# look2_structure_detail.gd — crafted details accumulated into the existing wall/roof meshes.
# Purpose: deterministic stone feet, framed log-wall apertures, layered eaves and ridge lashings;
# all vertices retain the owning builder's ground/cutaway metadata and material surface.
# Bridge methods called: none.
extends RefCounted

const Builder = preload("res://scripts/world3d/mesh_builder3d.gd")
const Textures = preload("res://scripts/world3d/textures3d.gd")
const DIRS: Array[Vector2i] = [Vector2i.UP, Vector2i.RIGHT, Vector2i.DOWN, Vector2i.LEFT]


static func window_tile(tile: Vector2i, walls: Dictionary) -> bool:
	if posmod(tile.x * 11 + tile.y * 7, 4) != 0:
		return false
	var arms: Array[Vector2i] = []
	for direction: Vector2i in DIRS:
		if walls.has(tile + direction):
			if bool((walls[tile + direction] as Dictionary).get("door", false)):
				return false
			arms.append(direction)
	return arms.size() == 2 and arms[0] == -arms[1]


static func foundation(b: Builder, centre: Vector3, arms: Array[Vector2i], half: float) -> void:
	var wall_texture: float = b.texture_tile
	b.texture_tile = Textures.index("surfaces", "stone")
	b.profile = 0.0
	var stone: Color = Color(0.43, 0.42, 0.34)
	b.box(centre + Vector3(-half - 0.055, -0.05, -half - 0.055),
		centre + Vector3(half + 0.055, 0.22, half + 0.055), stone, stone.lightened(0.13))
	for direction: Vector2i in arms:
		var along: Vector3 = Vector3(direction.x, 0, direction.y)
		var across: Vector3 = Vector3(absf(along.z), 0, absf(along.x)) * (half + 0.055)
		var a: Vector3 = centre + along * half
		var e: Vector3 = centre + along * 0.49
		var lo: Vector3 = a.min(e) - across + Vector3(0, -0.04, 0)
		var hi: Vector3 = a.max(e) + across + Vector3(0, 0.19, 0)
		b.box(lo, hi, stone.darkened(0.04), stone.lightened(0.08))
	b.texture_tile = wall_texture


static func window_frame(b: Builder, centre: Vector3, axis: Vector2i, half: float) -> void:
	var wall_texture: float = b.texture_tile
	b.texture_tile = Textures.index("surfaces", "wood_dark")
	b.profile = 3.0
	var along: Vector3 = Vector3(axis.x, 0, axis.y)
	var normal: Vector3 = Vector3(axis.y, 0, axis.x)
	var trim: Color = Color(0.37, 0.24, 0.13)
	for face: float in [-1.0, 1.0]:
		var origin: Vector3 = centre + normal * face * (half + 0.025)
		for side: float in [-1.0, 1.0]:
			var foot: Vector3 = origin + along * side * 0.275 + Vector3.UP * 0.96
			b.log_prism(foot, foot + Vector3.UP * 0.9, 0.065, trim, 5)
		for y: float in [0.99, 1.82]:
			b.log_prism(origin - along * 0.34 + Vector3.UP * y,
				origin + along * 0.34 + Vector3.UP * y, 0.075, trim.lightened(0.08), 5)
		b.log_prism(origin + Vector3.UP * 1.05, origin + Vector3.UP * 1.76, 0.022, trim, 4)
	b.texture_tile = wall_texture


static func gable_heights(heights: Dictionary, room: Rect2i) -> void:
	var ridge_x: bool = room.size.x >= room.size.y
	var lo: int = 2 * (room.position.y if ridge_x else room.position.x) - 1
	var hi: int = 2 * (room.end.y if ridge_x else room.end.x) + 1
	for vertex: Vector2i in heights:
		var across: int = vertex.y if ridge_x else vertex.x
		heights[vertex] = float(mini(across - lo, hi - across))


## Gable infill shares the roof material, so the existing roof fade also exposes its room.
static func gable_ends(b: Builder, room: Rect2i, position: Callable) -> void:
	var ridge_x: bool = room.size.x >= room.size.y
	var lo: int = 2 * (room.position.y if ridge_x else room.position.x) - 1
	var hi: int = 2 * (room.end.y if ridge_x else room.end.x) + 1
	var ends: Array[int] = [2 * (room.position.x if ridge_x else room.position.y) - 1,
		2 * (room.end.x if ridge_x else room.end.y) + 1]
	var infill: Color = Color(0.48, 0.34, 0.19, 0.0)
	for end_index: int in 2:
		var along: int = ends[end_index]
		var normal: Vector3 = Vector3(1, 0, 0) if ridge_x else Vector3(0, 0, 1)
		normal *= -1.0 if end_index == 0 else 1.0
		var foot: Vector3 = position.call(Vector2i(along, lo) if ridge_x else Vector2i(lo, along))
		for across: int in range(lo, hi):
			var a: Vector3 = position.call(Vector2i(along, across) if ridge_x else Vector2i(across, along))
			var e: Vector3 = position.call(Vector2i(along, across + 1) if ridge_x else Vector2i(across + 1, along))
			b.quad_out(a, e, Vector3(e.x, foot.y, e.z), Vector3(a.x, foot.y, a.z), infill, normal)
			if across > lo and posmod(across - lo, 2) == 0:
				var first: int = b.colours.size()
				b.log_prism(Vector3(a.x, foot.y, a.z) + normal * 0.035, a + normal * 0.035, 0.05, infill.lightened(0.10), 5)
				for index: int in range(first, b.colours.size()):
					b.colours[index].a = 0.0


## Layered bundles end inside the existing roof boundary, so adjoining rooms remain disjoint.
static func eave(b: Builder, edge: PackedVector3Array, thickness: float) -> void:
	var a: Vector3 = edge[0]
	var e: Vector3 = edge[1]
	var inner_a: Vector3 = edge[2]
	var inner_e: Vector3 = edge[3]
	var count: int = 3
	for i: int in count:
		var u: float = float(i) / float(count)
		var v: float = float(i + 1) / float(count)
		var jitter: float = 0.015 + 0.012 * float(posmod(roundi(a.x * 7.0 + a.z * 11.0) + i, 3))
		var outer_a: Vector3 = a.lerp(e, u + 0.018).lerp(inner_a.lerp(inner_e, u + 0.018), jitter)
		var outer_e: Vector3 = a.lerp(e, v - 0.018).lerp(inner_a.lerp(inner_e, v - 0.018), jitter)
		var back_a: Vector3 = a.lerp(e, u).lerp(inner_a.lerp(inner_e, u), 0.4) + Vector3.UP * 0.025
		var back_e: Vector3 = a.lerp(e, v).lerp(inner_a.lerp(inner_e, v), 0.4) + Vector3.UP * 0.025
		outer_a.y += 0.035
		outer_e.y += 0.035
		var down: Vector3 = Vector3.UP * (thickness + jitter)
		var tint: Color = Color.WHITE.darkened(0.025 * float(i))
		b.quad_out(back_a, back_e, outer_e, outer_a, tint, Vector3.UP)
		b.quad_out(outer_a, outer_e, outer_e - down, outer_a - down, tint.darkened(0.18),
			(a + e - inner_a - inner_e).normalized())
		b.quad_out(back_a, outer_a, outer_a - down, back_a - down, tint.darkened(0.12), a - e)
		b.quad_out(outer_e, back_e, back_e - down, outer_e - down, tint.darkened(0.08), e - a)


## Cap only real local maxima of the original heightfield; never bridge a valley or another roof.
static func ridge_caps(b: Builder, heights: Dictionary, position: Callable) -> void:
	var timber: Color = Color(0.46, 0.32, 0.18, 0.0)
	for vertex: Vector2i in heights:
		var height: float = float(heights[vertex])
		if height <= 0.0:
			continue
		for axis: Vector2i in [Vector2i.RIGHT, Vector2i.DOWN]:
			var end: Vector2i = vertex + axis
			var cross_axis: Vector2i = Vector2i(axis.y, axis.x)
			if not heights.has(end) or not is_equal_approx(height, float(heights[end])):
				continue
			var ridge: bool = true
			for point: Vector2i in [vertex, end]:
				for side: int in [-1, 1]:
					if float(heights.get(point + cross_axis * side, height)) >= height:
						ridge = false
			if not ridge:
				continue
			b.profile = float(axis.y)
			var a: Vector3 = position.call(vertex)
			var e: Vector3 = position.call(end)
			b.log_prism(a + Vector3.UP * 0.07, e + Vector3.UP * 0.07, 0.12, Color(1.0, 0.97, 0.9), 6)
			var centre: Vector3 = (a + e) * 0.5 + Vector3.UP * 0.13
			var spread: Vector3 = Vector3(cross_axis.x, 0.35, cross_axis.y) * 0.20
			b.log_prism(centre - spread, centre + spread, 0.028, timber, 4)
