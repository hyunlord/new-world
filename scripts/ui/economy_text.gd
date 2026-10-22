# economy_text.gd: the M2 economy's codes and names as player text (static helpers shared by the order bar, the
#   workshop panel, the priority table, the inspector and the overlay).
# Maps the bridge's codes to localization keys: item kinds (0 log, 1 firewood, 2 stone, 3 flint, 4 flint_blade,
#   5 stone_axe, 6 hide), materials by name ("flint", "chert", "granite", "limestone", "oak"), recipes by code
#   (0 knap_blade, 1 haft_axe, 2 split_firewood) or name, work types by name (craft, build, quarry, chop, split,
#   haul), skills by name (knap, woodwork, chop, quarry, build) and tiers 0–3, workshop/blueprint kinds (0 hut,
#   1 knapping_spot, 2 wood_bench, 3 drying_rack) and command reject reasons. Unknown codes fall back to UI_UNKNOWN.
# M3 (goal 3.4): quality as a word and a number ("좋음 (62)"), five bands of 20 (0–19 조악 … 80–100 걸작).
# M3 (goal 9): the space blueprint kinds (4 log_wall, 5 stone_wall, 6 door, 7 packed_floor, 8 bedroll; one tile,
#   walls and floors placed by drag), the build menu's order (5 통나무 벽, 6 문, 7 바닥, 8 침상, 9 돌담), work type
#   `mine`, skill `mining`, reasons person/hill/edge/built, and room roles by code (0 unassigned, 1 bedroom,
#   2 storehouse, 3 workshop, 4 hall).
# Bridge methods called: none (callers pass the dictionaries they read).
extends RefCounted

const LocaleScript = preload("res://scripts/core/locale.gd")

const ITEM_KEYS: Array[String] = ["ITEM_LOG", "ITEM_FIREWOOD", "ITEM_STONE", "ITEM_FLINT", "ITEM_FLINT_BLADE", "ITEM_STONE_AXE", "ITEM_HIDE"]
const ITEM_FMT_KEYS: Array[String] = ["ITEM_LOG_FMT", "ITEM_FIREWOOD_FMT", "ITEM_STONE_FMT", "ITEM_FLINT_FMT", "ITEM_FLINT_BLADE_FMT", "ITEM_STONE_AXE_FMT", "ITEM_HIDE_FMT"]
## The overlay sprite per item kind (logs use the log piles by count).
const ITEM_SPRITES: Array[String] = ["logs_s", "item_firewood", "item_stone", "item_flint", "item_blade", "item_axe", "item_hide"]
const MATERIAL_KEYS: Dictionary = {
	"flint": "MATERIAL_FLINT", "chert": "MATERIAL_CHERT", "granite": "MATERIAL_GRANITE",
	"limestone": "MATERIAL_LIMESTONE", "oak": "MATERIAL_OAK",
}
const RECIPE_NAMES: Array[String] = ["knap_blade", "haft_axe", "split_firewood"]
const RECIPE_KEYS: Array[String] = ["RECIPE_KNAP_BLADE", "RECIPE_HAFT_AXE", "RECIPE_SPLIT_FIREWOOD"]
const WORK_KEYS: Dictionary = {
	"craft": "WORK_CRAFT", "build": "WORK_BUILD", "quarry": "WORK_QUARRY",
	"chop": "WORK_CHOP", "split": "WORK_SPLIT", "haul": "WORK_HAUL", "mine": "WORK_MINE",
}
const SKILL_KEYS: Dictionary = {
	"knap": "SKILL_KNAP", "woodwork": "SKILL_WOODWORK", "chop": "SKILL_CHOP",
	"quarry": "SKILL_QUARRY", "build": "SKILL_BUILD", "mining": "SKILL_MINING",
}
const TIER_KEYS: Array[String] = ["SKILL_TIER_0", "SKILL_TIER_1", "SKILL_TIER_2", "SKILL_TIER_3"]
const QUALITY_KEYS: Array[String] = ["QUALITY_BAND_0", "QUALITY_BAND_1", "QUALITY_BAND_2", "QUALITY_BAND_3", "QUALITY_BAND_4"]
## Blueprint / workshop kinds by code (`BlueprintKind::code`): bridge name, footprint, localization key, sheet
## sprite; M3 space kinds add `drag` ("wall": line or hollow rectangle, "floor": line or filled rectangle, "": one
## click) and `ghost` (the blueprint sprite of a site or a placement preview).
const BUILD_KINDS: Array[Dictionary] = [
	{"name": "hut", "size": Vector2i(3, 3), "key": "BUILDING_HUT", "sprite": "hut"},
	{"name": "knapping_spot", "size": Vector2i(1, 1), "key": "BUILDING_KNAPPING_SPOT", "sprite": "knapping_spot"},
	{"name": "wood_bench", "size": Vector2i(2, 1), "key": "BUILDING_WOOD_BENCH", "sprite": "wood_bench"},
	{"name": "drying_rack", "size": Vector2i(2, 1), "key": "BUILDING_DRYING_RACK", "sprite": "drying_rack"},
	{"name": "log_wall", "size": Vector2i(1, 1), "key": "BUILDING_LOG_WALL", "sprite": "log_wall_10", "drag": "wall", "ghost": "ghost_wall"},
	{"name": "stone_wall", "size": Vector2i(1, 1), "key": "BUILDING_STONE_WALL", "sprite": "stone_wall_10", "drag": "wall", "ghost": "ghost_wall"},
	{"name": "door", "size": Vector2i(1, 1), "key": "BUILDING_DOOR", "sprite": "door_h", "drag": "", "ghost": "ghost_door"},
	{"name": "packed_floor", "size": Vector2i(1, 1), "key": "BUILDING_PACKED_FLOOR", "sprite": "floor_packed", "drag": "floor", "ghost": "ghost_floor"},
	{"name": "bedroll", "size": Vector2i(1, 1), "key": "BUILDING_BEDROLL", "sprite": "bedroll", "drag": "", "ghost": "ghost_bedroll"},
]
## The F3 build menu: BUILD_KINDS codes in menu order (keys 1–9).
const MENU_ORDER: Array[int] = [0, 1, 2, 3, 4, 6, 7, 8, 5]
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
}


