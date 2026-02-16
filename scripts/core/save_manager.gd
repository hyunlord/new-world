extends RefCounted

## Binary save/load system (version 2).
## Structure: user://saves/quicksave/ directory with:
##   meta.json, entities.bin, buildings.bin, relationships.bin,
##   settlements.bin, world.bin, stats.json

const SAVE_VERSION: int = 3

## Reverse-lookup arrays for binary enum deserialization
var _genders: Array = ["male", "female"]
var _age_stages: Array = ["infant", "toddler", "child", "teen", "adult", "elder"]
var _jobs: Array = ["none", "gatherer", "lumberjack", "builder", "miner"]
var _rel_types: Array = ["stranger", "acquaintance", "friend", "close_friend", "romantic", "partner", "rival"]


## Convert unsigned 32-bit read back to signed
func _s32(v: int) -> int:
	return v if v < 0x80000000 else v - 0x100000000


## ═══════════════════════════════════════════════════
## SAVE
## ═══════════════════════════════════════════════════

func save_game(dir_path: String, sim_engine: RefCounted, entity_manager: RefCounted, building_manager: RefCounted, resource_map: RefCounted, settlement_manager: RefCounted, relationship_manager: RefCounted, stats_recorder: RefCounted) -> bool:
	DirAccess.make_dir_recursive_absolute(dir_path)

	# meta.json
	var date: Dictionary = GameConfig.tick_to_date(sim_engine.current_tick)
	var meta: Dictionary = {
		"version": SAVE_VERSION,
		"current_tick": sim_engine.current_tick,
		"seed": sim_engine._seed,
		"speed_index": sim_engine.speed_index,
		"rng_state": sim_engine.rng.state,
		"ui_scale": GameConfig.ui_scale,
		"population": entity_manager.get_alive_count(),
		"game_date": "Y%d M%d D%d" % [date.year, date.month, date.day],
	}
	var mf: FileAccess = FileAccess.open(dir_path + "/meta.json", FileAccess.WRITE)
	if mf == null:
		push_warning("[SaveManager] Cannot write meta.json")
		return false
	mf.store_string(JSON.stringify(meta))
	mf = null

	if not _save_entities(dir_path + "/entities.bin", entity_manager):
		return false
	if not _save_buildings(dir_path + "/buildings.bin", building_manager):
		return false
	if not _save_relationships(dir_path + "/relationships.bin", relationship_manager):
		return false
	if not _save_settlements(dir_path + "/settlements.bin", settlement_manager):
		return false
	if not _save_resource_map(dir_path + "/world.bin", resource_map):
		return false
	_save_stats(dir_path + "/stats.json", stats_recorder)

	SimulationBus.emit_event("game_saved", {"path": dir_path, "tick": sim_engine.current_tick})
	return true


## ═══════════════════════════════════════════════════
## LOAD
## ═══════════════════════════════════════════════════

func load_game(dir_path: String, sim_engine: RefCounted, entity_manager: RefCounted, building_manager: RefCounted, resource_map: RefCounted, world_data: RefCounted, settlement_manager: RefCounted, relationship_manager: RefCounted, stats_recorder: RefCounted) -> bool:
	var meta_path: String = dir_path + "/meta.json"
	if not FileAccess.file_exists(meta_path):
		push_warning("[SaveManager] Save not found: %s" % meta_path)
		return false
	var mf: FileAccess = FileAccess.open(meta_path, FileAccess.READ)
	if mf == null:
		return false
	var json: JSON = JSON.new()
	if json.parse(mf.get_as_text()) != OK:
		push_warning("[SaveManager] Corrupted meta.json")
		return false
	mf = null
	var meta: Dictionary = json.data
	if meta.get("version", 0) != SAVE_VERSION:
		push_warning("[SaveManager] Incompatible save version: %d" % meta.get("version", 0))
		return false

	sim_engine.current_tick = int(meta.get("current_tick", 0))
	sim_engine.speed_index = int(meta.get("speed_index", 0))
	if meta.has("rng_state"):
		sim_engine.rng.state = int(meta.get("rng_state", 0))
	GameConfig.ui_scale = float(meta.get("ui_scale", 1.0))

	world_data.clear_entities()

	if not _load_entities(dir_path + "/entities.bin", entity_manager, world_data):
		return false
	if not _load_buildings(dir_path + "/buildings.bin", building_manager):
		return false
	if not _load_relationships(dir_path + "/relationships.bin", relationship_manager):
		return false
	if not _load_settlements(dir_path + "/settlements.bin", settlement_manager):
		return false
	if not _load_resource_map(dir_path + "/world.bin", resource_map):
		return false
	_load_stats(dir_path + "/stats.json", stats_recorder)

	SimulationBus.emit_event("game_loaded", {"path": dir_path, "tick": sim_engine.current_tick})
	return true


