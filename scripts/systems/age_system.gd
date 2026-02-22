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
		# [Layer 1.5] 연간 body 재계산 + 아동기 환경 추적
		if entity.age > 0 and entity.age % GameConfig.TICKS_PER_YEAR < tick_interval:
			if entity.body != null:
				var body_age_y: float = GameConfig.get_age_years(entity.age)
				# ── [A] 아동기 환경 추적 (0~12세) ──
				if not entity.body.childhood_finalized:
					if body_age_y < 12.0:
						entity.body.child_nutrition_sum += entity.hunger
						entity.body.child_nutrition_count += 1
						var is_active: float = 0.0 if entity.current_action in ["idle", "sleep"] else 1.0
						entity.body.child_activity_sum += is_active
						entity.body.child_activity_count += 1
					else:
						entity.body.finalize_childhood_environment()
						emit_event("childhood_finalized", {
							"entity_id": entity.id,
							"nutrition_avg": entity.body.child_nutrition_sum / maxf(float(entity.body.child_nutrition_count), 1.0),
							"activity_avg": entity.body.child_activity_sum / maxf(float(entity.body.child_activity_count), 1.0),
						})
				# ── [B] realized 재계산 (5축: potential + training_gain × age_curve) ──
				for body_axis in ["str", "agi", "end", "tou", "rec"]:
					var old_realized: int = entity.body.realized.get(body_axis, 0)
					var gain: int = entity.body.calc_training_gain(body_axis)
					var age_c: float = BodyAttributes.compute_age_curve(body_axis, body_age_y)
					var new_realized: int = clampi(int(float(entity.body.potential.get(body_axis, 700) + gain) * age_c), 0, 15000)
					entity.body.realized[body_axis] = new_realized
					if abs(new_realized - old_realized) >= 50:
						emit_event("body_attribute_changed", {
							"entity_id": entity.id,
							"axis": body_axis,
							"old": old_realized,
							"new": new_realized,
							"age_years": body_age_y,
						})
				# DR: potential × age_curve only (exposure system Phase 5)
				entity.body.realized["dr"] = clampi(int(float(entity.body.potential.get("dr", 700)) * BodyAttributes.compute_age_curve("dr", body_age_y)), 0, 10000)
				# ── [C] entity 속도/근력 갱신 ──
				entity.speed = float(entity.body.realized.get("agi", 700)) * GameConfig.BODY_SPEED_SCALE + GameConfig.BODY_SPEED_BASE
				entity.strength = float(entity.body.realized.get("str", 700)) / 1000.0


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
