# Actual existing bridge get_trees integration; Rust opt-in probe logs frozen packing and real GString calls.
# Bridge methods: bootstrap_catalog/create, reset_world, get_trees, get_colony_versions.
extends SceneTree
var checks: int = 0
var failures: int = 0

func _init() -> void:
	call_deferred("_run")

func _check(label: String, valid: bool) -> void:
	checks += 1
	failures += 0 if valid else 1
	print("MUX2_TREE_BRIDGE_%s %s" % ["PASS" if valid else "FAIL", label])

func _run() -> void:
	_check("probe_explicitly_enabled", OS.get_environment("WORLDSIM_MUX2_TREE_CONVERSION_PROBE") == "1")
	var receipt: Dictionary = WorldSimNode.bootstrap_catalog()
	_check("actual_content_bootstrap", bool(receipt.get("ok", false)))
	var world: WorldSimNode = WorldSimNode.create()
	_check("actual_bridge_factory", world != null)
	if world == null:
		quit(1)
		return
	world.process_mode = Node.PROCESS_MODE_DISABLED
	root.add_child(world)
	var first: Dictionary = {}
	for invocation: int in 4:
		if invocation == 2:
			_check("world_reset", world.reset_world(1))
		var snapshot: Dictionary = world.get_trees()
		var keys: Array = snapshot.keys()
		keys.sort()
		_check("call%d_exact_schema" % invocation, keys == ["species", "version", "xs", "ys"])
		var xs: PackedInt32Array = snapshot["xs"]
		var ys: PackedInt32Array = snapshot["ys"]
		var species: PackedStringArray = snapshot["species"]
		_check("call%d_aligned_columns" % invocation, xs.size() == ys.size() and xs.size() == species.size())
		var ordered: bool = true
		for i: int in range(1, xs.size()):
			ordered = ordered and (xs[i - 1] < xs[i] or (xs[i - 1] == xs[i] and ys[i - 1] <= ys[i]))
		_check("call%d_original_tile_order" % invocation, ordered)
		_check("call%d_actual_version" % invocation, snapshot["version"] == world.get_colony_versions()["trees_version"])
		if invocation % 2 == 0:
			first = snapshot.duplicate(true)
		else:
			_check("call%d_repeated_exact_snapshot" % invocation, snapshot == first)
	world.free()
	print("MUX2_TREE_BRIDGE_DONE checks=%d failures=%d" % [checks, failures])
	quit(0 if failures == 0 else 1)
