# chronicle_text.gd: static helpers that turn packed chronicle columns into rows and localized BBCode lines
#   (day prefix, person/village links, death and porter wording); used by chronicle_panel.gd and agent_inspector_panel.gd.
#   M3 kind 23 (RoomFormed: cause = the room's role code, value = its id): "새 방이 생김 — 침실".
#   M4 kinds 24 ToolBroke (subject holder, cause item code, value uses) "Fox의 돌도끼가 부러졌다 (40번 씀)",
#   25 KnowledgeLearned (subject learner, other teacher, cause channel 2 oral / 3 observed / 4 apprenticed, value node)
#   "Fox가 Mia에게 문 만들기를 가르쳤다" (or "Mia가 문 만들기를 익혔다 (구전)" without a teacher), 26 KnowledgeDiscovered
#   "Fox가 간석기를 발견했다!", 27 KnowledgeAtRisk "⚠ 문 만들기: Fox만 안다", 28 KnowledgeLost "기술 소실: 문 만들기
#   (마지막: Fox)"; lost_plain() is the loss line without BBCode for the toast.
# Bridge methods: none (formats chronicle rows and world info its callers read: ticks_per_day, ticks_per_year,
#   clock_offset_ticks from get_world_info).
extends RefCounted
const LocaleScript = preload("res://scripts/core/locale.gd")
const Names = preload("res://scripts/ui/person_names.gd")
const Selection = preload("res://scripts/core/settlement_selection.gd")
const EconomyText = preload("res://scripts/ui/economy_text.gd")
const KnowledgeText = preload("res://scripts/ui/knowledge_text.gd")
## The last chronicle kind this text knows (M4: 28 KnowledgeLost).
const LAST_KIND: int = 28
const COLUMNS: Array[String] = ["tick", "kind", "subject", "other", "third", "settlement", "settlement2", "cause", "value"]
const DEPART: Array[String] = ["CHRON_CAUSE_RANKED", "CHRON_CAUSE_SEED", "CHRON_CAUSE_HOUSEHOLD", "CHRON_CAUSE_ELDER_PARTNER", "CHRON_CAUSE_TOP_UP"]
const DEATH: Array[String] = ["CHRON_DEATH_STARVATION", "CHRON_DEATH_DEHYDRATION", "CHRON_DEATH_COMBAT", "CHRON_DEATH_OLD_AGE", "CHRON_DEATH_EXPOSURE"]

static func language(locale: Node) -> String:
	return str(locale.get("_current_lang")) if locale != null else "en"

static func loc(locale: Node, key: String) -> String:
	return str(locale.call("ltr", key)) if locale != null else key

static func day(tick: int, info: Dictionary) -> int:
	return floori(float(tick + int(info.get("clock_offset_ticks", 0))) / float(maxi(1, int(info.get("ticks_per_day", 1))))) + 1

## Whole years in `ticks` (M1.5: world info `ticks_per_year`; a missing or zero value counts as 1).
static func age_years(ticks: int, info: Dictionary) -> int:
	return floori(float(ticks) / float(maxi(1, int(info.get("ticks_per_year", 1)))))

static func rows(packed: Dictionary) -> Array[Dictionary]:
	var result: Array[Dictionary] = []
	for index: int in packed.get("tick", PackedInt64Array()).size():
		var row: Dictionary = {}
		for key: String in COLUMNS:
			var column: Variant = packed.get(key, [])
			row[key] = int(column[index]) if index < column.size() else -1
		result.append(row)
	return result

static func person(id: int, locale: Node) -> String:
	if id < 0:
		return loc(locale, "PANEL_FAMILY_NONE")
	return "[url=person:%d][b]%s[/b][/url]" % [id, Names.present(id, language(locale))]

static func village(id: int, locale: Node) -> String:
	if id < 0:
		return loc(locale, "PANEL_FAMILY_NONE")
	Selection.register(id)
	return "[url=village:%d][b]%s[/b][/url]" % [id, Names.unbroken(Selection.name_for(id, locale, false))]

static func death(cause: int, locale: Node) -> String:
	return loc(locale, DEATH[cause]) if cause >= 0 and cause < DEATH.size() else loc(locale, "UI_UNKNOWN")

static func porter_purpose(cause: int, locale: Node) -> String:
	return loc(locale, "CHRON_PORTER_AID" if cause % 10 < 2 else "CHRON_PORTER_EXCHANGE")

static func porter_load(cause: int, locale: Node) -> String:
	return loc(locale, "CHRON_LOAD_FOOD" if cause % 10 % 2 == 0 else "CHRON_LOAD_WOOD")