## ── Entities ───────────────────────────────────────

func _save_entities(path: String, em: RefCounted) -> bool:
	var f: FileAccess = FileAccess.open(path, FileAccess.WRITE)
	if f == null:
		return false
	f.big_endian = false
	var all: Array = em._entities.values()
	f.store_32(all.size())
	for i in range(all.size()):
		var e: RefCounted = all[i]
		f.store_32(e.id)
		f.store_pascal_string(e.entity_name)
		f.store_32(e.position.x)
		f.store_32(e.position.y)
		f.store_8(1 if e.is_alive else 0)
		f.store_float(e.hunger)
		f.store_float(e.energy)
		f.store_float(e.social)
		f.store_32(e.age)
		f.store_float(e.speed)
		f.store_float(e.strength)
		f.store_8(maxi(_genders.find(e.gender), 0))
		f.store_8(maxi(_age_stages.find(e.age_stage), 0))
		f.store_32(e.birth_tick)
		f.store_32(e.partner_id)
		f.store_32(e.pregnancy_tick)
		f.store_float(e.frailty)
		# Personality (5 floats, fixed order)
		f.store_float(e.personality.get("openness", 0.5))
		f.store_float(e.personality.get("agreeableness", 0.5))
		f.store_float(e.personality.get("extraversion", 0.5))
		f.store_float(e.personality.get("diligence", 0.5))
		f.store_float(e.personality.get("emotional_stability", 0.5))
		# Emotions (5 floats, fixed order)
		f.store_float(e.emotions.get("happiness", 0.5))
		f.store_float(e.emotions.get("loneliness", 0.0))
		f.store_float(e.emotions.get("stress", 0.0))
		f.store_float(e.emotions.get("grief", 0.0))
		f.store_float(e.emotions.get("love", 0.0))
		# Job + settlement
		f.store_8(maxi(_jobs.find(e.job), 0))
		f.store_32(e.settlement_id)
		# AI state
		f.store_pascal_string(e.current_action)
		f.store_pascal_string(e.current_goal)
		f.store_32(e.action_target.x)
		f.store_32(e.action_target.y)
		f.store_32(e.action_timer)
		f.store_32(e.starving_timer)
		# Inventory
		f.store_float(e.inventory.get("food", 0.0))
		f.store_float(e.inventory.get("wood", 0.0))
		f.store_float(e.inventory.get("stone", 0.0))
		# Lifetime stats
		f.store_float(e.total_gathered)
		f.store_32(e.buildings_built)
		# Parent IDs (variable length, max 2)
		f.store_8(e.parent_ids.size())
		for j in range(e.parent_ids.size()):
			f.store_32(e.parent_ids[j])
		# Children IDs (variable length)
		f.store_8(e.children_ids.size())
		for j in range(e.children_ids.size()):
			f.store_32(e.children_ids[j])
	return true


func _load_entities(path: String, em: RefCounted, world_data: RefCounted) -> bool:
	if not FileAccess.file_exists(path):
		return false
	var f: FileAccess = FileAccess.open(path, FileAccess.READ)
	if f == null:
		return false
	f.big_endian = false
	em._entities.clear()
	em._next_id = 1
	em.chunk_index.clear()
	var EntityDataScript = load("res://scripts/core/entity_data.gd")
	var count: int = f.get_32()
	for _i in range(count):
		var e = EntityDataScript.new()
		e.id = f.get_32()
		e.entity_name = f.get_pascal_string()
		e.position = Vector2i(f.get_32(), f.get_32())
		e.is_alive = f.get_8() == 1
		e.hunger = f.get_float()
		e.energy = f.get_float()
		e.social = f.get_float()
		e.age = f.get_32()
		e.speed = f.get_float()
		e.strength = f.get_float()
		e.gender = _genders[mini(f.get_8(), _genders.size() - 1)]
		e.age_stage = _age_stages[mini(f.get_8(), _age_stages.size() - 1)]
		e.birth_tick = f.get_32()
		e.partner_id = _s32(f.get_32())
		e.pregnancy_tick = _s32(f.get_32())
		e.frailty = f.get_float()
		e.personality = {
			"openness": f.get_float(),
			"agreeableness": f.get_float(),
			"extraversion": f.get_float(),
			"diligence": f.get_float(),
			"emotional_stability": f.get_float(),
		}
		e.emotions = {
			"happiness": f.get_float(),
			"loneliness": f.get_float(),
			"stress": f.get_float(),
			"grief": f.get_float(),
			"love": f.get_float(),
		}
		e.job = _jobs[mini(f.get_8(), _jobs.size() - 1)]
		e.settlement_id = f.get_32()
		e.current_action = f.get_pascal_string()
		e.current_goal = f.get_pascal_string()
		e.action_target = Vector2i(_s32(f.get_32()), _s32(f.get_32()))
		e.action_timer = f.get_32()
		e.starving_timer = f.get_32()
		e.inventory = {
			"food": f.get_float(),
			"wood": f.get_float(),
			"stone": f.get_float(),
		}
		e.total_gathered = f.get_float()
		e.buildings_built = f.get_32()
		var pc: int = f.get_8()
		e.parent_ids = []
		for _j in range(pc):
			e.parent_ids.append(f.get_32())
		var cc: int = f.get_8()
		e.children_ids = []
		for _j in range(cc):
			e.children_ids.append(f.get_32())
		em._entities[e.id] = e
		if e.is_alive:
			world_data.register_entity(e.position, e.id)
			em.chunk_index.add_entity(e.id, e.position)
		if e.id >= em._next_id:
			em._next_id = e.id + 1
	return true


