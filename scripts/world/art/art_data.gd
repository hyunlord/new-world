# art_data.gd — the renderer's registry of imported art (ADR 0033, 0034).
# Purpose: one place that reads assets/v8art/: `manifest.json` (every imported file: id, role, path, size, anchor,
#   canvas, footprint; the 36 logical looks; the people and animal frame layouts), `bindings.json` (sim codes → art
#   ids, with family fallbacks) and `render_params.json` (every tunable; `reload_params()` re-reads it, the F12 key in
#   debug builds). Layers ask it for textures, images, anchors and bindings by id or by sim code and never name a
#   content id themselves. Unknown ids return null / an empty Dictionary and the caller draws the family fallback.
# Bridge methods called: none.
extends RefCounted

const Content = preload("res://scripts/ui/content_context.gd")

const ROOT: String = "res://assets/v8art/"

static var _selected_bindings: Dictionary = {}
static var _manifest: Dictionary = {}
static var _bindings: Dictionary = {}
static var _params: Dictionary = {}
static var _params_version: int = 0
static var _by_id: Dictionary = {}
static var _by_source: Dictionary = {}
static var _textures: Dictionary = {}
static var _images: Dictionary = {}
static var _codes: Dictionary = {}
## The params file `reload_params()` reads (tests point it at a copy under user://).
static var params_path: String = ROOT + "render_params.json"


static func _read_json(path: String) -> Dictionary:
	var text: String = FileAccess.get_file_as_string(path)
	var parsed: Variant = JSON.parse_string(text)
	if parsed is Dictionary:
		return parsed
	push_warning("art_data: cannot read %s" % path)
	return {}


static func _load() -> void:
	if not _manifest.is_empty():
		return
	_manifest = _read_json(ROOT + "manifest.json")
	_bindings = _read_json(ROOT + "bindings.json")
	_apply_selected_bindings()
	if _params.is_empty():
		_params = _read_json(ROOT + "render_params.json")
	_by_id.clear()
	_by_source.clear()
	var files: Variant = _manifest.get("files", [])
	if files is Array:
		for row: Variant in files:
			if row is Dictionary:
				var d: Dictionary = row
				_by_id[str(d.get("id", ""))] = d
				_by_source[str(d.get("source", ""))] = d
	_codes.clear()


## The whole manifest (read-only use).
static func manifest() -> Dictionary:
	_load()
	return _manifest


## The whole bindings document (read-only use).
static func bindings() -> Dictionary:
	_load()
	return _bindings


## The render parameters document.
static func params() -> Dictionary:
	_load()
	return _params


## Counts reloads, so layers can rebuild what depends on parameters.
static func params_version() -> int:
	return _params_version


## Re-read render_params.json (F12 in debug builds). Returns false when the file does not parse (the old values stay).
static func reload_params() -> bool:
	var fresh: Dictionary = _read_json(params_path)
	if fresh.is_empty() or int(fresh.get("schema_version", 0)) != 1:
		return false
	_params = fresh
	_params_version += 1
	return true


## A parameter by dotted path ("walls.top_value"); `fallback` when it is missing or of another type.
static func param(path: String, fallback: Variant) -> Variant:
	_load()
	var node: Variant = _params
	for key: String in path.split("."):
		if node is Dictionary and (node as Dictionary).has(key):
			node = (node as Dictionary)[key]
		else:
			return fallback
	if fallback != null and typeof(node) != typeof(fallback):
		if (typeof(node) == TYPE_INT or typeof(node) == TYPE_FLOAT) and (typeof(fallback) == TYPE_INT or typeof(fallback) == TYPE_FLOAT):
			return float(node) if typeof(fallback) == TYPE_FLOAT else int(node)
		return fallback
	return node


## A manifest row by id (empty when unknown).
static func row(id: String) -> Dictionary:
	_load()
	return _by_id.get(id, {})


## Whether the manifest has `id`.
static func has(id: String) -> bool:
	_load()
	return _by_id.has(id)


## The res:// path of an imported file by id ("" when unknown).
static func path(id: String) -> String:
	var r: Dictionary = row(id)
	return ROOT + str(r["path"]) if r.has("path") else ""


## The res:// path of the file imported from art path `source` (as the art JSON names it; "" when not imported).
static func path_of_source(source: String) -> String:
	_load()
	var r: Dictionary = _by_source.get(source, {})
	return ROOT + str(r["path"]) if r.has("path") else ""


## The imported texture `id` (null when unknown).
static func texture(id: String) -> Texture2D:
	var p: String = path(id)
	return texture_at(p)


