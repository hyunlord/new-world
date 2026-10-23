# neighbour_text.gd: the M5.5 neighbour codes as player text (static helpers shared by the neighbours panel (N), the
#   migrant and trade dialogs, the visitor banner and the visitor layer, the tech map's rumour line and the chronicle).
# Maps the bridge's codes to localization keys: neighbour names (`name_keys`: the locale key itself, from the sim's
#   data file sim-core/data/colony/neighbours.ron),
#   edges (0 north, 1 east, 2 south, 3 west → 북/동/남/서), visit states (0 walking in, 1 waiting at the hearth,
#   2 leaving, 3 gone), attitude words by 20s (0 적대 … 4 친밀), sexes, the countdown in hours from a deadline tick,
#   the rumoured Neolithic nodes of a `rumoured` bitmask (node id = bit index) and the trade balance (Σ give value ×
#   buy_permille vs Σ take value × sell_permille, the bridge's rule). Chronicle rows name a neighbour by id only, so
#   observe() keeps the last get_neighbours read's names in a static table that name_of() reads; an unknown id reads
#   "이웃". Unknown codes fall back to UI_UNKNOWN.
# Bridge methods called: none (callers pass the dictionaries they read: get_neighbours, get_wave, get_caravan,
#   get_world_info).
extends RefCounted

const EconomyText = preload("res://scripts/ui/economy_text.gd")
const KnowledgeText = preload("res://scripts/ui/knowledge_text.gd")

const DIR_KEYS: Array[String] = ["M55_DIR_NORTH", "M55_DIR_EAST", "M55_DIR_SOUTH", "M55_DIR_WEST"]
const ATTITUDE_KEYS: Array[String] = ["M55_ATT_0", "M55_ATT_1", "M55_ATT_2", "M55_ATT_3", "M55_ATT_4"]
const ATTITUDE_COLORS: Array[Color] = [Color("e05a48"), Color("d8a050"), Color("c8b89a"), Color("a8d888"), Color("8ee07a")]
## Visit states (`wave_states`, `caravan_states`).
const WALKING: int = 0
const WAITING: int = 1
const LEAVING: int = 2
const GONE: int = 3
## Wave answers (`wave_answers`).
const ANSWER_NONE: int = -1
const ANSWER_ACCEPTED: int = 0
const ANSWER_DECLINED: int = 1
const ANSWER_UNANSWERED: int = 2

## Neighbour id → name locale key, from the last get_neighbours read observed (chronicle rows carry only the id).
static var _names: Dictionary = {}


static func ltr(locale: Node, key: String) -> String:
	return EconomyText.ltr(locale, key)


static func trf(locale: Node, key: String, params: Dictionary) -> String:
	return EconomyText.trf(locale, key, params)


static func language(locale: Node) -> String:
	return str(locale.get("_current_lang")) if locale != null else "en"


## Remember the neighbours' names of a get_neighbours read (for rows that name a neighbour by id).
static func observe(read: Dictionary) -> void:
	var ids: PackedInt32Array = read.get("ids", PackedInt32Array())
	var keys: PackedStringArray = read.get("name_keys", PackedStringArray())
	for i: int in mini(ids.size(), keys.size()):
		_names[int(ids[i])] = String(keys[i])


## The name behind locale key `key` (a neighbour's `name_keys` entry); "이웃" for an empty key.
static func name_of_key(locale: Node, key: String) -> String:
	if key.is_empty():
		return ltr(locale, "M55_NB_SOMEONE")
	return ltr(locale, key)


## The name of neighbour `id` from the observed read; "이웃" when it was never seen.
static func name_of(locale: Node, id: int) -> String:
	return name_of_key(locale, String(_names[id])) if _names.has(id) else ltr(locale, "M55_NB_SOMEONE")


## The name of neighbour `id` in a get_neighbours read (observes the read first).
static func name_in(locale: Node, read: Dictionary, id: int) -> String:
	observe(read)
	return name_of(locale, id)


static func direction(locale: Node, edge: int) -> String:
	return ltr(locale, DIR_KEYS[edge]) if edge >= 0 and edge < DIR_KEYS.size() else ltr(locale, "UI_UNKNOWN")


## Attitude band 0–4 of attitude 0–100 (by 20s).
static func attitude_band(value: int) -> int:
	return clampi(floori(value / 20.0), 0, ATTITUDE_KEYS.size() - 1)


static func attitude_word(locale: Node, value: int) -> String:
	return ltr(locale, ATTITUDE_KEYS[attitude_band(value)])


static func attitude_color(value: int) -> Color:
	return ATTITUDE_COLORS[attitude_band(value)]


