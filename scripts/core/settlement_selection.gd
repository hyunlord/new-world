extends RefCounted
## Presentation identity persists through dissolution and is cleared only on world reset.
const PALETTE: Array[Color] = [Color("E0B060"), Color("7FB0D8"), Color("B88AD0"), Color("8CC07A")]
static var selected_id: int = -1
static var _numbers: Dictionary = {}
static var _history: Dictionary = {}
static var _homes: Dictionary = {}
static var _alive: Array[int] = []
static var _retired: Dictionary = {}
static var _casts: Dictionary = {}

static func select(id: int) -> void:
	selected_id = id

static func clear() -> void:
	selected_id = -1

static func reset() -> void:
	clear()
	_numbers.clear()
	_history.clear()
	_homes.clear()
	_alive.clear()
	_retired.clear()
	_casts.clear()

static func observe(society: Dictionary, settlements: Dictionary = {}) -> void:
	var ids: PackedInt32Array = society.get("settlement_ids", settlements.get("settlement_ids", PackedInt32Array()))
	var known: Array[int] = []
	for key: String in ["settlement_ids", "parent_ids", "fission_parents", "fission_daughters"]:
		for id: int in society.get(key, PackedInt32Array()):
			if id >= 0 and not known.has(id):
				known.append(id)
	for id: int in ids:
		if not known.has(id):
			known.append(id)
	known.sort()
	for id: int in known:
		if not _numbers.has(id):
			_numbers[id] = _numbers.size() + 1
			_history[id] = {"parent_id": -1, "member_count": 0}
	_alive.assign(Array(ids))
	_alive.sort()
	var parents: PackedInt32Array = society.get("parent_ids", PackedInt32Array())
	var counts: PackedInt32Array = society.get("member_counts", settlements.get("member_counts", PackedInt32Array()))
	for index: int in ids.size():
		var row: Dictionary = _history[ids[index]]
		if index < parents.size():
			row["parent_id"] = parents[index]
		row["member_count"] = int(counts[index]) if index < counts.size() else 0
	var formation_ids: PackedInt32Array = settlements.get("settlement_ids", PackedInt32Array())
	var xs: PackedInt32Array = settlements.get("formation_xs", PackedInt32Array())
	var ys: PackedInt32Array = settlements.get("formation_ys", PackedInt32Array())
	for index: int in mini(formation_ids.size(), mini(xs.size(), ys.size())):
		if _history.has(formation_ids[index]):
			_history[formation_ids[index]]["tile"] = Vector2i(xs[index], ys[index])
	var daughters: PackedInt32Array = society.get("fission_daughters", PackedInt32Array())
	var fparents: PackedInt32Array = society.get("fission_parents", PackedInt32Array())
	var fx: PackedInt32Array = society.get("fission_site_xs", PackedInt32Array())
	var fy: PackedInt32Array = society.get("fission_site_ys", PackedInt32Array())
	for index: int in daughters.size():
		if daughters[index] >= 0 and _history.has(daughters[index]):
			if index < fparents.size():
				_history[daughters[index]]["parent_id"] = fparents[index]
			if index < fx.size() and index < fy.size() and not _history[daughters[index]].has("tile"):
				_history[daughters[index]]["tile"] = Vector2i(fx[index], fy[index])
	_homes.clear()
	var agents: PackedInt64Array = society.get("home_agent_ids", PackedInt64Array())
	var homes: PackedInt32Array = society.get("home_settlement_ids", PackedInt32Array())
	for index: int in mini(agents.size(), homes.size()):
		_homes[agents[index]] = homes[index]
	for agent: int in society.get("founder_agent_ids", PackedInt64Array()):
		var home: int = home_for_agent(agent)
		if _history.has(home):
			_history[home]["member_count"] = maxi(0, int(_history[home]["member_count"]) - 1)
	_retired.clear()
	var rx: PackedInt32Array = society.get("retired_hearth_xs", PackedInt32Array())
	var ry: PackedInt32Array = society.get("retired_hearth_ys", PackedInt32Array())
	for index: int in mini(rx.size(), ry.size()):
		_retired[Vector2i(rx[index], ry[index])] = true
	if selected_id >= 0 and not _alive.has(selected_id):
		clear()

static func number_for(id: int) -> int:
	return int(_numbers.get(id, 0))

static func color_for(id: int) -> Color:
	return PALETTE[posmod(number_for(id) - 1, PALETTE.size())]

static func name_for(id: int, locale: Node, daughter: bool = true) -> String:
	var parent: int = int(_history.get(id, {}).get("parent_id", -1))
	var key: String = "SETTLEMENT_LABEL_DAUGHTER_FMT" if daughter and parent >= 0 else "SETTLEMENT_LABEL_FMT"
	var format: String = str(locale.call("ltr", key)) if locale != null else key
	if not format.contains("%d"):
		return format
	return format % [number_for(id), number_for(parent)] if daughter and parent >= 0 else format % number_for(id)

static func label_for(id: int, locale: Node = null) -> String:
	return name_for(id, locale)

static func home_for_agent(agent_id: int) -> int:
	return int(_homes.get(agent_id, -1))

static func displayed_members(id: int) -> int:
	return int(_history.get(id, {}).get("member_count", 0))

static func formation_for(id: int) -> Vector2i:
	return _history.get(id, {}).get("tile", Vector2i(-1, -1))

static func alive_ids() -> Array[int]:
	return _alive.duplicate()

static func known_ids() -> Array:
	return _numbers.keys()

static func is_abandoned(id: int) -> bool:
	return not _alive.has(id) and _retired.has(formation_for(id))

static func retired_building(tile: Vector2i) -> bool:
	if _retired.has(tile):
		return true
	var nearest: int = -1
	var distance: float = INF
	var tied: bool = false
	for id: int in _history:
		var site: Vector2i = formation_for(id)
		if site.x < 0:
			continue
		var d: float = Vector2(tile).distance_squared_to(Vector2(site))
		if d < distance:
			nearest = id
			distance = d
			tied = false
		elif is_equal_approx(d, distance):
			tied = true
	return nearest >= 0 and not tied and distance <= 25.0 and is_abandoned(nearest)

static func resolved_target() -> int:
	return selected_id if selected_id >= 0 else (_alive[0] if not _alive.is_empty() else -1)

static func record_cast(tick: int, kind: String, target: int) -> void:
	_casts["%d:%s" % [tick, kind]] = target

static func cast_target(tick: int, kind: String) -> int:
	return int(_casts.get("%d:%s" % [tick, kind], -1))

static func capture_metadata(locale: Node = null) -> Dictionary:
	var rows: Array[Dictionary] = []
	for id: int in _numbers:
		var tile: Vector2i = formation_for(id)
		rows.append({"id": id, "number": number_for(id), "name": name_for(id, locale), "parent_id": int(_history[id].get("parent_id", -1)), "parent_number": number_for(int(_history[id].get("parent_id", -1))), "member_count": displayed_members(id), "tile": [tile.x, tile.y], "alive": _alive.has(id), "abandoned": is_abandoned(id), "color": color_for(id).to_html(false)})
	return {"settlements": rows, "selected_id": selected_id}
