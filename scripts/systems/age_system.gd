extends "res://scripts/core/simulation_system.gd"

## Checks age stage transitions, emits growth notifications,
## and applies yearly personality maturation.
## Runs every 50 ticks (~4 days).

const PersonalityMaturation = preload("res://scripts/systems/personality_maturation.gd")
const BodyAttributes = preload("res://scripts/core/body_attributes.gd")

var _entity_manager: RefCounted
var _personality_maturation: RefCounted


func _init() -> void:
	system_name = "aging"
	priority = 48
	tick_interval = 50


func init(entity_manager: RefCounted, rng: RandomNumberGenerator = null) -> void:
	_entity_manager = entity_manager
	if rng != null:
		_personality_maturation = PersonalityMaturation.new()
		_personality_maturation.init(rng)


func execute_tick(tick: int) -> void:
	var alive: Array = _entity_manager.get_alive_entities()
	for i in range(alive.size()):
		var entity: RefCounted = alive[i]
		var new_stage: String = GameConfig.get_age_stage(entity.age)
		if new_stage != entity.age_stage:
			var old_stage: String = entity.age_stage
			entity.age_stage = new_stage
			_on_stage_changed(entity, old_stage, new_stage, tick)
		# Yearly personality maturation (on birthday ± tick_interval window)
		if _personality_maturation != null and entity.personality != null:
			if entity.age > 0 and entity.age % GameConfig.TICKS_PER_YEAR < tick_interval:
				var age_years: int = int(entity.age / GameConfig.TICKS_PER_YEAR)
				_personality_maturation.apply_maturation(entity.personality, age_years)
		# [Layer 1.5] 연간 body realized 재계산 [Gurven et al. 2008]
		# potential은 불변 — 재계산 안 함
		if entity.age > 0 and entity.age % GameConfig.TICKS_PER_YEAR < tick_interval:
			if entity.body != null:
				var body_age_y: float = GameConfig.get_age_years(entity.age)
				var new_realized: Dictionary = BodyAttributes.compute_realized(
					entity.body.potentials, body_age_y
				)
				for body_axis in new_realized:
					var old_bval: float = entity.body.realized.get(body_axis, 0.0)
					var new_bval: float = new_realized[body_axis]
					entity.body.realized[body_axis] = new_bval
					if absf(new_bval - old_bval) >= 0.02:
						emit_event("body_attribute_changed", {
							"entity_id": entity.id,
							"axis": body_axis,
							"old_val": old_bval,
							"new_val": new_bval,
							"age_years": body_age_y,
						})
				entity.speed = entity.body.realized["agi"] * GameConfig.BODY_SPEED_SCALE + GameConfig.BODY_SPEED_BASE
				entity.strength = entity.body.realized["str"]


func _on_stage_changed(entity: RefCounted, old_stage: String, new_stage: String, tick: int) -> void:
	var age_years: float = GameConfig.get_age_years(entity.age)
	emit_event("age_stage_changed", {
		"entity_id": entity.id,
		"entity_name": entity.entity_name,
		"from_stage": old_stage,
		"to_stage": new_stage,
		"age_years": age_years,
		"tick": tick,
	})
	match new_stage:
		"teen":
			SimulationBus.emit_signal("ui_notification",
				"%s grew up (teen, %.0fy)" % [entity.entity_name, age_years], "growth")
		"adult":
			SimulationBus.emit_signal("ui_notification",
				"%s is now adult (%.0fy)" % [entity.entity_name, age_years], "growth")
		"elder":
			SimulationBus.emit_signal("ui_notification",
				"%s became elder (%.0fy)" % [entity.entity_name, age_years], "growth")
			# Elders can't be builders — clear for reassignment
			if entity.job == "builder":
				entity.job = "none"
