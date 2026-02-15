extends "res://scripts/core/simulation_system.gd"

var _entity_manager: RefCounted
var _building_manager: RefCounted
var _settlement_manager: RefCounted
var history: Array = []
const MAX_HISTORY: int = 200
var peak_pop: int = 0
var total_births: int = 0
var total_deaths: int = 0


func _init() -> void:
	system_name = "stats_recorder"
	priority = 90
	tick_interval = 50


func init(entity_manager: RefCounted, building_manager: RefCounted, settlement_manager: RefCounted = null) -> void:
	_entity_manager = entity_manager
	_building_manager = building_manager
	_settlement_manager = settlement_manager


func execute_tick(_tick: int) -> void:
	var snap: Dictionary = {
		"tick": _tick,
		"pop": _entity_manager.get_alive_count(),
		"food": 0.0,
		"wood": 0.0,
		"stone": 0.0,
		"gatherers": 0,
		"lumberjacks": 0,
		"builders": 0,
		"miners": 0,
		"none_job": 0,
	}
	var alive: Array = _entity_manager.get_alive_entities()
	for i in range(alive.size()):
		var e: RefCounted = alive[i]
		match e.job:
			"gatherer":
				snap.gatherers += 1
			"lumberjack":
				snap.lumberjacks += 1
			"builder":
				snap.builders += 1
			"miner":
				snap.miners += 1
			_:
				snap.none_job += 1
	var stockpiles: Array = _building_manager.get_buildings_by_type("stockpile")
	for i in range(stockpiles.size()):
		var sp = stockpiles[i]
		if sp.is_built:
			snap.food += sp.storage.get("food", 0.0)
			snap.wood += sp.storage.get("wood", 0.0)
			snap.stone += sp.storage.get("stone", 0.0)
	# Track peak population
	if snap.pop > peak_pop:
		peak_pop = snap.pop

	history.append(snap)
	if history.size() > MAX_HISTORY:
		history.pop_front()


## Get resource delta per 100 ticks (rate of change)
func get_resource_deltas() -> Dictionary:
	if history.size() < 3:
		return {"food": 0.0, "wood": 0.0, "stone": 0.0}
	var latest: Dictionary = history[history.size() - 1]
	var older: Dictionary = history[maxi(0, history.size() - 3)]
	var tick_diff: float = float(latest.tick - older.tick)
	if tick_diff <= 0.0:
		return {"food": 0.0, "wood": 0.0, "stone": 0.0}
	var scale: float = 100.0 / tick_diff
	return {
		"food": (latest.food - older.food) * scale,
		"wood": (latest.wood - older.wood) * scale,
		"stone": (latest.stone - older.stone) * scale,
	}


## Get per-settlement stats
func get_settlement_stats() -> Array:
	if _settlement_manager == null:
		return []
	var result: Array = []
	var active: Array = _settlement_manager.get_active_settlements()
	for i in range(active.size()):
		var s: RefCounted = active[i]
		var pop: int = _settlement_manager.get_settlement_population(s.id)
		var bld_count: int = s.building_ids.size()
		var food: float = 0.0
		var wood: float = 0.0
		var stone: float = 0.0
		if _building_manager != null:
			var stockpiles: Array = _building_manager.get_buildings_by_type("stockpile")
			for j in range(stockpiles.size()):
				var sp = stockpiles[j]
				if sp.settlement_id == s.id and sp.is_built:
					food += sp.storage.get("food", 0.0)
					wood += sp.storage.get("wood", 0.0)
					stone += sp.storage.get("stone", 0.0)
		result.append({
			"id": s.id, "pop": pop, "buildings": bld_count,
			"food": food, "wood": wood, "stone": stone,
		})
	return result