static func sex(locale: Node, female: bool) -> String:
	return ltr(locale, "M55_SEX_FEMALE" if female else "M55_SEX_MALE")


## Whole hours (rounded up, at least 0) from `now` to tick `deadline`; -1 when there is no deadline.
static func hours_left(deadline: int, now: int, info: Dictionary) -> int:
	if deadline < 0:
		return -1
	var per_hour: float = maxf(1.0, float(int(info.get("ticks_per_day", 1440))) / 24.0)
	return maxi(0, ceili(float(deadline - now) / per_hour))


## The Neolithic node ids of a `rumoured` bitmask (bit = node id), ascending.
static func rumoured_nodes(mask: int) -> Array[int]:
	var out: Array[int] = []
	for bit: int in 63:
		if (mask >> bit) & 1 == 1:
			out.append(bit)
	return out


## The neighbours of a get_neighbours read that know node `node` by rumour, as their ids.
static func rumour_holders(read: Dictionary, node: int) -> Array[int]:
	var out: Array[int] = []
	if node < 0 or node >= 63:
		return out
	var ids: PackedInt32Array = read.get("ids", PackedInt32Array())
	var masks: PackedInt64Array = read.get("rumoured", PackedInt64Array())
	for i: int in mini(ids.size(), masks.size()):
		if (int(masks[i]) >> node) & 1 == 1:
			out.append(int(ids[i]))
	return out


## The tech map's line "이웃 붉은바위가 앎 (소문)" for neighbour `id` (the name with the Korean subject particle).
static func rumour_line(locale: Node, read: Dictionary, id: int) -> String:
	return trf(locale, "M55_TECH_RUMOUR_FMT", {"neighbour": KnowledgeText.with_subject(locale, name_in(locale, read, id))})


## Rows of a get_neighbours read: [{id, name_key, edge, days, population, attitude, last_contact, rumoured}].
static func neighbours(read: Dictionary) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var ids: PackedInt32Array = read.get("ids", PackedInt32Array())
	var keys: PackedStringArray = read.get("name_keys", PackedStringArray())
	var edges: PackedInt32Array = read.get("edges", PackedInt32Array())
	var days: PackedInt32Array = read.get("distance_days", PackedInt32Array())
	var pops: PackedInt32Array = read.get("populations", PackedInt32Array())
	var atts: PackedInt32Array = read.get("attitudes", PackedInt32Array())
	var contacts: PackedInt32Array = read.get("last_contact", PackedInt32Array())
	var masks: PackedInt64Array = read.get("rumoured", PackedInt64Array())
	var n: int = mini(mini(ids.size(), keys.size()), mini(edges.size(), days.size()))
	n = mini(n, mini(mini(pops.size(), atts.size()), mini(contacts.size(), masks.size())))
	for i: int in n:
		out.append({"id": int(ids[i]), "name_key": String(keys[i]), "edge": int(edges[i]), "days": int(days[i]), "population": int(pops[i]),
			"attitude": int(atts[i]), "last_contact": int(contacts[i]), "rumoured": int(masks[i])})
	return out


## Waves of a get_neighbours read: [{id, neighbour, state, tile, size, deadline, answer}].
static func waves(read: Dictionary) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var ids: PackedInt32Array = read.get("wave_ids", PackedInt32Array())
	var nbs: PackedInt32Array = read.get("wave_neighbours", PackedInt32Array())
	var states: PackedInt32Array = read.get("wave_states", PackedInt32Array())
	var xs: PackedInt32Array = read.get("wave_x", PackedInt32Array())
	var ys: PackedInt32Array = read.get("wave_y", PackedInt32Array())
	var sizes: PackedInt32Array = read.get("wave_sizes", PackedInt32Array())
	var deadlines: PackedInt64Array = read.get("wave_deadline", PackedInt64Array())
	var answers: PackedInt32Array = read.get("wave_answers", PackedInt32Array())
	var n: int = mini(mini(ids.size(), nbs.size()), mini(states.size(), xs.size()))
	n = mini(n, mini(mini(ys.size(), sizes.size()), mini(deadlines.size(), answers.size())))
	for i: int in n:
		out.append({"id": int(ids[i]), "neighbour": int(nbs[i]), "state": int(states[i]), "tile": Vector2i(xs[i], ys[i]), "size": int(sizes[i]),
			"deadline": int(deadlines[i]), "answer": int(answers[i])})
	return out


