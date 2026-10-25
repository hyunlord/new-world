# Converts the immutable bulk observer into selectable alert causes.
# Bridge methods: none; caller supplies one SimReads get_ui_facts snapshot.
extends RefCounted

const Content = preload("res://scripts/ui/content_context.gd")

const TABLES: Dictionary = {
	"people": ["ids", "xs", "ys", "hps", "children", "infants", "idle", "sleeping", "roofed", "bed_xs", "bed_ys", "has_job", "drafted", "sick", "breakdown", "mothers", "fathers", "partners", "household_ids"],
	"settlements": ["ids", "xs", "ys", "food", "food_need", "winter_wood_need"],
	"stockpile_tiles": ["zones", "xs", "ys", "can_hold", "rooms", "room_roles"],
	"stacks": ["xs", "ys", "kinds", "counts", "capacities", "reserved", "zones"],
	"unreachable": ["agent_ids", "xs", "ys", "until_ticks"],
	"meat_lots": ["days", "counts"], "tech": ["ids", "practitioners"],
	"waits": ["kinds", "a", "b", "nodes"],
	"animals": ["ids", "kinds", "xs", "ys"],
	"rooms": ["ids", "roles", "centre_xs", "centre_ys"],
	"workshops": ["ids", "work_xs", "work_ys"],
}
const COLLECTIONS: Array[String] = ["settlements", "people", "storehouses", "unreachable_orders", "knowledge_waits", "techniques", "animals", "migrants", "traders", "stacks", "households"]
static var _policy: Dictionary = {}


