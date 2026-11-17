# Optional generated UI art; missing assets retain each caller's established presentation.
# Bridge methods: none; portrait identity uses caller-supplied immutable snapshots.
extends RefCounted

const ROOT: String = "res://assets/ui/generated/"
const Content = preload("res://scripts/ui/content_context.gd")
const CHRONICLE_ART: String = "res://assets/ui/chronicle_art.json"
static var _textures: Dictionary = {}
static var _chronicle_rules: Array = []


static func texture(family: String, id: String) -> Texture2D:
	if id.is_empty() or not id.is_valid_filename() or not family.is_valid_filename():
		return null
	var path: String = ROOT + family + "/" + id + ".webp"
	if not _textures.has(path):
		_textures[path] = load(path) as Texture2D if ResourceLoader.exists(path) else null
	return _textures[path]


static func image(size: Vector2) -> TextureRect:
	var rect := TextureRect.new()
	rect.name = "GeneratedArt"
	rect.custom_minimum_size = size
	rect.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	rect.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
	rect.texture_filter = CanvasItem.TEXTURE_FILTER_LINEAR
	rect.mouse_filter = Control.MOUSE_FILTER_IGNORE
	return rect


static func set_image(rect: TextureRect, family: String, id: String) -> void:
	rect.texture = texture(family, id)
	rect.visible = rect.texture != null


static func portrait_id(snapshot: Dictionary, bits: int, tech: Dictionary) -> String:
	var ids: Variant = snapshot.get("ids")
	var ages: Variant = snapshot.get("life_stages")
	var sexes: Variant = snapshot.get("sexes")
	if not ids is PackedInt64Array or not ages is PackedByteArray or not sexes is PackedByteArray:
		return ""
	var index: int = ids.find(bits)
	if index < 0 or index >= ages.size() or index >= sexes.size() or sexes[index] > 1:
		return ""
	var age: String
	match int(ages[index]):
		0: age = "adult"
		1: age = "child"
		2: age = "elder"
		_: return ""
	var wardrobe: String = "neolithic"
	var eras: Variant = tech.get("eras", [])
	if not eras is Array:
		eras = []
	var current: Variant = tech.get("current_era_order", 0)
	if not current is int:
		current = 0
	for value: Variant in eras:
		if value is Dictionary and value.get("order", -1) == current:
			var era: String = str(value.get("id", ""))
			if era in ["bronze", "iron"]:
				wardrobe = era
	var sex: String = "female" if sexes[index] == 1 else "male"
	var variant: int = posmod(hash([bits, age, sex, wardrobe]), 8)
	return "%s_%s_%s_%d" % [wardrobe, age, sex, variant]


## Keep the original four looks available when an optional expansion is missing.
static func portrait(snapshot: Dictionary, bits: int, tech: Dictionary) -> Texture2D:
	var id: String = portrait_id(snapshot, bits, tech)
	var selected: Texture2D = texture("portraits", id)
	if selected == null and not id.is_empty():
		selected = texture("portraits", id.left(-1) + str(id.right(1).to_int() % 4))
	return selected


## Rules use native event fields and catalogue identities; unmatched rows stay text-only.
static func chronicle_art(row: Dictionary) -> Dictionary:
	if _chronicle_rules.is_empty():
		var raw: Variant = JSON.parse_string(FileAccess.get_file_as_string(CHRONICLE_ART))
		if raw is Array:
			_chronicle_rules = raw
	for value: Variant in _chronicle_rules:
		if not value is Dictionary or not value.get("match") is Dictionary:
			continue
		var rule: Dictionary = value
		var matches: bool = true
		for key: String in rule["match"]:
			if row.get(key) != rule["match"][key]:
				matches = false
		if not matches:
			continue
		var content: Dictionary = {}
		if rule.has("catalogue"):
			var code: Variant = row.get(str(rule.get("column", "value")))
			if not code is int:
				continue
			content = Content.row(str(rule["catalogue"]), code)
			var field: String = str(rule.get("field", "id"))
			if not content.get(field) in rule.get("values", []):
				continue
		return {"family": str(rule.get("family", "events")), "id": str(rule.get("id", content.get("id", "")))}
	return {}
