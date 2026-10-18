extends SceneTree
const Reader = preload("res://scripts/ui/sim_reads.gd")
const GETTERS: Array[String] = ["get_porter_snapshot", "get_settlement_relations"]
var _failures: int = 0
var _assertions: int = 0

class FakeWorld extends Node:
	var calls: int = 0
	func get_porter_snapshot() -> Dictionary:
		calls += 1
		return {"value": calls}
	func get_settlement_relations() -> Dictionary:
		calls += 1
		return {"value": calls}

func _initialize() -> void:
	_run.call_deferred()

func _check(value: bool, message: String) -> void:
	_assertions += 1
	if not value:
		_failures += 1
		push_error("SIM READS RELATIONS P12: " + message)

func _fixture(world: Node) -> Node:
	var parent := Node.new()
	world.name = "WorldSim"
	parent.add_child(world)
	var reader := Reader.new()
	reader.name = "SimReads"
	parent.add_child(reader)
	return parent

func _run() -> void:
	for getter: String in GETTERS:
		_check(getter in Reader.GETTERS, getter + " is a supported getter")
	if _failures > 0:
		_finish()
		return
	var world: Node = ClassDB.instantiate("WorldSimNode")
	world.set_process(false)
	world.call("set_sim_speed", 0.0)
	var parent: Node = _fixture(world)
	root.add_child(parent)
	var reader: SimReads = parent.get_node("SimReads") as SimReads
	reader.reset_world(19400)
	reader.advance_ticks(121080)
	await process_frame
	await process_frame
	var first: Dictionary = {}
	for getter: String in GETTERS:
		var snapshot: Dictionary = reader.read_raw(getter)
		first[getter] = snapshot
		_check(snapshot == world.call(getter), getter + " matches live bridge")
		_check(snapshot.is_read_only(), getter + " real snapshot frozen")
		_check(is_same(snapshot, Reader.read(world, getter)), getter + " shared same-frame identity")
		_check(reader.frame_call_counts().get(getter, 0) == 1, getter + " exactly one FFI call")
		_check(reader.frame_call_usec().has(getter), getter + " FFI timing recorded")
	_check(first[GETTERS[0]].get("ids") is PackedInt64Array, "porter ids packed entity bits")
	_check(first[GETTERS[0]].get("stage") is PackedByteArray, "porter stages packed bytes")
	_check(first[GETTERS[1]].get("value") is PackedInt32Array, "relation values packed signed integers")
	_check(first[GETTERS[1]].get("estranged") is PackedByteArray, "estranged packed bytes")
	_check(first[GETTERS[0]].get("ids", []).size() > 0, "W-multi actually has porters")
	_check(first[GETTERS[1]].get("a", []).size() > 0, "W-multi actually has relation pairs")
	var agents: Dictionary = reader.agent_snapshot()
	var terrain: Dictionary = reader.terrain_snapshot()
	for _frame: int in range(2):
		await process_frame
		for getter: String in GETTERS:
			_check(is_same(first[getter], reader.read_raw(getter)), getter + " same tick survives next render frame")
			_check(reader.frame_call_counts().get(getter, 0) == 0, getter + " cached next frame has no FFI call")
	_check(not is_same(agents, reader.agent_snapshot()), "existing ordinary frame cache remains frame based")
	reader.advance_ticks(1)
	_check(is_same(terrain, reader.terrain_snapshot()), "advance retains static terrain")
	for getter: String in GETTERS:
		var next: Dictionary = reader.read_raw(getter)
		_check(not is_same(first[getter], next), getter + " next tick refreshes")
		_check(next == world.call(getter), getter + " next tick value correct")
		_check(reader.frame_call_counts().get(getter, 0) == 1, getter + " next tick counted once")
		first[getter] = next
	# Native tick advancement between frames must refresh even without explicit reader invalidation.
	world.call("advance_ticks", 1)
	await process_frame
	for getter: String in GETTERS:
		_check(not is_same(first[getter], reader.read_raw(getter)), getter + " observed native next tick refreshes")
		_check(reader.frame_call_counts().get(getter, 0) == 1, getter + " native next tick counted once")
	reader.invalidate()
	for getter: String in GETTERS:
		var before: int = int(reader.frame_call_counts().get(getter, 0))
		var peek: Dictionary = reader.peek_uncounted(getter)
		_check(reader.frame_call_counts().get(getter, 0) == before, getter + " peek uncounted")
		var counted: Dictionary = reader.read_raw(getter)
		_check(not is_same(peek, counted), getter + " peek does not warm cache")
		_check(reader.frame_call_counts().get(getter, 0) == before + 1, getter + " postpeek refresh counted")
		first[getter] = counted
	reader.reset_world(19400)
	for getter: String in GETTERS:
		var reset: Dictionary = reader.read_raw(getter)
		_check(not is_same(first[getter], reset), getter + " reset refreshes before deferred signal")
		_check(reset == world.call(getter), getter + " reset values match bridge immediately")
		first[getter] = reset
	reader.reset_world_stress(15100, 20)
	for getter: String in GETTERS:
		_check(not is_same(first[getter], reader.read_raw(getter)), getter + " stress reset refreshes")
	parent.remove_child(world)
	world.free()
	var replacement: Node = ClassDB.instantiate("WorldSimNode")
	replacement.name = "WorldSim"
	replacement.set_process(false)
	parent.add_child(replacement)
	for getter: String in GETTERS:
		_check(reader.read_raw(getter) == replacement.call(getter), getter + " world replacement rebinds")
	parent.free()
	_test_fake_paths()
	_finish()

func _test_fake_paths() -> void:
	var fake := FakeWorld.new()
	var parent: Node = _fixture(fake)
	root.add_child(parent)
	var reader: SimReads = parent.get_node("SimReads") as SimReads
	for getter: String in GETTERS:
		var first: Dictionary = Reader.read(fake, getter)
		var second: Dictionary = reader.read_raw(getter)
		_check(not is_same(first, second) and int(second["value"]) == int(first["value"]) + 1, getter + " fake remains live pass-through")
		_check(not first.is_read_only(), getter + " fake remains mutable")
		_check(reader.frame_call_counts().is_empty(), getter + " fake remains uncounted")
	parent.free()
	var missing := Node.new()
	parent = _fixture(missing)
	root.add_child(parent)
	reader = parent.get_node("SimReads") as SimReads
	for getter: String in GETTERS:
		_check(Reader.read(missing, getter).is_empty(), getter + " absent optional fake getter is empty")
		_check(reader.peek_uncounted(getter).is_empty(), getter + " absent optional diagnostic getter is empty")
	parent.free()
	var detached := FakeWorld.new()
	for getter: String in GETTERS:
		_check(not Reader.read(detached, getter).is_empty(), getter + " detached fake uses direct fallback")
	detached.free()

func _finish() -> void:
	print("SIM READS RELATIONS P12 assertions=%d failures=%d" % [_assertions, _failures])
	quit(0 if _failures == 0 else 1)