## ── Buildings ──────────────────────────────────────

func _save_buildings(path: String, bm: RefCounted) -> bool:
	var f: FileAccess = FileAccess.open(path, FileAccess.WRITE)
	if f == null:
		return false
	f.big_endian = false
	var all: Array = bm._buildings.values()
	f.store_32(all.size())
	for i in range(all.size()):
		var b: RefCounted = all[i]
		f.store_32(b.id)
		f.store_pascal_string(b.building_type)
		f.store_32(b.tile_x)
		f.store_32(b.tile_y)
		f.store_8(1 if b.is_built else 0)
		f.store_float(b.build_progress)
		f.store_float(b.storage.get("food", 0.0))
		f.store_float(b.storage.get("wood", 0.0))
		f.store_float(b.storage.get("stone", 0.0))
		f.store_32(b.settlement_id)
	return true


func _load_buildings(path: String, bm: RefCounted) -> bool:
	if not FileAccess.file_exists(path):
		return false
	var f: FileAccess = FileAccess.open(path, FileAccess.READ)
	if f == null:
		return false
	f.big_endian = false
	bm._buildings.clear()
	bm._tile_map.clear()
	bm._next_id = 1
	var BuildingDataScript = load("res://scripts/core/building_data.gd")
	var count: int = f.get_32()
	for _i in range(count):
		var b = BuildingDataScript.new()
		b.id = f.get_32()
		b.building_type = f.get_pascal_string()
		b.tile_x = f.get_32()
		b.tile_y = f.get_32()
		b.is_built = f.get_8() == 1
		b.build_progress = f.get_float()
		b.storage = {
			"food": f.get_float(),
			"wood": f.get_float(),
			"stone": f.get_float(),
		}
		b.settlement_id = f.get_32()
		bm._buildings[b.id] = b
		bm._tile_map[bm._tile_key(b.tile_x, b.tile_y)] = b.id
		if b.id >= bm._next_id:
			bm._next_id = b.id + 1
	return true


## ── Relationships ──────────────────────────────────

func _save_relationships(path: String, rm: RefCounted) -> bool:
	var f: FileAccess = FileAccess.open(path, FileAccess.WRITE)
	if f == null:
		return false
	f.big_endian = false
	var keys: Array = rm._relationships.keys()
	f.store_32(keys.size())
	for i in range(keys.size()):
		var key: String = keys[i]
		var rel: RefCounted = rm._relationships[key]
		var parts: PackedStringArray = key.split(":")
		f.store_32(int(parts[0]))
		f.store_32(int(parts[1]))
		f.store_float(rel.affinity)
		f.store_float(rel.trust)
		f.store_float(rel.romantic_interest)
		f.store_32(rel.interaction_count)
		f.store_32(rel.last_interaction_tick)
		f.store_8(maxi(_rel_types.find(rel.type), 0))
	return true


func _load_relationships(path: String, rm: RefCounted) -> bool:
	if not FileAccess.file_exists(path):
		return false
	var f: FileAccess = FileAccess.open(path, FileAccess.READ)
	if f == null:
		return false
	f.big_endian = false
	rm._relationships.clear()
	var RelationshipData = load("res://scripts/core/relationship_data.gd")
	var count: int = f.get_32()
	for _i in range(count):
		var id_a: int = f.get_32()
		var id_b: int = f.get_32()
		var rel = RelationshipData.new()
		rel.affinity = f.get_float()
		rel.trust = f.get_float()
		rel.romantic_interest = f.get_float()
		rel.interaction_count = f.get_32()
		rel.last_interaction_tick = f.get_32()
		rel.type = _rel_types[mini(f.get_8(), _rel_types.size() - 1)]
		rm._relationships["%d:%d" % [id_a, id_b]] = rel
	return true


## ── Settlements ────────────────────────────────────

