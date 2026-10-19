extends SceneTree

func _vbits(value: Vector2) -> Array[int]:
	var bytes: PackedByteArray = PackedFloat32Array([value.x, value.y]).to_byte_array()
	return [bytes.decode_u32(0), bytes.decode_u32(4)]

func _rbits(value: Rect2) -> Dictionary:
	return {"position": _vbits(value.position), "size": _vbits(value.size), "area_f64_le": PackedFloat64Array([value.get_area()]).to_byte_array().hex_encode()}

func _from_bits(bits: int) -> float:
	var bytes := PackedByteArray()
	bytes.resize(4)
	bytes.encode_u32(0, bits)
	return bytes.decode_float(0)

func _initialize() -> void:
	var output: String = OS.get_environment("P13_NUMERIC_OUT")
	if output.is_empty():
		quit(2)
		return
	var result: Dictionary = {"engine": Engine.get_version_info(), "double": OS.has_feature("double"), "rounding": [], "distance": [], "rectangles": [], "tile_conversion": []}
	for value: float in [-2.5, -1.5, -0.5, _from_bits(2147483648), _from_bits(0), 0.5, 1.5, 2.5]:
		var vector := Vector2(value, value)
		result["rounding"].append({"input": _vbits(vector), "rounded": _vbits(vector.round())})
	for pair: Array in [[Vector2(0, 0), Vector2(3, 4)], [Vector2(100000.125, 1.23456789), Vector2(-30000.75, -8.7654321)], [Vector2(-0.0, 0.0), Vector2(0.0, -0.0)], [Vector2(1e20, 1e20), Vector2.ZERO]]:
		var a: Vector2 = pair[0]
		var b: Vector2 = pair[1]
		result["distance"].append({"a": _vbits(a), "b": _vbits(b), "squared_f64_le": PackedFloat64Array([a.distance_squared_to(b)]).to_byte_array().hex_encode()})
	var negative := Rect2(4, 5, -3, -2)
	result["negative_size_abs"] = {"input": _rbits(negative), "abs": _rbits(negative.abs())}
	for pair: Array in [[Rect2(1, 2, 3, 4), Rect2(2, 3, 4, 5)], [Rect2(0, 0, 2, 2), Rect2(2, 0, 2, 2)], [Rect2(0, 0, 2, 2), Rect2(3, 0, 2, 2)], [Rect2(-0.0, -0.0, -0.0, 2), Rect2(0, 0, 1, 1)], [Rect2(100000.125, 1.23456789, 17.1234567, 23.9876543), Rect2(100001, 2, 20, 30)]]:
		var a: Rect2 = pair[0]
		var b: Rect2 = pair[1]
		result["rectangles"].append({"a": _rbits(a), "b": _rbits(b), "intersection": _rbits(a.intersection(b)), "abs_a": _rbits(a.abs())})
	for value: float in [-2.5, -1.5, -0.5, 0.5, 1.5, 2.5]:
		var point := Vector2(456 + value * 16, 42 + value * 16)
		var tile := Vector2i(((point - Vector2(456, 42)) / 16.0).round())
		result["tile_conversion"].append({"point": _vbits(point), "tile": [tile.x, tile.y]})
	var file := FileAccess.open(output, FileAccess.WRITE)
	if file == null:
		quit(2)
		return
	file.store_string(JSON.stringify(result, "\t") + "\n")
	quit(0)
