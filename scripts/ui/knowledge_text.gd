# knowledge_text.gd: the M4 knowledge codes as player text (static helpers shared by the HUD, the tech map (T), the
#   teach dialog (K), the inspector's 지식 tab, the chronicle, the workshop panel and the "why?" panel).
# Maps node ids to their registry name keys (the selected knowledge catalog from
#   file order; a get_tech read's `name_keys` wins when the caller has one), lanes (0 survival … 5 social), eras
#   (columns 0 paleolithic / 1 neolithic; labels 0 구석기, 1 신석기 전환기, 2 신석기 시대), tech statuses (0 unknown,
#   1 at risk, 2 known, 3 stable, 4 lost) with their card colours, sources (0 seeded 전승, 1 discovered 발견, 2 oral 구전,
#   3 observed 관찰, 4 apprenticed 도제), entry states (0 숙련, 1 배우는 중, 2 잊어 감), discovery conditions, gates and
#   the "기다리는 중: … 아는 사람이 없음" line (get_knowledge_waits). Korean object/subject particles (을/를, 이/가) are
#   chosen from the last Hangul syllable of a name.
# M5: the hunt designation (gated by tracking) in DESIGNATION_KEYS.
# Bridge methods called: none (callers pass the dictionaries they read: get_tech, get_tech_node, get_knowledge_waits,
#   get_colony_overlay).
extends RefCounted

const EconomyText = preload("res://scripts/ui/economy_text.gd")

const Content = preload("res://scripts/ui/content_context.gd")

const LANE_KEYS: Array[String] = ["M4_LANE_SURVIVAL", "M4_LANE_CRAFT", "M4_LANE_BUILD", "M4_LANE_MINE", "M4_LANE_POTTERY", "M4_LANE_SOCIAL"]
const ERA_COLUMN_KEYS: Array[String] = ["M4_ERA_COL_PALEOLITHIC", "M4_ERA_COL_NEOLITHIC"]
const ERA_LABEL_KEYS: Array[String] = ["M4_ERA_PALEOLITHIC", "M4_ERA_TRANSITION", "M4_ERA_NEOLITHIC"]
const STATUS_KEYS: Array[String] = ["M4_STATUS_UNKNOWN", "M4_STATUS_AT_RISK", "M4_STATUS_KNOWN", "M4_STATUS_STABLE", "M4_STATUS_LOST"]
## Card border/accent colour per status: 미지 grey, 위험 red, 알려짐 yellow, 안정 green, 소실 black.
const STATUS_COLORS: Array[Color] = [Color("8a8278"), Color("e05a48"), Color("e8c46e"), Color("8ee07a"), Color("0c0a08")]
const SOURCE_KEYS: Array[String] = ["M4_SOURCE_SEEDED", "M4_SOURCE_DISCOVERED", "M4_SOURCE_ORAL", "M4_SOURCE_OBSERVED", "M4_SOURCE_APPRENTICED"]
const STATE_KEYS: Array[String] = ["M4_ENTRY_SKILLED", "M4_ENTRY_LEARNING", "M4_ENTRY_FORGETTING"]
const STATE_COLORS: Array[Color] = [Color("8ee07a"), Color("e8d26e"), Color("b08878")]
const COND_KEYS: Array[String] = ["M4_COND_STOCKPILE_FMT", "M4_COND_SKILL_FMT", "M4_COND_KNOWS_FMT", "M4_COND_CLAY_FMT"]
const GATE_KEYS: Array[String] = ["M4_GATE_RECIPE_FMT", "M4_GATE_BLUEPRINT_FMT", "M4_GATE_WORKSHOP_FMT", "M4_GATE_DESIGNATION_FMT", "M4_GATE_CHANNEL_FMT", "M4_GATE_LORE_FMT"]
const DESIGNATION_KEYS: Dictionary = {"chop": "UI_ORDER_CHOP", "quarry": "UI_ORDER_QUARRY", "dig": "UI_ORDER_DIG", "deconstruct": "UI_ORDER_DECONSTRUCT", "clay_pit": "UI_ORDER_CLAY",
	"hunt": "M5_ORDER_HUNT"}
