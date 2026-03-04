extends "res://scripts/core/simulation/simulation_system.gd"

const BodyAttributes = preload("res://scripts/core/entity/body_attributes.gd")

var _entity_manager: RefCounted
var _resource_map: RefCounted


func init(entity_manager: RefCounted, resource_map: RefCounted) -> void:
	system_name = "gathering"
	priority = 25
	tick_interval = GameConfig.GATHERING_TICK_INTERVAL
	_entity_manager = entity_manager
	_resource_map = resource_map