func _save_settlements(path: String, sm: RefCounted) -> bool:
	var f: FileAccess = FileAccess.open(path, FileAccess.WRITE)
	if f == null:
		return false
	f.big_endian = false
	var all: Array = sm._settlements.values()
	f.store_32(all.size())
	for i in range(all.size()):
		var s: RefCounted = all[i]
		f.store_32(s.id)
		f.store_32(s.center_x)
		f.store_32(s.center_y)
		f.store_32(s.founding_tick)
		f.store_32(s.member_ids.size())
		for j in range(s.member_ids.size()):
			f.store_32(s.member_ids[j])
		f.store_32(s.building_ids.size())
		for j in range(s.building_ids.size()):
			f.store_32(s.building_ids[j])
	return true


func _load_settlements(path: String, sm: RefCounted) -> bool:
	if not FileAccess.file_exists(path):
		return false
	var f: FileAccess = FileAccess.open(path, FileAccess.READ)
	if f == null:
		return false
	f.big_endian = false
	sm._settlements.clear()
	sm._next_id = 1
	var SettlementDataScript = load("res://scripts/core/settlement_data.gd")
	var count: int = f.get_32()
	for _i in range(count):
		var s = SettlementDataScript.new()
		s.id = f.get_32()
		s.center_x = f.get_32()
		s.center_y = f.get_32()
		s.founding_tick = f.get_32()
		var mc: int = f.get_32()
		s.member_ids = []
		for _j in range(mc):
			s.member_ids.append(f.get_32())
		var bc: int = f.get_32()
		s.building_ids = []
		for _j in range(bc):
			s.building_ids.append(f.get_32())
		sm._settlements[s.id] = s
		if s.id >= sm._next_id:
			sm._next_id = s.id + 1
	return true


## ── Resource Map (world.bin) ───────────────────────

func _save_resource_map(path: String, rm: RefCounted) -> bool:
	var f: FileAccess = FileAccess.open(path, FileAccess.WRITE)
	if f == null:
		return false
	f.big_endian = false
	f.store_32(rm._width)
	f.store_32(rm._height)
	var food_bytes: PackedByteArray = rm._food.to_byte_array()
	f.store_32(food_bytes.size())
	f.store_buffer(food_bytes)
	var wood_bytes: PackedByteArray = rm._wood.to_byte_array()
	f.store_32(wood_bytes.size())
	f.store_buffer(wood_bytes)
	var stone_bytes: PackedByteArray = rm._stone.to_byte_array()
	f.store_32(stone_bytes.size())
	f.store_buffer(stone_bytes)
	return true


func _load_resource_map(path: String, rm: RefCounted) -> bool:
	if not FileAccess.file_exists(path):
		return false
	var f: FileAccess = FileAccess.open(path, FileAccess.READ)
	if f == null:
		return false
	f.big_endian = false
	rm._width = f.get_32()
	rm._height = f.get_32()
	var size: int = rm._width * rm._height
	rm._food.resize(size)
	rm._wood.resize(size)
	rm._stone.resize(size)
	var food_byte_size: int = f.get_32()
	var food_bytes: PackedByteArray = f.get_buffer(food_byte_size)
	for i in range(size):
		rm._food[i] = food_bytes.decode_float(i * 4)
	var wood_byte_size: int = f.get_32()
	var wood_bytes: PackedByteArray = f.get_buffer(wood_byte_size)
	for i in range(size):
		rm._wood[i] = wood_bytes.decode_float(i * 4)
	var stone_byte_size: int = f.get_32()
	var stone_bytes: PackedByteArray = f.get_buffer(stone_byte_size)
	for i in range(size):
		rm._stone[i] = stone_bytes.decode_float(i * 4)
	return true


## ── Stats (JSON) ───────────────────────────────────

func _save_stats(path: String, sr: RefCounted) -> void:
	var data: Dictionary = {
		"peak_pop": sr.peak_pop,
		"total_births": sr.total_births,
		"total_deaths": sr.total_deaths,
		"history": sr.history,
	}
	var f: FileAccess = FileAccess.open(path, FileAccess.WRITE)
	if f != null:
		f.store_string(JSON.stringify(data))


func _load_stats(path: String, sr: RefCounted) -> void:
	if not FileAccess.file_exists(path):
		return
	var f: FileAccess = FileAccess.open(path, FileAccess.READ)
	if f == null:
		return
	var json: JSON = JSON.new()
	if json.parse(f.get_as_text()) != OK:
		return
	var data: Dictionary = json.data
	sr.peak_pop = int(data.get("peak_pop", 0))
	sr.total_births = int(data.get("total_births", 0))
	sr.total_deaths = int(data.get("total_deaths", 0))
	sr.history.clear()
	var raw_history: Array = data.get("history", [])
	for i in range(raw_history.size()):
		sr.history.append(raw_history[i])