static func ltr(locale: Node, key: String) -> String:
	return str(locale.call("ltr", key)) if locale != null else key


static func trf(locale: Node, key: String, params: Dictionary) -> String:
	return str(locale.call("trf", key, params)) if locale != null else LocaleScript.format_text(key, params)


## The plain name of item kind `kind`.
static func item(locale: Node, kind: int) -> String:
	return ltr(locale, ITEM_KEYS[kind]) if kind >= 0 and kind < ITEM_KEYS.size() else ltr(locale, "UI_UNKNOWN")


## The name of item kind `kind` made of `material` (a bridge material name; "" or unknown = the plain name).
static func item_of(locale: Node, kind: int, material: String) -> String:
	if kind < 0 or kind >= ITEM_FMT_KEYS.size() or not MATERIAL_KEYS.has(material):
		return item(locale, kind)
	return trf(locale, ITEM_FMT_KEYS[kind], {"material": ltr(locale, str(MATERIAL_KEYS[material]))})


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
	return ltr(locale, RECIPE_KEYS[code]) if code >= 0 and code < RECIPE_KEYS.size() else ltr(locale, "UI_UNKNOWN")


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
	return ltr(locale, str(BUILD_KINDS[code]["key"])) if code >= 0 and code < BUILD_KINDS.size() else ltr(locale, "UI_UNKNOWN")


## The blueprint kind code of bridge name `name` (-1 when unknown).
static func build_code(build_name: String) -> int:
	for i: int in BUILD_KINDS.size():
		if str(BUILD_KINDS[i]["name"]) == build_name:
			return i
	return -1


## The player text for a command result's reject reason.
static func reason(locale: Node, reason_name: String) -> String:
	return ltr(locale, str(REASON_KEYS.get(reason_name, "UI_REJECT_NOTHING")))


## Whether blueprint kind `code` is an M3 space kind (one tile; walls, doors, floors, bedrolls).
static func is_space(code: int) -> bool:
	return code >= 0 and code < BUILD_KINDS.size() and BUILD_KINDS[code].has("drag")


## How blueprint kind `code` is placed: "wall" / "floor" (drag a line, Shift for a rectangle) or "" (one click).
static func drag_of(code: int) -> String:
	return str(BUILD_KINDS[code].get("drag", "")) if code >= 0 and code < BUILD_KINDS.size() else ""


## The room role text of role code `code` (unknown codes = 미정).
static func role(locale: Node, code: int) -> String:
	return ltr(locale, ROLE_KEYS[code] if code >= 0 and code < ROLE_KEYS.size() else ROLE_KEYS[0])
