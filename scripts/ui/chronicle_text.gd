# chronicle_text.gd: static helpers that turn packed chronicle columns into rows and localized BBCode lines
#   (day prefix, person/village links, death and porter wording); used by chronicle_panel.gd and agent_inspector_panel.gd.
#   M3 kind 23 (RoomFormed: cause = the room's role code, value = its id): "새 방이 생김 — 침실".
#   M4 kinds 24 ToolBroke (subject holder, cause item code, value uses) "Fox의 돌도끼가 부러졌다 (40번 씀)",
#   25 KnowledgeLearned (subject learner, other teacher, cause channel 2 oral / 3 observed / 4 apprenticed, value node)
#   "Fox가 Mia에게 문 만들기를 가르쳤다" (or "Mia가 문 만들기를 익혔다 (구전)" without a teacher), 26 KnowledgeDiscovered
#   "Fox가 간석기를 발견했다!", 27 KnowledgeAtRisk "⚠ 문 만들기: Fox만 안다", 28 KnowledgeLost "기술 소실: 문 만들기
#   (마지막: Fox)"; lost_plain() is the loss line without BBCode for the toast.
#   M5 (goal 3.4): kinds 29 PressureBegan "한파가 닥쳤다" / "늑대 3마리가 나타났다", 30 PressureEnded "한파가 지나갔다 · 죽은
#   사람 0", 31 Hunted "Fox가 사슴을 잡았다 (고기 3)", 32 Fight, 33 Breakdown (value = three modifier codes packed by 8
#   bits, cause = their count) "Fox가 무너졌다 — 춥게 잠, 배고픔", 34 Drafted (cause 0 drafted, 1 released, 2 left at a
#   critical need), 35 WolfKilled, 36 Sick (other = who passed it on), 37 Recovered; and the M5 Quarrel (kind 19 with
#   cause ≥ 1 = 1 + the quarrel cause) "Fox가 Mia와 다퉜다 — 배고파서". An entry with `link` ≥ 0 ends in a "→ 왜?" link
#   (`why:<index>`) when the caller asks for it; why_lines() renders a get_chronicle_why chain (entries as their text,
#   modifiers by name, 100 as "사이가 나빠서", pressures by name); self_undraft_plain() is the toast.
#   M5.5 (goal §9): kinds 38 MigrantsArrived (cause neighbour, value wave << 8 | count) "붉은바위에서 이주민 3명이 왔다",
#   39 MigrantsJoined (subject the first person, cause neighbour, value wave << 8 | count), 40 MigrantsDeclined (value
#   reason << 24 | wave << 8 | count; reason 0 declined, 1 unanswered), 41 TraderArrived, 42 Traded (value give << 16 |
#   take; take 0 = a gift), 43 TraderLeft (cause neighbour) and 44 Eureka (subject discoverer, cause the artifact's item
#   code, value node) "카토가 토기 보고 토기 빚기를 발견했다! (단서)"; neighbours are named through NeighbourText (the last
#   get_neighbours read its callers observed).
# Bridge methods: none (formats chronicle rows and world info its callers read: ticks_per_day, ticks_per_year,
#   clock_offset_ticks from get_world_info).
extends RefCounted

const KoreanText = preload("res://scripts/ui/mux/korean_text.gd")
const LocaleScript = preload("res://scripts/core/locale.gd")
const Names = preload("res://scripts/ui/person_names.gd")
const Selection = preload("res://scripts/core/settlement_selection.gd")
const EconomyText = preload("res://scripts/ui/economy_text.gd")
const KnowledgeText = preload("res://scripts/ui/knowledge_text.gd")
const StoryText = preload("res://scripts/ui/story_text.gd")
const NeighbourText = preload("res://scripts/ui/neighbour_text.gd")
## MindChoice (47) records only entry into the critical band; its node slot is not a display name.
const LAST_KIND: int = 52
static var steward_labels: Dictionary = {}
static var food_chronicle: Dictionary = {}
## M5 adds `index` (the entry's chronicle index) and its "→" link columns (absent in older reads: -1).
const COLUMNS: Array[String] = ["tick", "kind", "subject", "other", "third", "settlement", "settlement2", "cause", "value",
	"index", "link", "link_entry", "link_code", "food_daily_milli"]
