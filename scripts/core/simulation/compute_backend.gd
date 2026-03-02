extends Node

signal compute_mode_changed(new_mode: String, resolved_mode: String)
signal compute_domain_mode_changed(domain: String, new_mode: String, resolved_mode: String)

const SETTINGS_PATH: String = "user://settings.json"
const SUPPORTED_MODES: Array[String] = ["cpu", "gpu_auto", "gpu_force"]
const DEFAULT_MODE: String = "gpu_auto"
const COMPUTE_DOMAINS: Array[String] = [
	"pathfinding",
	"needs",
	"stress",
	"emotion",
	"orchestration",
]
const GPU_ENABLED_DOMAINS: Array[String] = ["pathfinding"]

var _mode: String = DEFAULT_MODE
var _domain_modes: Dictionary = {}


func _ready() -> void:
	_load_settings()
	_ensure_domain_modes()
	_sync_pathfinding_backend_mode()
	_sync_runtime_compute_modes()


## Returns configured compute mode (`cpu`, `gpu_auto`, `gpu_force`).
func get_mode() -> String:
	return _mode


## Sets compute mode and persists to user settings.
func set_mode(new_mode: String) -> void:
	if new_mode not in SUPPORTED_MODES:
		push_warning("[ComputeBackend] Unsupported mode: %s" % new_mode)
		return
	if _mode == new_mode:
		return
	_mode = new_mode
	for i in range(COMPUTE_DOMAINS.size()):
		var domain: String = COMPUTE_DOMAINS[i]
		_domain_modes[domain] = _mode if _domain_supports_gpu(domain) else "cpu"
	_save_settings()
	_sync_pathfinding_backend_mode()
	_queue_runtime_command(StringName("set_compute_mode_all"), {"mode": _mode})
	compute_mode_changed.emit(_mode, resolve_mode())
	_emit_domain_mode_signals()


## Returns configured compute mode for a domain.
func get_mode_for_domain(domain: String) -> String:
	if not _domain_supports_gpu(domain):
		return "cpu"
	if domain in _domain_modes:
		return str(_domain_modes[domain])
	return _mode


## Sets compute mode for a specific domain and persists settings.
func set_mode_for_domain(domain: String, new_mode: String) -> void:
	if domain not in COMPUTE_DOMAINS:
		push_warning("[ComputeBackend] Unsupported compute domain: %s" % domain)
		return
	if new_mode not in SUPPORTED_MODES:
		push_warning("[ComputeBackend] Unsupported mode: %s" % new_mode)
		return
	var normalized_mode: String = new_mode if _domain_supports_gpu(domain) else "cpu"
	var old_mode: String = get_mode_for_domain(domain)
	if old_mode == normalized_mode:
		return
	_domain_modes[domain] = normalized_mode
	_save_settings()
	if domain == "pathfinding":
		_sync_pathfinding_backend_mode()
	_queue_runtime_command(StringName("set_compute_domain_mode"), {
		"domain": domain,
		"mode": normalized_mode,
	})
	compute_domain_mode_changed.emit(domain, normalized_mode, resolve_mode_for_domain(domain))


## Returns resolved execution mode (`cpu` or `gpu`) for the given domain.
func resolve_mode_for_domain(domain: String) -> String:
	if not _domain_supports_gpu(domain):
		return "cpu"
	var configured: String = get_mode_for_domain(domain)
	if configured == "cpu":
		return "cpu"
	if is_gpu_capable():
		return "gpu"
	return "cpu"


## Returns true when current runtime can use GPU compute path.
func is_gpu_capable() -> bool:
	var renderer_method: String = str(ProjectSettings.get_setting(
		"rendering/renderer/rendering_method", "mobile"
	))
	if renderer_method == "compatibility" or renderer_method == "gl_compatibility":
		return false
	var rd: RenderingDevice = RenderingServer.get_rendering_device()
	return rd != null


## Resolves active execution mode at runtime (`cpu` or `gpu`).
func resolve_mode() -> String:
	if _mode == "cpu":
		return "cpu"
	if is_gpu_capable():
		return "gpu"
	return "cpu"


