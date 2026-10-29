# Data-defined colony alert predicates, occurrence dismissal and stable cause cycling.
# Bridge methods: none directly; the panel supplies get_ui_facts through SimReads.
extends RefCounted

const Content = preload("res://scripts/ui/content_context.gd")

const INTERVAL: float = 0.5
const OPERATORS: Array[String] = ["eq", "ne", "lt", "lte", "gt", "gte"]
const TARGETS: Array[String] = ["tile", "person", "room", "technique", "animal", "migrant", "trader"]

var last_error: String = ""
var data_error: String = ""
var _registry: Dictionary = {}
var _matches: Dictionary = {}
var _dismissed: Dictionary = {}
var _cursors: Dictionary = {}
var _elapsed: float = INTERVAL


func load_registry(path: String = "res://assets/ui/alerts.json") -> bool:
	var frozen: Dictionary = Content.document("alerts")
	if path == "res://assets/ui/alerts.json" and not frozen.is_empty():
		return configure(frozen)
	if not FileAccess.file_exists(path):
		last_error = "missing alert registry: " + path
		return false
	var parsed: Variant = JSON.parse_string(FileAccess.get_file_as_string(path))
	if not parsed is Dictionary:
		last_error = "alert registry must be an object"
		return false
	return configure(parsed)


func configure(registry: Dictionary) -> bool:
	last_error = _validate(registry)
	if not last_error.is_empty():
		return false
	_registry = registry.duplicate(true)
	reset()
	return true


func reset() -> void:
	_matches.clear()
	_dismissed.clear()
	_cursors.clear()
	_elapsed = INTERVAL
	data_error = ""


func poll(delta: float, read_facts: Callable) -> bool:
	_elapsed += maxf(0.0, delta)
	if _elapsed < INTERVAL or not read_facts.is_valid():
		return false
	_elapsed = 0.0
	var facts: Variant = read_facts.call()
	if not facts is Dictionary:
		data_error = "UI facts must be an object"
		return true
	evaluate(facts)
	return true


func evaluate(facts: Dictionary) -> bool:
	data_error = ""
	if _registry.is_empty() or facts.get("schema_version") != 1:
		data_error = "unsupported UI facts schema or missing alert registry"
		return false
	var next_matches: Dictionary = {}
	for definition: Dictionary in _registry["alerts"]:
		var condition: Dictionary = _registry["conditions"][definition["condition_id"]]
		var rows: Variant = facts.get(condition["collection"])
		if not rows is Array:
			data_error = "missing facts collection: " + str(condition["collection"])
			return false
		var causes: Array[Dictionary] = []
		for row: Variant in rows:
			if not row is Dictionary or not row.has_all(["id", "x", "y"]):
				data_error = "cause needs id, x and y"
				return false
			if not _number(row["id"]) or not _number(row["x"]) or not _number(row["y"]):
				data_error = "cause identity and coordinates must be numeric"
				return false
			if _matches_condition(row, condition["predicates"]):
				causes.append(row.duplicate(true))
		if not data_error.is_empty():
			return false
		causes.sort_custom(func(a: Dictionary, b: Dictionary) -> bool:
			if a["id"] != b["id"]:
				return a["id"] < b["id"]
			return a["x"] < b["x"] if a["x"] != b["x"] else a["y"] < b["y"])
		next_matches[definition["id"]] = causes
	for id: String in next_matches:
		if next_matches[id].is_empty():
			_dismissed.erase(id)
			_cursors.erase(id)
	_matches = next_matches
	return true


func active_alerts() -> Array[Dictionary]:
	var result: Array[Dictionary] = []
	for definition: Dictionary in _registry.get("alerts", []):
		var id: String = definition["id"]
		var causes: Array = _matches.get(id, [])
		if causes.is_empty() or _dismissed.has(id):
			continue
		var row: Dictionary = definition.duplicate(true)
		var severity: Dictionary = _registry["severities"].get(row["severity"], _registry["severities"][_registry["fallback_severity"]])
		row.merge(severity, true)
		row["count"] = causes.size()
		row["facts"] = causes[0]
		result.append(row)
	result.sort_custom(func(a: Dictionary, b: Dictionary) -> bool:
		return a["rank"] < b["rank"] if a["rank"] != b["rank"] else a["id"] < b["id"])
	return result


func dismiss(id: String) -> void:
	if _matches.has(id) and not _matches[id].is_empty():
		_dismissed[id] = true


func next_target(id: String) -> Dictionary:
	var causes: Array = _matches.get(id, [])
	if causes.is_empty() or _dismissed.has(id):
		return {}
	var index: int = int(_cursors.get(id, 0)) % causes.size()
	_cursors[id] = index + 1
	var target: Dictionary = causes[index].duplicate(true)
	for definition: Dictionary in _registry["alerts"]:
		if definition["id"] == id:
			var kind: String = definition["jump_target"]
			target["kind"] = kind if kind in TARGETS else _registry["fallback_jump_target"]
			if target["kind"] == "technique" and target.has("node_id"):
				target["id"] = target["node_id"]
			return target
	return {}


