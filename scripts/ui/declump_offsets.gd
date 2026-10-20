extends RefCounted

const Seats = preload("res://scripts/ui/camp_seats.gd")

static func pack_inputs(rows: Array[Dictionary], targets: Dictionary, terrain: Dictionary, blocked: Dictionary, rects: Dictionary) -> Dictionary:
	var row_ids := PackedInt64Array()
	var visible := PackedByteArray()
	var walking := PackedByteArray()
	var place_known := PackedByteArray()
	row_ids.resize(rows.size())
	visible.resize(rows.size())
	walking.resize(rows.size())
	place_known.resize(rows.size())
	for index: int in rows.size():
		var row: Dictionary = rows[index]
		row_ids[index] = int(row["id"])
		visible[index] = int(bool(row.get("visible", false)))
		walking[index] = int(Seats._walking(row))
		place_known[index] = int(bool(row.get("activity_place_known", false)))
	var target_ids := PackedInt64Array(targets.keys())
	var target_xy := PackedFloat32Array()
	target_xy.resize(target_ids.size() * 2)
	for index: int in target_ids.size():
		var point: Vector2 = targets[target_ids[index]]
		target_xy[index * 2] = point.x
		target_xy[index * 2 + 1] = point.y
	var rect_ids := PackedInt64Array(rects.keys())
	var rect_xywh := PackedFloat32Array()
	rect_xywh.resize(rect_ids.size() * 4)
	for index: int in rect_ids.size():
		var rect: Rect2 = rects[rect_ids[index]]
		rect_xywh[index * 4] = rect.position.x
		rect_xywh[index * 4 + 1] = rect.position.y
		rect_xywh[index * 4 + 2] = rect.size.x
		rect_xywh[index * 4 + 3] = rect.size.y
	var blocked_xy := PackedInt32Array()
	blocked_xy.resize(blocked.size() * 2)
	var blocked_index: int = 0
	for tile: Vector2i in blocked:
		blocked_xy[blocked_index] = tile.x
		blocked_xy[blocked_index + 1] = tile.y
		blocked_index += 2
	return {"row_ids": row_ids, "visible": visible, "walking": walking, "place_known": place_known, "target_ids": target_ids, "target_xy": target_xy, "rect_ids": rect_ids, "rect_xywh": rect_xywh, "width": int(terrain.get("width", 0)), "height": int(terrain.get("height", 0)), "terrain": terrain.get("terrain", PackedByteArray()), "blocked_xy": blocked_xy}

static func declump(bridge: Node, rows: Array[Dictionary], targets: Dictionary, terrain: Dictionary, blocked: Dictionary, rects: Dictionary, spatial: bool = true) -> Dictionary:
	if bridge == null or not bridge.has_method("art_declump"):
		push_error("declump requires actual art_declump bridge")
		return {}
	var inputs: Dictionary = pack_inputs(rows, targets, terrain, blocked, rects)
	var packed: Dictionary = bridge.call("art_declump", inputs["row_ids"], inputs["visible"], inputs["walking"], inputs["place_known"], inputs["target_ids"], inputs["target_xy"], inputs["rect_ids"], inputs["rect_xywh"], inputs["width"], inputs["height"], inputs["terrain"], inputs["blocked_xy"], spatial)
	if typeof(packed.get("ids")) != TYPE_PACKED_INT64_ARRAY or typeof(packed.get("offsets")) != TYPE_PACKED_FLOAT32_ARRAY:
		push_error("declump bridge returned malformed packed output")
		return {}
	var ids: PackedInt64Array = packed["ids"]
	var offsets: PackedFloat32Array = packed["offsets"]
	if offsets.size() != ids.size() * 2:
		push_error("declump bridge returned mismatched output columns")
		return {}
	var result: Dictionary = {}
	for index: int in ids.size():
		if result.has(ids[index]):
			push_error("declump bridge returned duplicate output ids")
			return {}
		result[ids[index]] = Vector2(offsets[index * 2], offsets[index * 2 + 1])
	return result

static func bit_equal(reference: Dictionary, candidate: Dictionary) -> bool:
	if reference.keys() != candidate.keys():
		return false
	for agent_id: int in reference:
		var expected: Vector2 = reference[agent_id]
		var actual: Vector2 = candidate[agent_id]
		if PackedFloat32Array([expected.x, expected.y]).to_byte_array() != PackedFloat32Array([actual.x, actual.y]).to_byte_array():
			return false
	return true
