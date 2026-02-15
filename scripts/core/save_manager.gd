extends RefCounted


func save_game(path: String, sim_engine: RefCounted, entity_manager: RefCounted, building_manager: RefCounted, resource_map: RefCounted, settlement_manager: RefCounted) -> bool:
	var data: Dictionary = {
		"version": 1,
		"sim_engine": {
			"current_tick": sim_engine.current_tick,
			"seed": sim_engine._seed,
			"speed_index": sim_engine.speed_index,
			"rng_state": sim_engine.rng.state,
		},
		"entities": entity_manager.to_save_data(),
		"buildings": building_manager.to_save_data(),
		"resource_map": resource_map.to_save_data(),
		"settlements": settlement_manager.to_save_data(),
	}
	var json_string: String = JSON.stringify(data)
	var file: FileAccess = FileAccess.open(path, FileAccess.WRITE)
	if file == null:
		push_warning("[SaveManager] Failed to open file for writing: %s" % path)
		return false
	file.store_string(json_string)
	SimulationBus.emit_event("game_saved", {"path": path, "tick": sim_engine.current_tick})
	return true


func load_game(path: String, sim_engine: RefCounted, entity_manager: RefCounted, building_manager: RefCounted, resource_map: RefCounted, world_data: RefCounted, settlement_manager: RefCounted) -> bool:
	if not FileAccess.file_exists(path):
		push_warning("[SaveManager] Save file not found: %s" % path)
		return false
	var file: FileAccess = FileAccess.open(path, FileAccess.READ)
	if file == null:
		push_warning("[SaveManager] Failed to open file for reading: %s" % path)
		return false
	var json_string: String = file.get_as_text()
	var json: JSON = JSON.new()
	var parse_result: int = json.parse(json_string)
	if parse_result != OK:
		push_warning("[SaveManager] Failed to parse save file: %s" % path)
		return false
	var data: Dictionary = json.data
	if not data.has("version"):
		push_warning("[SaveManager] Invalid save file format")
		return false

	# Restore sim engine state
	var se_data: Dictionary = data.get("sim_engine", {})
	sim_engine.current_tick = int(se_data.get("current_tick", 0))
	sim_engine.speed_index = int(se_data.get("speed_index", 0))
	if se_data.has("rng_state"):
		sim_engine.rng.state = int(se_data.get("rng_state", 0))

	# Clear world entity registrations before loading
	world_data.clear_entities()

	# Restore entities
	var entity_data: Array = []
	var raw_entities = data.get("entities", [])
	for i in range(raw_entities.size()):
		entity_data.append(raw_entities[i])
	entity_manager.load_save_data(entity_data, world_data)

	# Restore buildings
	var building_data: Array = []
	var raw_buildings = data.get("buildings", [])
	for i in range(raw_buildings.size()):
		building_data.append(raw_buildings[i])
	building_manager.load_save_data(building_data)

	# Restore settlements
	var settlement_data: Array = []
	var raw_settlements: Array = data.get("settlements", [])
	for i in range(raw_settlements.size()):
		settlement_data.append(raw_settlements[i])
	settlement_manager.load_save_data(settlement_data)

	# Restore resource map
	var res_data: Dictionary = data.get("resource_map", {})
	resource_map.load_save_data(res_data)

	SimulationBus.emit_event("game_loaded", {"path": path, "tick": sim_engine.current_tick})
	return true
