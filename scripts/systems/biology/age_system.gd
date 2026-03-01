extends "res://scripts/core/simulation/simulation_system.gd"

## Checks age stage transitions, emits growth notifications,
## and applies yearly personality maturation.
## Runs every 50 ticks (~4 days).

const PersonalityMaturation = preload("res://scripts/systems/psychology/personality_maturation.gd")
const _SIM_BRIDGE_NODE_NAME: String = "SimBridge"
const _SIM_BRIDGE_AGE_SPEED_METHOD: String = "body_age_body_speed"
const _SIM_BRIDGE_AGE_STRENGTH_METHOD: String = "body_age_body_strength"
var _entity_manager: RefCounted
var _personality_maturation: RefCounted
var _bridge_checked: bool = false
var _sim_bridge: Object = null


func _init() -> void:
	system_name = "aging"
	priority = 48
	tick_interval = 50


func init(entity_manager: RefCounted, rng: RandomNumberGenerator = null) -> void:
	_entity_manager = entity_manager
	if rng != null:
		_personality_maturation = PersonalityMaturation.new()
		_personality_maturation.init(rng)


func _get_sim_bridge() -> Object:
	if _bridge_checked:
		return _sim_bridge
	_bridge_checked = true
	var tree: SceneTree = Engine.get_main_loop() as SceneTree
	if tree == null:
		return null
	var root: Node = tree.get_root()
	if root == null:
		return null
	var node: Node = root.get_node_or_null(_SIM_BRIDGE_NODE_NAME)
	if node != null \
	and node.has_method(_SIM_BRIDGE_AGE_SPEED_METHOD) \
	and node.has_method(_SIM_BRIDGE_AGE_STRENGTH_METHOD):
		_sim_bridge = node
	return _sim_bridge


func execute_tick(tick: int) -> void:
	var alive: Array = _entity_manager.get_alive_entities()
	var bridge: Object = _get_sim_bridge()
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
						entity.body.child_nutrition_sum += StatQuery.get_normalized(entity, &"NEED_HUNGER")
						entity.body.child_nutrition_count += 1
						var activity_level: float = 0.0 if entity.current_action in ["idle", "sleep"] else 1.0
						entity.body.child_activity_sum += activity_level
						entity.body.child_activity_count += 1
					else:
						entity.body.finalize_childhood_environment()
						emit_event("childhood_finalized", {
							"entity_id": entity.id,
							"nutrition_avg": entity.body.child_nutrition_sum / maxf(float(entity.body.child_nutrition_count), 1.0),
							"activity_avg": entity.body.child_activity_sum / maxf(float(entity.body.child_activity_count), 1.0),
						})
				# ── [B] realized 재계산 (5축: potential + training_gain × age_curve) ──
				var realized_values: PackedInt32Array = entity.body.calc_realized_values_packed(body_age_y)
				var realized_axes: Array[String] = ["str", "agi", "end", "tou", "rec"]
				for i_axis in range(realized_axes.size()):
					var body_axis: String = realized_axes[i_axis]
					var old_realized: int = entity.body.realized.get(body_axis, 0)
					var new_realized: int = int(realized_values[i_axis])
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
				entity.body.realized["dr"] = int(realized_values[5])
				# ── [C] entity 속도/근력 갱신 ──
				var agi_realized: int = int(entity.body.realized.get("agi", 700))
				var str_realized: int = int(entity.body.realized.get("str", 700))
				entity.speed = float(agi_realized) * GameConfig.BODY_SPEED_SCALE + GameConfig.BODY_SPEED_BASE
				entity.strength = float(str_realized) / 1000.0
				if bridge != null:
					var speed_variant: Variant = bridge.call(
						_SIM_BRIDGE_AGE_SPEED_METHOD,
						agi_realized,
						float(GameConfig.BODY_SPEED_SCALE),
						float(GameConfig.BODY_SPEED_BASE),
					)
					if speed_variant != null:
						entity.speed = float(speed_variant)
					var strength_variant: Variant = bridge.call(
						_SIM_BRIDGE_AGE_STRENGTH_METHOD,
						str_realized,
					)
					if strength_variant != null:
						entity.strength = float(strength_variant)


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
