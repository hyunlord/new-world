# economy_text.gd — registry-backed player labels and placement metadata.
# Bridge methods: none; ContentContext is installed before gameplay ready.
extends RefCounted

const Content = preload("res://scripts/ui/content_context.gd")

const LocaleScript = preload("res://scripts/core/locale.gd")

const WORK_KEYS: Dictionary = {
	"craft": "WORK_CRAFT", "build": "WORK_BUILD", "quarry": "WORK_QUARRY",
	"chop": "WORK_CHOP", "split": "WORK_SPLIT", "haul": "WORK_HAUL", "mine": "WORK_MINE", "hunt": "M5_WORK_HUNT",
}
const SKILL_KEYS: Dictionary = {
	"knap": "SKILL_KNAP", "woodwork": "SKILL_WOODWORK", "chop": "SKILL_CHOP",
	"quarry": "SKILL_QUARRY", "build": "SKILL_BUILD", "mining": "SKILL_MINING", "pottery": "SKILL_POTTERY",
	"hunting": "M5_SKILL_HUNTING",
}
const TIER_KEYS: Array[String] = ["SKILL_TIER_0", "SKILL_TIER_1", "SKILL_TIER_2", "SKILL_TIER_3"]
const QUALITY_KEYS: Array[String] = ["QUALITY_BAND_0", "QUALITY_BAND_1", "QUALITY_BAND_2", "QUALITY_BAND_3", "QUALITY_BAND_4"]
## Room roles by `RoomRole::code` (0 unassigned, 1 bedroom, 2 storehouse, 3 workshop, 4 hall): bridge name, key.
const ROLE_NAMES: Array[String] = ["unassigned", "bedroom", "storehouse", "workshop", "hall"]
const ROLE_KEYS: Array[String] = ["ROOM_ROLE_UNASSIGNED", "ROOM_ROLE_BEDROOM", "ROOM_ROLE_STOREHOUSE", "ROOM_ROLE_WORKSHOP", "ROOM_ROLE_HALL"]
const REASON_KEYS: Dictionary = {
	"blocked": "UI_REJECT_BLOCKED",
	"water": "UI_REJECT_WATER",
	"out_of_map": "UI_REJECT_OUT_OF_MAP",
	"nothing": "UI_REJECT_NOTHING",
	"no_colony": "UI_REJECT_NO_COLONY",
	"not_yet": "UI_REJECT_NOT_YET",
	"wrong_workshop": "UI_REJECT_WRONG_WORKSHOP",
	"unknown": "UI_REJECT_UNKNOWN",
	"person": "UI_REJECT_PERSON",
	"hill": "UI_REJECT_HILL",
	"edge": "UI_REJECT_EDGE",
	"built": "UI_REJECT_BUILT",
	"not_hill": "UI_REJECT_NOT_HILL",
	"unreachable": "UI_REJECT_UNREACHABLE",
	"no_practitioner": "UI_REJECT_NO_PRACTITIONER",
	"too_young": "UI_REJECT_TOO_YOUNG",
	"not_drafted": "M5_REJECT_NOT_DRAFTED",
	"no_visitor": "M55_REJECT_NO_VISITOR",
	"unbalanced": "M55_REJECT_UNBALANCED",
}


static func ltr(locale: Node, key: String) -> String:
	return str(locale.call("ltr", key)) if locale != null else key


static func trf(locale: Node, key: String, params: Dictionary) -> String:
	return str(locale.call("trf", key, params)) if locale != null else LocaleScript.format_text(key, params)


## The plain name of item kind `kind`.
static func item(locale: Node, kind: int) -> String:
	return ltr(locale, str(Content.row("items", kind).get("name_key", "UI_UNKNOWN")))