const CHANNEL_KEYS: Dictionary = {"apprenticeship": "M4_CHANNEL_APPRENTICESHIP", "oral": "M4_CHANNEL_ORAL"}
const LORE_KEYS: Dictionary = {"hearth": "M4_LORE_HEARTH", "forage": "M4_LORE_FORAGE", "water": "M4_LORE_WATER"}
## Lanes and era columns as the bridge codes them.
const LANES: int = 6
const ERAS: int = 2


static func ltr(locale: Node, key: String) -> String:
	return EconomyText.ltr(locale, key)


static func trf(locale: Node, key: String, params: Dictionary) -> String:
	return EconomyText.trf(locale, key, params)


static func language(locale: Node) -> String:
	return str(locale.get("_current_lang")) if locale != null else "en"


## The locale key of node `id` (`tech` = a get_tech read, may be empty).
static func node_key(id: int, tech: Dictionary = {}) -> String:
	var keys: PackedStringArray = tech.get("name_keys", PackedStringArray())
	if id >= 0 and id < keys.size():
		return keys[id]
	return str(Content.row("knowledge", id).get("name_key", "UI_UNKNOWN"))


## The player name of node `id`.
static func node_name(locale: Node, id: int, tech: Dictionary = {}) -> String:
	return ltr(locale, node_key(id, tech))


## The node id of stable name `node_id_name` (a knowledge.ron id), -1 when unknown.
static func node_id(node_id_name: String) -> int:
	return Content.code("knowledge", node_id_name)


static func lane(locale: Node, code: int) -> String:
	return ltr(locale, LANE_KEYS[code]) if code >= 0 and code < LANE_KEYS.size() else ltr(locale, "UI_UNKNOWN")


static func era_column(locale: Node, code: int) -> String:
	return ltr(locale, ERA_COLUMN_KEYS[code]) if code >= 0 and code < ERA_COLUMN_KEYS.size() else ltr(locale, "UI_UNKNOWN")


static func era_label(locale: Node, code: int) -> String:
	return ltr(locale, ERA_LABEL_KEYS[clampi(code, 0, ERA_LABEL_KEYS.size() - 1)])


static func status(locale: Node, code: int) -> String:
	return ltr(locale, STATUS_KEYS[code]) if code >= 0 and code < STATUS_KEYS.size() else ltr(locale, "UI_UNKNOWN")


static func status_color(code: int) -> Color:
	return STATUS_COLORS[code] if code >= 0 and code < STATUS_COLORS.size() else STATUS_COLORS[0]


static func source(locale: Node, code: int) -> String:
	return ltr(locale, SOURCE_KEYS[code]) if code >= 0 and code < SOURCE_KEYS.size() else ltr(locale, "UI_UNKNOWN")


static func entry_state(locale: Node, code: int) -> String:
	return ltr(locale, STATE_KEYS[code]) if code >= 0 and code < STATE_KEYS.size() else ltr(locale, "UI_UNKNOWN")


static func state_color(code: int) -> Color:
	return STATE_COLORS[code] if code >= 0 and code < STATE_COLORS.size() else STATE_COLORS[0]


## The HUD era line "신석기 전환기 31 %" from a get_tech read (era label code, progress ‰ shown as whole percent).
static func era_line(locale: Node, tech: Dictionary) -> String:
	@warning_ignore("integer_division")
	var percent: int = maxi(0, int(tech.get("progress", 0))) / 10
	return trf(locale, "M4_HUD_ERA_FMT", {"era": era_label(locale, int(tech.get("era", 0))), "percent": percent})


## Whether the last Hangul syllable of `text` has a final consonant (받침); false when there is no Hangul.
static func has_final(text: String) -> bool:
	for i: int in range(text.length() - 1, -1, -1):
		var code: int = text.unicode_at(i)
		if code >= 0xAC00 and code <= 0xD7A3:
			return (code - 0xAC00) % 28 != 0
	return false