## The texture at a res:// path, cached (null when missing).
static func texture_at(p: String) -> Texture2D:
	if p == "":
		return null
	if not _textures.has(p):
		_textures[p] = load(p) as Texture2D if ResourceLoader.exists(p) else null
	return _textures[p]


## The decoded RGBA8 image of texture `p` (a res:// path), cached; null when missing.
static func image_at(p: String) -> Image:
	if p == "":
		return null
	if not _images.has(p):
		var t: Texture2D = texture_at(p)
		var img: Image = t.get_image() if t != null else null
		if img != null:
			if img.is_compressed():
				img.decompress()
			img.convert(Image.FORMAT_RGBA8)
		_images[p] = img
	return _images[p]


## The decoded image of `id`.
static func image(id: String) -> Image:
	return image_at(path(id))


## The declared anchor of `id` in its canvas (Vector2i; the canvas's bottom centre when none is declared).
static func anchor(id: String) -> Vector2i:
	var r: Dictionary = row(id)
	var meta: Dictionary = r.get("meta", {})
	var a: Variant = meta.get("anchor", null)
	if a is Array and (a as Array).size() == 2:
		return Vector2i(int(a[0]), int(a[1]))
	var size: Variant = r.get("size", [0, 0])
	if size is Array and (size as Array).size() == 2:
		return Vector2i(floori(float(size[0]) * 0.5), int(size[1]) - 1)
	return Vector2i.ZERO


## A footprint in tiles (1×1 when none is declared).
static func footprint(id: String) -> Vector2i:
	var meta: Dictionary = row(id).get("meta", {})
	var f: Variant = meta.get("footprint", null)
	if f is Array and (f as Array).size() == 2:
		return Vector2i(maxi(1, int(f[0])), maxi(1, int(f[1])))
	return Vector2i.ONE


## The bindings row of `family` ("items", "buildings", "workshops", "nodes", "animals", "terrain", "resources")
## for sim code `code`; the family's fallback row (its `fallback` id, or `fallback_code`) when the code is unknown.
static func binding(family: String, code: int) -> Dictionary:
	_load()
	var key: String = "%s/%d" % [family, code]
	if _codes.has(key):
		return _codes[key]
	var fam: Dictionary = _bindings.get(family, {})
	var found: Dictionary = {}
	var fallback_row: Dictionary = {}
	var fallback_code: int = int(fam.get("fallback_code", -1))
	for r: Variant in fam.get("codes", []):
		if r is Dictionary:
			var d: Dictionary = r
			if int(d.get("code", -999)) == code:
				found = d
			if int(d.get("code", -999)) == fallback_code:
				fallback_row = d
	if found.is_empty():
		if family == "animals":
			var category: String = str(Content.row("animals", code).get("category", ""))
			var category_code: int = int(fam.get("category_fallbacks", {}).get(category, -1))
			for candidate: Dictionary in fam.get("codes", []):
				if int(candidate["code"]) == category_code:
					found = candidate
		if not found.is_empty():
			pass
		elif not fallback_row.is_empty():
			found = fallback_row
		elif fam.get("fallback", null) is String:
			found = {"code": code, "structure": fam["fallback"], "asset": fam["fallback"], "ground": fam["fallback"], "fallback": true}
	_codes[key] = found
	return found


## The logical look `index` (age × sex × top × hair) with its frame folder and fallback flag.
static func look(index: int) -> Dictionary:
	var looks: Variant = manifest().get("looks", [])
	if looks is Array and index >= 0 and index < (looks as Array).size():
		return (looks as Array)[index]
	return {}


## The number of logical looks.
static func look_count() -> int:
	var looks: Variant = manifest().get("looks", [])
	return (looks as Array).size() if looks is Array else 0


## The res:// path of a person frame (look folder, facing "S"/"E", pose) — "" when not imported.
static func frame_path(frames: String, facing: String, pose: String) -> String:
	return path("%s/%s/%s" % [frames, facing, pose])


## The res:// path of a portrait (look folder, variant id "v1".."v6").
static func portrait_path(frames: String, variant: String) -> String:
	return path("%s/%s" % [frames, variant])


## Drop every cache (tests; a re-import while running).
static func clear() -> void:
	_manifest = {}
	_bindings = {}
	_params = {}
	_textures.clear()
	_images.clear()
	_codes.clear()


