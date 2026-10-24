# settings_store.gd: user interface preferences and learning progress, separate from world saves.
# Bridge methods: none. Saving steward_default affects future new games only through the coordinator.
extends RefCounted

signal changed(key: String, value: Variant)
signal persistence_failed(code: int)

const DEFAULTS: Dictionary = {"edge_pan": false, "invert_pan": false, "ui_scale": 1.0, "helper_on": true, "steward_default": true}
var path: String
var last_error: Error = OK
var _config: ConfigFile = ConfigFile.new()
var _values: Dictionary = DEFAULTS.duplicate()
var _completed: PackedStringArray = []
var _skipped: PackedStringArray = []
var _events: Dictionary = {}
var _observed: PackedStringArray = []
var _steward_observed: PackedStringArray = []

func _init(file_path: String = "") -> void:
	path = resolve_path(file_path)
	load_settings()

static func resolve_path(file_path: String = "") -> String:
	if not file_path.is_empty():
		return file_path
	var test_dir: String = OS.get_environment("WORLDSIM_TEST_SETTINGS_DIR")
	if test_dir.is_empty():
		return "user://settings.cfg"
	if not test_dir.is_absolute_path() or test_dir.begins_with("user://") or test_dir.begins_with("res://"):
		return ""
	return test_dir.path_join("settings-%d.cfg" % OS.get_process_id())

func load_settings() -> Error:
	_config = ConfigFile.new()
	_values = DEFAULTS.duplicate()
	_completed = []
	_skipped = []
	_events = {}
	_observed = []
	_steward_observed = []
	last_error = ERR_INVALID_PARAMETER if path.is_empty() else _config.load(path)
	if last_error == ERR_FILE_NOT_FOUND:
		last_error = OK
		return OK
	if last_error != OK:
		return last_error
	for key: String in DEFAULTS:
		var candidate: Variant = _config.get_value("interface", key, DEFAULTS[key])
		if _valid_value(key, candidate):
			_values[key] = candidate
	_observed = _ids(_config.get_value("learning", "observed", []))
	_steward_observed = _ids(_config.get_value("learning", "steward_observed", []))
	_completed = _ids(_config.get_value("learning", "completed", []))
	_skipped = _ids(_config.get_value("learning", "skipped", []))
	for id: String in _ids(_config.get_value("learning", "events", [])):
		_events[id] = true
	return OK

func value(key: String) -> Variant:
	return _values.get(key)

func set_value(key: String, next_value: Variant) -> bool:
	if not _valid_value(key, next_value):
		return false
	if _values[key] == next_value:
		return true
	_values[key] = next_value
	_config.set_value("interface", key, next_value)
	_persist()
	changed.emit(key, next_value)
	return true

func progress() -> Dictionary:
	return {"completed": _completed.duplicate(), "skipped": _skipped.duplicate(), "events": _events.duplicate(), "observed": _observed.duplicate(), "steward_observed": _steward_observed.duplicate()}

func save_progress(completed: PackedStringArray, skipped: PackedStringArray, events: Dictionary, observed: PackedStringArray = [], steward_observed: PackedStringArray = []) -> Error:
	_completed = completed.duplicate()
	_skipped = skipped.duplicate()
	_events = events.duplicate()
	_observed = observed.duplicate()
	_steward_observed = steward_observed.duplicate()
	_config.set_value("learning", "observed", _observed)
	_config.set_value("learning", "steward_observed", _steward_observed)
	_config.set_value("learning", "completed", _completed)
	_config.set_value("learning", "skipped", _skipped)
	_config.set_value("learning", "events", PackedStringArray(_events.keys()))
	return _persist()

func _persist() -> Error:
	last_error = ERR_INVALID_PARAMETER if path.is_empty() else _config.save(path)
	if last_error != OK:
		persistence_failed.emit(last_error)
	return last_error

static func _valid_value(key: String, candidate: Variant) -> bool:
	if not DEFAULTS.has(key):
		return false
	if key == "ui_scale":
		return (candidate is float or candidate is int) and float(candidate) in [1.0, 1.25, 1.5]
	return candidate is bool

static func _ids(raw: Variant) -> PackedStringArray:
	var result: PackedStringArray = []
	if raw is Array or raw is PackedStringArray:
		for entry: Variant in raw:
			if entry is String and not entry.is_empty() and not result.has(entry):
				result.append(entry)
	return result
