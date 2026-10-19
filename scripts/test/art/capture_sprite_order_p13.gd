extends RefCounted
## Godot 4.6 renderer_canvas_cull: y-sort groups, behind/body/front, then z buckets.
var errors: Array[String] = []
var _rows: Array[Dictionary] = []
var _snap: bool = false
var _group: String = ""

func collect(objects: Node2D) -> Array[Dictionary]:
	errors.clear()
	_rows.clear()
	_snap = bool(ProjectSettings.get_setting("rendering/2d/snap/snap_2d_transforms_to_pixel", false))
	if not objects.y_sort_enabled or objects.is_set_as_top_level():
		errors.append("unsupported WorldObjects sorting/top-level configuration")
		return []
	if bool(ProjectSettings.get_setting("physics/common/physics_interpolation", false)):
		errors.append("physics-interpolated ordering is unsupported")
		return []
	var parent_z: int = 0
	var parent_alpha: float = 1.0
	var ancestors: Array[CanvasItem] = []
	var ancestor: Node = objects
	while ancestor is CanvasItem:
		ancestors.push_front(ancestor as CanvasItem)
		ancestor = ancestor.get_parent()
	for item: CanvasItem in ancestors:
		parent_alpha *= item.modulate.a
		parent_z = clampi(parent_z + item.z_index, -4096, 4096) if item.z_as_relative else item.z_index
	var groups: Array[Node2D] = []
	for node: Node in objects.get_children():
		if node is Node2D:
			groups.append(node as Node2D)
		else:
			errors.append("unsupported direct world child: " + str(node.get_path()))
	groups.sort_custom(_group_before)
	for group: Node2D in groups:
		_group = str(group.get_path())
		_visit(group, parent_z, parent_alpha)
	_rows.sort_custom(func(a: Dictionary, b: Dictionary) -> bool:
		if int(a["effective_z"]) != int(b["effective_z"]):
			return int(a["effective_z"]) < int(b["effective_z"])
		return int(a["traversal_index"]) < int(b["traversal_index"]))
	for i: int in _rows.size():
		_rows[i]["draw_rank"] = i
	return _rows

func _group_before(a: Node2D, b: Node2D) -> bool:
	var ay: float = floorf(a.position.y + 0.5) if _snap else a.position.y
	var by: float = floorf(b.position.y + 0.5) if _snap else b.position.y
	return a.get_index() < b.get_index() if is_equal_approx(ay, by) else ay < by

func _visit(node: Node2D, parent_z: int, parent_alpha: float) -> void:
	if not node.is_visible_in_tree():
		return
	if node.visibility_layer != 1:
		errors.append("unsupported nondefault visibility layer: " + str(node.get_path()))
		return
	if node.y_sort_enabled or node.is_set_as_top_level() or node is CanvasGroup or node.clip_children != CanvasItem.CLIP_CHILDREN_DISABLED:
		errors.append("unsupported nested y-sort/top-level/group/clip: " + str(node.get_path()))
		return
	var alpha: float = parent_alpha * node.modulate.a
	if alpha < 0.007:
		return
	var z: int = clampi(parent_z + node.z_index, -4096, 4096) if node.z_as_relative else node.z_index
	for child: Node in node.get_children():
		if not child is Node2D:
			errors.append("unsupported world descendant: " + str(child.get_path()))
		elif (child as Node2D).show_behind_parent:
			_visit(child as Node2D, z, alpha)
	if node is Sprite2D:
		var sprite: Sprite2D = node as Sprite2D
		if sprite.texture != null and alpha * sprite.self_modulate.a > 0.0:
			var rect: Rect2 = sprite.get_global_transform_with_canvas() * sprite.get_rect()
			_rows.append({"node_path": str(sprite.get_path()), "parent_path": str(sprite.get_parent().get_path()),
				"top_y_sort_group": _group, "effective_z": z, "traversal_index": _rows.size(),
				"show_behind_parent": sprite.show_behind_parent, "entity_bits": int(sprite.get_meta("entity_bits", -1)),
				"kind": str(sprite.get_meta("art_kind", "")), "rect": [rect.position.x, rect.position.y, rect.size.x, rect.size.y]})
	elif node.get_class() != "Node2D":
		errors.append("unsupported non-sprite drawable: " + str(node.get_path()))
	for child: Node in node.get_children():
		if child is Node2D and not (child as Node2D).show_behind_parent:
			_visit(child as Node2D, z, alpha)
