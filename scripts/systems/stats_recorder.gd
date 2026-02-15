extends "res://scripts/core/simulation_system.gd"

var _entity_manager: RefCounted
var _building_manager: RefCounted
var history: Array = []
const MAX_HISTORY: int = 200


func _init() -> void:
	system_name = "stats_recorder"
	priority = 90
	tick_interval = 50


func init(entity_manager: RefCounted, building_manager: RefCounted) -> void:
	_entity_manager = entity_manager
	_building_manager = building_manager


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
	history.append(snap)
	if history.size() > MAX_HISTORY:
		history.pop_front()
