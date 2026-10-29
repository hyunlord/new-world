# story_text.gd: the M5 story codes as player text (static helpers shared by the story banner, the inspector, the
#   chronicle, the "→ 왜?" panel, the animal layer and the order tool).
# Maps the bridge's codes to localization keys: pressures (`PressureKind::code` 0 cold snap, 1 drought, 2 beasts,
#   3 illness), mood modifiers (`Modifier::code` 0 slept_roofed … 13 pressure, and 100 = the relation itself), the
#   quarrel causes of a chronicle Quarrel/Fight row (cause − 1: 0 hunger, 1 cold, 2 sleep, 3 grief, 4 pressure,
#   5 relation), mood bands (0 none, 1 우울, 2 보통, 3 흡족, 4 무너짐) and the mood overlay's tints, tops (0 loincloth,
#   1 hide wrap, 2 fur cloak) with "추위 −30 %" from `top_cold_permille`, draft orders (0 hold, 1 move, 2 attack an
#   animal, 3 attack a person), animal kinds (0 wolf, 1 deer) and states (0 wander, 1 flee, 2 rest, 3 hunt, 4 attack,
#   5 retreat). Unknown codes fall back to UI_UNKNOWN.
# Mood overlay tints (M): 우울 blue, 보통 none, 흡족 yellow, 무너짐 (a breakdown) a stronger red — the one state that
#   refuses work, so it must not read as "a little sadder than blue".
# M5.5 (goal §9): the inspector's "도망 중: 늑대" (get_agent_m5 `fleeing`); the year line appends migrants joined, trades
#   and eurekas when the summary has them (`migrants_joined`, `trades`, `eurekas`).
# Bridge methods called: none (callers pass the dictionaries they read: get_story, get_agent_m5, get_drafted,
#   get_animals, get_year_summary).
extends RefCounted

const Content = preload("res://scripts/ui/content_context.gd")

const EconomyText = preload("res://scripts/ui/economy_text.gd")
const Names = preload("res://scripts/ui/person_names.gd")

const MODIFIER_KEYS: Array[String] = ["M5_MOD_SLEPT_ROOFED", "M5_MOD_SLEPT_OUTSIDE", "M5_MOD_FED", "M5_MOD_HUNGRY", "M5_MOD_THIRSTY",
	"M5_MOD_COLD", "M5_MOD_QUARRELLED", "M5_MOD_LEARNED", "M5_MOD_DISCOVERED", "M5_MOD_GRIEF", "M5_MOD_SICK", "M5_MOD_CRAFTED",
	"M5_MOD_DRAFTED", "M5_MOD_PRESSURE", "M6A_MOD_VARIED", "M6A_MOD_MONOTONOUS"]
## Modifier names as the bridge gives them (`modifier_names`), in code order.
const MODIFIER_NAMES: Array[String] = ["slept_roofed", "slept_outside", "fed", "hungry", "thirsty", "cold", "quarrelled", "learned",
	"discovered", "grief", "sick", "crafted", "drafted", "pressure", "food_varied", "food_monotonous"]
## `Modifier::code` of the relation cause (no mood modifier: the pair's bad blood).
const RELATION_CODE: int = 100
const RELATION_KEY: String = "M5_MOD_RELATION"
const QUARREL_KEYS: Array[String] = ["M5_QUARREL_HUNGER", "M5_QUARREL_COLD", "M5_QUARREL_SLEEP", "M5_QUARREL_GRIEF", "M5_QUARREL_PRESSURE",
	"M5_QUARREL_RELATION"]
