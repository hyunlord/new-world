extends SceneTree

# Native Godot operations only: this records the oracle before Rust is accepted.
# The output is deliberately fresh and separate from real-renderer call fixtures.
func _initialize() -> void:
	var output: String = OS.get_environment("P13B_WALKER_NUMERIC_OUT")
	if output.is_empty() or FileAccess.file_exists(output) or OS.has_feature("double"):
		push_error("Walker numeric probe requires fresh output and single-precision Godot")
		quit(2)
		return
	if DirAccess.dir_exists_absolute("/Users/rexxa/work/.mac_quiet.lock"):
		push_error("Walker numeric probe cannot run during a measurement quiet hold")
		quit(2)
		return
	var file: FileAccess = FileAccess.open(output, FileAccess.WRITE)
	if file == null:
		push_error("Cannot create walker numeric output")
		quit(2)
		return
	var minus_zero: float = _float_from_bits(0x80000000)
	var subnormal: float = _float_from_bits(1)
	var canvases: Array[Transform2D] = [
		Transform2D.IDENTITY,
		Transform2D(Vector2(2, 0), Vector2(0, 4), Vector2(10, -20)),
		Transform2D(Vector2(0.05, 0), Vector2(0, 0.04), Vector2(200, -50)),
		Transform2D(Vector2(-2, minus_zero), Vector2(0, 0.7), Vector2(minus_zero, 0)),
		Transform2D(0.37, Vector2(100, -200)),
		Transform2D(-1.13, Vector2(-9, 3)),
		Transform2D(Vector2(0.75, -0.25), Vector2(0.125, 1.5), Vector2(12.5, -6.25))]
	var directions: Array[Vector2] = [Vector2.RIGHT, Vector2.DOWN, Vector2.ONE, Vector2.ZERO,
		Vector2(minus_zero, 0), Vector2(1.0e-30, -1.0e-30), Vector2(subnormal, -subnormal),
		Vector2(-1, 2), Vector2(0.99999, -0.00001), Vector2(3.14159, -2.71828)]
	var screens: Array[Vector2] = [Vector2(0, 3), Vector2(3, 0), Vector2(0, 0.001), Vector2(0.001, 0),
		Vector2(-0.5, 0.5), Vector2(-1.5, 2.5), Vector2(minus_zero, 0), Vector2(0.49999997, -0.49999997)]
	var source_hashes: Dictionary = {}
	for path: String in ["res://scripts/test/art/walker_numeric_probe_p13b.gd", "res://scripts/ui/camp_seats.gd"]:
		source_hashes[path] = FileAccess.get_sha256(path)
	var buffer := StreamPeerBuffer.new()
	buffer.big_endian = false
	buffer.put_data("P13BNM01".to_ascii_buffer())
	buffer.put_u32(1)
	buffer.put_u32(canvases.size() * directions.size() * screens.size())
	for canvas: Transform2D in canvases:
		if canvas.determinant() == 0.0:
			push_error("Numeric probe must not introduce singular engine errors")
			file.close()
			quit(2)
			return
		for raw_direction: Vector2 in directions:
			for screen: Vector2 in screens:
				_transform(buffer, canvas)
				_point(buffer, raw_direction)
				_point(buffer, screen)
				var inverse: Transform2D = canvas.affine_inverse()
				var transformed: Vector2 = canvas.basis_xform(raw_direction)
				var direction: Vector2 = transformed.normalized()
				_transform(buffer, inverse)
				_point(buffer, direction)
				_point(buffer, transformed)
				buffer.put_float(screen.length())
				buffer.put_float(screen.dot(direction))
				buffer.put_u8(int(direction.is_zero_approx()))
				buffer.put_u8(0)
				buffer.put_u8(0)
				buffer.put_u8(0)
				_point(buffer, screen.round())
				_point(buffer, inverse.basis_xform(screen))
	file.store_buffer(buffer.data_array)
	file.close()
	var errors: Array[String] = []
	for path: String in source_hashes:
		if source_hashes[path] != FileAccess.get_sha256(path):
			errors.append("Source changed during numeric probe: " + path)
	var metadata: Dictionary = {"schema": "P13BNM01", "version": 1, "cases": canvases.size() * directions.size() * screens.size(),
		"canvas_count": canvases.size(), "direction_count": directions.size(), "screen_count": screens.size(),
		"godot": Engine.get_version_info(), "executable": OS.get_executable_path(),
		"executable_sha256": FileAccess.get_sha256(OS.get_executable_path()), "double": OS.has_feature("double"),
		"source_sha256": source_hashes, "fixture_sha256": FileAccess.get_sha256(output), "errors": errors,
		"valid": errors.is_empty(), "argv": OS.get_cmdline_args(), "user_argv": OS.get_cmdline_user_args()}
	var json_file: FileAccess = FileAccess.open(output + ".json", FileAccess.WRITE)
	if json_file == null:
		push_error("Cannot create numeric metadata")
		quit(2)
		return
	json_file.store_string(JSON.stringify(metadata, "\t") + "\n")
	json_file.close()
	print("P13B_WALKER_NUMERIC cases=%d valid=%s" % [metadata["cases"], metadata["valid"]])
	quit(0 if errors.is_empty() else 1)

func _float_from_bits(bits: int) -> float:
	var bytes := PackedByteArray()
	bytes.resize(4)
	bytes.encode_u32(0, bits)
	return bytes.decode_float(0)

func _point(buffer: StreamPeerBuffer, point: Vector2) -> void:
	buffer.put_float(point.x)
	buffer.put_float(point.y)

func _transform(buffer: StreamPeerBuffer, transform: Transform2D) -> void:
	_point(buffer, transform.x)
	_point(buffer, transform.y)
	_point(buffer, transform.origin)
