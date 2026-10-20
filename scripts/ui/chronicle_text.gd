# chronicle_text.gd: static helpers that turn packed chronicle columns into rows and localized BBCode lines
#   (day prefix, person/village links, death and porter wording); used by chronicle_panel.gd and agent_inspector_panel.gd.
# Bridge methods: none (formats chronicle rows and world info its callers read).
extends RefCounted
const LocaleScript = preload("res://scripts/core/locale.gd")
const Names = preload("res://scripts/ui/person_names.gd")
const Selection = preload("res://scripts/core/settlement_selection.gd")
const COLUMNS: Array[String] = ["tick", "kind", "subject", "other", "third", "settlement", "settlement2", "cause", "value"]
const DEPART: Array[String] = ["CHRON_CAUSE_RANKED", "CHRON_CAUSE_SEED", "CHRON_CAUSE_HOUSEHOLD", "CHRON_CAUSE_ELDER_PARTNER", "CHRON_CAUSE_TOP_UP"]
const DEATH: Array[String] = ["CHRON_DEATH_STARVATION", "CHRON_DEATH_DEHYDRATION", "CHRON_DEATH_COMBAT", "CHRON_DEATH_OLD_AGE", "CHRON_DEATH_EXPOSURE"]

static func language(locale: Node) -> String:
	return str(locale.get("_current_lang")) if locale != null else "en"

static func loc(locale: Node, key: String) -> String:
	return str(locale.call("ltr", key)) if locale != null else key

static func day(tick: int, info: Dictionary) -> int:
	return floori(float(tick + int(info.get("clock_offset_ticks", 0))) / float(maxi(1, int(info.get("ticks_per_day", 1))))) + 1

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
			return loc(locale, "CHRON_DIED_FMT") % [floori(float(value) / float(maxi(1, int(info.get("ticks_per_day", 1))))), death(cause, locale)] if value >= 0 else loc(locale, "CHRON_DIED_NO_AGE_FMT") % death(cause, locale)
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
	return ""


## M1 kind 22 (HutBuilt): "A hut was finished in {settlement}".
static func hut_built(row: Dictionary, locale: Node) -> String:
	return LocaleScript.format_text(loc(locale, "CHRONICLE_HUT_BUILT"), {"settlement": village(int(row.get("settlement", -1)), locale)})

static func line(row: Dictionary, viewed: int, locale: Node, info: Dictionary, world_feed: bool = false) -> String:
	var text: String = sentence(row, viewed, locale, info)
	if text.is_empty():
		return ""
	var prefix: String = loc(locale, "CHRONICLE_DAY_FMT") % day(int(row.get("tick", 0)), info)
	if world_feed and int(row.get("subject", -1)) >= 0 and int(row.get("porter_count", 0)) == 0 and int(row.get("kind", -1)) not in [20, 21]:
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