const BAND_KEYS: Array[String] = ["M5_MOOD_NONE", "M5_MOOD_LOW", "M5_MOOD_NORMAL", "M5_MOOD_CONTENT", "M5_MOOD_BREAKDOWN"]
## The mood overlay's modulate per band (white = no tint).
const BAND_TINTS: Array[Color] = [Color.WHITE, Color(0.55, 0.72, 1.3), Color.WHITE, Color(1.3, 1.18, 0.5), Color(1.45, 0.42, 0.38)]
## The band colours for text (the inspector's mood line, the legend).
const BAND_COLORS: Array[Color] = [Color("c8b89a"), Color("8cb4f0"), Color("f2e6c8"), Color("f0d860"), Color("f07060")]
const TOP_KEYS: Array[String] = ["M5_TOP_LOINCLOTH", "M5_TOP_HIDE_WRAP", "M5_TOP_FUR_CLOAK"]
const ANIMAL_STATE_KEYS: Array[String] = ["M5_ANIMAL_WANDER", "M5_ANIMAL_FLEE", "M5_ANIMAL_REST", "M5_ANIMAL_HUNT", "M5_ANIMAL_ATTACK",
	"M5_ANIMAL_RETREAT"]
## Historical fixture aliases only; live consumers read registered capabilities.
const WOLF: int = 0
const DEER: int = 1
## Historical fixture aliases only; live consumers resolve definitions by category.
const COLD_SNAP: int = 0
const DROUGHT: int = 1
const BEASTS: int = 2
const ILLNESS: int = 3


static func ltr(locale: Node, key: String) -> String:
	return EconomyText.ltr(locale, key)


static func trf(locale: Node, key: String, params: Dictionary) -> String:
	return EconomyText.trf(locale, key, params)


static func language(locale: Node) -> String:
	return str(locale.get("_current_lang")) if locale != null else "en"


static func _pick(locale: Node, keys: Array[String], code: int) -> String:
	return ltr(locale, keys[code]) if code >= 0 and code < keys.size() else ltr(locale, "UI_UNKNOWN")


## The name of pressure `code` (한파, 가뭄, 늑대 무리, 병).
static func pressure(locale: Node, code: int) -> String:
	return ltr(locale, str(Content.row("pressures", code).get("name_key", "UI_UNKNOWN")))


## The word of mood modifier `code` (100 = the relation: "사이가 나빠서").
static func modifier(locale: Node, code: int) -> String:
	if code == RELATION_CODE:
		return ltr(locale, RELATION_KEY)
	return _pick(locale, MODIFIER_KEYS, code)


## The word of a modifier by its bridge name (`modifier_names`); "relation" is the relation cause.
static func modifier_named(locale: Node, modifier_name: String) -> String:
	if modifier_name == "relation":
		return ltr(locale, RELATION_KEY)
	return modifier(locale, MODIFIER_NAMES.find(modifier_name))


## The cause phrase of a chronicle Quarrel/Fight row's `cause` (1 + the quarrel cause code): "배고파서".
static func quarrel_cause(locale: Node, cause: int) -> String:
	return _pick(locale, QUARREL_KEYS, cause - 1)


static func band(locale: Node, code: int) -> String:
	return _pick(locale, BAND_KEYS, code)


static func band_tint(code: int) -> Color:
	return BAND_TINTS[code] if code >= 0 and code < BAND_TINTS.size() else Color.WHITE


static func band_color(code: int) -> Color:
	return BAND_COLORS[code] if code >= 0 and code < BAND_COLORS.size() else BAND_COLORS[0]


static func top(locale: Node, code: int) -> String:
	return _pick(locale, TOP_KEYS, code)


static func animal(locale: Node, kind: int) -> String:
	return ltr(locale, str(Content.row("animals", kind).get("name_key", "UI_UNKNOWN")))


static func animal_huntable(kind: int) -> bool:
	return bool(Content.row("animals", kind).get("huntable", false))