## The name of item kind `kind` made of `material` (a bridge material name; "" or unknown = the plain name).
static func item_of(locale: Node, kind: int, material: String) -> String:
	var key: String = str(Content.row("items", kind).get("material_format_key", ""))
	var material_row: Dictionary = Content.row("materials", Content.code("materials", material))
	if key.is_empty() or material_row.is_empty():
		return item(locale, kind)
	return trf(locale, key, {"material": ltr(locale, str(material_row["name_key"]))})


## The quality band 0–4 of quality 0–100 (0–19, 20–39, 40–59, 60–79, 80–100).
static func quality_band(value: int) -> int:
	return clampi(floori(value / 20.0), 0, QUALITY_KEYS.size() - 1)


## "좋음 (62)": the band word and the number.
static func quality(locale: Node, value: int) -> String:
	return trf(locale, "UI_QUALITY_FMT", {"word": ltr(locale, QUALITY_KEYS[quality_band(value)]), "value": value})


## "{item} ×{count} · 좋음 (62)".
static func stack(locale: Node, kind: int, material: String, count: int, value: int) -> String:
	return trf(locale, "UI_STACK_FMT", {"item": item_of(locale, kind, material), "count": count, "quality": quality(locale, value)})


## The recipe name for recipe code `code`.
static func recipe(locale: Node, code: int) -> String:
	return ltr(locale, str(Content.row("recipes", code).get("name_key", "UI_UNKNOWN")))


## The column title of work type `work` (bridge name).
static func work(locale: Node, work_name: String) -> String:
	return ltr(locale, str(WORK_KEYS.get(work_name, "UI_UNKNOWN")))


## The skill name of `skill` (bridge name).
static func skill(locale: Node, skill_name: String) -> String:
	return ltr(locale, str(SKILL_KEYS.get(skill_name, "UI_UNKNOWN")))


## The tier name (0 초보 … 3 달인).
static func tier(locale: Node, code: int) -> String:
	return ltr(locale, TIER_KEYS[code]) if code >= 0 and code < TIER_KEYS.size() else ltr(locale, "UI_UNKNOWN")


## The building name of blueprint/workshop kind `code`.
static func building(locale: Node, code: int) -> String:
	return ltr(locale, str(Content.row("blueprints", code).get("name_key", "UI_UNKNOWN")))


## The blueprint kind code of bridge name `name` (-1 when unknown).
static func build_code(build_name: String) -> int:
	return Content.code("blueprints", build_name)


## The player text for a command result's reject reason.
static func reason(locale: Node, reason_name: String) -> String:
	return ltr(locale, str(REASON_KEYS.get(reason_name, "UI_REJECT_NOTHING")))


## Whether blueprint kind `code` is an M3 space kind (one tile; walls, doors, floors, bedrolls).
static func is_space(code: int) -> bool:
	return str(Content.row("blueprints", code).get("completion_category", "")) in ["wall", "door", "floor", "bed"]


## How blueprint kind `code` is placed: "wall" / "floor" (drag a line, Shift for a rectangle) or "" (one click).
static func drag_of(code: int) -> String:
	var category: String = str(Content.row("blueprints", code).get("placement_category", ""))
	return category if category in ["wall", "floor"] else ""


## The room role text of role code `code` (unknown codes = 미정).
static func role(locale: Node, code: int) -> String:
	return ltr(locale, ROLE_KEYS[code] if code >= 0 and code < ROLE_KEYS.size() else ROLE_KEYS[0])


static func build_kind(code: int) -> Dictionary:
	var row: Dictionary = Content.row("blueprints", code)
	if row.is_empty():
		return {}
	return {"name": row["id"], "key": row["name_key"], "size": Vector2i(int(row["width"]), int(row["height"])), "placeable": row["placeable"]}


static func menu_order() -> Array[int]:
	var out: Array[int] = []
	for row: Dictionary in Content.menu():
		out.append(int(row["code"]))
	return out


static func item_code(id: String) -> int:
	return Content.code("items", id)


static func recipe_code(id: String) -> int:
	return Content.code("recipes", id)