## Selected bindings add aliases of validated existing visual rows; content IDs remain independent.
static func validate_extension(document: Dictionary, dto: Dictionary) -> String:
	_load()
	if document.get("schema_version") != 1:
		return "bindings: unsupported schema_version"
	var families: Dictionary = {"items": "items", "buildings": "blueprints", "workshops": "blueprints", "animals": "animals"}
	for family: Variant in document:
		if family == "schema_version":
			continue
		if family == "construction_styles":
			var style_error: String = _validate_style_aliases(document[family], dto)
			if not style_error.is_empty(): return style_error
			continue
		if not families.has(family) or not document[family] is Dictionary or document[family].size() != 1 or not document[family].get("codes") is Array:
			return "bindings: unsupported family/schema " + str(family)
		var allowed: Dictionary = {}
		for definition: Dictionary in dto.get(families[family], []):
			allowed[int(definition["code"])] = true
		var used: Dictionary = {}
		for existing: Dictionary in _bindings.get(family, {}).get("codes", []):
			used[int(existing["code"])] = true
		for value: Variant in document[family]["codes"]:
			if not value is Dictionary or value.size() != 2 or not value.has("code") or not value.has("alias_code"):
				return "bindings: expected code and alias_code"
			for field: String in ["code", "alias_code"]:
				var number: Variant = value[field]
				if not (number is int or number is float) or not is_finite(float(number)) or float(number) != floor(float(number)) or float(number) < 0 or float(number) > 4294967295:
					return "bindings: invalid u32 " + field
			var code: int = int(value["code"])
			if not allowed.has(code) or used.has(code):
				return "bindings: unknown or duplicate content code " + str(code)
			var source_found: bool = false
			for existing: Dictionary in _bindings.get(family, {}).get("codes", []):
				if int(existing["code"]) == int(value["alias_code"]):
					source_found = true
			if not source_found:
				return "bindings: unknown bundled visual alias"
			used[code] = true
	return ""


static func install_extension(document: Dictionary) -> void:
	_selected_bindings = document.duplicate(true)
	_bindings = {}
	_manifest = {}
	_load()


static func _apply_selected_bindings() -> void:
	for family: Variant in _selected_bindings:
		if family in ["schema_version", "construction_styles"]:
			continue
		for addition: Dictionary in _selected_bindings[family]["codes"]:
			var source: Dictionary = {}
			for existing: Dictionary in _bindings[family]["codes"]:
				if int(existing["code"]) == int(addition["alias_code"]):
					source = existing.duplicate(true)
					break
			if not source.is_empty():
				source["code"] = int(addition["code"])
				_bindings[family]["codes"].append(source)


static func _validate_style_aliases(value: Variant, dto: Dictionary) -> String:
	if not value is Dictionary or value.size() != 1 or not value.get("aliases") is Array:
		return "bindings: expected construction_styles.aliases"
	var parsed: Variant = JSON.parse_string(FileAccess.get_file_as_string(ROOT + "materials/construction_styles.json"))
	if not parsed is Dictionary or not parsed.get("styles") is Array:
		return "bindings: invalid bundled construction styles"
	var sources: Dictionary = {}
	var used: Dictionary = {}
	for style: Dictionary in parsed["styles"]:
		sources[str(style["id"])] = style
		used[str(style["id"])] = true
		for legacy: String in style.get("sim_blueprints", []): used[legacy] = true
	for alias: Variant in value["aliases"]:
		if not alias is Dictionary or alias.size() != 3 or not alias.has_all(["id", "alias_id", "kind"]):
			return "bindings: style alias requires id, alias_id, kind"
		for field: String in ["id", "alias_id", "kind"]:
			if not alias[field] is String or alias[field].is_empty():
				return "bindings: invalid style alias " + field
		if used.has(alias["id"]) or not sources.has(alias["alias_id"]):
			return "bindings: duplicate style id or unknown bundled style alias"
		if alias["kind"] not in ["wall", "door", "floor"] or sources[alias["alias_id"]]["kind"] != alias["kind"]:
			return "bindings: incompatible style alias kind"
		var referenced: bool = false
		for blueprint: Dictionary in dto.get("blueprints", []):
			if blueprint.get("construction_style") == alias["id"]:
				if blueprint.get("completion_category") != alias["kind"]:
					return "bindings: style alias disagrees with blueprint kind"
				referenced = true
		if not referenced: return "bindings: style alias has no registered blueprint"
		used[alias["id"]] = true
	return ""


## Expand only preflight-validated aliases; original rows stay byte-for-byte equivalent as dictionaries.
static func construction_styles(document: Dictionary, extension: Variant = null) -> Dictionary:
	var selected: Dictionary = _selected_bindings if extension == null else extension
	var result: Dictionary = document.duplicate(true)
	for alias: Dictionary in selected.get("construction_styles", {}).get("aliases", []):
		for source: Dictionary in document.get("styles", []):
			if source["id"] == alias["alias_id"]:
				var row: Dictionary = source.duplicate(true)
				row["id"] = alias["id"]
				row["sim_blueprints"] = []
				result["styles"].append(row)
				break
	return result