## Caravans of a get_neighbours read: [{id, neighbour, state, tile, leaves, trades}].
static func caravans(read: Dictionary) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var ids: PackedInt32Array = read.get("caravan_ids", PackedInt32Array())
	var nbs: PackedInt32Array = read.get("caravan_neighbours", PackedInt32Array())
	var states: PackedInt32Array = read.get("caravan_states", PackedInt32Array())
	var xs: PackedInt32Array = read.get("caravan_x", PackedInt32Array())
	var ys: PackedInt32Array = read.get("caravan_y", PackedInt32Array())
	var leaves: PackedInt64Array = read.get("caravan_leaves", PackedInt64Array())
	var trades: PackedInt32Array = read.get("caravan_trades", PackedInt32Array())
	var n: int = mini(mini(ids.size(), nbs.size()), mini(states.size(), xs.size()))
	n = mini(n, mini(ys.size(), mini(leaves.size(), trades.size())))
	for i: int in n:
		out.append({"id": int(ids[i]), "neighbour": int(nbs[i]), "state": int(states[i]), "tile": Vector2i(xs[i], ys[i]), "leaves": int(leaves[i]),
			"trades": int(trades[i])})
	return out


## The clue (eureka) item of node `node` in a get_neighbours read, -1 when the node has no clue.
static func clue_item(read: Dictionary, node: int) -> int:
	var nodes: PackedInt32Array = read.get("clue_nodes", PackedInt32Array())
	var items: PackedInt32Array = read.get("clue_items", PackedInt32Array())
	var i: int = nodes.find(node)
	return int(items[i]) if i >= 0 and i < items.size() else -1


## People of a get_wave read: [{agent, female, age, top, know: [{node, permille, practice}]}] (know without pads; top =
## the worn top's code from `tops`, 0 none).
static func wave_people(wave: Dictionary) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var agents: PackedInt64Array = wave.get("agents", PackedInt64Array())
	var female: PackedInt32Array = wave.get("female", PackedInt32Array())
	var ages: PackedInt32Array = wave.get("age_years", PackedInt32Array())
	var tops: PackedInt32Array = wave.get("tops", PackedInt32Array())
	var nodes: PackedInt32Array = wave.get("know_nodes", PackedInt32Array())
	var permille: PackedInt32Array = wave.get("know_permille", PackedInt32Array())
	var practice: PackedInt32Array = wave.get("know_practice", PackedInt32Array())
	for i: int in mini(mini(agents.size(), female.size()), mini(ages.size(), tops.size())):
		var know: Array[Dictionary] = []
		for k: int in 3:
			var j: int = i * 3 + k
			if j < nodes.size() and int(nodes[j]) >= 0:
				know.append({"node": int(nodes[j]), "permille": int(permille[j]) if j < permille.size() else 0,
					"practice": j < practice.size() and practice[j] != 0})
		out.append({"agent": int(agents[i]), "female": female[i] != 0, "age": int(ages[i]), "top": int(tops[i]), "know": know})
	return out


## One migrant's knowledge as "문 만들기 693‰ 숙련" pieces joined by " · " ("아는 기술 없음" when none).
static func know_text(locale: Node, know: Array[Dictionary]) -> String:
	if know.is_empty():
		return ltr(locale, "M55_MIGRANT_KNOWS_NONE")
	var parts: Array[String] = []
	for k: Dictionary in know:
		var text: String = trf(locale, "M55_MIGRANT_KNOW_FMT", {"node": KnowledgeText.node_name(locale, int(k["node"])), "permille": int(k["permille"])})
		if bool(k["practice"]):
			text += " " + ltr(locale, "M55_MIGRANT_SKILLED")
		parts.append(text)
	return " · ".join(parts)


## The rate of a ‰ as text ("0.8" for 800, "1.2" for 1,200).
static func rate(permille: int) -> String:
	var whole: int = floori(permille / 1000.0)
	var rest: int = permille - whole * 1000
	if rest == 0:
		return str(whole)
	var text: String = "%d.%03d" % [whole, rest]
	while text.ends_with("0"):
		text = text.left(text.length() - 1)
	return text


## The balance of a trade: {give, take, give_weighted, take_weighted, ok} with values at 100 % (Σ unit value × count),
## weighted = value × ‰ (the bridge's rule: give × buy ‰ ≥ take × sell ‰; weighted values in thousandths).
static func balance(give_value: int, take_value: int, buy_permille: int, sell_permille: int) -> Dictionary:
	var gw: int = give_value * buy_permille
	var tw: int = take_value * sell_permille
	return {"give": give_value, "take": take_value, "give_weighted": gw, "take_weighted": tw, "ok": gw >= tw and (give_value > 0 or take_value > 0)}


## A weighted value in thousandths as text with at most one decimal ("11.2").
static func weighted(thousandths: int) -> String:
	var tenths: int = roundi(thousandths / 100.0)
	var whole: int = floori(tenths / 10.0)
	var rest: int = tenths - whole * 10
	return str(whole) if rest == 0 else "%d.%d" % [whole, rest]