## `text` with the Korean object particle (을/를) in Korean; unchanged in other languages.
static func with_object(locale: Node, text: String) -> String:
	if language(locale) != "ko":
		return text
	return text + ("을" if has_final(text) else "를")


## `text` with the Korean subject particle (이/가) in Korean; unchanged in other languages.
static func with_subject(locale: Node, text: String) -> String:
	if language(locale) != "ko":
		return text
	return text + ("이" if has_final(text) else "가")


## "3일 전" (`days` whole days since the last use; 0 = 오늘).
static func days_ago(locale: Node, days: int) -> String:
	return ltr(locale, "M4_USED_TODAY") if days <= 0 else trf(locale, "M4_DAYS_AGO_FMT", {"days": days})


## The item kind code of a bridge item name ("flint_blade" -> 4), -1 when unknown.
static func item_code(item_name: String) -> int:
	return EconomyText.item_code(item_name)


## One discovery condition of a get_tech_node read (row `i`) as player text, without the met mark.
static func condition(locale: Node, node: Dictionary, i: int) -> String:
	var kinds: PackedInt64Array = node.get("cond_kinds", PackedInt64Array())
	var names: PackedStringArray = node.get("cond_names", PackedStringArray())
	var amounts: PackedInt64Array = node.get("cond_amounts", PackedInt64Array())
	if i < 0 or i >= mini(kinds.size(), mini(names.size(), amounts.size())):
		return ltr(locale, "UI_UNKNOWN")
	var kind: int = int(kinds[i])
	var amount: int = int(amounts[i])
	var name: String = names[i]
	match kind:
		0: return trf(locale, COND_KEYS[0], {"item": EconomyText.item(locale, item_code(name)), "amount": amount})
		1: return trf(locale, COND_KEYS[1], {"skill": EconomyText.skill(locale, name), "amount": amount})
		2:
			@warning_ignore("integer_division")
			var percent: int = amount / 10
			return trf(locale, COND_KEYS[2], {"node": with_object(locale, node_name(locale, node_id(name))), "percent": percent})
		3: return trf(locale, COND_KEYS[3], {"amount": amount})
	return ltr(locale, "UI_UNKNOWN")


## What a gate of a get_tech_node read (row `i`) covers, as "제작: 토기 굽기".
static func gate(locale: Node, node: Dictionary, i: int) -> String:
	var kinds: PackedInt64Array = node.get("gate_kinds", PackedInt64Array())
	var names: PackedStringArray = node.get("gate_names", PackedStringArray())
	if i < 0 or i >= mini(kinds.size(), names.size()):
		return ltr(locale, "UI_UNKNOWN")
	var kind: int = int(kinds[i])
	var name: String = names[i]
	var what: String = ltr(locale, "UI_UNKNOWN")
	match kind:
		0: what = EconomyText.recipe(locale, EconomyText.recipe_code(name))
		1, 2: what = EconomyText.building(locale, EconomyText.build_code(name))
		3: what = ltr(locale, str(DESIGNATION_KEYS.get(name, "UI_UNKNOWN")))
		4: what = ltr(locale, str(CHANNEL_KEYS.get(name, "UI_UNKNOWN")))
		5: what = ltr(locale, str(LORE_KEYS.get(name, "UI_UNKNOWN")))
	if kind < 0 or kind >= GATE_KEYS.size():
		return what
	return trf(locale, GATE_KEYS[kind], {"name": what})


## "기다리는 중: 토기 굽기를 아는 사람이 없음" for node `id`.
static func wait_line(locale: Node, id: int, tech: Dictionary = {}) -> String:
	return trf(locale, "M4_WAIT_FMT", {"node": with_object(locale, node_name(locale, id, tech))})