func _matches_condition(row: Dictionary, predicates: Array) -> bool:
	var matched: bool = true
	for predicate: Dictionary in predicates:
		var left: Variant = row.get(predicate["field"])
		var right: Variant = predicate.get("value")
		if predicate.has("rhs_field"):
			right = row.get(predicate["rhs_field"])
			var factor: Variant = row.get(predicate["factor_field"]) if predicate.has("factor_field") else 1
			if not _number(right) or not _number(factor):
				data_error = "numeric comparison facts missing"
				return false
			right = float(right) * float(predicate.get("multiplier", 1)) * float(factor)
		if left == null or right == null:
			data_error = "comparison facts missing: " + str(predicate["field"])
			return false
		match predicate["op"]:
			"eq": matched = matched and left == right
			"ne": matched = matched and left != right
			_:
				if not _number(left) or not _number(right):
					data_error = "ordered comparison requires numbers"
					return false
				match predicate["op"]:
					"lt": matched = matched and float(left) < float(right)
					"lte": matched = matched and float(left) <= float(right)
					"gt": matched = matched and float(left) > float(right)
					"gte": matched = matched and float(left) >= float(right)
	return matched


func _number(value: Variant) -> bool:
	return value is int or value is float


func _keys(value: Dictionary, required: Array, optional: Array = []) -> bool:
	if not value.has_all(required):
		return false
	for key: Variant in value:
		if key not in required and key not in optional:
			return false
	return true


func _validate(registry: Dictionary) -> String:
	if not _keys(registry, ["schema_version", "fallback_severity", "fallback_jump_target", "severities", "conditions", "alerts", "fact_selectors"]) or registry["schema_version"] != 1:
		return "unsupported alert schema or unknown root field"
	if not registry["fact_selectors"] is Dictionary or not _keys(registry["fact_selectors"], ["spoilable_kind", "spoil_horizon_ticks"]):
		return "invalid fact selectors"
	for value: Variant in registry["fact_selectors"].values():
		if not _number(value) or value < 0:
			return "invalid fact selector value"
	if not registry["severities"] is Dictionary or not registry["conditions"] is Dictionary or not registry["alerts"] is Array:
		return "invalid alert registry containers"
	if not registry["severities"].has(registry["fallback_severity"]) or registry["fallback_jump_target"] not in TARGETS:
		return "missing category fallback"
	for severity: Variant in registry["severities"].values():
		if not severity is Dictionary or not _keys(severity, ["icon", "color", "rank"]):
			return "invalid severity fields"
		if not severity["icon"] is String or not severity["color"] is String or not Color.html_is_valid(severity["color"]) or not _number(severity["rank"]):
			return "invalid severity values"
	for condition: Variant in registry["conditions"].values():
		if not condition is Dictionary or not _keys(condition, ["collection", "predicates"]) or not condition["collection"] is String or not condition["predicates"] is Array:
			return "invalid condition"
		for predicate: Variant in condition["predicates"]:
			if not predicate is Dictionary or not _keys(predicate, ["field", "op"], ["value", "rhs_field", "multiplier", "factor_field"]):
				return "invalid predicate fields"
			if not predicate["field"] is String or predicate["op"] not in OPERATORS or predicate.has("value") == predicate.has("rhs_field"):
				return "invalid predicate operation"
			if predicate.has("multiplier") and not predicate.has("rhs_field"):
				return "literal predicate cannot have multiplier"
			if predicate.has("rhs_field") and (not predicate["rhs_field"] is String or not _number(predicate.get("multiplier", 1))):
				return "invalid predicate reference"
			if predicate.has("factor_field") and (not predicate.has("rhs_field") or not predicate["factor_field"] is String):
				return "invalid predicate factor"
			if predicate.has("value") and not (_number(predicate["value"]) or predicate["value"] is bool or predicate["value"] is String):
				return "invalid predicate literal"
	var ids: Dictionary = {}
	for definition: Variant in registry["alerts"]:
		if not definition is Dictionary or not _keys(definition, ["id", "condition_id", "severity", "text_key", "explanation_key", "fix_key", "jump_target"]):
			return "invalid alert fields"
		for value: Variant in definition.values():
			if not value is String or value.is_empty():
				return "alert fields must be nonempty strings"
		if ids.has(definition["id"]) or not registry["conditions"].has(definition["condition_id"]):
			return "duplicate alert id or unknown condition"
		ids[definition["id"]] = true
	return ""
