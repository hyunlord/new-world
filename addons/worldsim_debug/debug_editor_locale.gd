@tool
class_name DebugEditorLocale
extends RefCounted

const _SETTINGS_PATH: String = "user://settings.json"
const _COMPILED_LOCALE_DIR: String = "res://localization/compiled/"
const _DEFAULT_LOCALE: String = "ko"
const _SUPPORTED_LOCALES: Array[String] = ["ko", "en"]

static var _cached_locale: String = ""
static var _cached_strings: Dictionary = {}


static func ltr(key: String) -> String:
	var locale: String = _load_current_locale()
	if locale != _cached_locale:
		_load_locale_strings(locale)
	return str(_cached_strings.get(key, key))


static func _load_current_locale() -> String:
	if not FileAccess.file_exists(_SETTINGS_PATH):
		return _DEFAULT_LOCALE
	var file: FileAccess = FileAccess.open(_SETTINGS_PATH, FileAccess.READ)
	if file == null:
		return _DEFAULT_LOCALE
	var parsed: Variant = JSON.parse_string(file.get_as_text())
	file.close()
	if parsed is Dictionary:
		var locale: String = str(parsed.get("locale", _DEFAULT_LOCALE))
		if locale in _SUPPORTED_LOCALES:
			return locale
	return _DEFAULT_LOCALE


static func _load_locale_strings(locale: String) -> void:
	_cached_locale = locale
	_cached_strings = {}
	var path: String = _COMPILED_LOCALE_DIR + locale + ".json"
	if not FileAccess.file_exists(path):
		if locale != _DEFAULT_LOCALE:
			_load_locale_strings(_DEFAULT_LOCALE)
		return
	var file: FileAccess = FileAccess.open(path, FileAccess.READ)
	if file == null:
		return
	var parsed: Variant = JSON.parse_string(file.get_as_text())
	file.close()
	if parsed is Dictionary:
		var strings: Variant = parsed.get("strings", {})
		if strings is Dictionary:
			_cached_strings = strings
