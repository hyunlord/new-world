extends "res://scripts/core/simulation/simulation_system.gd"

const BodyAttributes = preload("res://scripts/core/entity/body_attributes.gd")

var _entity_manager: RefCounted
var _building_manager: RefCounted


func init(entity_manager: RefCounted, building_manager: RefCounted) -> void:
	system_name = "construction"
	priority = 28
	tick_interval = GameConfig.CONSTRUCTION_TICK_INTERVAL
	_entity_manager = entity_manager
	_building_manager = building_manager
