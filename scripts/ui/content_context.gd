# content_context.gd — process-selected immutable UI catalogue facade.
# Bridge methods: none; the engine-free bootstrap supplies get_content_catalog's DTO.
extends RefCounted

static var _revision: String = ""
static var _item_roles: Dictionary = {}
static var _families: Dictionary = {}
static var _names: Dictionary = {}
static var _menu: Array[Dictionary] = []
static var _presentation: Dictionary = {}


static func validate(dto: Dictionary) -> String:
	if dto.get("schema_version") != 1 or not dto.get("registry_revision") is String or str(dto.get("registry_revision", "")).is_empty():
		return "content catalog: invalid schema/revision"
	for family: String in ["items", "blueprints", "recipes"]:
		if not dto.get(family) is Array:
			return "content catalog: missing " + family
		var codes: Dictionary = {}
		var names: Dictionary = {}
		for value: Variant in dto[family]:
			if not value is Dictionary:
				return "content catalog: invalid row in " + family
			var row: Dictionary = value
			if not row.get("code") is int or int(row["code"]) < 0 or int(row["code"]) > 4294967295:
				return "content catalog: invalid u32 code in " + family
			if not row.get("id") is String or str(row["id"]).is_empty() or not row.get("name_key") is String or str(row["name_key"]).is_empty():
				return "content catalog: missing identity in " + family
			if codes.has(row["code"]) or names.has(row["id"]):
				return "content catalog: duplicate identity in " + family
			codes[row["code"]] = true
			names[row["id"]] = true
	return ""


static func install(dto: Dictionary, presentation: Dictionary) -> String:
	var error: String = validate(dto)
	if not error.is_empty():
		return error
	var revision: String = dto["registry_revision"]
	if not _revision.is_empty():
		return "" if _revision == revision else "content catalog already installed"
	for family: String in ["items", "blueprints", "recipes", "materials", "knowledge", "pressures", "animals"]:
		var rows: Dictionary = {}
		var names: Dictionary = {}
		for value: Variant in dto.get(family, []):
			var row: Dictionary = (value as Dictionary).duplicate(true)
			var code: int = int(row.get("code", row.get("raw_id", -1)))
			rows[code] = row
			names[str(row["id"])] = code
			if family == "blueprints" and bool(row.get("placeable", false)) and int(row.get("menu_order", -1)) >= 0:
				_menu.append(row)
		_families[family] = rows
		_names[family] = names
	_menu.sort_custom(func(a: Dictionary, b: Dictionary) -> bool:
		return int(a["menu_order"]) < int(b["menu_order"]) if a["menu_order"] != b["menu_order"] else int(a["code"]) < int(b["code"]))
	_item_roles = dto.get("item_roles", {}).duplicate(true)
	_presentation = presentation.duplicate(true)
	_revision = revision
	return ""


static func row(family: String, code: int) -> Dictionary:
	return (_families.get(family, {}).get(code, {}) as Dictionary).duplicate(true)


static func code(family: String, id: String) -> int:
	return int(_names.get(family, {}).get(id, -1))


static func menu() -> Array[Dictionary]:
	return _menu.duplicate(true)


static func document(family: String) -> Dictionary:
	return (_presentation.get(family, {}) as Dictionary).duplicate(true)


static func revision() -> String:
	return _revision


static func rows(family: String) -> Array[Dictionary]:
	var result: Array[Dictionary] = []
	for value: Dictionary in _families.get(family, {}).values():
		result.append(value.duplicate(true))
	return result


static func item_role(role: String) -> int:
	return int(_item_roles.get(role, -1))