const DEPART: Array[String] = ["CHRON_CAUSE_RANKED", "CHRON_CAUSE_SEED", "CHRON_CAUSE_HOUSEHOLD", "CHRON_CAUSE_ELDER_PARTNER", "CHRON_CAUSE_TOP_UP"]
const DEATH: Array[String] = ["CHRON_DEATH_STARVATION", "CHRON_DEATH_DEHYDRATION", "CHRON_DEATH_COMBAT", "CHRON_DEATH_OLD_AGE", "CHRON_DEATH_EXPOSURE"]

static func language(locale: Node) -> String:
	return str(locale.get("_current_lang")) if locale != null else "en"

static func loc(locale: Node, key: String) -> String:
	return KoreanText.words(str(locale.call("ltr", key))) if locale != null else key

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
			19:
				if cause >= 1:
					return story_sentence(row, locale)
				return loc(locale, "CHRON_QUARREL_FMT") % [person(subject, locale), village(int(row.get("settlement", -1)), locale)]
			22: return hut_built(row, locale)
			32, 36: return story_sentence(row, locale)
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
		19:
			if cause >= 1:
				return story_sentence(row, locale)
			return loc(locale, "CHRON_QUARREL_FMT") % [person(other, locale), village(parent, locale)]
		20: return loc(locale, "CHRON_VILLAGES_ESTRANGED_FMT") % [village(place, locale), village(parent, locale)]
		21: return loc(locale, "CHRON_VILLAGES_RECONCILED_FMT") % [village(place, locale), village(parent, locale)]
		22: return hut_built(row, locale)
		23: return room_formed(row, locale)
		24, 25, 26, 27, 28: return knowledge_sentence(row, locale)
		29, 30, 31, 32, 33, 34, 35, 36, 37: return story_sentence(row, locale)
		38, 39, 40, 41, 42, 43, 44: return neighbour_sentence(row, locale)
		46: return loc(locale, "MUX_STEWARD_DEFENCE_ACTION").format({"count": value})
		45: return loc(locale, "MUX_STEWARD_ACTION") + " " + loc(locale, str(steward_labels.get(cause, "UI_UNKNOWN")))
		47: return loc(locale, "CHRON_MIND_CRITICAL")
		48: return loc(locale, "M6A_CHRON_COOKED").format({"person": person(subject, locale), "item": KnowledgeText.with_object(locale, EconomyText.item(locale, cause))})
		49: return loc(locale, "M6A_CHRON_DRIED").format({"item": EconomyText.item(locale, cause), "count": value})
		50: return winter_food_sentence(row, locale)
		51: return loc(locale, "M6A_CHRON_WINTER_SURVIVED")
		52: return food_hunger_sentence(cause, value, locale)
	return ""


## M5 kinds 29–37 and the M5 quarrel (19, cause ≥ 1) as BBCode sentences with person links.
static func story_sentence(row: Dictionary, locale: Node) -> String:
	var kind: int = int(row.get("kind", -1))
	var subject: int = int(row.get("subject", -1))
	var other: int = int(row.get("other", -1))
	var cause: int = int(row.get("cause", -1))
	var value: int = int(row.get("value", -1))
	var who: String = person(subject, locale)
	match kind:
		19:
			return LocaleScript.format_text(loc(locale, "M5_CHRON_QUARREL_FMT"), {"a": who, "b": person(other, locale), "cause": StoryText.quarrel_cause(locale, cause)})
		29:
			var event: String = "began_person" if subject >= 0 else "began"
			return StoryText.content_phrase(locale, "pressure", cause, event, {"person": who, "count": value})
		30:
			return LocaleScript.format_text(loc(locale, "M5_CHRON_ENDED_FMT"), {"pressure": KnowledgeText.with_subject(locale, StoryText.pressure(locale, cause)), "deaths": maxi(0, value)})
		31:
			return StoryText.content_phrase(locale, "animal", cause, "hunted", {"person": who, "meat": maxi(0, value)})
		32:
			return LocaleScript.format_text(loc(locale, "M5_CHRON_FIGHT_FMT"), {"a": who, "b": person(other, locale), "cause": StoryText.quarrel_cause(locale, cause)})
		33:
			var words: Array[String] = []
			for k: int in clampi(cause, 0, 3):
				words.append(StoryText.modifier(locale, (value >> (8 * k)) & 0xff))
			if words.is_empty():
				return LocaleScript.format_text(loc(locale, "M5_CHRON_BREAKDOWN_PLAIN_FMT"), {"person": who})
			return LocaleScript.format_text(loc(locale, "M5_CHRON_BREAKDOWN_FMT"), {"person": who, "mods": ", ".join(words)})
		34:
			var keys: Array[String] = ["M5_CHRON_DRAFTED_FMT", "M5_CHRON_UNDRAFTED_FMT", "M5_CHRON_SELF_UNDRAFTED_FMT"]
			return LocaleScript.format_text(loc(locale, keys[clampi(cause, 0, 2)]), {"person": who})
		35:
			return StoryText.content_phrase(locale, "animal", cause, "killed" if subject >= 0 else "died", {"person": who, "left": maxi(0, value)})
		36:
			if other >= 0:
				return LocaleScript.format_text(loc(locale, "M5_CHRON_SICK_FROM_FMT"), {"person": who, "other": person(other, locale)})
			return LocaleScript.format_text(loc(locale, "M5_CHRON_SICK_FMT"), {"person": who})
		37:
			return LocaleScript.format_text(loc(locale, "M5_CHRON_RECOVERED_FMT"), {"person": who})
	return ""


