# economy_text.gd: the M2 economy's codes and names as player text (static helpers shared by the order bar, the
#   workshop panel, the priority table, the inspector and the overlay).
# Maps the bridge's codes to localization keys: item kinds (0 log, 1 firewood, 2 stone, 3 flint, 4 flint_blade,
#   5 stone_axe, 6 hide), materials by name ("flint", "chert", "granite", "limestone", "oak"), recipes by code
#   (0 knap_blade, 1 haft_axe, 2 split_firewood) or name, work types by name (craft, build, quarry, chop, split,
#   haul), skills by name (knap, woodwork, chop, quarry, build) and tiers 0–3, workshop/blueprint kinds (0 hut,
#   1 knapping_spot, 2 wood_bench, 3 drying_rack) and command reject reasons. Unknown codes fall back to UI_UNKNOWN.
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
	"chop": "WORK_CHOP", "split": "WORK_SPLIT", "haul": "WORK_HAUL",
}
const SKILL_KEYS: Dictionary = {
	"knap": "SKILL_KNAP", "woodwork": "SKILL_WOODWORK", "chop": "SKILL_CHOP",
	"quarry": "SKILL_QUARRY", "build": "SKILL_BUILD",
}
const TIER_KEYS: Array[String] = ["SKILL_TIER_0", "SKILL_TIER_1", "SKILL_TIER_2", "SKILL_TIER_3"]
## Blueprint / workshop kinds by code: bridge name, footprint, localization key, sheet sprite.
const BUILD_KINDS: Array[Dictionary] = [
	{"name": "hut", "size": Vector2i(3, 3), "key": "BUILDING_HUT", "sprite": "hut"},
	{"name": "knapping_spot", "size": Vector2i(1, 1), "key": "BUILDING_KNAPPING_SPOT", "sprite": "knapping_spot"},
	{"name": "wood_bench", "size": Vector2i(2, 1), "key": "BUILDING_WOOD_BENCH", "sprite": "wood_bench"},
	{"name": "drying_rack", "size": Vector2i(2, 1), "key": "BUILDING_DRYING_RACK", "sprite": "drying_rack"},
]
const REASON_KEYS: Dictionary = {
	"blocked": "UI_REJECT_BLOCKED",
	"water": "UI_REJECT_WATER",
	"out_of_map": "UI_REJECT_OUT_OF_MAP",
	"nothing": "UI_REJECT_NOTHING",
	"no_colony": "UI_REJECT_NO_COLONY",
	"not_yet": "UI_REJECT_NOT_YET",
	"wrong_workshop": "UI_REJECT_WRONG_WORKSHOP",
	"unknown": "UI_REJECT_UNKNOWN",
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


## "{item} ×{count} · 품질 {quality}".
static func stack(locale: Node, kind: int, material: String, count: int, quality: int) -> String:
	return trf(locale, "UI_STACK_FMT", {"item": item_of(locale, kind, material), "count": count, "quality": quality})


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