static func adapt(raw: Dictionary) -> Dictionary:
	if raw.get("schema_version") != 1 or not raw.has_all(["enabled", "tick", "ticks_per_day", "clock_offset_ticks", "season", "season_day", "days_per_season", "meat_spoil_days"]):
		return {}
	for table: String in TABLES:
		if not raw.get(table) is Dictionary or not _columns(raw[table], TABLES[table]):
			return {}
	if not raw.get("neighbours") is Dictionary or not raw.get("story") is Dictionary:
		return {}
	var neighbours: Dictionary = raw["neighbours"]
	for prefix: String in ["wave", "caravan"]:
		if not _columns(neighbours, [prefix + "_ids", prefix + "_states", prefix + "_x", prefix + "_y"]):
			return {}
	var out: Dictionary = {"schema_version": 1, "tick": raw["tick"]}
	for collection: String in COLLECTIONS:
		out[collection] = []
	if not raw["enabled"]:
		return out
	if not raw["story"].has("firewood") or int(raw["ticks_per_day"]) <= 0:
		return {}
	var settlements: Dictionary = raw["settlements"]
	var camp: Vector2i = Vector2i.ZERO
	if not settlements["ids"].is_empty():
		camp = Vector2i(settlements["xs"][0], settlements["ys"][0])
	var storage: Dictionary = raw["stockpile_tiles"]
	var stacks: Dictionary = raw["stacks"]
	var stack_at: Dictionary = {}
	for i: int in stacks["xs"].size():
		stack_at[Vector2i(stacks["xs"][i], stacks["ys"][i])] = i
	var room_capacity: Dictionary = {}
	var room_used: Dictionary = {}
	var cells: int = 0
	for i: int in storage["xs"].size():
		if not storage["can_hold"][i]:
			continue
		cells += 1
		var room_id: int = storage["rooms"][i]
		room_capacity[room_id] = int(room_capacity.get(room_id, 0)) + 1
		var index: int = stack_at.get(Vector2i(storage["xs"][i], storage["ys"][i]), -1)
		if index >= 0 and stacks["capacities"][index] > 0 and stacks["counts"][index] >= stacks["capacities"][index]:
			room_used[room_id] = int(room_used.get(room_id, 0)) + 1
	var winter_days: int = 0
	if int(raw["season"]) >= 0:
		winter_days = maxi(0, int(raw["days_per_season"]) - int(raw["season_day"]) + 1) if int(raw["season"]) == 3 else int(raw["days_per_season"])
	# Usable story fuel is global, so every jump cause must compare it with global demand.
	var daily_firewood_need: float = 0.0
	for requirement: float in settlements["winter_wood_need"]:
		daily_firewood_need += maxf(0.0, requirement)
	for i: int in settlements["ids"].size():
		var row: Dictionary = _row(settlements["ids"][i], settlements["xs"][i], settlements["ys"][i])
		row.merge({"food": settlements["food"][i], "daily_food_need": settlements["food_need"][i], "firewood": raw["story"]["firewood"], "daily_firewood_need": daily_firewood_need, "winter_days": winter_days, "stockpile_cells": cells})
		out["settlements"].append(row)
	var people: Dictionary = raw["people"]
	for i: int in people["ids"].size():
		var row: Dictionary = _row(people["ids"][i], people["xs"][i], people["ys"][i])
		row.merge({"adult": not bool(people["children"][i]), "working": bool(people["has_job"][i]) or not bool(people["idle"][i]), "sleeping": bool(people["sleeping"][i]), "roofed": bool(people["roofed"][i]), "drafted": bool(people["drafted"][i]), "sick": bool(people["sick"][i]), "breakdown": bool(people["breakdown"][i]), "hp": people["hps"][i]})
		out["people"].append(row)
	var households: Dictionary = {}
	for i: int in people["ids"].size():
		var household_id: int = people["household_ids"][i]
		if not households.has(household_id):
			households[household_id] = {"beds": {}, "infants": []}
		var household: Dictionary = households[household_id]
		if people["bed_xs"][i] >= 0 and people["bed_ys"][i] >= 0:
			household["beds"][Vector2i(people["bed_xs"][i], people["bed_ys"][i])] = true
		if people["infants"][i]:
			household["infants"].append(_row(people["ids"][i], people["xs"][i], people["ys"][i]))
	for household: Dictionary in households.values():
		var infants: Array = household["infants"]
		if infants.is_empty():
			continue
		infants.sort_custom(func(a: Dictionary, b: Dictionary) -> bool: return a["id"] < b["id"])
		var row: Dictionary = infants[0]
		row.merge({"newborns": infants.size(), "beds": household["beds"].size()})
		out["households"].append(row)
	var rooms: Dictionary = raw["rooms"]
	for i: int in rooms["ids"].size():
		var row: Dictionary = _row(rooms["ids"][i], rooms["centre_xs"][i], rooms["centre_ys"][i])
		row.merge({"role": rooms["roles"][i], "used": room_used.get(row["id"], 0), "capacity": room_capacity.get(row["id"], 0)})
		out["storehouses"].append(row)
	var unreachable: Dictionary = raw["unreachable"]
	var unreachable_tiles: Dictionary = {}
	for i: int in unreachable["xs"].size():
		if unreachable["until_ticks"][i] <= raw["tick"]:
			continue
		var tile: Vector2i = Vector2i(unreachable["xs"][i], unreachable["ys"][i])
		if unreachable_tiles.has(tile):
			continue
		unreachable_tiles[tile] = true
		# Cooldowns retain a location, not the identity of the failed order.
		var tile_id: int = (int(tile.x) << 32) | int(tile.y)
		out["unreachable_orders"].append(_row(tile_id, tile.x, tile.y))
	var workshops: Dictionary = raw["workshops"]
	var workshop_at: Dictionary = {}
	for i: int in workshops["ids"].size():
		workshop_at[workshops["ids"][i]] = Vector2i(workshops["work_xs"][i], workshops["work_ys"][i])
	var waits: Dictionary = raw["waits"]
	for i: int in waits["nodes"].size():
		var tile: Vector2i = workshop_at.get(waits["a"][i], camp) if waits["kinds"][i] == 0 else Vector2i(waits["a"][i], waits["b"][i])
		var row: Dictionary = _row(i, tile.x, tile.y)
		row["node_id"] = waits["nodes"][i]
		out["knowledge_waits"].append(row)
	var tech: Dictionary = raw["tech"]
	for i: int in tech["ids"].size():
		var row: Dictionary = _row(tech["ids"][i], camp.x, camp.y)
		row["practitioners"] = tech["practitioners"][i]
		out["techniques"].append(row)
	var animals: Dictionary = raw["animals"]
	for i: int in animals["ids"].size():
		var row: Dictionary = _row(animals["ids"][i], animals["xs"][i], animals["ys"][i])
		var distance: float = INF
		for j: int in settlements["ids"].size():
			distance = minf(distance, Vector2(row["x"], row["y"]).distance_to(Vector2(settlements["xs"][j], settlements["ys"][j])))
		row.merge({"kind": animals["kinds"][i], "distance_to_camp": distance})
		out["animals"].append(row)
	for prefix: String in ["wave", "caravan"]:
		for i: int in neighbours[prefix + "_ids"].size():
			var row: Dictionary = _row(neighbours[prefix + "_ids"][i], neighbours[prefix + "_x"][i], neighbours[prefix + "_y"][i])
			row["waiting" if prefix == "wave" else "present"] = neighbours[prefix + "_states"][i] == 1
			out["migrants" if prefix == "wave" else "traders"].append(row)
	out["stacks"] = _spoilage(raw)
	return out


