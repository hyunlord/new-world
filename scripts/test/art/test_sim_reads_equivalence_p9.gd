extends SceneTree

# Also preloaded by baseline-compatible tools; only a SceneTree instance runs tests.
static func resolve(world: Node) -> Node:
	if not is_instance_valid(world) or world.get_parent() == null:
		return null
	var reader: Node = world.get_parent().get_node_or_null("SimReads")
	if reader != null and reader.has_method("owns_world") and bool(reader.call("owns_world", world)):
		return reader
	return null

static func read(world: Node, getter: String, uncounted: bool = false) -> Dictionary:
	var reader: Node = resolve(world)
	if reader != null:
		return reader.call("peek_uncounted" if uncounted else "read_raw", getter)
	return old_read(world, getter)

static func mutate(world: Node, method: String, arguments: Array = []) -> Variant:
	var reader: Node = resolve(world)
	if reader != null:
		return reader.call("mutate_raw", method, arguments)
	return old_mutate(world, method, arguments)

static func old_read(world: Node, getter: String) -> Dictionary:
	return world.call(getter)

static func old_mutate(world: Node, method: String, arguments: Array) -> Variant:
	return world.callv(method, arguments)

const GETTERS: Array[String] = [
	"get_agent_snapshot", "get_settlement_snapshot", "get_construction_snapshot",
	"get_building_snapshot", "get_resource_snapshot", "get_time_snapshot",
	"get_tallies", "get_recent_deaths", "get_society_snapshot",
	"get_intervention_state", "get_world_info", "get_terrain_snapshot",
	"get_activity_snapshot",
]
var _failures: int = 0
var _assertions: int = 0

class FakeWorld extends Node:
	var tick: int = 0
	func get_time_snapshot() -> Dictionary:
		return {"tick": tick}
	func advance_ticks(count: int) -> int:
		tick += count
		return tick

class ForeignReader extends Node:
	var owner_world: Node
	var routed_calls: int = 0
	func owns_world(world: Node) -> bool:
		return world == owner_world
	func read_raw(_getter: String) -> Dictionary:
		routed_calls += 1
		return {"tick": -999}
	func peek_uncounted(_getter: String) -> Dictionary:
		routed_calls += 1
		return {"tick": -999}
	func mutate_raw(_method: String, _arguments: Array) -> Variant:
		routed_calls += 1
		return -999

func _initialize() -> void:
	call_deferred("_run")

func _run() -> void:
	_test_fake_and_foreign_reader()
	_test_advanced_worlds()
	print("SIM READS EQUIVALENCE P9: %d assertions, %d failures" % [_assertions, _failures])
	quit(0 if _failures == 0 else 1)

func _check(condition: bool, message: String) -> void:
	_assertions += 1
	if not condition:
		_failures += 1
		push_error("SIM READS EQUIVALENCE P9: " + message)

func _test_fake_and_foreign_reader() -> void:
	var fake := FakeWorld.new()
	_check(resolve(fake) == null, "detached fake has no reader")
	_check(read(fake, "get_time_snapshot") == old_read(fake, "get_time_snapshot"), "detached old read")
	var parent := Node.new()
	fake.name = "WorldSim"
	parent.add_child(fake)
	var other := FakeWorld.new()
	var foreign := ForeignReader.new()
	foreign.name = "SimReads"
	foreign.owner_world = other
	parent.add_child(foreign)
	root.add_child(parent)
	_check(resolve(fake) == null, "sibling owning a different world is rejected")
	for tick: int in [7, 23, 91]:
		fake.tick = tick
		_check(read(fake, "get_time_snapshot")["tick"] == tick, "fake same-frame data stays live")
		_check(read(fake, "get_time_snapshot", true)["tick"] == tick, "fake uncounted pass-through")
	_check(mutate(fake, "advance_ticks", [5]) == 96, "fake mutation returns old-path value")
	_check(read(fake, "get_time_snapshot")["tick"] == 96, "fake mutation is immediately visible")
	_check(foreign.routed_calls == 0, "unmatched sibling receives no reads or mutations")
	parent.free()
	other.free()

