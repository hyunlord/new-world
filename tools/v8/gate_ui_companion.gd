# Exact native horizon companion for command-free smoke only, never a UI oracle.
# Bridge reads: get_time_snapshot, get_command_log; advance_ticks is the existing manual tick API.
# Runner must create exclusive fresh output_root and bind settings to output_root/settings.
extends SceneTree

const GateObserver = preload("res://tools/v8/gate_ui_observer.gd")
var _observer: RefCounted = GateObserver.new("smoke.companion", "res://tools/v8/gate_ui_companion.gd")

func _init() -> void:
	call_deferred("_run")

func _fail(message: String) -> void:
	push_error("GATE_COMPANION " + message)
	quit(_observer.finish(1))

func _run() -> void:
	if not _observer.enabled:
		_fail("gate observer must be enabled")
		return
	var args: PackedStringArray = OS.get_cmdline_user_args()
	if args.size() != 6 or args[0] != "--ticks" or args[2] != "--original-producer" or args[4] != "--original-sidecar-sha256":
		_fail("expected exact tick, original producer and authenticated sidecar hash arguments")
		return
	var decimal: RegEx = RegEx.new()
	decimal.compile("^(0|[1-9][0-9]{0,5})$")
	var digest: RegEx = RegEx.new()
	digest.compile("^[0-9a-f]{64}$")
	if decimal.search(args[1]) == null or int(args[1]) > 200000 or digest.search(args[5]) == null or args[3] not in ["smoke.boot", "smoke.labels"]:
		_fail("unsupported original or invalid exact horizon/hash")
		return
	var settings: String = OS.get_environment("WORLDSIM_TEST_SETTINGS_DIR")
	if not settings.is_absolute_path() or FileAccess.file_exists(settings.path_join("settings-%d.cfg" % OS.get_process_id())):
		_fail("fresh isolated settings directory required")
		return
	if DirAccess.make_dir_recursive_absolute(settings) != OK:
		_fail("cannot create isolated settings directory")
		return
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(main)
	var world: Node = main.get_node_or_null("WorldSim")
	if world == null or not bool(main.get("boot_ok")):
		_fail("actual main scene bootstrap failed")
		return
	world.process_mode = Node.PROCESS_MODE_DISABLED
	main.process_mode = Node.PROCESS_MODE_DISABLED
	_observer.capture(main, "bootstrap")
	if int(world.get_time_snapshot().get("tick", -1)) != 0 or not _native_commands_only(world.get_command_log(0), true):
		_fail("bootstrap advanced or changed intrinsic steward command")
		return
	var target: int = int(args[1])
	var actual: int = int(world.advance_ticks(target))
	if actual != target or int(world.get_time_snapshot().get("tick", -1)) != target or not _native_commands_only(world.get_command_log(0), false):
		_fail("native exact horizon or intrinsic native command contract failed")
		return
	_observer.release(main, "companion_terminal", {"requested_tick": target, "original_producer": args[3], "original_sidecar_sha256": args[5]})
	quit(_observer.finish(0))

func _native_commands_only(log: Dictionary, bootstrap: bool) -> bool:
	var ticks: Array = Array(log.get("ticks", []))
	var kinds: Array = Array(log.get("kinds", []))
	var accepted: Array = Array(log.get("accepted", []))
	var total: int = int(log.get("total", -1))
	if total < 1 or ticks.size() != total or kinds.size() != total or accepted.size() != total:
		return false
	if ticks[0] != 0 or kinds[0] != "SetSteward" or accepted[0] != 1 or (bootstrap and total != 1):
		return false
	for index: int in range(1, total):
		if kinds[index] != "Steward":
			return false
	return true
