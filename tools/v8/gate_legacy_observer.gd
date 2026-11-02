# Gate-only autoload for exact historical preknowledge sources in an isolated export.
# Native reads: get_world_info, get_time_snapshot, get_agent_snapshot, get_agent_id,
# get_tallies, get_chronicle_since. No commands, pause, reset, advance or replacement scene.
extends Node

const PRODUCTS: Array[String] = ["892edca1d31b26e535a3e9b62b37d62b99bb1130", "b1ca032f8fa84cee87d68a185e471b8e2c2bddce"]
const COLUMNS: Array[String] = ["tick", "kind", "subject", "other", "third", "settlement", "settlement2", "cause", "value"]
var _source: Dictionary = {}
var _instances: Dictionary = {}
var _errors: Array[String] = []
var _path: String = ""
var _enabled: bool = false

func _enter_tree() -> void:
	_enabled = OS.get_environment("WORLDSIM_GATE_OBSERVER") == "1"
	if not _enabled:
		return
	var directory: String = OS.get_environment("WORLDSIM_GATE_UI_OUT").simplify_path().trim_suffix("/")
	var project: String = ProjectSettings.globalize_path("res://").simplify_path().trim_suffix("/")
	if not directory.is_absolute_path() or not DirAccess.dir_exists_absolute(directory) or directory == project or directory.begins_with(project + "/"):
		_error("missing existing external sidecar directory")
		return
	_path = directory.path_join("legacy-%d.json" % OS.get_process_id())
	if FileAccess.file_exists(_path):
		_path = ""
		_error("refusing existing legacy sidecar")
		return
	var parsed: Variant = JSON.parse_string(FileAccess.get_file_as_string("res://gate_observer/provenance.json"))
	if not parsed is Dictionary:
		_error("missing export provenance")
		return
	_source = parsed
	if str(_source.get("product_commit", "")) not in PRODUCTS or _source.get("knowledge_enabled") != false or _source.get("knowledge_capability_source") != "pinned_preknowledge_native_schema":
		_error("unqualified historical source/capability")
		return
	if OS.get_environment("WORLDSIM_GATE_INVOCATION_ID").is_empty():
		_error("missing invocation identity")
		return
	for pair: Array in [["res://project.godot", "project_overlay_sha256"], ["res://gate_observer/legacy.gd", "observer_sha256"], ["res://gate_observer/build-receipt.json", "build_receipt_sha256"], ["res://rust/target/debug/libsim_bridge.dylib", "binary_sha256"]]:
		if FileAccess.get_sha256(str(pair[0])) != str(_source.get(str(pair[1]), "")):
			_error("export hash mismatch " + str(pair[0]))
			return
	get_tree().node_added.connect(_node_added)
	_write()

func _node_added(node: Node) -> void:
	if not node.is_class("WorldSimNode"):
		return
	var key: String = str(node.get_instance_id())
	if _instances.has(key):
		_error("duplicate native instance identity")
		return
	_instances[key] = {"initial": _snapshot(node), "terminal": null, "history_complete": false,
		"census_qualification": "PENDING", "lifecycle_qualification": "PENDING"}
	if (_instances[key]["initial"] as Dictionary).get("tick") != "0":
		_error("initial observation was not at native tick zero")
	node.tree_exiting.connect(_terminal.bind(node, key))
	if node.has_signal("world_reset"):
		node.connect("world_reset", _unexpected_reset.bind(key))
	_write()

func _unexpected_reset(key: String) -> void:
	_error("unexpected historical reset; discarded engine cannot be recovered " + key)

