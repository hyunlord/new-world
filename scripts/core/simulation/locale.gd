extends Node

## Autoload: Locale
## All text lookups go through this singleton.
## No human-readable text in code — only keys.

signal locale_changed(new_locale: String)

const LOCALES_DIR: String = "res://localization/"
const SETTINGS_PATH: String = "user://settings.json"
const MANIFEST_PATH: String = LOCALES_DIR + "manifest.json"
const SUPPORTED_LOCALES: Array = ["ko", "en"]
const DEFAULT_LOCALE: String = "ko"
const COMPILED_DIR_DEFAULT: String = "compiled"

var current_locale: String = DEFAULT_LOCALE
var _supported_locales: Array = SUPPORTED_LOCALES.duplicate()
var _default_locale: String = DEFAULT_LOCALE
var _compiled_dir: String = COMPILED_DIR_DEFAULT

## Loaded translation data: { "ui": {"UI_SAVE": "...", ...}, "game": {...}, ... }
var _strings: Dictionary = {}
## Flattened lookup table: { "UI_SAVE": "...", "TECH_FIRE": "...", ... }
var _flat_strings: Dictionary = {}

## All category file names (no extension)
var _categories: Array = ["ui", "game", "traits", "emotions", "events", "deaths", "buildings", "tutorial", "debug", "coping", "childhood", "reputation", "economy", "tech", "data_generated"]


func _ready() -> void:
	_load_manifest()
	_load_settings()
	load_locale(current_locale)


## Switch locale at runtime
func set_locale(locale: String) -> void:
	if locale not in _supported_locales:
		push_warning("[Locale] Unsupported locale: %s" % locale)
		return
	if locale == current_locale:
		return
	current_locale = locale
	load_locale(locale)
	_save_settings()
	locale_changed.emit(locale)


## Load all translation files for a locale
func load_locale(locale: String) -> void:
	if locale not in _supported_locales:
		locale = _default_locale
	current_locale = locale
	_strings.clear()
	_flat_strings.clear()
	if _load_compiled_locale(locale):
		return

	for cat in _categories:
		var path: String = LOCALES_DIR + locale + "/" + cat + ".json"
		if not FileAccess.file_exists(path):
			path = LOCALES_DIR + "en/" + cat + ".json"
		if not FileAccess.file_exists(path):
			_strings[cat] = {}
			continue
		var file: FileAccess = FileAccess.open(path, FileAccess.READ)
		var json: JSON = JSON.new()
		json.parse(file.get_as_text())
		var cat_data: Dictionary = json.data if json.data else {}
		_strings[cat] = cat_data
		var keys: Array = cat_data.keys()
		for i in range(keys.size()):
			var key: String = str(keys[i])
			if not _flat_strings.has(key):
				_flat_strings[key] = str(cat_data[key])


## Lookup translation string by key (searches all categories)
func ltr(key: String) -> String:
	if _flat_strings.has(key):
		return str(_flat_strings[key])
	return key


## Format string with placeholder substitution
## Example: Locale.trf("EVT_CHILD_BORN", {"name": "Aria", "mother": "Bea", "father": "Cal"})
func trf(key: String, params: Dictionary = {}) -> String:
	var text = ltr(key)
	for p in params:
		text = text.replace("{%s}" % p, str(params[p]))
	return text


## Game internal ID -> translation (job, status, death cause, etc.)
## Example: Locale.tr_id("STATUS", "gather_wood") -> "Gather Wood" or "목재 채집"
func tr_id(prefix: String, id: String) -> String:
	var key = prefix + "_" + id.to_upper()
	var result = ltr(key)
	if result == key:
		return id
	return result


## @deprecated: tr_data()는 폐기됨. ltr(data["name_key"]) 또는 ltr(data["desc_key"])를 사용하라.
## name_key / desc_key가 있으면 ltr()로 위임. 없으면 하위 호환 fallback.
func tr_data(data: Dictionary, field: String = "name") -> String:
	push_warning("[Locale] tr_data() is deprecated. Use ltr(data['name_key']) instead.")
	if field == "name" and "name_key" in data:
		return ltr(str(data["name_key"]))
	if field == "description" and "desc_key" in data:
		return ltr(str(data["desc_key"]))
	var key_lookup: String = field + "_key"
	if key_lookup in data:
		return ltr(str(data[key_lookup]))
	var key_direct = field + "_" + current_locale
	if key_direct in data:
		return str(data[key_direct])
	if current_locale == "ko":
		var key_kr = field + "_kr"
		if key_kr in data:
			return str(data[key_kr])
	var key_en = field + "_en"
	if key_en in data:
		return str(data[key_en])
	return str(data.get(field, "???"))


