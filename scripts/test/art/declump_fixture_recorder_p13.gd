extends RefCounted

const Seats = preload("res://scripts/ui/camp_seats.gd")
const Reads = preload("res://scripts/test/art/test_sim_reads_equivalence_p9.gd")
const WORLDS: Array[String] = ["W0", "W1", "W200", "W500", "W-multi"]
const REQUIRED_CALLS: int = 200
var world: Node
var file: FileAccess
var output_path: String
var world_id: int = -1
var running: bool = false
var seed: int = 0
var active: bool = false
var calls: int = 0
var records: int = 0
var block_calls: int = 0
var counts: Dictionary = {}
var observations: Array[Dictionary] = []
var unique_inputs: Dictionary = {}
var source_hashes: Dictionary = {}
var errors: Array[String] = []
var last_frame: int = -1
var build_commit: String

func begin(path: String) -> bool:
	output_path = ProjectSettings.globalize_path(path)
	if OS.has_feature("double") or FileAccess.file_exists(output_path):
		errors.append("Requires single precision and a fresh fixture output")
		return false
	DirAccess.make_dir_recursive_absolute(output_path.get_base_dir())
	file = FileAccess.open(output_path, FileAccess.WRITE)
	if file == null:
		errors.append("Cannot open fixture output")
		return false
	file.big_endian = false
	file.store_buffer("P12DCL01".to_ascii_buffer())
	file.store_32(1)
	file.store_32(0)
	for path_name: String in ["scripts/ui/camp_seats.gd", "scripts/ui/agent_renderer.gd", "scripts/ui/art_declump_p12.gd", "scripts/test/art/declump_fixture_recorder_p13.gd", "scripts/test/art/capture_declump_fixture_p13.gd", "rust/target/debug/libsim_bridge.dylib"]:
		source_hashes[path_name] = FileAccess.get_sha256("res://" + path_name)
	var git_output: Array = []
	var code: int = OS.execute("/usr/bin/git", PackedStringArray(["-C", ProjectSettings.globalize_path("res://"), "rev-parse", "HEAD"]), git_output, true)
	build_commit = "".join(git_output).strip_edges() if code == 0 else ""
	if build_commit.is_empty():
		errors.append("Missing capture HEAD")
	return errors.is_empty()

func start_block(label: String, is_running: bool, world_seed: int) -> void:
	world_id = WORLDS.find(label)
	running = is_running
	seed = world_seed
	block_calls = 0
	last_frame = -1
	active = world_id >= 0

func record(rows: Array[Dictionary], targets: Dictionary, terrain: Dictionary, blocked: Dictionary, rects: Dictionary) -> void:
	if not active or block_calls >= REQUIRED_CALLS:
		return
	var frame: int = Engine.get_process_frames()
	if frame == last_frame:
		errors.append("Duplicate renderer invocation in one observed frame")
		active = false
		return
	last_frame = frame
	var tick: int = int(Reads.read(world, "get_time_snapshot", true)["tick"])
	var population: int = Reads.read(world, "get_agent_snapshot", true)["agent_ids"].size()
	for spatial: bool in [false, true]:
		var expected: Dictionary = Seats.declump(rows, {}, targets, terrain, blocked, rects, spatial)
		var payload: PackedByteArray = _payload(rows, targets, terrain, blocked, rects, expected, spatial, tick, frame)
		file.store_32(payload.size())
		file.store_buffer(payload)
		records += 1
		var key: String = "%s/%s/%s" % [WORLDS[world_id], "running_2x" if running else "frozen", "spatial" if spatial else "brute"]
		counts[key] = int(counts.get(key, 0)) + 1
		var hash := HashingContext.new()
		hash.start(HashingContext.HASH_SHA256)
		hash.update(payload.slice(28))
		unique_inputs[key + "/" + hash.finish().hex_encode()] = true
	observations.append({"world": WORLDS[world_id], "running": running, "seed": seed, "tick": tick, "render_frame": frame, "population": population, "renderer_rows": rows.size(), "records": 2})
	calls += 1
	block_calls += 1
	if block_calls == REQUIRED_CALLS:
		active = false
		file.flush()

