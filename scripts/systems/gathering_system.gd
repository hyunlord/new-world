extends "res://scripts/core/simulation_system.gd"

var _entity_manager: RefCounted
var _resource_map: RefCounted


func init(entity_manager: RefCounted, resource_map: RefCounted) -> void:
	system_name = "gathering"
	priority = 25
	tick_interval = GameConfig.GATHERING_TICK_INTERVAL
	_entity_manager = entity_manager
	_resource_map = resource_map


func execute_tick(tick: int) -> void:
	var entities: Array = _entity_manager.get_alive_entities()
	for i in range(entities.size()):
		var entity = entities[i]
		var res_type: int = -1
		var res_name: String = ""
		match entity.current_action:
			"gather_food":
				res_type = GameConfig.ResourceType.FOOD
				res_name = "food"
			"gather_wood":
				res_type = GameConfig.ResourceType.WOOD
				res_name = "wood"
			"gather_stone":
				res_type = GameConfig.ResourceType.STONE
				res_name = "stone"
			_:
				continue

		var x: int = entity.position.x
		var y: int = entity.position.y
		var available: float = _resource_map.get_resource(x, y, res_type)
		if available < 0.5:
			continue

		# Age restriction: infants and toddlers can't gather
		if entity.age_stage == "infant" or entity.age_stage == "toddler":
			continue

		var remaining_cap: float = GameConfig.MAX_CARRY - entity.get_total_carry()
		if remaining_cap <= 0.0:
			continue

		# Age efficiency defaults to 1.0 and can be overridden per stage in config
		var age_efficiency: float = GameConfig.CHILD_GATHER_EFFICIENCY.get(entity.age_stage, 1.0)
		var amount: float = minf(GameConfig.GATHER_AMOUNT * entity.speed * age_efficiency, minf(available, remaining_cap))
		var harvested: float = _resource_map.harvest(x, y, res_type, amount)
		if harvested > 0.0:
			entity.add_item(res_name, harvested)
			entity.total_gathered += harvested
			emit_event("resource_gathered", {
				"entity_id": entity.id,
				"entity_name": entity.entity_name,
				"resource_type": res_name,
				"amount": harvested,
				"tile_x": x,
				"tile_y": y,
				"tick": tick,
			})
			# [훈련 XP 누적] 채집 활동 → 지구력/근력/민첩 훈련
			if entity.body != null:
				entity.body.training_xp["end"] += GameConfig.GATHER_XP_END
				entity.body.training_xp["str"] += GameConfig.GATHER_XP_STR
				entity.body.training_xp["agi"] += GameConfig.GATHER_XP_AGI
