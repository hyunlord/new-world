# Gate-only sidecars for actual native nodes; disabled unless WORLDSIM_GATE_OBSERVER=1.
# Bridge reads: get_gate_observation, get_tallies, get_command_log, get_time_snapshot.
extends RefCounted

var enabled: bool = OS.get_environment("WORLDSIM_GATE_OBSERVER") == "1"
var _path: String = ""
var _argv: PackedStringArray = OS.get_cmdline_args()
var _user_args: PackedStringArray = OS.get_cmdline_user_args()
var _nodes: Dictionary = {}
var _checkpoints: Array[Dictionary] = []
var _errors: Array[String] = []
var _producer: String
var _script: String
var _complete: bool = false
var _native_exit: int = -1
var _completion: String = "pending"
var _classification: String = "native_world_producer"
var _no_native_reason: String = ""

func _init(producer: String, script: String) -> void:
	_producer = producer
	_script = script
	if not enabled:
		return
	var directory: String = OS.get_environment("WORLDSIM_GATE_UI_OUT").simplify_path().trim_suffix("/")
	var project: String = ProjectSettings.globalize_path("res://").simplify_path().trim_suffix("/")
	if not directory.is_absolute_path() or directory == project or directory.begins_with(project + "/") or not DirAccess.dir_exists_absolute(directory):
		_errors.append("WORLDSIM_GATE_UI_OUT must be an existing external directory")
		return
	_path = directory.path_join("%s-%d.json" % [producer, OS.get_process_id()])
	if FileAccess.file_exists(_path):
		_errors.append("refusing existing sidecar " + _path)
		_path = ""
		return
	if OS.get_environment("WORLDSIM_GATE_INVOCATION_ID").is_empty():
		_errors.append("missing WORLDSIM_GATE_INVOCATION_ID")
	_write()

func capture(main: Node, label: String, context: Dictionary = {}) -> int:
	if not enabled:
		return -1
	if not is_instance_valid(main):
		_errors.append("missing scene at " + label)
		_write()
		return -1
	var world: Node = main.get_node_or_null("WorldSim")
	if world == null or not world.has_method("get_gate_observation"):
		_errors.append("missing native observer at " + label)
		_write()
		return -1
	var key: String = str(world.get_instance_id())
	_nodes[key] = {"main": weakref(main), "released": false}
	var before: Dictionary = world.call("get_time_snapshot")
	var observation: Dictionary = world.call("get_gate_observation")
	var tallies: Dictionary = world.call("get_tallies")
	var commands: Dictionary = world.call("get_command_log", 0)
	for column: String in ["ticks", "kinds", "accepted"]:
		commands[column] = Array(commands.get(column, []))
	var after: Dictionary = world.call("get_time_snapshot")
	if observation.get("status") != "OBSERVED" or observation.get("schema_version") != 1 or observation.get("node_instance_id") != key or not observation.get("instances") is Array:
		_errors.append("invalid native observation at " + label)
	if not before.has("tick") or before.get("tick") != after.get("tick"):
		_errors.append("tick changed during read at " + label)
	var index: int = _checkpoints.size()
	_checkpoints.append({"index": index, "label": label, "node_instance_id": key,
		"observation": observation, "tallies": tallies, "commands": commands,
		"clock": after, "command_log_capability": "ticks_kinds_accepted_only", "context": context.duplicate(true)})
	_write()
	return index

func release(main: Node, label: String, context: Dictionary = {}) -> void:
	if not enabled:
		return
	capture(main, label, context)
	if is_instance_valid(main):
		var world: Node = main.get_node_or_null("WorldSim")
		if world != null and _nodes.has(str(world.get_instance_id())):
			_nodes[str(world.get_instance_id())]["released"] = true

func finish(native_exit: int) -> int:
	if not enabled:
		return native_exit
	if _classification == "synthetic_renderer_only" and not _nodes.is_empty():
		_errors.append("synthetic-only producer observed native nodes")
	for key: String in _nodes.keys():
		var row: Dictionary = _nodes[key]
		if bool(row["released"]):
			continue
		var main: Node = (row["main"] as WeakRef).get_ref() as Node
		if is_instance_valid(main):
			capture(main, "terminal")
		else:
			_errors.append("node freed without terminal receipt " + key)
	_native_exit = native_exit
	_complete = true
	_completion = "producer_finish"
	_write()
	for error: String in _errors:
		push_error("GATE_UI_OBSERVER " + error)
	return native_exit if native_exit != 0 else (0 if _errors.is_empty() else 1)

func shutdown(main: Node) -> void:
	if not enabled:
		return
	release(main, "native_tree_exit")
	_complete = true
	_completion = "native_tree_exit"
	_write()
	for error: String in _errors:
		push_error("GATE_UI_OBSERVER " + error)

func declare_no_native(reason: String) -> void:
	if not enabled:
		return
	if not _nodes.is_empty():
		_errors.append("no-native declaration conflicts with observed native nodes")
	_classification = "synthetic_renderer_only"
	_no_native_reason = reason
	_write()

func _write() -> void:
	if _path.is_empty():
		return
	var receipt: Dictionary = {"schema_version": 1, "producer": _producer,
		"invocation_id": OS.get_environment("WORLDSIM_GATE_INVOCATION_ID"),
		"pid": OS.get_process_id(), "argv": _argv,
		"user_args": _user_args, "project": ProjectSettings.globalize_path("res://"),
		"script": _script, "script_sha256": FileAccess.get_sha256(_script),
		"helper_sha256": FileAccess.get_sha256("res://tools/v8/gate_ui_observer.gd"),
		"complete": _complete, "completion": _completion, "native_exit_code": _native_exit,
		"classification": _classification, "no_native_reason": _no_native_reason,
		"observed_native_nodes": _nodes.size(),
		"checkpoints": _checkpoints, "errors": _errors}
	var file: FileAccess = FileAccess.open(_path + ".tmp", FileAccess.WRITE)
	if file == null:
		_errors.append("sidecar write failed " + _path)
		return
	file.store_string(JSON.stringify(receipt, "\t"))
	file.close()
	if DirAccess.rename_absolute(_path + ".tmp", _path) != OK:
		_errors.append("sidecar rename failed " + _path)
