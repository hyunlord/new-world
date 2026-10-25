# presentation_bootstrap.gd — engine-free validation and one-shot UI binding.
# Bridge methods: none; Rust supplies a candidate catalogue and frozen selected JSON bytes.
extends RefCounted

const Content = preload("res://scripts/ui/content_context.gd")
const Catalog = preload("res://scripts/ui/mux/controls_catalog.gd")
const Evaluator = preload("res://scripts/ui/mux/alert_evaluator.gd")
const Lessons = preload("res://scripts/ui/mux/learning_helper.gd")
const LocaleScript = preload("res://scripts/core/locale.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
var _error: String = ""


func prepare(dto: Dictionary, selected: Dictionary) -> Dictionary:
	var error: String = Content.validate(dto)
	if not error.is_empty():
		return {"ok": false, "error": error}
	var presentation: Dictionary = {}
	for family: String in ["alerts", "lessons", "controls", "goals"]:
		presentation[family] = _parse(FileAccess.get_file_as_string("res://assets/ui/%s.json" % family), family)
	if not _error.is_empty():
		return {"ok": false, "error": _error}
	var locale_tables: Dictionary = {}
	var additions: Dictionary = {}
	for lang: String in ["en", "ko"]:
		var base: Dictionary = _parse(FileAccess.get_file_as_string("res://localization/compiled/%s.json" % lang), "locale_" + lang)
		if not base.get("strings") is Dictionary:
			return {"ok": false, "error": "Missing compiled locale strings: " + lang}
		locale_tables[lang] = base["strings"].duplicate(true)
		additions[lang] = {}
		if selected.has("locale_" + lang):
			var extra: Dictionary = _parse(selected["locale_" + lang], "locale_" + lang)
			if extra.size() != 2 or extra.get("schema_version") != 1 or not extra.get("strings") is Dictionary:
				return {"ok": false, "error": "Invalid selected locale schema: " + lang}
			for key: Variant in extra["strings"]:
				var value: Variant = extra["strings"][key]
				if not key is String or key.is_empty() or not value is String or value.is_empty():
					return {"ok": false, "error": "Invalid selected locale entry: " + lang}
				if locale_tables[lang].has(key) and locale_tables[lang][key] != value:
					return {"ok": false, "error": "Selected locale overrides bundled key: " + str(key)}
				additions[lang][key] = value
				locale_tables[lang][key] = value
	var alert_vocabulary: Dictionary = _alert_vocabulary(presentation["alerts"])
	var lesson_vocabulary: Dictionary = _lesson_vocabulary(presentation["lessons"]["lessons"])
	if selected.has("alerts"):
		var extension: Dictionary = _parse(selected["alerts"], "alerts")
		if extension.size() != 3 or extension.get("schema_version") != 1 or not extension.get("conditions") is Dictionary or not extension.get("alerts") is Array:
			return {"ok": false, "error": "Selected alerts require schema_version, conditions, alerts"}
		for id: Variant in extension["conditions"]:
			if presentation["alerts"]["conditions"].has(id):
				return {"ok": false, "error": "Duplicate alert condition: " + str(id)}
			presentation["alerts"]["conditions"][id] = extension["conditions"][id]
		presentation["alerts"]["alerts"].append_array(extension["alerts"])
	var evaluator: RefCounted = Evaluator.new()
	if not evaluator.configure(presentation["alerts"]):
		return {"ok": false, "error": str(evaluator.last_error)}
	for condition: Dictionary in presentation["alerts"]["conditions"].values():
		var collection: String = str(condition["collection"])
		if not alert_vocabulary.has(collection):
			return {"ok": false, "error": "Unsupported alert collection: " + collection}
		for predicate: Dictionary in condition["predicates"]:
			for field: String in ["field", "rhs_field", "factor_field"]:
				if predicate.has(field) and not alert_vocabulary[collection].has(predicate[field]):
					return {"ok": false, "error": "Unsupported alert fact: " + str(predicate[field])}
	for definition: Dictionary in presentation["alerts"]["alerts"]:
		if definition["jump_target"] not in Evaluator.TARGETS:
			return {"ok": false, "error": "Unsupported alert jump target"}
	if selected.has("lessons"):
		var extension: Dictionary = _parse(selected["lessons"], "lessons")
		var parsed: Dictionary = Catalog.parse_registry(extension, "lessons", Lessons.FIELDS)
		if not str(parsed["error"]).is_empty():
			return {"ok": false, "error": str(parsed["error"])}
		presentation["lessons"]["lessons"].append_array(extension["lessons"])
	var lessons: Dictionary = Catalog.parse_registry(presentation["lessons"], "lessons", Lessons.FIELDS)
	if not str(lessons["error"]).is_empty():
		return {"ok": false, "error": str(lessons["error"])}
	for lesson: Dictionary in lessons["rows"]:
		for field: String in ["highlight", "steward_rule"]:
			if not lesson_vocabulary[field].has(lesson[field]):
				return {"ok": false, "error": "Unsupported lesson " + field}
		for field: String in ["trigger", "completion"]:
			if not _known_condition(lesson[field], lesson_vocabulary):
				return {"ok": false, "error": "Unsupported lesson fact/event"}
	var material_data: Dictionary = _parse(FileAccess.get_file_as_string("res://assets/ui/content_presentation.json"), "content presentation")
	dto["materials"] = material_data.get("materials", [])
	var labels: Array[String] = []
	for family: String in ["items", "materials", "blueprints", "recipes", "knowledge", "pressures", "animals"]:
		for row: Dictionary in dto.get(family, []):
			labels.append(str(row["name_key"]))
			if not str(row.get("material_format_key", "")).is_empty():
				labels.append(str(row["material_format_key"]))
	for row: Dictionary in presentation["alerts"]["alerts"]:
		for key: String in ["text_key", "explanation_key", "fix_key"]:
			labels.append(str(row[key]))
	for row: Dictionary in lessons["rows"]:
		for key: String in ["title_key", "body_key", "steward_key"]:
			labels.append(str(row[key]))
	for lang: String in ["en", "ko"]:
		for key: String in labels:
			if not locale_tables[lang].has(key):
				return {"ok": false, "error": "Missing " + lang + " label: " + key}
	var styles: Dictionary = _parse(FileAccess.get_file_as_string("res://assets/v8art/materials/construction_styles.json"), "construction styles")
	for blueprint: Dictionary in dto["blueprints"]:
		var kind: String = str(blueprint["completion_category"])
		if kind not in ["wall", "door", "floor"]:
			continue
		var style_id: String = str(blueprint["construction_style"])
		var found: bool = false
		for style: Dictionary in styles.get("styles", []):
			if str(style["kind"]) == kind and (str(style["id"]) == style_id or style.get("sim_blueprints", []).has(style_id)):
				found = true
		if not found:
			return {"ok": false, "error": "Unknown or incompatible construction style: " + style_id}
	var bindings: Dictionary = {}
	if selected.has("bindings"):
		bindings = _parse(selected["bindings"], "bindings")
		var binding_error: String = ArtData.validate_extension(bindings, dto)
		if not binding_error.is_empty():
			return {"ok": false, "error": binding_error}
	if not _error.is_empty():
		return {"ok": false, "error": _error}
	error = Content.install(dto, presentation)
	if not error.is_empty():
		return {"ok": false, "error": error}
	LocaleScript.install_additions(additions)
	ArtData.install_extension(bindings)
	return {"ok": true, "error": ""}


func _parse(text: String, context: String) -> Dictionary:
	var parser: JSON = JSON.new()
	if parser.parse(text) != OK or not parser.data is Dictionary:
		_error = context + ": invalid JSON object: " + parser.get_error_message()
		return {}
	return parser.data


func _alert_vocabulary(registry: Dictionary) -> Dictionary:
	var result: Dictionary = {}
	for condition: Dictionary in registry["conditions"].values():
		var collection: String = condition["collection"]
		if not result.has(collection):
			result[collection] = {"id": true, "x": true, "y": true}
		for predicate: Dictionary in condition["predicates"]:
			for field: String in ["field", "rhs_field", "factor_field"]:
				if predicate.has(field):
					result[collection][predicate[field]] = true
	return result


func _lesson_vocabulary(rows: Array) -> Dictionary:
	var result: Dictionary = {"highlight": {}, "steward_rule": {}, "fact": {}, "event": {}}
	for lesson: Dictionary in rows:
		for field: String in ["highlight", "steward_rule"]:
			result[field][lesson[field]] = true
		_collect_condition(lesson["trigger"], result)
		_collect_condition(lesson["completion"], result)
	return result


func _collect_condition(condition: Dictionary, vocabulary: Dictionary) -> void:
	for group: String in ["all", "any"]:
		for member: Dictionary in condition.get(group, []):
			_collect_condition(member, vocabulary)
	for field: String in ["fact", "event"]:
		if condition.has(field):
			vocabulary[field][condition[field]] = true


func _known_condition(condition: Dictionary, vocabulary: Dictionary) -> bool:
	for group: String in ["all", "any"]:
		for member: Dictionary in condition.get(group, []):
			if not _known_condition(member, vocabulary):
				return false
	for field: String in ["fact", "event"]:
		if condition.has(field) and not vocabulary[field].has(condition[field]):
			return false
	return true
