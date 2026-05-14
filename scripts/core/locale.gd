extends Node

# Locale autoload — compiled JSON loader (en+ko)
# V7 Phase 3-γ (γ-2-α): first autoload, consumed by causal_panel.gd
# Primary source: localization/compiled/{lang}.json (output of
# tools/localization_compile.py — flat "strings" map keyed by uppercase ID).
# Fallback: per-category JSON files under localization/{lang}/ if the
# compiled artifact is unavailable (dev convenience).
# Public API:
#   ltr(key)           returns the localized string, or the literal key
#                      itself if missing (visibility fallback during dev).
#   set_language(lang) reloads strings for the new language.
#   key_count()        returns the number of loaded keys.

var _strings: Dictionary = {}
var _current_lang: String = "en"

func _ready() -> void:
	_load_lang(_current_lang)

func _load_lang(lang: String) -> void:
	_strings.clear()
	var compiled_path := "res://localization/compiled/%s.json" % lang
	if _load_compiled(compiled_path):
		return
	_load_category_dir(lang)

func _load_compiled(path: String) -> bool:
	var file := FileAccess.open(path, FileAccess.READ)
	if file == null:
		push_warning("Locale: compiled file unavailable at %s; falling back to category JSON" % path)
		return false
	var json_text := file.get_as_text()
	file.close()
	var parsed: Variant = JSON.parse_string(json_text)
	if not (parsed is Dictionary):
		push_warning("Locale: malformed compiled JSON at %s" % path)
		return false
	var compiled := parsed as Dictionary
	var strings: Variant = compiled.get("strings", null)
	if not (strings is Dictionary):
		push_warning("Locale: compiled JSON missing 'strings' table at %s" % path)
		return false
	_merge_dict(strings as Dictionary)
	return true

func _load_category_dir(lang: String) -> void:
	var dir := DirAccess.open("res://localization/%s/" % lang)
	if dir == null:
		push_error("Locale: cannot open localization/%s/" % lang)
		return
	dir.list_dir_begin()
	var filename := dir.get_next()
	while filename != "":
		if not dir.current_is_dir() and filename.ends_with(".json"):
			var path := "res://localization/%s/%s" % [lang, filename]
			var file := FileAccess.open(path, FileAccess.READ)
			if file:
				var json_text := file.get_as_text()
				file.close()
				var parsed: Variant = JSON.parse_string(json_text)
				if parsed is Dictionary:
					_merge_dict(parsed as Dictionary)
				else:
					push_warning("Locale: invalid JSON in %s" % filename)
		filename = dir.get_next()
	dir.list_dir_end()

func _merge_dict(dict: Dictionary) -> void:
	for key in dict:
		_strings[key] = dict[key]

func ltr(key: String) -> String:
	return _strings.get(key, key)

func set_language(lang: String) -> void:
	_current_lang = lang
	_load_lang(lang)

func key_count() -> int:
	return _strings.size()
