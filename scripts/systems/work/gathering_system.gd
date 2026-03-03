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
			# [훈련 XP 누적] 채집 종류별 신체능력 XP (Heritage 1999, Ahtiainen 2016)
			if entity.body != null:
				var _xp_age: float = float(entity.age)
				var _age_mods: PackedFloat32Array = BodyAttributes.get_age_trainability_modifier_packed(_xp_age)
				match res_name:
					"food":
						var _m_end: float = float(_age_mods[2])
						var _m_str: float = float(_age_mods[0])
						entity.body.training_xp["end"] = entity.body.training_xp.get("end", 0.0) + GameConfig.BODY_XP_GATHER_FOOD * _m_end
						entity.body.training_xp["str"] = entity.body.training_xp.get("str", 0.0) + GameConfig.BODY_XP_GATHER_FOOD * 0.5 * _m_str
					"wood":
						var _m_str: float = float(_age_mods[0])
						var _m_end: float = float(_age_mods[2])
						entity.body.training_xp["str"] = entity.body.training_xp.get("str", 0.0) + GameConfig.BODY_XP_GATHER_WOOD * _m_str
						entity.body.training_xp["end"] = entity.body.training_xp.get("end", 0.0) + GameConfig.BODY_XP_GATHER_WOOD * 0.5 * _m_end
					"stone":
						var _m_str: float = float(_age_mods[0])
						var _m_tou: float = float(_age_mods[3])
						entity.body.training_xp["str"] = entity.body.training_xp.get("str", 0.0) + GameConfig.BODY_XP_GATHER_STONE * _m_str
						entity.body.training_xp["tou"] = entity.body.training_xp.get("tou", 0.0) + GameConfig.BODY_XP_GATHER_STONE * 0.5 * _m_tou
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