func _snapshot(world: Node) -> Dictionary:
	for method: String in ["get_world_info", "get_time_snapshot", "get_agent_snapshot", "get_agent_id", "get_tallies", "get_chronicle_since"]:
		if not world.has_method(method):
			_error("missing historical API " + method)
			return {}
	var clock: Dictionary = world.call("get_time_snapshot")
	var agents: Dictionary = world.call("get_agent_snapshot")
	if not agents.get("agent_ids") is PackedInt64Array or not agents.get("ids") is PackedInt64Array or not clock.has("tick"):
		_error("invalid historical census/clock schema")
		return {}
	if agents["ids"].size() != agents["agent_ids"].size():
		_error("historical entity/stable identity columns disagree")
		return {}
	var stable_ids: Array[String] = []
	for index: int in agents["ids"].size():
		var identity: int = int(world.call("get_agent_id", int(agents["ids"][index])))
		if identity == -1 or identity != int(agents["agent_ids"][index]):
			_error("ambiguous or mismatched historical AgentId")
		stable_ids.append(str(identity))
	stable_ids.sort()
	var pages: Array[Dictionary] = []
	var cursor: int = 0
	var expected_total: int = -1
	while true:
		var page: Dictionary = world.call("get_chronicle_since", cursor, 512)
		var total: int = int(page.get("total", -1))
		var next: int = int(page.get("next", -1))
		if expected_total < 0:
			expected_total = total
		if total < cursor or total != expected_total or next < cursor or next > total or next - cursor > 512 or not bool(page.get("found", false)):
			_error("inconsistent historical chronicle pagination")
			return {}
		var columns: Dictionary = {}
		for column: String in COLUMNS:
			var values: Variant = page.get(column)
			if not (values is PackedInt64Array or values is PackedInt32Array) or values.size() != next - cursor:
				_error("malformed historical chronicle column " + column)
				return {}
			var signed_values: Array[String] = []
			for value: int in values:
				signed_values.append(str(value))
			columns[column] = signed_values
		pages.append({"start": cursor, "next": next, "total": total, "columns_signed_decimal": columns})
		if next == total:
			break
		if next == cursor:
			_error("historical chronicle cursor stalled")
			return {}
		cursor = next
	var after: Dictionary = world.call("get_time_snapshot")
	if clock.get("tick") != after.get("tick"):
		_error("historical clock changed during read")
	return {"tick": str(clock["tick"]), "population": stable_ids.size(), "agent_id_bits_signed": stable_ids,
		"census_source": "get_agent_snapshot Agent+Position; native completeness qualification pending",
		"world_info": world.call("get_world_info"), "tallies": world.call("get_tallies"),
		"chronicle_total": expected_total, "chronicle_pages": pages,
		"knowledge_enabled": false, "knowledge_lost": null,
		"knowledge_capability_source": "pinned_preknowledge_native_schema"}

func _terminal(world: Node, key: String) -> void:
	var row: Dictionary = _instances[key]
	row["terminal"] = _snapshot(world)
	row["history_complete"] = false
	# The raw pages can be complete while constructor/census/shutdown qualification is still pending.
	row["history_status"] = "OBSERVED_PAGES_QUALIFICATION_PENDING"
	_write()

func _error(message: String) -> void:
	_errors.append(message)
	push_error("GATE_LEGACY_OBSERVER " + message)
	_write()

func _write() -> void:
	if _path.is_empty():
		return
	var file: FileAccess = FileAccess.open(_path + ".tmp", FileAccess.WRITE)
	if file == null:
		push_error("GATE_LEGACY_OBSERVER cannot write sidecar")
		return
	var complete: bool = _instances.size() == 1 and _errors.is_empty()
	for row: Dictionary in _instances.values():
		complete = complete and row.get("terminal") is Dictionary and not (row["terminal"] as Dictionary).is_empty()
	file.store_string(JSON.stringify({"schema_version": 1, "invocation_id": OS.get_environment("WORLDSIM_GATE_INVOCATION_ID"),
		"pid": OS.get_process_id(), "argv": OS.get_cmdline_args(), "product_commit": _source.get("product_commit"),
		"export_provenance_sha256": FileAccess.get_sha256("res://gate_observer/provenance.json"),
		"complete": complete, "qualified": false, "native_exit_code": null,
		"instances": _instances, "errors": _errors}, "\t"))
	file.close()
	if DirAccess.rename_absolute(_path + ".tmp", _path) != OK:
		push_error("GATE_LEGACY_OBSERVER cannot publish sidecar")