## Bundled wording is an optional data override; new identities use category templates and their own name.
static func content_phrase(locale: Node, family: String, code: int, event: String, params: Dictionary) -> String:
	var definition: Dictionary = Content.row(family + "s", code)
	var presentation: Dictionary = Content.document("story")
	var overrides: Dictionary = presentation.get(family + "_overrides", {}).get(str(code), {})
	if overrides.has(event):
		return trf(locale, str(overrides[event]), params)
	var category: String = str(definition.get("category", ""))
	var templates: Dictionary = presentation.get(family + "_categories", {}).get(category, {})
	var template: String = str(templates.get(event, {}).get(language(locale), ""))
	var values: Dictionary = params.duplicate()
	values["name"] = ltr(locale, str(definition.get("name_key", "UI_UNKNOWN")))
	for key: Variant in values:
		template = template.replace("{" + str(key) + "}", str(values[key]))
	return template if not template.is_empty() else str(values["name"])


static func animal_state(locale: Node, state: int) -> String:
	return _pick(locale, ANIMAL_STATE_KEYS, state)


## "기분: 우울 (210) — 춥게 잠, 배고픔, 다툼" from a get_agent_m5 read (the first three modifiers, largest first);
## "" without a mood.
static func mood_line(locale: Node, m5: Dictionary) -> String:
	var value: int = int(m5.get("mood", -1))
	if value < 0:
		return ""
	var names: PackedStringArray = m5.get("modifier_names", PackedStringArray())
	var words: Array[String] = []
	for i: int in mini(3, names.size()):
		words.append(modifier_named(locale, names[i]))
	var params: Dictionary = {"band": band(locale, int(m5.get("band", 0))), "value": value, "mods": ", ".join(words)}
	return trf(locale, "M5_INSPECT_MOOD_FMT" if not words.is_empty() else "M5_INSPECT_MOOD_PLAIN_FMT", params)


## "옷: 가죽 랩 (추위 −30 %)" (−(1000 − top_cold_permille) / 10 %); the loincloth has no percentage.
static func apparel_line(locale: Node, m5: Dictionary) -> String:
	var code: int = int(m5.get("top", 0))
	var permille: int = int(m5.get("top_cold_permille", 1000))
	if code <= 0 or permille >= 1000:
		return trf(locale, "M5_INSPECT_TOP_PLAIN_FMT", {"top": top(locale, code)})
	@warning_ignore("integer_division")
	var percent: int = (1000 - permille) / 10
	return trf(locale, "M5_INSPECT_TOP_FMT", {"top": top(locale, code), "percent": percent})


## The draft order as words: "대기", "(40, 52)로 이동", "늑대 공격" / "카토 공격" (`target` = the attack's id, `x`/`y`
## the move's tile; `animals` = a get_animals read to name the animal, may be empty).
static func order_text(locale: Node, order: int, x: int, y: int, animals: Dictionary = {}) -> String:
	match order:
		0: return ltr(locale, "M5_DRAFT_HOLD")
		1: return trf(locale, "M5_DRAFT_MOVE_FMT", {"x": x, "y": y})
		2:
			var kind: int = animal_kind(animals, x)
			return trf(locale, "M5_DRAFT_ATTACK_FMT", {"target": animal(locale, kind) if kind >= 0 else ltr(locale, "M5_ANIMAL_ANY")})
		3: return trf(locale, "M5_DRAFT_ATTACK_FMT", {"target": Names.present(x, language(locale))})
	return ltr(locale, "UI_UNKNOWN")


## The kind of animal `id` in a get_animals read, -1 when it is not there.
static func animal_kind(animals: Dictionary, id: int) -> int:
	var ids: PackedInt64Array = animals.get("ids", PackedInt64Array())
	var kinds: PackedInt64Array = animals.get("kinds", PackedInt64Array())
	var i: int = ids.find(id)
	return kinds[i] if i >= 0 and i < kinds.size() else -1


## The held weapon's name (an item kind code; -1 = bare hands).
static func weapon(locale: Node, code: int) -> String:
	return ltr(locale, "M5_WEAPON_BARE") if code < 0 else EconomyText.item(locale, code)


