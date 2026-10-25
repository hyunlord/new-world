# controls_catalog.gd: validated data registries, control text and declarative UI conditions.
# Bridge methods: none; conditions consume facts supplied by the read-only UI coordinator.
extends RefCounted

const KoreanText = preload("res://scripts/ui/mux/korean_text.gd")

const Content = preload("res://scripts/ui/content_context.gd")

const PATH: String = "res://assets/ui/controls.json"
var entries: Array[Dictionary] = []
var error: String = ""

func _init(path: String = PATH) -> void:
	var result: Dictionary = read_registry(path, "controls", ["id", "input", "input_key", "text_key", "group", "source"])
	entries.assign(result.get("rows", []))
	error = str(result.get("error", ""))

func tooltip(id: String, translator: Callable = Callable()) -> String:
	for entry: Dictionary in entries:
		if str(entry["id"]) == id:
			return "%s · %s" % [_text(str(entry["text_key"]), translator), input_text(entry, translator)]
	return _text("MUX_CONTROL_UNKNOWN", translator)

func markdown_table(translator: Callable) -> String:
	var lines: PackedStringArray = ["| 입력 | 동작 |", "|---|---|"]
	for entry: Dictionary in entries:
		lines.append("| %s | %s |" % [input_text(entry, translator).replace("|", "\\|"), _text(str(entry["text_key"]), translator).replace("|", "\\|")])
	return "\n".join(lines)

static func input_text(entry: Dictionary, translator: Callable = Callable()) -> String:
	var key: String = str(entry["input_key"])
	var translated: String = _text(key, translator)
	return str(entry["input"]) if translated == key else translated

static func _text(key: String, translator: Callable) -> String:
	if translator.is_valid():
		return str(translator.call(key))
	var tree: SceneTree = Engine.get_main_loop() as SceneTree
	var locale: Node = tree.root.get_node_or_null("Locale") if tree != null else null
	return KoreanText.words(str(locale.call("ltr", key))) if locale != null else key

static func read_registry(path: String, family: String, fields: Array) -> Dictionary:
	var frozen: Dictionary = Content.document(family)
	if path == "res://assets/ui/%s.json" % family and not frozen.is_empty():
		return parse_registry(frozen, family, fields)
	var file: FileAccess = FileAccess.open(path, FileAccess.READ)
	if file == null:
		return {"rows": [], "error": "Cannot open %s" % path}
	return parse_registry(JSON.parse_string(file.get_as_text()), family, fields)

static func parse_registry(raw: Variant, family: String, fields: Array) -> Dictionary:
	if not raw is Dictionary:
		return {"rows": [], "error": "Registry must be an object"}
	var document: Dictionary = raw
	if document.size() != 2 or document.get("schema_version", 0) != 1 or not document.get(family) is Array:
		return {"rows": [], "error": "Unknown schema or fields"}
	var seen: Dictionary = {}
	var rows: Array[Dictionary] = []
	for candidate: Variant in document[family]:
		if not candidate is Dictionary:
			return {"rows": [], "error": "Entry must be an object"}
		var row: Dictionary = candidate
		if row.size() != fields.size():
			return {"rows": [], "error": "Unknown or missing entry fields"}
		for field: String in fields:
			if not row.has(field):
				return {"rows": [], "error": "Missing %s" % field}
			if field in ["trigger", "completion"]:
				if not row[field] is Dictionary or not valid_condition(row[field]):
					return {"rows": [], "error": "Invalid condition"}
			elif not row[field] is String or str(row[field]).is_empty():
				return {"rows": [], "error": "Expected nonempty string"}
		var id: String = str(row.get("id", ""))
		if id.is_empty() or seen.has(id):
			return {"rows": [], "error": "Duplicate or empty id"}
		seen[id] = true
		rows.append(row.duplicate(true))
	return {"rows": rows, "error": ""}

static func valid_condition(rule: Dictionary) -> bool:
	if rule.size() != 1:
		return false
	for group: String in ["all", "any"]:
		if rule.has(group):
			if not rule[group] is Array or rule[group].is_empty():
				return false
			for member: Variant in rule[group]:
				if not member is Dictionary or not valid_condition(member):
					return false
			return true
	for field: String in ["event", "fact"]:
		if rule.has(field):
			return rule[field] is String and not str(rule[field]).is_empty()
	return rule.size() == 1 and rule.get("always") is bool

static func condition_met(rule: Dictionary, facts: Dictionary, events: Dictionary) -> bool:
	if rule.size() != 1:
		return false
	if rule.has("all") and rule["all"] is Array:
		var members: Array = rule["all"]
		if members.is_empty():
			return false
		for member: Variant in members:
			if not member is Dictionary or not condition_met(member, facts, events):
				return false
		return true
	if rule.has("any") and rule["any"] is Array:
		for member: Variant in rule["any"]:
			if member is Dictionary and condition_met(member, facts, events):
				return true
		return false
	if rule.has("fact"):
		return facts.get(str(rule["fact"]), false) == true
	if rule.has("event"):
		return events.get(str(rule["event"]), false) == true
	if rule.has("always"):
		return rule["always"] == true
	return false
