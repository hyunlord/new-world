# material_resolver.gd — the material system's resolver: (construction style, simulation material) → what to draw.
# Purpose: a GDScript port of the art line's pure resolver (`art/materials/resolve.py` at v8/art-polish2 c8ed8608) over
#   the imported JSON (assets/v8art/materials/{construction_styles,material_looks,category_defaults}.json), plus the
#   loader's relational checks from `ADDING_A_STYLE.md` ("M-ART2 loading contract"): schema_version exactly 1, unique
#   style / material / category ids and raw material ids, kind defaults that exist and have the right kind, ends ==
#   textures.end and corner == textures.junction, every texture path imported. No style or material is named here:
#   an era is added by data (ADR 0033). Resolution order — style: exact style id → a style whose `sim_blueprints`
#   holds the id → the material category's kind default → the document's kind default; tint: material → category →
#   style; a material's `texture_override` for the chosen style merges over the style's textures. Material ids are
#   the canonical key (String) or the simulation's raw u32 `MaterialId` (int; djb2 of the key).
#   `resolve()` returns {style_id, material_id, material_key, category, kind, textures {role: art path},
#   tint {dL, da, db}, tint_source, style_source, top, face}; `res_path(art path)` maps a texture to its import.
#   The parity test (gate step resolver_parity) checks this file against resolve.py on a fixture.
# Bridge methods called: none.
extends RefCounted

const ArtData = preload("res://scripts/world/art/art_data.gd")
const KINDS: Array[String] = ["wall", "floor", "door"]

var styles: Array[Dictionary] = []
var materials: Array[Dictionary] = []
var categories: Array[Dictionary] = []
var default_styles: Dictionary = {}
## Problems found while loading (empty = the documents satisfy the loading contract).
var problems: Array[String] = []
## Field defaults of the schema's `Top` and `Face` records (a style without them, or missing a field, gets these).
var top_defaults: Dictionary = {}
var face_defaults: Dictionary = {}


## Read the `Top` / `Face` defaults from the construction-style JSON Schema (its `$defs`).
func load_schema_defaults(schema: Dictionary) -> void:
	var defs: Dictionary = schema.get("$defs", {})
	top_defaults = _defaults_of(defs.get("Top", {}))
	face_defaults = _defaults_of(defs.get("Face", {}))


func _defaults_of(record: Dictionary) -> Dictionary:
	var out: Dictionary = {}
	var props: Dictionary = record.get("properties", {})
	for key: String in props.keys():
		var p: Dictionary = props[key]
		if p.has("default"):
			out[key] = p["default"]
	return out


func _with_defaults(value: Variant, defaults: Dictionary) -> Dictionary:
	var out: Dictionary = defaults.duplicate()
	if value is Dictionary:
		for key: String in (value as Dictionary).keys():
			out[key] = (value as Dictionary)[key]
	return out


## Load from the three documents (Dictionaries as parsed from JSON); `problems` lists contract violations.
func load_documents(construction: Dictionary, looks: Dictionary, defaults: Dictionary) -> void:
	problems.clear()
	styles.clear()
	materials.clear()
	categories.clear()
	for doc: Dictionary in [construction, looks, defaults]:
		if int(doc.get("schema_version", 0)) != 1 or typeof(doc.get("schema_version")) == TYPE_STRING:
			problems.append("schema_version is not 1")
	for s: Variant in construction.get("styles", []):
		if s is Dictionary:
			styles.append(s)
	for m: Variant in looks.get("materials", []):
		if m is Dictionary:
			materials.append(m)
	for c: Variant in defaults.get("categories", []):
		if c is Dictionary:
			categories.append(c)
	default_styles = construction.get("default_styles", {})
	_check()


## Load from the imported files under assets/v8art/materials/.
func load_imported() -> void:
	var base: String = ArtData.ROOT + "materials/"
	load_schema_defaults(_json(base + "construction_styles.schema.json"))
	load_documents(_json(base + "construction_styles.json"), _json(base + "material_looks.json"), _json(base + "category_defaults.json"))


func _json(path: String) -> Dictionary:
	var parsed: Variant = JSON.parse_string(FileAccess.get_file_as_string(path))
	return parsed if parsed is Dictionary else {}


