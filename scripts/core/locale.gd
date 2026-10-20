# locale.gd: the `Locale` autoload (project.godot); loads the localized string table and serves ltr() to the
#   HUD, panels and renderers; set_language() reloads and emits language_changed.
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
#   trf(key, params)   ltr(key) with "{name}" placeholders replaced (M1).
# Bridge methods: none.
extends Node

signal language_changed(language: String)

var _strings: Dictionary = {}
var _current_lang: String = "ko"

func _ready() -> void:
	var file: FileAccess = FileAccess.open("res://localization/manifest.json", FileAccess.READ)
	if file != null:
		var parsed: Variant = JSON.parse_string(file.get_as_text())
		if parsed is Dictionary:
			_current_lang = str(parsed.get("default_locale", "ko"))
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

## ltr(key) with every "{name}" placeholder replaced by str(params[name]) (M1).
## Placeholders without a parameter stay as they are; extra parameters are ignored.
func trf(key: String, params: Dictionary) -> String:
	return format_text(ltr(key), params)

## The placeholder substitution behind trf(), on any text (pure; unit-tested).
static func format_text(text: String, params: Dictionary) -> String:
	var out: String = text
	for name: Variant in params.keys():
		out = out.replace("{%s}" % str(name), str(params[name]))
	return out

func set_language(lang: String) -> void:
	_current_lang = lang
	_load_lang(lang)
	language_changed.emit(lang)

func key_count() -> int:
	return _strings.size()