static func sentence(row: Dictionary, viewed: int, locale: Node, info: Dictionary) -> String:
	var kind: int = int(row.get("kind", -1))
	var subject: int = int(row.get("subject", -1))
	var other: int = int(row.get("other", -1))
	var third: int = int(row.get("third", -1))
	var cause: int = int(row.get("cause", -1))
	var value: int = int(row.get("value", -1))
	if viewed >= 0 and viewed != subject:
		if viewed != other and viewed != third:
			return ""
		match kind:
			0: return loc(locale, "CHRON_CHILD_BORN_FMT") % person(subject, locale)
			2: return loc(locale, "CHRON_PAIRED_FMT") % person(subject, locale)
			19: return loc(locale, "CHRON_QUARREL_FMT") % [person(subject, locale), village(int(row.get("settlement", -1)), locale)]
			22: return hut_built(row, locale)
			25: return knowledge_sentence(row, locale)
			_: return ""
	var place: int = int(row.get("settlement", -1))
	var parent: int = int(row.get("settlement2", -1))
	match kind:
		0:
			var text: String = loc(locale, "CHRON_BORN_FMT") % [village(place, locale), person(other, locale), person(third, locale)]
			return text + (" " + loc(locale, "CHRON_BORN_FALLBACK_NOTE") if cause == 1 else "")
		1: return loc(locale, "CHRON_FOUNDER_FMT") % village(place, locale)
		2: return loc(locale, "CHRON_PAIRED_FMT") % person(other, locale)
		3: return loc(locale, "CHRON_CAME_OF_AGE")
		4: return loc(locale, "CHRON_BECAME_ELDER")
		5:
			var reason: String = loc(locale, DEPART[cause]) if cause >= 0 and cause < DEPART.size() else loc(locale, "UI_UNKNOWN")
			return loc(locale, "CHRON_DEPARTED_FMT") % [village(place, locale), person(other, locale), reason]
		6: return loc(locale, "CHRON_DEPARTURE_CANCELLED_FMT") % village(place, locale)
		7: return loc(locale, "CHRON_SETTLED_FMT") % [village(place, locale), village(parent, locale)]
		8: return loc(locale, "CHRON_JOINED_FMT") % village(place, locale)
		9: return loc(locale, "CHRON_LEFT_BROKE_UP_FMT" if cause == 0 else "CHRON_LEFT_FMT") % village(place, locale)
		10: return loc(locale, "CHRON_WIDOWED_FMT") % person(other, locale)
		11:
			# M1.5: the age at death (value, in ticks) is shown in whole years.
			return loc(locale, "CHRON_DIED_FMT") % [age_years(value, info), death(cause, locale)] if value >= 0 else loc(locale, "CHRON_DIED_NO_AGE_FMT") % death(cause, locale)
		12: return loc(locale, "CHRON_SETTLEMENT_FORMED_FMT") % [village(place, locale), value]
		13: return loc(locale, "CHRON_FISSION_STARTED_FMT") % [village(place, locale), person(other, locale)]
		14: return loc(locale, "CHRON_DAUGHTER_FOUNDED_FMT") % [village(place, locale), village(parent, locale)]
		15: return loc(locale, "CHRON_FISSION_CANCELLED_FMT") % village(place, locale)
		16: return loc(locale, "CHRON_SETTLEMENT_DISSOLVED_FMT") % village(place, locale)
		17: return loc(locale, "CHRON_PORTER_DEPARTED_FMT") % [village(place, locale), village(parent, locale), porter_load(cause, locale), value, porter_purpose(cause, locale)]
		18:
			var purpose: String = porter_purpose(cause, locale)
			var count: int = int(row.get("porter_count", 0))
			if count > 0:
				if cause >= 10:
					return loc(locale, "CHRON_PORTER_GROUP_FAILED_FMT") % [count, village(parent, locale), village(place, locale), purpose]
				return loc(locale, "CHRON_PORTER_GROUP_FMT") % [count, porter_load(cause, locale), value, village(parent, locale), village(place, locale), purpose]
			if cause >= 10:
				return loc(locale, "CHRON_PORTER_FAILED_FMT") % [village(place, locale), purpose]
			return loc(locale, "CHRON_PORTER_DELIVERED_FMT") % [porter_load(cause, locale), value, village(parent, locale), village(place, locale), purpose]
		19: return loc(locale, "CHRON_QUARREL_FMT") % [person(other, locale), village(parent, locale)]
		20: return loc(locale, "CHRON_VILLAGES_ESTRANGED_FMT") % [village(place, locale), village(parent, locale)]
		21: return loc(locale, "CHRON_VILLAGES_RECONCILED_FMT") % [village(place, locale), village(parent, locale)]
		22: return hut_built(row, locale)
		23: return room_formed(row, locale)
		24, 25, 26, 27, 28: return knowledge_sentence(row, locale)
	return ""


## M1 kind 22 (HutBuilt): "A hut was finished in {settlement}".
static func hut_built(row: Dictionary, locale: Node) -> String:
	return LocaleScript.format_text(loc(locale, "CHRONICLE_HUT_BUILT"), {"settlement": village(int(row.get("settlement", -1)), locale)})