func _check() -> void:
	var ids: Dictionary = {}
	for s: Dictionary in styles:
		var id: String = str(s.get("id", ""))
		if ids.has(id):
			problems.append("duplicate style id %s" % id)
		ids[id] = true
		if not KINDS.has(str(s.get("kind", ""))):
			problems.append("style %s has an unknown kind" % id)
		var tex: Dictionary = s.get("textures", {})
		if str(s.get("kind", "")) == "wall":
			if str(s.get("ends", "")) != str(tex.get("end", "")):
				problems.append("style %s: ends differs from textures.end" % id)
			if str(s.get("corner", "")) != str(tex.get("junction", "")):
				problems.append("style %s: corner differs from textures.junction" % id)
		for role: String in tex.keys():
			var p: String = str(tex[role])
			if p == "" or p.begins_with("/") or p.contains("..") or p.contains("\\") or p.contains(":") or p.contains(" "):
				problems.append("style %s: bad path %s" % [id, p])
	for kind: String in KINDS:
		var d: String = str(default_styles.get(kind, ""))
		if _style_by_id(d, kind).is_empty():
			problems.append("default %s style %s is missing or of another kind" % [kind, d])
	var keys: Dictionary = {}
	var raws: Dictionary = {}
	for m: Dictionary in materials:
		var key: String = str(m.get("id", ""))
		var raw: int = int(m.get("sim_id", -1))
		if keys.has(key):
			problems.append("duplicate material id %s" % key)
		if raws.has(raw):
			problems.append("duplicate raw material id %d" % raw)
		keys[key] = true
		raws[raw] = true
		var cat: String = str(m.get("category", ""))
		if _category(cat).is_empty():
			problems.append("material %s has an unknown category %s" % [key, cat])
		var overrides: Variant = m.get("texture_override", {})
		if overrides is Dictionary:
			for sid: String in (overrides as Dictionary).keys():
				if not ids.has(sid):
					problems.append("material %s overrides unknown style %s" % [key, sid])
	var cats: Dictionary = {}
	for c: Dictionary in categories:
		var name: String = str(c.get("category", ""))
		if cats.has(name):
			problems.append("duplicate category %s" % name)
		cats[name] = true
		var kd: Dictionary = c.get("kind_defaults", {})
		for kind: String in KINDS:
			if _style_by_id(str(kd.get(kind, "")), kind).is_empty():
				problems.append("category %s: %s default missing or of another kind" % [name, kind])
		if _style_by_id(str(c.get("default_style", "")), "wall").is_empty():
			problems.append("category %s: default_style is not a wall style" % name)
	var blueprints: Dictionary = {}
	for s: Dictionary in styles:
		for bp: Variant in s.get("sim_blueprints", []):
			var k: String = "%s/%s" % [str(s.get("kind", "")), str(bp)]
			if blueprints.has(k):
				problems.append("blueprint %s maps to two styles" % k)
			blueprints[k] = true


func _style_by_id(id: String, kind: String) -> Dictionary:
	for s: Dictionary in styles:
		if str(s.get("id", "")) == id and str(s.get("kind", "")) == kind:
			return s
	return {}


func _category(name: String) -> Dictionary:
	for c: Dictionary in categories:
		if str(c.get("category", "")) == name:
			return c
	return {}


## The material row whose key or raw id is `material_id` (empty when unknown).
func material(material_id: Variant) -> Dictionary:
	for m: Dictionary in materials:
		if material_id is String and str(m.get("id", "")) == material_id:
			return m
		if (material_id is int) and int(m.get("sim_id", -1)) == int(material_id):
			return m
	return {}


## Resolve without I/O or mutation (resolve.py `resolve`); `category` is the simulation's category for a material
## the data does not list ("" when unknown).
func resolve(style_id: String, material_id: Variant, category: String = "", kind: String = "wall") -> Dictionary:
	var mat: Dictionary = material(material_id)
	var category_id: String = str(mat.get("category", "")) if not mat.is_empty() else category
	var fallback: Dictionary = _category(category_id)
	var kind_styles: Array[Dictionary] = []
	for s: Dictionary in styles:
		if str(s.get("kind", "")) == kind:
			kind_styles.append(s)
	var style: Dictionary = {}
	var style_source: String = "style"
	for s: Dictionary in kind_styles:
		if str(s.get("id", "")) == style_id:
			style = s
			break
	if style.is_empty():
		style_source = "blueprint"
		for s: Dictionary in kind_styles:
			if (s.get("sim_blueprints", []) as Array).has(style_id):
				style = s
				break
	if style.is_empty():
		var default_id: String = str((fallback.get("kind_defaults", {}) as Dictionary).get(kind, "")) if not fallback.is_empty() else str(default_styles.get(kind, ""))
		style = _style_by_id(default_id, kind)
		style_source = "category" if not fallback.is_empty() else "default"
	var tint: Dictionary = style.get("default_tint", {"dL": 0, "da": 0, "db": 0})
	var tint_source: String = "style"
	if not fallback.is_empty():
		tint = fallback.get("tint", tint)
		tint_source = "category"
	if not mat.is_empty() and mat.get("tint", null) is Dictionary:
		tint = mat["tint"]
		tint_source = "material"
	var textures: Dictionary = (style.get("textures", {}) as Dictionary).duplicate()
	if not mat.is_empty():
		var overrides: Variant = mat.get("texture_override", {})
		if overrides is Dictionary and (overrides as Dictionary).has(str(style.get("id", ""))):
			var o: Dictionary = (overrides as Dictionary)[str(style.get("id", ""))]
			for role: String in o.keys():
				if o[role] != null:
					textures[role] = o[role]
	return {
		"style_id": str(style.get("id", "")),
		"material_id": material_id,
		"material_key": mat.get("id", null) if not mat.is_empty() else null,
		"category": category_id if category_id != "" else null,
		"kind": kind,
		"textures": textures,
		"tint": tint,
		"tint_source": tint_source,
		"style_source": style_source,
		"top": _with_defaults(style.get("top", null), top_defaults),
		"face": _with_defaults(style.get("face", null), face_defaults),
	}


## The res:// path of an art texture path named by the documents ("" when it was not imported).
static func res_path(art_path: String) -> String:
	return ArtData.path_of_source(art_path)
