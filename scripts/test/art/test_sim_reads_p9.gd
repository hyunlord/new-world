extends SceneTree

const READER = preload("res://scripts/ui/sim_reads.gd")
var _failures: int = 0

class FakeWorld extends Node:
	signal world_reset
	var tick: int = 3
	var calls: int = 0
	func get_time_snapshot() -> Dictionary:
		calls += 1
		return {"tick": tick, "nested": [{"value": tick}]}
	func advance_ticks(count: int) -> int:
		tick += count
		return count

func _initialize() -> void:
	call_deferred("_run")

func _check(condition: bool, message: String) -> void:
	if not condition:
		_failures += 1
		push_error("SIM READS P9: " + message)

func _fixture(world: Node) -> Node:
	var container := Node.new()
	world.name = "WorldSim"
	container.add_child(world)
	var reader := READER.new()
	reader.name = "SimReads"
	container.add_child(reader)
	return container

func _run() -> void:
	_test_fake_and_identity()
	_test_recursive_freeze()
	var world: Node = ClassDB.instantiate("WorldSimNode")
	var container: Node = _fixture(world)
	var reader: SimReads = container.get_node("SimReads") as SimReads
	_check(READER.resolve(world) == reader, "Detached sibling resolves by identity")
	root.add_child(container)
	world.set_process(false)
	world.call("set_sim_speed", 0.0)
	_check(reader.reset_world(20261010), "Reset returns native success")
	await process_frame
	await process_frame
	var first: Dictionary = reader.time_snapshot()
	var second: Dictionary = READER.read(world, "get_time_snapshot")
	_check(is_same(first, second), "Same-frame reads share dictionary identity")
	_check(first.is_read_only(), "Real FFI snapshot is readonly")
	_check(reader.frame_call_counts().get("get_time_snapshot", 0) == 1, "Cache hit costs no FFI call")
	_check(reader.frame_call_usec().has("get_time_snapshot"), "FFI wall time recorded")
	_check(reader.frame_id() == Engine.get_process_frames(), "Counter frame matches engine frame")
	var detached_counts: Dictionary = reader.frame_call_counts()
	detached_counts.clear()
	_check(reader.frame_call_counts().get("get_time_snapshot", 0) == 1, "Counter copies cannot corrupt accounting")
	reader.invalidate()
	reader.time_snapshot()
	_check(reader.frame_call_counts().get("get_time_snapshot", 0) == 2, "Invalidation preserves same-frame counts")
	reader.peek_uncounted("get_time_snapshot")
	_check(reader.frame_call_counts().get("get_time_snapshot", 0) == 2, "Diagnostic peek is uncounted")
	reader.invalidate()
	reader.peek_uncounted("get_time_snapshot")
	reader.time_snapshot()
	_check(reader.frame_call_counts().get("get_time_snapshot", 0) == 3, "Diagnostic peek does not warm cache")
	await process_frame
	var next: Dictionary = reader.time_snapshot()
	_check(not is_same(first, next), "Next engine frame refreshes dictionary identity")
	_check(reader.frame_call_counts().get("get_time_snapshot", 0) == 1, "New frame resets counters")
	var terrain: Dictionary = reader.terrain_snapshot()
	_check(reader.advance_ticks(1) == 1, "Advance preserves native progress result")
	_check(is_same(terrain, reader.terrain_snapshot()), "Advance keeps static terrain cache")
	_check(not is_same(next, reader.time_snapshot()), "Advance invalidates same-frame snapshot")
	_check(reader.frame_call_counts().get("get_time_snapshot", 0) == 2, "Mutation does not reset getter accounting")
	_check(reader.reset_world(20261013), "Normal reset succeeds")
	var reset_terrain: Dictionary = reader.terrain_snapshot()
	_check(not is_same(terrain, reset_terrain), "Normal reset refreshes terrain before deferred signal")
	_check(reader.reset_world_stress(15100, 200), "Stress reset succeeds")
	var stress_terrain: Dictionary = reader.terrain_snapshot()
	_check(not is_same(reset_terrain, stress_terrain), "Stress reset refreshes terrain before deferred signal")
	_check(reader.frame_call_counts().get("get_terrain_snapshot", 0) == 3, "Both resets retain cold-read accounting")
	await process_frame
	await process_frame
	var signaled_terrain: Dictionary = reader.terrain_snapshot()
	_check(not is_same(stress_terrain, signaled_terrain), "Deferred world reset also clears terrain")
	await process_frame
	_check(is_same(signaled_terrain, reader.terrain_snapshot()), "Terrain cache survives frame rollover")
	_check(reader.frame_call_counts().get("get_terrain_snapshot", 0) == 0, "Warm terrain costs no FFI across frames")
	for raw_getter: String in READER.GETTERS:
		_check(reader.read_raw(raw_getter).is_read_only(), "Readonly getter: " + raw_getter)
	container.free()
	print("SIM READS P9: failures=%d" % _failures)
	quit(0 if _failures == 0 else 1)

func _test_fake_and_identity() -> void:
	var fake := FakeWorld.new()
	var container: Node = _fixture(fake)
	var reader: SimReads = container.get_node("SimReads") as SimReads
	var first: Dictionary = READER.read(fake, "get_time_snapshot")
	_check(not first.is_read_only(), "Fake dictionary remains mutable")
	_check(not first.nested.is_read_only(), "Fake nested array remains mutable")
	_check(READER.mutate(fake, "advance_ticks", [7]) == 7, "Fake mutation returns native result")
	var second: Dictionary = READER.read(fake, "get_time_snapshot")
	_check(second.tick == 10 and fake.calls == 2, "Fake same-frame reads observe changed state")
	_check(reader.frame_call_counts().is_empty(), "Fake pass-through does not claim real FFI calls")
	var replacement := FakeWorld.new()
	container.remove_child(fake)
	replacement.name = "WorldSim"
	container.add_child(replacement)
	_check(not reader.owns_world(fake) and reader.owns_world(replacement), "Resolver revalidates replaced world")
	_check(READER.resolve(fake) == null, "Unparented world cannot borrow reader")
	fake.free()
	var other := FakeWorld.new()
	var other_container: Node = _fixture(other)
	_check(READER.resolve(other) != reader, "Two main-like trees keep distinct readers")
	other_container.free()
	container.free()
	var standalone := FakeWorld.new()
	_check(READER.resolve(standalone) == null, "Standalone fake has no reader")
	_check(READER.read(standalone, "get_time_snapshot").tick == 3, "Standalone read passes through")
	_check(READER.mutate(standalone, "advance_ticks", [2]) == 2, "Standalone mutation passes through")
	_check(READER.read(standalone, "get_time_snapshot").tick == 5, "Standalone mutation visible immediately")
	standalone.free()
	_check(READER.read(null, "get_time_snapshot").is_empty(), "Missing world read is empty")
	_check(READER.mutate(null, "advance_ticks", [1]) == null, "Missing world mutation is null")

func _test_recursive_freeze() -> void:
	var snapshot: Dictionary = {"rows": [{"values": [1, 2]}], "xs": PackedInt32Array([1, 2])}
	READER._freeze(snapshot)
	_check(snapshot.is_read_only(), "Root dictionary readonly")
	_check(snapshot.rows.is_read_only(), "Nested array readonly")
	_check(snapshot.rows[0].is_read_only(), "Dictionary inside array readonly")
	_check(snapshot["rows"][0]["values"].is_read_only(), "Nested leaf array readonly")
	var copied: PackedInt32Array = snapshot.xs.duplicate()
	copied[0] = 99
	_check(snapshot.xs[0] == 1, "Audited packed-array duplicate isolates caller mutation")
