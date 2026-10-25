# Frozen clipping body before owner-classification memo; geometry helpers are unchanged production helpers.
# Bridge methods: none. Only helper qualifications differ from the frozen source.
extends RefCounted

const Layout = preload("res://scripts/world/colony/marker_layout.gd")


static func clipped_aggregate(contours: PackedVector2Array, viewport: Rect2, common: Dictionary, bodies: Dictionary, members: Array, counters: Dictionary = {}) -> PackedVector2Array:
	var member_set: Dictionary = {}
	for bits: int in members:
		member_set[bits] = true
	var result: PackedVector2Array = PackedVector2Array()
	for i: int in range(0, contours.size(), 2):
		var start: Vector2 = contours[i]
		var finish: Vector2 = contours[i + 1]
		var visible: Vector2 = Layout._segment_interval(start, finish, viewport)
		if visible.x < 0.0 or visible.y <= visible.x:
			continue
		var clipped_start: Vector2 = start.lerp(finish, visible.x)
		finish = start.lerp(finish, visible.y)
		start = clipped_start
		var obstacles: Array[Rect2] = Layout._segment_obstacles(start, finish, common)
		for rectangle: Rect2 in Layout._segment_obstacles(start, finish, bodies["index"]):
			var nonmember: bool = false
			for bits: int in bodies["owners"][rectangle]:
				nonmember = nonmember or not member_set.has(bits)
			if nonmember:
				obstacles.append(rectangle)
		var intervals: Array[Vector2] = []
		for rectangle: Rect2 in obstacles:
			if not counters.is_empty():
				counters["clip_candidates"] = int(counters.get("clip_candidates", 0)) + 1
			var interval: Vector2 = Layout._segment_interval(start, finish, rectangle)
			if interval.x >= 0.0:
				intervals.append(interval)
		intervals.sort_custom(func(a: Vector2, b: Vector2) -> bool: return a.x < b.x)
		var cursor: float = 0.0
		for interval: Vector2 in intervals:
			if interval.x > cursor:
				result.append(start.lerp(finish, cursor))
				result.append(start.lerp(finish, interval.x))
			cursor = maxf(cursor, interval.y)
		if cursor < 1.0:
			result.append(start.lerp(finish, cursor))
			result.append(finish)
	if not counters.is_empty():
		counters["emitted_segments"] = int(counters.get("emitted_segments", 0)) + (result.size() >> 1)
	return result
