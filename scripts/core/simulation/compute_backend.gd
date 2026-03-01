extends Node

signal compute_mode_changed(new_mode: String, resolved_mode: String)

const SETTINGS_PATH: String = "user://settings.json"
const SUPPORTED_MODES: Array[String] = ["cpu", "gpu_auto", "gpu_force"]
const DEFAULT_MODE: String = "gpu_auto"

var _mode: String = DEFAULT_MODE


func _ready() -> void:
	_load_settings()
	_sync_pathfinding_backend_mode()


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
	_save_settings()
	_sync_pathfinding_backend_mode()
	compute_mode_changed.emit(_mode, resolve_mode())


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


func _save_settings() -> void:
	var data: Dictionary = {}
	if FileAccess.file_exists(SETTINGS_PATH):
		var f_read: FileAccess = FileAccess.open(SETTINGS_PATH, FileAccess.READ)
		var json: JSON = JSON.new()
		json.parse(f_read.get_as_text())
		if json.data:
			data = json.data
	data["compute_mode"] = _mode
	var f_write: FileAccess = FileAccess.open(SETTINGS_PATH, FileAccess.WRITE)
	f_write.store_string(JSON.stringify(data, "  "))


func _resolve_pathfinding_backend_mode() -> String:
	if _mode == "cpu":
		return "cpu"
	if _mode == "gpu_force":
		return "gpu"
	return "auto"


func _sync_pathfinding_backend_mode() -> void:
	if SimBridge == null:
		return
	if not SimBridge.has_method("set_pathfinding_backend"):
		return
	SimBridge.set_pathfinding_backend(_resolve_pathfinding_backend_mode())