## Rows of a get_knowledge_waits read: [{kind, a, b, node}].
static func waits(read: Dictionary) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var kinds: PackedInt64Array = read.get("kinds", PackedInt64Array())
	var a: PackedInt64Array = read.get("a", PackedInt64Array())
	var b: PackedInt64Array = read.get("b", PackedInt64Array())
	var nodes: PackedInt64Array = read.get("nodes", PackedInt64Array())
	for i: int in mini(mini(kinds.size(), a.size()), mini(b.size(), nodes.size())):
		out.append({"kind": int(kinds[i]), "a": int(a[i]), "b": int(b[i]), "node": int(nodes[i])})
	return out


## The node a bill waits for (kind 0, workshop `shop`, bill `bill`), -1 when it does not wait.
static func bill_wait(read: Dictionary, shop: int, bill: int) -> int:
	for row: Dictionary in waits(read):
		if int(row["kind"]) == 0 and int(row["a"]) == shop and int(row["b"]) == bill:
			return int(row["node"])
	return -1


## The node a site or mark at `tile` waits for, -1 when none: a mark (kind 2) on the tile itself, or a site (kind 1,
## keyed by its centre) whose footprint in `overlay` (a get_colony_overlay read) holds the tile.
static func tile_wait(read: Dictionary, overlay: Dictionary, tile: Vector2i) -> int:
	var rows: Array[Dictionary] = waits(read)
	for row: Dictionary in rows:
		if int(row["kind"]) == 2 and Vector2i(int(row["a"]), int(row["b"])) == tile:
			return int(row["node"])
	var sxs: PackedInt32Array = overlay.get("site_xs", PackedInt32Array())
	var sys: PackedInt32Array = overlay.get("site_ys", PackedInt32Array())
	var sws: PackedInt32Array = overlay.get("site_ws", PackedInt32Array())
	var shs: PackedInt32Array = overlay.get("site_hs", PackedInt32Array())
	for i: int in mini(mini(sxs.size(), sys.size()), mini(sws.size(), shs.size())):
		var rect: Rect2i = Rect2i(sxs[i], sys[i], sws[i], shs[i])
		if not rect.has_point(tile):
			continue
		for row: Dictionary in rows:
			if int(row["kind"]) == 1 and rect.has_point(Vector2i(int(row["a"]), int(row["b"]))):
				return int(row["node"])
	return -1


## Nodes of a get_tech read as rows {id, lane, era, status, practitioners, holders, marker, first, threshold}.
static func nodes(tech: Dictionary) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var ids: PackedInt64Array = tech.get("ids", PackedInt64Array())
	var lanes: PackedInt64Array = tech.get("lanes", PackedInt64Array())
	var eras: PackedInt64Array = tech.get("eras", PackedInt64Array())
	var statuses: PackedInt64Array = tech.get("statuses", PackedInt64Array())
	var practitioners: PackedInt64Array = tech.get("practitioners", PackedInt64Array())
	var holders: PackedInt64Array = tech.get("holders", PackedInt64Array())
	var markers: PackedInt64Array = tech.get("marker_weights", PackedInt64Array())
	var firsts: PackedInt64Array = tech.get("first_names", PackedInt64Array())
	var thresholds: PackedInt64Array = tech.get("thresholds", PackedInt64Array())
	var n: int = mini(mini(ids.size(), lanes.size()), mini(eras.size(), statuses.size()))
	n = mini(n, mini(mini(practitioners.size(), holders.size()), mini(markers.size(), mini(firsts.size(), thresholds.size()))))
	for i: int in n:
		out.append({"id": int(ids[i]), "lane": int(lanes[i]), "era": int(eras[i]), "status": int(statuses[i]),
			"practitioners": int(practitioners[i]), "holders": int(holders[i]), "marker": int(markers[i]),
			"first": int(firsts[i]), "threshold": int(thresholds[i])})
	return out


## The first at-risk node of a get_tech read as {id, holder}, or {} when none.
static func first_at_risk(tech: Dictionary) -> Dictionary:
	for row: Dictionary in nodes(tech):
		if int(row["status"]) == 1:
			return {"id": int(row["id"]), "holder": int(row["first"])}
	return {}



static func node_count() -> int:
	return Content.rows("knowledge").size()
