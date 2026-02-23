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
		## [Skill yield bonus — Newell & Rosenbloom 1981 Power Law of Practice]
		## Reads POWER curve params from skill JSON affects[] via StatQuery.get_skill_multiplier()
		## level 0 → ×1.0, level 50 → ×1.28, level 100 → ×1.70
		var _skill_mult: float = 1.0
		var _yield_skill_map: Dictionary = {
			"food":  &"SKILL_FORAGING",
			"wood":  &"SKILL_WOODCUTTING",
			"stone": &"SKILL_MINING",
		}
		var _yield_sid: StringName = _yield_skill_map.get(res_name, &"")
		if _yield_sid != &"":
			_skill_mult = StatQuery.get_skill_multiplier(entity, _yield_sid, &"gathering")
		var amount: float = minf(
			GameConfig.GATHER_AMOUNT * entity.speed * age_efficiency * _skill_mult,
			minf(available, remaining_cap)
		)
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
			## [Skill XP — Newell & Rosenbloom 1981 Power Law of Practice]
			var _skill_map: Dictionary = {
				"food":  &"SKILL_FORAGING",
				"wood":  &"SKILL_WOODCUTTING",
				"stone": &"SKILL_MINING",
			}
			var _skill_id: StringName = _skill_map.get(res_name, &"")
			if _skill_id != &"":
				var _xp_map: Dictionary = {
					&"SKILL_FORAGING":    GameConfig.SKILL_XP_FORAGING,
					&"SKILL_WOODCUTTING": GameConfig.SKILL_XP_WOODCUTTING,
					&"SKILL_MINING":      GameConfig.SKILL_XP_MINING,
				}
				var _xp_amt: float = _xp_map.get(_skill_id, 2.0)
				var _xp_result: Dictionary = StatQuery.add_xp(entity, _skill_id, _xp_amt)
				if _xp_result.get("leveled_up", false):
					emit_event("skill_leveled_up", {
						"entity_id":   entity.id,
						"entity_name": entity.entity_name,
						"skill_id":    str(_skill_id),
						"old_level":   _xp_result["old_level"],
						"new_level":   _xp_result["new_level"],
						"tick":        tick,
					})
					SimulationBus.skill_leveled_up.emit(
						entity.id,
						entity.entity_name,
						_skill_id,
						_xp_result["old_level"],
						_xp_result["new_level"],
						tick
					)