## Get month name from 1-based month number
func get_month_name(month: int) -> String:
	var key = "MONTH_%d" % clampi(month, 1, 12)
	return ltr(key)


func _load_settings() -> void:
	if not FileAccess.file_exists(SETTINGS_PATH):
		return
	var f: FileAccess = FileAccess.open(SETTINGS_PATH, FileAccess.READ)
	var json: JSON = JSON.new()
	json.parse(f.get_as_text())
	if json.data and json.data.has("locale"):
		current_locale = str(json.data.locale)
	if current_locale not in _supported_locales:
		current_locale = _default_locale


func _save_settings() -> void:
	var data: Dictionary = {}
	if FileAccess.file_exists(SETTINGS_PATH):
		@warning_ignore("confusable_local_declaration")
		var f: FileAccess = FileAccess.open(SETTINGS_PATH, FileAccess.READ)
		var json: JSON = JSON.new()
		json.parse(f.get_as_text())
		if json.data is Dictionary:
			data = json.data
	data["locale"] = current_locale
	@warning_ignore("confusable_local_declaration")
	var f: FileAccess = FileAccess.open(SETTINGS_PATH, FileAccess.WRITE)
	f.store_string(JSON.stringify(data, "  "))


func _load_manifest() -> void:
	if not FileAccess.file_exists(MANIFEST_PATH):
		return

	var file: FileAccess = FileAccess.open(MANIFEST_PATH, FileAccess.READ)
	var json: JSON = JSON.new()
	var parse_err: int = json.parse(file.get_as_text())
	if parse_err != OK:
		push_warning("[Locale] Failed to parse manifest: %s" % MANIFEST_PATH)
		return
	if not (json.data is Dictionary):
		return

	var manifest: Dictionary = json.data
	if manifest.has("default_locale"):
		_default_locale = str(manifest["default_locale"])
		if current_locale == DEFAULT_LOCALE:
			current_locale = _default_locale

	if manifest.has("supported_locales") and manifest["supported_locales"] is Array:
		var parsed_locales: Array = []
		var raw_locales: Array = manifest["supported_locales"]
		for i in range(raw_locales.size()):
			parsed_locales.append(str(raw_locales[i]))
		if parsed_locales.size() > 0:
			_supported_locales = parsed_locales

	if manifest.has("categories_order") and manifest["categories_order"] is Array:
		var parsed_categories: Array = []
		var raw_categories: Array = manifest["categories_order"]
		for i in range(raw_categories.size()):
			parsed_categories.append(str(raw_categories[i]))
		if parsed_categories.size() > 0:
			_categories = parsed_categories

	if manifest.has("compiled_dir"):
		_compiled_dir = str(manifest["compiled_dir"])

	if current_locale not in _supported_locales:
		current_locale = _default_locale


func _load_compiled_locale(locale: String) -> bool:
	var path: String = LOCALES_DIR + _compiled_dir + "/" + locale + ".json"
	if not FileAccess.file_exists(path):
		if locale != "en":
			path = LOCALES_DIR + _compiled_dir + "/en.json"
		if not FileAccess.file_exists(path):
			return false

	var file: FileAccess = FileAccess.open(path, FileAccess.READ)
	var json: JSON = JSON.new()
	var parse_err: int = json.parse(file.get_as_text())
	if parse_err != OK:
		return false
	if not (json.data is Dictionary):
		return false

	var root: Dictionary = json.data
	if not root.has("strings"):
		return false
	if not (root["strings"] is Dictionary):
		return false

	var strings: Dictionary = root["strings"]
	_strings["compiled"] = strings
	var keys: Array = strings.keys()
	for i in range(keys.size()):
		var key: String = str(keys[i])
		_flat_strings[key] = str(strings[key])
	return true