static func _spoilage(raw: Dictionary) -> Array[Dictionary]:
	var rows: Array[Dictionary] = []
	if int(raw["meat_spoil_days"]) < 0:
		return rows
	if _policy.is_empty():
		var parsed: Variant = Content.document("alerts")
		if parsed.is_empty():
			parsed = JSON.parse_string(FileAccess.get_file_as_string("res://assets/ui/alerts.json"))
		if not parsed is Dictionary:
			return rows
		_policy = parsed
	var selected_kind: int = int(_policy["fact_selectors"]["spoilable_kind"])
	var horizon: int = int(_policy["fact_selectors"]["spoil_horizon_ticks"])
	var ticks_per_day: int = int(raw["ticks_per_day"])
	var now: int = int(raw["tick"])
	# AnimalSystem runs every three ticks; a spoil pass needs both divisibilities.
	var period: int = ticks_per_day if ticks_per_day % 3 == 0 else ticks_per_day * 3
	var next_pass: int = now - now % period + period
	var due: int = 0
	var lots: Dictionary = raw["meat_lots"]
	var stacks: Dictionary = raw["stacks"]
	var allocated: Dictionary = {}
	while next_pass - now <= horizon:
		@warning_ignore("integer_division")
		var display_day: int = (next_pass + int(raw["clock_offset_ticks"])) / ticks_per_day
		var total_due: int = 0
		for i: int in lots["days"].size():
			if lots["days"][i] + int(raw["meat_spoil_days"]) <= display_day:
				total_due += int(lots["counts"][i])
		due = total_due
		for i: int in stacks["xs"].size():
			if stacks["kinds"][i] != selected_kind or stacks["reserved"][i]:
				continue
			var count: int = mini(due, int(stacks["counts"][i]))
			due -= count
			if count <= 0 or allocated.has(i):
				continue
			allocated[i] = true
			var row: Dictionary = _row(i, stacks["xs"][i], stacks["ys"][i])
			row.merge({"kind": selected_kind, "in_stockpile": stacks["zones"][i] >= 0, "spoil_ticks_left": next_pass - now, "count": count})
			rows.append(row)
		next_pass += period
	return rows


static func _columns(table: Dictionary, keys: Array) -> bool:
	var length: int = -1
	for key: String in keys:
		var column: Variant = table.get(key)
		if not (column is Array or column is PackedInt64Array or column is PackedInt32Array or column is PackedByteArray or column is PackedFloat64Array):
			return false
		if length >= 0 and column.size() != length:
			return false
		length = column.size()
	return true


static func _row(id: int, x: int, y: int) -> Dictionary:
	return {"id": id, "x": x, "y": y}
