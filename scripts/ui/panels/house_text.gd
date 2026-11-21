# house_text.gd: HOUSE1 house lookups and text shared by the room panel's house section and the inspector's home line
#   ("카토네 집", "사는 곳: 카토네 집 (더부살이)"). A lodger is a resident who is neither in the owning household nor in
#   one of its houseless sub-households (ADR 0073, D-HOUSE1-1).
# Bridge methods: via SimReads get_house_snapshot, get_household_snapshot, get_world_info.
extends RefCounted

const Names = preload("res://scripts/ui/person_names.gd")
const Chronicle = preload("res://scripts/ui/chronicle_text.gd")
const Content = preload("res://scripts/ui/content_context.gd")
const KoreanText = preload("res://scripts/ui/mux/korean_text.gd")


static func _ltr(locale: Node, key: String) -> String:
	return KoreanText.words(str(locale.call("ltr", key))) if locale != null else key


static func rows(world: Node) -> Array:
	return SimReads.read(world, "get_house_snapshot").get("rows", [])


static func household(world: Node, id: int) -> Dictionary:
	if id < 0:
		return {}
	for row: Dictionary in SimReads.read(world, "get_household_snapshot").get("rows", []):
		if int(row.get("id", -1)) == id:
			return row
	return {}


## The house row with id `house_id` ({} when none).
static func by_id(world: Node, house_id: int) -> Dictionary:
	for row: Dictionary in rows(world):
		if int(row.get("house_id", -1)) == house_id:
			return row
	return {}


## The house whose footprint holds room `room` ({} when none).
static func of_room(world: Node, room: int) -> Dictionary:
	if room < 0:
		return {}
	for row: Dictionary in rows(world):
		if (row.get("room_ids", PackedInt64Array()) as PackedInt64Array).has(room):
			return row
	return {}


## Whether resident `agent` of house `row` belongs to the owners (household or a houseless sub-household).
static func is_family(world: Node, row: Dictionary, agent: int) -> bool:
	if (row.get("household_member_ids", PackedInt64Array()) as PackedInt64Array).has(agent):
		return true
	var owner: int = int(row.get("owner_household", -1))
	if owner < 0:
		return false
	for other: Dictionary in SimReads.read(world, "get_household_snapshot").get("rows", []):
		if int(other.get("parent", -1)) == owner and (other.get("owned_houses", PackedInt64Array()) as PackedInt64Array).is_empty() \
				and (other.get("members", PackedInt64Array()) as PackedInt64Array).has(agent):
			return true
	return false


## Where `agent` lives: {house: row, lodger: bool}; the house they sleep in, else one their household owns; {} homeless.
static func home_of(world: Node, agent: int) -> Dictionary:
	var all: Array = rows(world)
	for row: Dictionary in all:
		if (row.get("resident_ids", PackedInt64Array()) as PackedInt64Array).has(agent):
			return {"house": row, "lodger": not is_family(world, row, agent)}
	for row: Dictionary in all:
		if (row.get("household_member_ids", PackedInt64Array()) as PackedInt64Array).has(agent):
			return {"house": row, "lodger": false}
	return {}


## "카토네 집" after the owning household's head; "빈 집" when vacant.
static func title(world: Node, row: Dictionary, locale: Node) -> String:
	var head: int = int(household(world, int(row.get("owner_household", -1))).get("head", -1))
	if head < 0:
		return _ltr(locale, "HOUSE_VACANT")
	return _ltr(locale, "HOUSE_NAME_FMT").format({"name": Names.present(head, Chronicle.language(locale))})


static func _names(ids: Array[int], locale: Node) -> String:
	if ids.is_empty():
		return _ltr(locale, "PANEL_FAMILY_NONE")
	var out: PackedStringArray = []
	for id: int in ids:
		out.append(Names.present(id, Chronicle.language(locale)))
	return ", ".join(out)


## The section lines: first completed day, template and era, rooms/beds/hearths, family, lodgers, free beds.
static func lines(world: Node, row: Dictionary, locale: Node) -> Array[String]:
	var first: int = int(row.get("first_completed_tick", -1))
	var info: Dictionary = SimReads.read(world, "get_world_info")
	var date: String = _ltr(locale, "CHRONICLE_DAY_FMT") % Chronicle.day(first, info) if first >= 0 else _ltr(locale, "UI_UNKNOWN")
	var era_key: String = str(Content.row("eras", Content.code("eras", str(row.get("era_key", "")))).get("name_key", "UI_UNKNOWN"))
	var beds: int = 0
	for slots: int in row.get("bed_slots", PackedInt64Array()):
		beds += slots
	var family: Array[int] = []
	var lodgers: Array[int] = []
	for id: int in row.get("household_member_ids", PackedInt64Array()):
		family.append(id)
	for id: int in row.get("resident_ids", PackedInt64Array()):
		if family.has(id):
			continue
		if is_family(world, row, id):
			family.append(id)
		else:
			lodgers.append(id)
	var capacity: int = int(row.get("capacity", 0))
	return [
		_ltr(locale, "HOUSE_FIRST_BUILT_FMT").format({"date": date}),
		_ltr(locale, "HOUSE_KIND_FMT").format({"template": _ltr(locale, str(row.get("template_name_key", "UI_UNKNOWN"))),
			"era": _ltr(locale, era_key)}),
		_ltr(locale, "HOUSE_SPACE_FMT").format({"rooms": (row.get("room_ids", PackedInt64Array()) as PackedInt64Array).size(),
			"beds": beds, "hearths": (row.get("hearth_cells", PackedInt32Array()) as PackedInt32Array).size() >> 1}),
		_ltr(locale, "HOUSE_FAMILY_FMT").format({"names": _names(family, locale)}),
		_ltr(locale, "HOUSE_LODGERS_FMT").format({"names": _names(lodgers, locale)}),
		_ltr(locale, "HOUSE_VACANCY_FMT").format({"free": maxi(0, capacity - int(row.get("resident_count", 0))), "capacity": capacity}),
	]


## The inspector's home line for a `home_of` result.
static func home_line(world: Node, home: Dictionary, locale: Node) -> String:
	if home.is_empty():
		return _ltr(locale, "HOUSE_HOME_NONE")
	var key: String = "HOUSE_HOME_LODGER_FMT" if bool(home["lodger"]) else "HOUSE_HOME_FMT"
	return _ltr(locale, key).format({"house": title(world, home["house"], locale)})