## M3 kind 23 (RoomFormed): "A new room: {role}" (cause = `RoomRole::code`, 0 unassigned … 4 hall).
static func room_formed(row: Dictionary, locale: Node) -> String:
	const ROLE_KEYS: Array[String] = ["ROOM_ROLE_UNASSIGNED", "ROOM_ROLE_BEDROOM", "ROOM_ROLE_STOREHOUSE", "ROOM_ROLE_WORKSHOP", "ROOM_ROLE_HALL"]
	var cause: int = int(row.get("cause", 0))
	var role_key: String = ROLE_KEYS[cause] if cause >= 0 and cause < ROLE_KEYS.size() else ROLE_KEYS[0]
	return LocaleScript.format_text(loc(locale, "CHRONICLE_ROOM_FORMED"), {"role": loc(locale, role_key)})

## M4 kinds 24–28 (tool wear and knowledge) as BBCode sentences with person links.
static func knowledge_sentence(row: Dictionary, locale: Node) -> String:
	var kind: int = int(row.get("kind", -1))
	var subject: int = int(row.get("subject", -1))
	var other: int = int(row.get("other", -1))
	var cause: int = int(row.get("cause", -1))
	var value: int = int(row.get("value", -1))
	var node: String = "[b]%s[/b]" % KnowledgeText.node_name(locale, value)
	var node_obj: String = "[b]%s[/b]" % KnowledgeText.with_object(locale, KnowledgeText.node_name(locale, value))
	match kind:
		24:
			var tool: String = KnowledgeText.with_subject(locale, EconomyText.item(locale, cause))
			return LocaleScript.format_text(loc(locale, "CHRONICLE_TOOL_BROKE_FMT"), {"person": person(subject, locale), "item": tool, "uses": value})
		25:
			if other >= 0:
				return LocaleScript.format_text(loc(locale, "CHRONICLE_KNOWLEDGE_TAUGHT_FMT"), {"teacher": person(other, locale), "learner": person(subject, locale), "node": node_obj})
			return LocaleScript.format_text(loc(locale, "CHRONICLE_KNOWLEDGE_LEARNED_FMT"), {"learner": person(subject, locale), "node": node_obj,
				"channel": KnowledgeText.source(locale, cause)})
		26: return LocaleScript.format_text(loc(locale, "CHRONICLE_KNOWLEDGE_DISCOVERED_FMT"), {"person": person(subject, locale), "node": node_obj})
		27: return LocaleScript.format_text(loc(locale, "CHRONICLE_KNOWLEDGE_AT_RISK_FMT"), {"person": person(subject, locale), "node": node})
		28:
			if subject < 0:
				return LocaleScript.format_text(loc(locale, "CHRONICLE_KNOWLEDGE_LOST_NOBODY_FMT"), {"node": node})
			return LocaleScript.format_text(loc(locale, "CHRONICLE_KNOWLEDGE_LOST_FMT"), {"person": person(subject, locale), "node": node})
	return ""

## The KnowledgeLost line as plain text (no BBCode; the toast): "기술 소실: 문 만들기 (마지막: 카토)".
static func lost_plain(row: Dictionary, locale: Node) -> String:
	var subject: int = int(row.get("subject", -1))
	var node: String = KnowledgeText.node_name(locale, int(row.get("value", -1)))
	if subject < 0:
		return LocaleScript.format_text(loc(locale, "CHRONICLE_KNOWLEDGE_LOST_NOBODY_FMT"), {"node": node})
	return LocaleScript.format_text(loc(locale, "CHRONICLE_KNOWLEDGE_LOST_FMT"), {"person": Names.display_name(subject, language(locale)), "node": node})

static func line(row: Dictionary, viewed: int, locale: Node, info: Dictionary, world_feed: bool = false) -> String:
	var text: String = sentence(row, viewed, locale, info)
	if text.is_empty():
		return ""
	var prefix: String = loc(locale, "CHRONICLE_DAY_FMT") % day(int(row.get("tick", 0)), info)
	if world_feed and int(row.get("subject", -1)) >= 0 and int(row.get("porter_count", 0)) == 0 and int(row.get("kind", -1)) not in [20, 21, 24, 25, 26, 27, 28]:
		text = person(int(row["subject"]), locale) + " — " + text
	return prefix + " · " + text

static func partner(entries: Array[Dictionary], viewed: int) -> int:
	var result: int = -1
	for row: Dictionary in entries:
		var subject: int = int(row.get("subject", -1))
		var other: int = int(row.get("other", -1))
		if int(row.get("kind", -1)) == 2:
			result = other if subject == viewed else subject
		elif int(row.get("kind", -1)) == 10 and subject == viewed:
			result = other
	return result