func _test_advanced_worlds() -> void:
	var packed: PackedScene = load("res://scenes/main.tscn") as PackedScene
	var main: Node = packed.instantiate()
	var reader: Node = main.get_node_or_null("SimReads")
	if reader == null:
		_check(false, "current main must contain reader for real-route equivalence")
		main.free()
		return
	var reader_script: Script = reader.get_script()
	var world: Node = main.get_node("WorldSim")
	world.process_mode = Node.PROCESS_MODE_DISABLED
	world.call("set_sim_speed", 0.0)
	root.add_child(main)
	current_scene = main
	for stress: bool in [false, true]:
		var label: String = "W500" if stress else "W0"
		var reset_method: String = "reset_world_stress" if stress else "reset_world"
		var reset_arguments: Array = [15100, 500] if stress else [20261010]
		var ticks: int = 1440 if stress else 40560
		_check(bool(mutate(world, reset_method, reset_arguments)), label + " reset succeeds")
		_check(int(mutate(world, "advance_ticks", [ticks])) == ticks, label + " exact frozen tick")
		_check(resolve(world) == reader, label + " matching real reader")
		_compare_getters(world, world, label + " reader")
		# Reuse this exact advanced world; remove the reader before re-adding Main.
		current_scene = null
		root.remove_child(main)
		var prior_owner: Node = reader.owner
		reader.owner = null
		main.remove_child(reader)
		root.add_child(main)
		current_scene = main
		_check(resolve(world) == null, label + " absent reader before Main enters tree")
		_compare_getters(world, world, label + " absent")
		main.add_child(reader)
		reader.owner = prior_owner
		_check(resolve(world) == reader, label + " reader restored")
		_compare_getters(world, world, label + " restored")
	current_scene = null
	main.free()
	_test_mutation_returns(reader_script)

func _test_mutation_returns(reader_script: Script) -> void:
	var parent := Node.new()
	var world: Node = ClassDB.instantiate("WorldSimNode")
	var reference: Node = ClassDB.instantiate("WorldSimNode")
	world.name = "WorldSim"
	world.process_mode = Node.PROCESS_MODE_DISABLED
	reference.process_mode = Node.PROCESS_MODE_DISABLED
	world.call("set_sim_speed", 0.0)
	reference.call("set_sim_speed", 0.0)
	parent.add_child(world)
	var reader: Node = reader_script.new()
	reader.name = "SimReads"
	parent.add_child(reader)
	root.add_child(parent)
	_check(resolve(world) == reader, "mutation fixture owns real world")
	# Small-state reference checks preserve all five native mutation return contracts.
	var operations: Array[Array] = [
		["reset_world", [20261010]],
		["advance_ticks", [1]],
		["apply_intervention", ["gift_food", -1, 5]],
		["on_building_placed", [32, 32, 1]],
		["advance_ticks", [1]],
		["reset_world_stress", [15100, 500]],
	]
	for operation: Array in operations:
		var method: String = operation[0]
		var arguments: Array = operation[1]
		# Warm all registered domains immediately before each mutation, without yielding.
		for getter: String in GETTERS:
			read(world, getter)
		var expected: Variant = old_mutate(reference, method, arguments)
		var actual: Variant = mutate(world, method, arguments)
		_check(actual == expected, "small-state mutation result " + method)
		_compare_getters(world, reference, "small-state after " + method)
	reference.free()
	parent.free()

func _compare_getters(world: Node, reference: Node, label: String) -> void:
	for getter: String in GETTERS:
		var expected: Dictionary = old_read(reference, getter)
		_check(read(world, getter) == expected, label + " routed " + getter)
		_check(read(world, getter) == old_read(world, getter), label + " local old-path " + getter)
		_check(read(world, getter, true) == expected, label + " uncounted " + getter)
