# learning_facts.gd: converts authoritative snapshots into declarative learning and goal facts.
# Bridge methods: via SimReads get_ui_facts and get_colony_overlay.
extends RefCounted

static func collect(world: Node, steward: Dictionary = {}) -> Dictionary:
	return from_snapshots(SimReads.read(world, "get_ui_facts"), SimReads.read(world, "get_colony_overlay"), steward)

static func from_snapshots(bulk: Dictionary, overlay: Dictionary, steward: Dictionary = {}) -> Dictionary:
	var people: Dictionary = bulk.get("people", {})
	var shops: Dictionary = bulk.get("workshops", {})
	var tech: Dictionary = bulk.get("tech", {})
	var story: Dictionary = bulk.get("story", {})
	var neighbours: Dictionary = bulk.get("neighbours", {})
	var storage: Dictionary = bulk.get("stockpile_tiles", {})
	var facts: Dictionary = {
		"stockpile_exists": not storage.get("zones", []).is_empty(),
		"chop_marked": overlay.get("designation_kinds", []).has(0),
		"house_beds_ready": beds_ready(people, bulk.get("rooms", {})),
		"workshop_bill_exists": not shops.get("bill_ids", []).is_empty(),
		"winter_firewood_ready": winter_wood_ready(bulk.get("settlements", {})),
		"first_winter_all_alive": bulk.get("first_winter_all_alive", false) == true,
		"taught_technique": int(bulk.get("taught_count", 0)) > 0,
		"drafted": people.get("drafted", []).has(1),
		"wolves_seen": int(bulk.get("wolves_spawned", 0)) > 0 or int(story.get("wolves", 0)) > 0,
		"migrants_seen": int(bulk.get("migrant_waves_arrived", 0)) > 0 or int(neighbours.get("migrants_joined", 0)) > 0,
		"era_progress_permille": clampi(int(tech.get("progress", 0)), 0, 1000),
		"era_markers": era_markers(tech),
		"steward_accepted_rules": steward.get("accepted_rules", []).duplicate(),
	}
	return facts

static func beds_ready(people: Dictionary, rooms: Dictionary) -> bool:
	var ids: Variant = people.get("ids", [])
	var xs: Variant = people.get("bed_xs", [])
	var ys: Variant = people.get("bed_ys", [])
	if ids.is_empty() or xs.size() != ids.size() or ys.size() != ids.size():
		return false
	var roofed_rooms: Dictionary = {}
	var room_ids: Variant = rooms.get("ids", [])
	var roofs: Variant = rooms.get("roofed", [])
	for i: int in mini(room_ids.size(), roofs.size()):
		if int(roofs[i]) != 0:
			roofed_rooms[int(room_ids[i])] = true
	var tiles: Dictionary = {}
	var tx: Variant = rooms.get("tile_xs", [])
	var ty: Variant = rooms.get("tile_ys", [])
	var tr: Variant = rooms.get("tile_rooms", [])
	for i: int in mini(tx.size(), mini(ty.size(), tr.size())):
		if roofed_rooms.has(int(tr[i])):
			tiles[Vector2i(int(tx[i]), int(ty[i]))] = true
	for i: int in ids.size():
		if int(xs[i]) < 0 or int(ys[i]) < 0 or not tiles.has(Vector2i(int(xs[i]), int(ys[i]))):
			return false
	return true

static func winter_wood_ready(settlements: Dictionary) -> bool:
	var ids: Variant = settlements.get("ids", [])
	var wood: Variant = settlements.get("wood", [])
	var need: Variant = settlements.get("winter_wood_need", [])
	if ids.is_empty() or wood.size() != ids.size() or need.size() != ids.size():
		return false
	for i: int in ids.size():
		if float(need[i]) <= 0.0 or float(wood[i]) < float(need[i]):
			return false
	return true

static func era_markers(tech: Dictionary) -> Array[Dictionary]:
	var result: Array[Dictionary] = []
	var weights: Variant = tech.get("marker_weights", [])
	var statuses: Variant = tech.get("statuses", [])
	var names: Variant = tech.get("names", [])
	var keys: Variant = tech.get("name_keys", [])
	for i: int in mini(weights.size(), mini(statuses.size(), mini(names.size(), keys.size()))):
		if int(weights[i]) > 0:
			result.append({"name": str(names[i]), "name_key": str(keys[i]), "done": int(statuses[i]) == 3})
	return result
