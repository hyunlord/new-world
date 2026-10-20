# nature_canopy.gd: buckets villager head positions and fades tree sprites that would hide a villager behind them;
#   used by nature_renderer.gd.
# Bridge methods: none.
extends RefCounted

static func buckets(objects: Node2D) -> Dictionary:
	var result: Dictionary = {}
	for node: Node in objects.get_children():
		if node is not Sprite2D or not node.visible or str(node.get_meta("art_kind","")) != "agent":
			continue
		var sprite: Sprite2D = node as Sprite2D
		var feet: Vector2 = sprite.position
		var head: Vector2 = feet-Vector2(0,20)
		var bucket: Vector2i = Vector2i(floori(head.x/16.0),floori(head.y/16.0))
		if not result.has(bucket):
			result[bucket] = PackedVector2Array()
		var points: PackedVector2Array = result[bucket]
		points.append(feet)
		result[bucket] = points
	return result

static func alpha(tree: Sprite2D, heads: Dictionary) -> float:
	var rect: Rect2 = tree.get_rect()
	rect.position += tree.position
	var minimum: Vector2i = Vector2i(floori(rect.position.x/16.0),floori(rect.position.y/16.0))
	var maximum: Vector2i = Vector2i(floori(rect.end.x/16.0),floori(rect.end.y/16.0))
	for y: int in range(minimum.y,maximum.y+1):
		for x: int in range(minimum.x,maximum.x+1):
			var feet_points: PackedVector2Array = heads.get(Vector2i(x,y),PackedVector2Array())
			for feet: Vector2 in feet_points:
				if tree.position.y > feet.y and rect.has_point(feet-Vector2(0,20)):
					return 0.45
	return 1.0