## M5.5 kinds 38–44 (migrants, traders, eureka) as BBCode sentences with person links.
static func neighbour_sentence(row: Dictionary, locale: Node) -> String:
	var kind: int = int(row.get("kind", -1))
	var subject: int = int(row.get("subject", -1))
	var cause: int = int(row.get("cause", -1))
	var value: int = int(row.get("value", 0))
	var neighbour: String = "[b]%s[/b]" % NeighbourText.name_of(locale, cause)
	var count: int = value & 0xff
	match kind:
		38:
			return LocaleScript.format_text(loc(locale, "M55_CHRON_MIGRANTS_ARRIVED_FMT"), {"neighbour": neighbour, "count": count})
		39:
			var joined_key: String = "M55_CHRON_MIGRANT_JOINED_ONE_FMT" if count <= 1 else "M55_CHRON_MIGRANTS_JOINED_FMT"
			return LocaleScript.format_text(loc(locale, joined_key), {"neighbour": neighbour, "count": count, "person": person(subject, locale)})
		40:
			var declined_key: String = "M55_CHRON_MIGRANTS_UNANSWERED_FMT" if (value >> 24) & 0xff == 1 else "M55_CHRON_MIGRANTS_DECLINED_FMT"
			return LocaleScript.format_text(loc(locale, declined_key), {"neighbour": neighbour, "count": count})
		41:
			return LocaleScript.format_text(loc(locale, "M55_CHRON_TRADER_ARRIVED_FMT"), {"neighbour": neighbour})
		42:
			var give: int = (value >> 16) & 0xffff
			var take: int = value & 0xffff
			if take == 0:
				return LocaleScript.format_text(loc(locale, "M55_CHRON_GIFT_FMT"), {"neighbour": neighbour, "give": give})
			return LocaleScript.format_text(loc(locale, "M55_CHRON_TRADED_FMT"), {"neighbour": neighbour, "give": give, "take": take})
		43:
			return LocaleScript.format_text(loc(locale, "M55_CHRON_TRADER_LEFT_FMT"), {"neighbour": neighbour})
		44:
			var item: String = KnowledgeText.with_object(locale, EconomyText.item(locale, cause))
			var node: String = "[b]%s[/b]" % KnowledgeText.with_object(locale, KnowledgeText.node_name(locale, value))
			return LocaleScript.format_text(loc(locale, "M55_CHRON_EUREKA_FMT"), {"person": person(subject, locale), "item": item, "node": node})
	return ""


## M5: the "→ 왜?" link of `row` ([url=why:<index>]), "" when the entry has no link.
static func why_link(row: Dictionary, locale: Node) -> String:
	if int(row.get("link", -1)) < 0 or int(row.get("index", -1)) < 0:
		return ""
	return "[url=why:%d][color=#e8c46e]%s[/color][/url]" % [int(row["index"]), loc(locale, "M5_CHRON_WHY")]