## Returns current configured modes for all compute domains.
func get_domain_modes_snapshot() -> Dictionary:
	return _domain_modes.duplicate(true)


func _load_settings() -> void:
	if not FileAccess.file_exists(SETTINGS_PATH):
		return
	var f: FileAccess = FileAccess.open(SETTINGS_PATH, FileAccess.READ)
	var json: JSON = JSON.new()
	json.parse(f.get_as_text())
	if json.data and json.data.has("compute_mode"):
		var saved_mode: String = str(json.data.compute_mode)
		if saved_mode in SUPPORTED_MODES:
			_mode = saved_mode
	if json.data and json.data.has("compute_domain_modes"):
		var raw_modes: Variant = json.data.compute_domain_modes
		if raw_modes is Dictionary:
			var mode_map: Dictionary = raw_modes
			for i in range(COMPUTE_DOMAINS.size()):
				var domain: String = COMPUTE_DOMAINS[i]
				if not mode_map.has(domain):
					continue
				var saved_domain_mode: String = str(mode_map[domain])
				if saved_domain_mode in SUPPORTED_MODES:
					_domain_modes[domain] = saved_domain_mode


func _save_settings() -> void:
	var data: Dictionary = {}
	if FileAccess.file_exists(SETTINGS_PATH):
		var f_read: FileAccess = FileAccess.open(SETTINGS_PATH, FileAccess.READ)
		var json: JSON = JSON.new()
		json.parse(f_read.get_as_text())
		if json.data:
			data = json.data
	data["compute_mode"] = _mode
	data["compute_domain_modes"] = _domain_modes
	var f_write: FileAccess = FileAccess.open(SETTINGS_PATH, FileAccess.WRITE)
	f_write.store_string(JSON.stringify(data, "  "))


func _resolve_pathfinding_backend_mode() -> String:
	var mode: String = get_mode_for_domain("pathfinding")
	if mode == "cpu":
		return "cpu"
	if mode == "gpu_force":
		return "gpu"
	return "auto"


func _sync_pathfinding_backend_mode() -> void:
	if SimBridge == null:
		return
	if not SimBridge.has_method("set_pathfinding_backend"):
		return
	SimBridge.set_pathfinding_backend(_resolve_pathfinding_backend_mode())


func _ensure_domain_modes() -> void:
	for i in range(COMPUTE_DOMAINS.size()):
		var domain: String = COMPUTE_DOMAINS[i]
		if _domain_modes.has(domain):
			_domain_modes[domain] = str(_domain_modes[domain]) if _domain_supports_gpu(domain) else "cpu"
			continue
		_domain_modes[domain] = _mode if _domain_supports_gpu(domain) else "cpu"


func _emit_domain_mode_signals() -> void:
	for i in range(COMPUTE_DOMAINS.size()):
		var domain: String = COMPUTE_DOMAINS[i]
		var mode: String = get_mode_for_domain(domain)
		compute_domain_mode_changed.emit(domain, mode, resolve_mode_for_domain(domain))


func _sync_runtime_compute_modes() -> void:
	_queue_runtime_command(StringName("set_compute_mode_all"), {"mode": _mode})
	for i in range(COMPUTE_DOMAINS.size()):
		var domain: String = COMPUTE_DOMAINS[i]
		var mode: String = get_mode_for_domain(domain) if _domain_supports_gpu(domain) else "cpu"
		_queue_runtime_command(StringName("set_compute_domain_mode"), {
			"domain": domain,
			"mode": mode,
		})


func _domain_supports_gpu(domain: String) -> bool:
	return domain in GPU_ENABLED_DOMAINS


func _queue_runtime_command(command_id: StringName, payload: Dictionary) -> void:
	var bus_v2: Object = get_node_or_null("/root/SimulationBusV2")
	if bus_v2 == null:
		return
	if not bus_v2.has_method("queue_runtime_command"):
		return
	bus_v2.call("queue_runtime_command", command_id, payload)