func _payload(rows: Array[Dictionary], targets: Dictionary, terrain: Dictionary, blocked: Dictionary, rects: Dictionary, expected: Dictionary, spatial: bool, tick: int, frame: int) -> PackedByteArray:
	var buffer := StreamPeerBuffer.new()
	buffer.big_endian = false
	buffer.put_u8(world_id)
	buffer.put_u8(int(running))
	buffer.put_u8(int(spatial))
	buffer.put_u8(0)
	buffer.put_64(seed)
	buffer.put_64(tick)
	buffer.put_u64(frame)
	buffer.put_64(int(terrain.get("width", 0)))
	buffer.put_64(int(terrain.get("height", 0)))
	buffer.put_u32(rows.size())
	for row: Dictionary in rows:
		buffer.put_64(int(row["id"]))
		buffer.put_u8(int(bool(row.get("visible", false))))
		buffer.put_u8(int(Seats._walking(row)))
		buffer.put_u8(int(bool(row.get("activity_place_known", false))))
		buffer.put_u8(0)
	buffer.put_u32(targets.size())
	for agent_id: int in targets:
		buffer.put_64(agent_id)
		var point: Vector2 = targets[agent_id]
		buffer.put_float(point.x)
		buffer.put_float(point.y)
	buffer.put_u32(rects.size())
	for agent_id: int in rects:
		buffer.put_64(agent_id)
		var rect: Rect2 = rects[agent_id]
		buffer.put_float(rect.position.x)
		buffer.put_float(rect.position.y)
		buffer.put_float(rect.size.x)
		buffer.put_float(rect.size.y)
	var cells: PackedByteArray = terrain.get("terrain", PackedByteArray())
	buffer.put_u32(cells.size())
	buffer.put_data(cells)
	buffer.put_u32(blocked.size())
	for tile: Vector2i in blocked:
		buffer.put_32(tile.x)
		buffer.put_32(tile.y)
	buffer.put_u32(expected.size())
	for agent_id: int in expected:
		buffer.put_64(agent_id)
		var offset: Vector2 = expected[agent_id]
		buffer.put_float(offset.x)
		buffer.put_float(offset.y)
	return buffer.data_array

func finish() -> bool:
	active = false
	if file != null:
		file.seek(12)
		file.store_32(records)
		file.flush()
		file.close()
	if calls < 2000 or records != calls * 2 or counts.size() != 20:
		errors.append("Incomplete five-world/two-condition/two-mode fixture")
	for key: String in counts:
		if int(counts[key]) != REQUIRED_CALLS:
			errors.append("Incomplete fixture block: " + key)
	for path_name: String in source_hashes:
		if FileAccess.get_sha256("res://" + path_name) != source_hashes[path_name]:
			errors.append("Source or dylib changed during fixture: " + path_name)
	var manifest: Dictionary = {"schema_version": 1, "magic": "P12DCL01", "format": "little endian; vector components encoded as exact IEEE754 f32 bits", "source": "real AgentRenderer._apply_drawn_positions inputs; original CampSeats.declump outputs before kernel", "real_calls": calls, "records": records, "counts": counts, "unique_input_output_records": unique_inputs.size(), "observations": observations, "source_sha256": source_hashes, "build_commit": build_commit, "godot": Engine.get_version_info(), "build_profile": OS.get_environment("P13_BUILD_PROFILE"), "release_dylib_sha256": FileAccess.get_sha256("res://rust/target/release/libsim_bridge.dylib"), "double": OS.has_feature("double"), "command": OS.get_cmdline_args(), "user_args": OS.get_cmdline_user_args(), "fixture_sha256": FileAccess.get_sha256(output_path), "errors": errors, "valid": errors.is_empty()}
	var metadata := FileAccess.open(output_path + ".json", FileAccess.WRITE)
	if metadata == null:
		return false
	metadata.store_string(JSON.stringify(manifest, "\t") + "\n")
	metadata.close()
	return errors.is_empty()
