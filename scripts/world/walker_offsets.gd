extends RefCounted

const Declump = preload("res://scripts/world/declump_offsets.gd")

static func pack_inputs(rows: Array[Dictionary], positions: Dictionary, rects: Dictionary, terrain: Dictionary, blocked: Dictionary, canvas: Transform2D) -> Dictionary:
	var inputs: Dictionary = Declump.pack_inputs(rows, positions, terrain, blocked, rects)
	var direction_xy := PackedFloat32Array()
	direction_xy.resize(rows.size() * 2)
	for index: int in rows.size():
		var direction: Vector2 = rows[index].get("walking_direction", Vector2.ZERO)
		direction_xy[index * 2] = direction.x
		direction_xy[index * 2 + 1] = direction.y
	inputs["direction_xy"] = direction_xy
	inputs["canvas"] = PackedFloat32Array([canvas.x.x, canvas.x.y, canvas.y.x, canvas.y.y, canvas.origin.x, canvas.origin.y])
	return inputs

static func walker_offsets(bridge: Node, rows: Array[Dictionary], positions: Dictionary, rects: Dictionary, terrain: Dictionary, blocked: Dictionary, canvas: Transform2D, spatial: bool = true) -> Dictionary:
	if bridge == null or not bridge.has_method("art_walker_offsets"):
		push_error("walker offsets require actual art_walker_offsets bridge")
		return {}
	var inputs: Dictionary = pack_inputs(rows, positions, rects, terrain, blocked, canvas)
	var packed: Dictionary = bridge.call("art_walker_offsets", inputs["row_ids"], inputs["visible"], inputs["walking"], inputs["place_known"], inputs["direction_xy"], inputs["target_ids"], inputs["target_xy"], inputs["rect_ids"], inputs["rect_xywh"], Vector2i(inputs["width"], inputs["height"]), inputs["terrain"], inputs["blocked_xy"], inputs["canvas"], spatial)
	if packed.has("error"):
		push_error("walker offsets bridge error: " + str(packed["error"]))
		return {}
	if typeof(packed.get("ids")) != TYPE_PACKED_INT64_ARRAY or typeof(packed.get("offsets")) != TYPE_PACKED_FLOAT32_ARRAY:
		push_error("walker offsets bridge returned malformed packed output")
		return {}
	var ids: PackedInt64Array = packed["ids"]
	var offsets: PackedFloat32Array = packed["offsets"]
	if offsets.size() != ids.size() * 2:
		push_error("walker offsets bridge returned mismatched output columns")
		return {}
	var result: Dictionary = {}
	for index: int in ids.size():
		if result.has(ids[index]):
			push_error("walker offsets bridge returned duplicate output ids")
			return {}
		result[ids[index]] = Vector2(offsets[index * 2], offsets[index * 2 + 1])
	return result

static func bit_equal(reference: Dictionary, candidate: Dictionary) -> bool:
	return Declump.bit_equal(reference, candidate)