## "소집됨 · (40, 52)로 이동" from a get_agent_m5 read; "" when not drafted.
static func draft_line(locale: Node, m5: Dictionary, animals: Dictionary = {}) -> String:
	if not bool(m5.get("drafted", false)):
		return ""
	return trf(locale, "M5_INSPECT_DRAFT_FMT", {"order": order_text(locale, int(m5.get("draft_order", 0)), int(m5.get("draft_x", -1)),
		int(m5.get("draft_y", -1)), animals)})


## "아픔 · 14시간 남음" from a get_agent_m5 read (`ticks_per_hour` from the clock); "" when not sick.
static func sick_line(locale: Node, m5: Dictionary, ticks_per_hour: int) -> String:
	if not bool(m5.get("sick", false)):
		return ""
	var hours: int = ceili(float(int(m5.get("sick_ticks_left", 0))) / float(maxi(1, ticks_per_hour)))
	return trf(locale, "M5_INSPECT_SICK_FMT", {"hours": hours})


## M5.5: "도망 중: 늑대" from a get_agent_m5 read; "" when not fleeing (only wolves make people flee).
static func flee_line(locale: Node, m5: Dictionary) -> String:
	return ltr(locale, "M55_INSPECT_FLEE") if bool(m5.get("fleeing", false)) else ""


## The banner lines of a get_story read, in order: wolves (while any are on the map), then one per pressure in force
## (cold snap with the firewood line, drought, illness; beasts only through the wolves line). Each is
## {text, kind, color}.
static func banners(locale: Node, story: Dictionary) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	if not bool(story.get("on", false)):
		return out
	var wolves: int = int(story.get("wolves", 0))
	var pack_kind: int = int(story.get("pack_kind", -1))
	if wolves > 0 and pack_kind >= 0:
		out.append({"text": content_phrase(locale, "pressure", pack_kind, "banner", {"count": wolves, "day": 1}), "kind": pack_kind, "color": Color("f07060")})
	var kinds: PackedInt64Array = story.get("kinds", PackedInt64Array())
	var days: PackedInt64Array = story.get("days_of", PackedInt64Array())
	for i: int in kinds.size():
		var kind: int = kinds[i]
		var definition: Dictionary = Content.row("pressures", kind)
		var category: String = str(definition.get("category", ""))
		if category == "pack":
			continue
		var params: Dictionary = {"day": int(days[i]) if i < days.size() else 1, "wood": int(story.get("firewood", 0)), "need": int(story.get("firewood_day_need", 0)), "count": int(story.get("sick", 0))}
		var style: Dictionary = Content.document("story").get("pressure_categories", {}).get(category, {})
		var row: Dictionary = {"text": content_phrase(locale, "pressure", kind, "banner", params), "kind": kind, "color": Color(str(style.get("color", "ffffff")))}
		out.append(row)
	return out


## "정리: 1년 차 · 출생 2 · 사망 1 · …" from a get_year_summary read (`year` 0-based); M5.5 appends " · 이주 n · 거래 n · 단서 n".
static func year_line(locale: Node, summary: Dictionary) -> String:
	var deaths: int = 0
	for n: int in summary.get("deaths", PackedInt32Array()):
		deaths += n
	var pressures: int = 0
	for n: int in summary.get("pressures", PackedInt32Array()):
		pressures += n
	var text: String = trf(locale, "M5_YEAR_LINE_FMT", {"year": int(summary.get("year", 0)) + 1, "births": int(summary.get("births", 0)), "deaths": deaths,
		"pressures": pressures, "discoveries": int(summary.get("discoveries", 0)), "quarrels": int(summary.get("quarrels", 0)),
		"hunted": int(summary.get("hunted", 0)), "wolves": int(summary.get("wolves_killed", 0))})
	if summary.has("migrants_joined"):
		text += trf(locale, "M55_YEAR_EXTRA_FMT", {"migrants": int(summary.get("migrants_joined", 0)), "trades": int(summary.get("trades", 0)),
			"eurekas": int(summary.get("eurekas", 0))})
	return text