## M5: a get_chronicle_why read as lines: the entry itself first (its sentence), then each cause — an entry as its
## sentence with its day, a mood modifier by name ("→ 배고픔"), the relation ("→ 사이가 나빠서"), a pressure by
## name ("→ 한파"). BBCode; the first line has no arrow.
static func why_lines(read: Dictionary, locale: Node, info: Dictionary) -> Array[String]:
	var out: Array[String] = []
	var links: PackedInt32Array = read.get("links", PackedInt32Array())
	var codes: PackedInt64Array = read.get("codes", PackedInt64Array())
	var ticks: PackedInt64Array = read.get("ticks", PackedInt64Array())
	var kinds: PackedInt32Array = read.get("kinds", PackedInt32Array())
	var subjects: PackedInt64Array = read.get("subjects", PackedInt64Array())
	var others: PackedInt64Array = read.get("others", PackedInt64Array())
	var causes: PackedInt64Array = read.get("causes", PackedInt64Array())
	var values: PackedInt64Array = read.get("values", PackedInt64Array())
	var food_daily: PackedInt64Array = read.get("food_daily_milli", PackedInt64Array())
	for i: int in links.size():
		var text: String = ""
		match links[i]:
			0:
				var row: Dictionary = {"tick": ticks[i] if i < ticks.size() else 0, "kind": kinds[i] if i < kinds.size() else -1,
					"subject": subjects[i] if i < subjects.size() else -1, "other": others[i] if i < others.size() else -1, "third": -1,
					"settlement": -1, "settlement2": -1, "cause": causes[i] if i < causes.size() else -1, "value": values[i] if i < values.size() else 0, "food_daily_milli": food_daily[i] if i < food_daily.size() else -1}
				text = line(row, -1, locale, info, true)
				if text.is_empty():
					text = loc(locale, "UI_UNKNOWN")
			1: text = LocaleScript.format_text(loc(locale, "M5_WHY_MODIFIER_FMT"), {"name": StoryText.modifier(locale, int(codes[i]) if i < codes.size() else -1)})
			2: text = LocaleScript.format_text(loc(locale, "M5_WHY_PRESSURE_FMT"), {"name": StoryText.pressure(locale, int(codes[i]) if i < codes.size() else -1)})
			_: text = loc(locale, "UI_UNKNOWN")
		out.append(text if i == 0 else "→ " + text)
	return out


## M5: the toast when someone left the draft at a critical need (kind 34 cause 2), without BBCode.
static func self_undraft_plain(row: Dictionary, locale: Node) -> String:
	return LocaleScript.format_text(loc(locale, "M5_TOAST_SELF_UNDRAFT_FMT"), {"person": Names.display_name(int(row.get("subject", -1)), language(locale))})


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

static func line(row: Dictionary, viewed: int, locale: Node, info: Dictionary, world_feed: bool = false, with_why: bool = false) -> String:
	var text: String = sentence(row, viewed, locale, info)
	if text.is_empty():
		return ""
	var prefix: String = loc(locale, "CHRONICLE_DAY_FMT") % day(int(row.get("tick", 0)), info)
	var kind: int = int(row.get("kind", -1))
	var named: bool = kind in [20, 21, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 48] or (kind == 19 and int(row.get("cause", 0)) >= 1)
	if world_feed and int(row.get("subject", -1)) >= 0 and int(row.get("porter_count", 0)) == 0 and not named:
		text = person(int(row["subject"]), locale) + " — " + text
	var why: String = why_link(row, locale) if with_why else ""
	return prefix + " · " + text + (" " + why if why != "" else "")

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


static func food_hunger_sentence(stage: int, value: int, locale: Node) -> String:
	if food_chronicle.is_empty():
		var parsed: Variant = JSON.parse_string(FileAccess.get_file_as_string("res://assets/ui/food_chronicle.json"))
		if parsed is Dictionary:
			food_chronicle = parsed
	var key: String = str(food_chronicle.get("hunger_stages", {}).get(str(stage), "UI_UNKNOWN"))
	return loc(locale, key).format({"value": value})


static func winter_food_sentence(row: Dictionary, locale: Node) -> String:
	var daily: int = int(row.get("food_daily_milli", -1))
	var stored: int = int(row.get("value", 0))
	var needed: int = int(row.get("cause", 0))
	var quantities: Dictionary = {"stored": stored, "needed": needed}
	if daily <= 0:
		return loc(locale, "M6A_WINTER_UNITS").format(quantities) + " · " + loc(locale, "M6A_DEMAND_ZERO" if daily == 0 else "M6A_DEMAND_UNKNOWN")
	quantities["stored_days"] = "%.1f" % (float(stored) * 1000.0 / float(daily))
	quantities["needed_days"] = "%.1f" % (float(needed) * 1000.0 / float(daily))
	return loc(locale, "M6A_WINTER_DAYS").format(quantities)
