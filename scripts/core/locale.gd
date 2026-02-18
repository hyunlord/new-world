extends Node

## Autoload: Locale
## All text lookups go through this singleton.
## No human-readable text in code — only keys.

signal locale_changed(new_locale: String)

const LOCALES_DIR: String = "res://localization/"
const SETTINGS_PATH: String = "user://settings.json"
const SUPPORTED_LOCALES: Array = ["ko", "en"]
const DEFAULT_LOCALE: String = "ko"

var current_locale: String = DEFAULT_LOCALE

## Loaded translation data: { "ui": {"UI_SAVE": "...", ...}, "game": {...}, ... }
var _strings: Dictionary = {}

## All category file names (no extension)
var _categories: Array = ["ui", "game", "traits", "emotions", "events", "deaths", "buildings", "tutorial", "debug"]


func _ready() -> void:
	_load_settings()
	load_locale(current_locale)


## Switch locale at runtime
func set_locale(locale: String) -> void:
	if locale not in SUPPORTED_LOCALES:
		push_warning("[Locale] Unsupported locale: %s" % locale)
		return
	if locale == current_locale:
		return
	current_locale = locale
	load_locale(locale)
	_save_settings()
	locale_changed.emit(locale)
	print("[Locale] Changed to: %s" % locale)


## Load all translation files for a locale
func load_locale(locale: String) -> void:
	_strings.clear()
	for cat in _categories:
		var path = LOCALES_DIR + locale + "/" + cat + ".json"
		if not FileAccess.file_exists(path):
			path = LOCALES_DIR + "en/" + cat + ".json"
		if not FileAccess.file_exists(path):
			_strings[cat] = {}
			continue
		var file = FileAccess.open(path, FileAccess.READ)
		var json = JSON.new()
		json.parse(file.get_as_text())
		_strings[cat] = json.data if json.data else {}
	print("[Locale] Loaded %s: %d categories" % [locale, _strings.size()])


## Lookup translation string by key (searches all categories)
func ltr(key: String) -> String:
	for cat in _strings:
		if key in _strings[cat]:
			return str(_strings[cat][key])
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


## Get localized field from JSON data Dictionary
## Example: Locale.tr_data(trait_def, "name") -> name_kr or name_en
func tr_data(data: Dictionary, field: String = "name") -> String:
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
	var f = FileAccess.open(SETTINGS_PATH, FileAccess.READ)
	var json = JSON.new()
	json.parse(f.get_as_text())
	if json.data and json.data.has("locale"):
		current_locale = str(json.data.locale)


func _save_settings() -> void:
	var data = {}
	if FileAccess.file_exists(SETTINGS_PATH):
		var f = FileAccess.open(SETTINGS_PATH, FileAccess.READ)
		var json = JSON.new()
		json.parse(f.get_as_text())
		if json.data:
			data = json.data
	data["locale"] = current_locale
	var f = FileAccess.open(SETTINGS_PATH, FileAccess.WRITE)
	f.store_string(JSON.stringify(data, "  "))
