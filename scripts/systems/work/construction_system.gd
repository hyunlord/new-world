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


func execute_tick(tick: int) -> void:
	var entities: Array = _entity_manager.get_alive_entities()
	for i in range(entities.size()):
		var entity = entities[i]
		if entity.current_action != "build":
			continue
		# Only adults can build
		if entity.age_stage != "adult":  # Only adults can build
			continue

		var tx: int = entity.action_target.x
		var ty: int = entity.action_target.y
		var building = _building_manager.get_building_at(tx, ty)
		if building == null or building.is_built:
			continue

		# Check entity is at or adjacent to building tile
		var dx: int = absi(entity.position.x - tx)
		var dy: int = absi(entity.position.y - ty)
		if dx > 1 or dy > 1:
			continue

		# Calculate progress per tick from build_ticks config
		var build_ticks: int = GameConfig.BUILDING_TYPES.get(
			building.building_type, {}
		).get("build_ticks", 50)
		@warning_ignore("integer_division")
		var ticks_per_cycle: int = build_ticks / GameConfig.CONSTRUCTION_TICK_INTERVAL
		if ticks_per_cycle < 1:
			ticks_per_cycle = 1
		## [Skill build speed bonus — Newell & Rosenbloom 1981]
		## SKILL_CONSTRUCTION affects build_speed: level 0 → ×1.0, level 100 → ×1.70
		var _build_skill_mult: float = StatQuery.get_skill_multiplier(entity, &"SKILL_CONSTRUCTION", &"gathering")
		var progress_per_tick: float = (1.0 / float(ticks_per_cycle)) * _build_skill_mult

		building.build_progress += progress_per_tick
		# [훈련 XP 누적] 건설 활동 → 근력/강인함/민첩 훈련
		if entity.body != null:
			entity.body.training_xp["str"] += GameConfig.CONSTRUCT_XP_STR
			entity.body.training_xp["tou"] += GameConfig.CONSTRUCT_XP_TOU
			entity.body.training_xp["agi"] += GameConfig.CONSTRUCT_XP_AGI
		## [Skill XP — 건설 활동]
		var _xp_result: Dictionary = StatQuery.add_xp(entity, &"SKILL_CONSTRUCTION", GameConfig.SKILL_XP_CONSTRUCTION)
		if _xp_result.get("leveled_up", false):
			emit_event("skill_leveled_up", {
				"entity_id":   entity.id,
				"entity_name": entity.entity_name,
				"skill_id":    "SKILL_CONSTRUCTION",
				"old_level":   _xp_result["old_level"],
				"new_level":   _xp_result["new_level"],
				"tick":        tick,
			})
			SimulationBus.skill_leveled_up.emit(
				entity.id,
				entity.entity_name,
				&"SKILL_CONSTRUCTION",
				_xp_result["old_level"],
				_xp_result["new_level"],
				tick
			)
		if building.build_progress >= 1.0:
			building.build_progress = 1.0
			building.is_built = true
			entity.buildings_built += 1
			# [훈련 XP 누적] 건물 완공 보너스 — str + agi (Heritage 1999)
			if entity.body != null:
				var _age = float(entity.age)
				var _age_mods: Dictionary = BodyAttributes.get_age_trainability_modifier_batch(_age)
				var _m_str: float = float(_age_mods.get("str", 1.0))
				var _m_agi: float = float(_age_mods.get("agi", 1.0))
				entity.body.training_xp["str"] = entity.body.training_xp.get("str", 0.0) + GameConfig.BODY_XP_BUILD * _m_str
				entity.body.training_xp["agi"] = entity.body.training_xp.get("agi", 0.0) + GameConfig.BODY_XP_BUILD * 0.6 * _m_agi
			emit_event("building_completed", {
				"building_id": building.id,
				"building_type": building.building_type,
				"tile_x": building.tile_x,
				"tile_y": building.tile_y,
				"tick": tick,
			})
