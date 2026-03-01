extends "res://scripts/core/simulation/simulation_system.gd"

const BodyAttributes = preload("res://scripts/core/entity/body_attributes.gd")
const _BASE_DECAY_SCALAR_COUNT: int = 14
const _BASE_DECAY_FLAG_COUNT: int = 2
const _CRITICAL_SEVERITY_SCALAR_COUNT: int = 6
const _ERG_FRUSTRATION_SCALAR_COUNT: int = 10
const _ERG_FRUSTRATION_FLAG_COUNT: int = 2

var _entity_manager: RefCounted
var _building_manager: RefCounted
var _mortality_system: RefCounted
var _world_data: RefCounted = null
var _stress_system = null
var _last_tick: int = 0
var _base_decay_scalar_inputs: PackedFloat32Array = PackedFloat32Array()
var _base_decay_flag_inputs: PackedByteArray = PackedByteArray()
var _critical_severity_scalar_inputs: PackedFloat32Array = PackedFloat32Array()
var _erg_frustration_scalar_inputs: PackedFloat32Array = PackedFloat32Array()
var _erg_frustration_flag_inputs: PackedByteArray = PackedByteArray()


func _init() -> void:
	system_name = "needs"
	priority = 10
	tick_interval = GameConfig.NEEDS_TICK_INTERVAL


## Initialize with entity/building manager references
func init(entity_manager: RefCounted, building_manager: RefCounted, world_data: RefCounted = null) -> void:
	_entity_manager = entity_manager
	_building_manager = building_manager
	_world_data = world_data


func execute_tick(tick: int) -> void:
	_last_tick = tick
	if _base_decay_scalar_inputs.size() < _BASE_DECAY_SCALAR_COUNT:
		_base_decay_scalar_inputs.resize(_BASE_DECAY_SCALAR_COUNT)
	if _base_decay_flag_inputs.size() < _BASE_DECAY_FLAG_COUNT:
		_base_decay_flag_inputs.resize(_BASE_DECAY_FLAG_COUNT)
	if _critical_severity_scalar_inputs.size() < _CRITICAL_SEVERITY_SCALAR_COUNT:
		_critical_severity_scalar_inputs.resize(_CRITICAL_SEVERITY_SCALAR_COUNT)
	if _erg_frustration_scalar_inputs.size() < _ERG_FRUSTRATION_SCALAR_COUNT:
		_erg_frustration_scalar_inputs.resize(_ERG_FRUSTRATION_SCALAR_COUNT)
	if _erg_frustration_flag_inputs.size() < _ERG_FRUSTRATION_FLAG_COUNT:
		_erg_frustration_flag_inputs.resize(_ERG_FRUSTRATION_FLAG_COUNT)
	var alive: Array = _entity_manager.get_alive_entities()
	for i in range(alive.size()):
		var entity = alive[i]
		# Decay needs
		var hunger_mult: float = GameConfig.CHILD_HUNGER_DECAY_MULT.get(entity.age_stage, 1.0)
		var tile_temp: float = GameConfig.WARMTH_TEMP_NEUTRAL
		var has_tile_temp: bool = false
		if _world_data != null:
			tile_temp = _world_data.get_temperature(int(entity.position.x), int(entity.position.y))
			has_tile_temp = true
		var has_rust_base_decay: bool = false
		var rust_hunger_decay: float = 0.0
		var rust_energy_decay: float = 0.0
		var rust_social_decay: float = 0.0
		var rust_thirst_decay: float = 0.0
		var rust_warmth_decay: float = 0.0
		var rust_safety_decay: float = 0.0
		_base_decay_scalar_inputs[0] = entity.hunger
		_base_decay_scalar_inputs[1] = GameConfig.HUNGER_DECAY_RATE
		_base_decay_scalar_inputs[2] = hunger_mult
		_base_decay_scalar_inputs[3] = GameConfig.HUNGER_METABOLIC_MIN
		_base_decay_scalar_inputs[4] = GameConfig.HUNGER_METABOLIC_RANGE
		_base_decay_scalar_inputs[5] = GameConfig.ENERGY_DECAY_RATE
		_base_decay_scalar_inputs[6] = GameConfig.SOCIAL_DECAY_RATE
		_base_decay_scalar_inputs[7] = GameConfig.SAFETY_DECAY_RATE
		_base_decay_scalar_inputs[8] = GameConfig.THIRST_DECAY_RATE
		_base_decay_scalar_inputs[9] = GameConfig.WARMTH_DECAY_RATE
		_base_decay_scalar_inputs[10] = tile_temp
		_base_decay_scalar_inputs[11] = GameConfig.WARMTH_TEMP_NEUTRAL
		_base_decay_scalar_inputs[12] = GameConfig.WARMTH_TEMP_FREEZING
		_base_decay_scalar_inputs[13] = GameConfig.WARMTH_TEMP_COLD
		_base_decay_flag_inputs[0] = 1 if has_tile_temp else 0
		_base_decay_flag_inputs[1] = 1 if GameConfig.NEEDS_EXPANSION_ENABLED else 0
		var base_decay_variant: Variant = SimBridge.body_needs_base_decay_step_packed(
			_base_decay_scalar_inputs,
			_base_decay_flag_inputs
		)
		if base_decay_variant is PackedFloat32Array:
			var packed_base_decay: PackedFloat32Array = base_decay_variant
			if packed_base_decay.size() >= 6:
				rust_hunger_decay = float(packed_base_decay[0])
				rust_energy_decay = float(packed_base_decay[1])
				rust_social_decay = float(packed_base_decay[2])
				rust_thirst_decay = float(packed_base_decay[3])
				rust_warmth_decay = float(packed_base_decay[4])
				rust_safety_decay = float(packed_base_decay[5])
				has_rust_base_decay = true
		if has_rust_base_decay:
			entity.hunger -= rust_hunger_decay
			entity.energy -= rust_energy_decay
			entity.social -= rust_social_decay
			if GameConfig.NEEDS_EXPANSION_ENABLED:
				entity.safety = maxf(0.0, entity.safety - rust_safety_decay)
		else:
			var metabolic_factor: float = GameConfig.HUNGER_METABOLIC_MIN + GameConfig.HUNGER_METABOLIC_RANGE * entity.hunger
			entity.hunger -= GameConfig.HUNGER_DECAY_RATE * hunger_mult * metabolic_factor
			entity.energy -= GameConfig.ENERGY_DECAY_RATE
			entity.social -= GameConfig.SOCIAL_DECAY_RATE
			if GameConfig.NEEDS_EXPANSION_ENABLED:
				entity.safety = maxf(0.0, entity.safety - GameConfig.SAFETY_DECAY_RATE)

		## [Maslow (1943) L1 — 갈증 소모]
		## 기본 소모 + 더운 타일에서 가속 (최대 2배)
		if GameConfig.NEEDS_EXPANSION_ENABLED:
			var thirst_decay: float = GameConfig.THIRST_DECAY_RATE
			if has_rust_base_decay:
				thirst_decay = rust_thirst_decay
			elif has_tile_temp and tile_temp > GameConfig.WARMTH_TEMP_NEUTRAL:
				thirst_decay *= 1.0 + (tile_temp - GameConfig.WARMTH_TEMP_NEUTRAL) * 2.0
			entity.thirst = maxf(0.0, entity.thirst - thirst_decay)

		## [Cannon (1932) 항상성 — 체온 소모]
		## 중립 온도(0.5) 이상이면 소모 없음, 추울수록 가속
		if GameConfig.NEEDS_EXPANSION_ENABLED:
			var warmth_decay: float = 0.0
			if has_rust_base_decay:
				warmth_decay = rust_warmth_decay
			elif has_tile_temp:
				if tile_temp < GameConfig.WARMTH_TEMP_NEUTRAL:
					if tile_temp < GameConfig.WARMTH_TEMP_FREEZING:
						warmth_decay = GameConfig.WARMTH_DECAY_RATE * 5.0
					elif tile_temp < GameConfig.WARMTH_TEMP_COLD:
						warmth_decay = GameConfig.WARMTH_DECAY_RATE * 3.0
					else:
						var cold_ratio: float = (GameConfig.WARMTH_TEMP_NEUTRAL - tile_temp) / (GameConfig.WARMTH_TEMP_NEUTRAL - GameConfig.WARMTH_TEMP_COLD)
						warmth_decay = GameConfig.WARMTH_DECAY_RATE * (1.0 + cold_ratio * 2.0)
			else:
				warmth_decay = GameConfig.WARMTH_DECAY_RATE
			entity.warmth = maxf(0.0, entity.warmth - warmth_decay)

		# Extra energy cost when performing actions / recovery when resting
		if entity.current_action != "idle" and entity.current_action != "rest":
			var _end_norm: float = 0.5
			if entity.body != null and entity.body.realized.has("end"):
				_end_norm = clampf(float(entity.body.realized["end"]) / float(GameConfig.BODY_REALIZED_MAX), 0.0, 1.0)
			var _action_cost: float = BodyAttributes.compute_action_energy_cost(_end_norm)
			entity.energy -= _action_cost
		elif entity.current_action == "rest":
			var _rec_norm: float = 0.5
			if entity.body != null and entity.body.realized.has("rec"):
				_rec_norm = clampf(float(entity.body.realized["rec"]) / float(GameConfig.BODY_REALIZED_MAX), 0.0, 1.0)
			entity.energy += BodyAttributes.compute_rest_energy_recovery(_rec_norm)
			# [훈련 XP 누적] 휴식 → 회복력 훈련 (Buchheit & Laursen 2013)
			if entity.body != null:
				var _age_mod: float = BodyAttributes.get_rec_age_trainability_modifier(float(entity.age))
				entity.body.training_xp["rec"] = entity.body.training_xp.get("rec", 0.0) + GameConfig.BODY_XP_REST * _age_mod * 0.01

		# Age: derive from birth_tick (not incremental — avoids drift)
		entity.age = tick - entity.birth_tick

		# Auto-eat from inventory when hungry
		if entity.hunger < GameConfig.HUNGER_EAT_THRESHOLD:
			var food_in_inv: float = entity.inventory.get("food", 0.0)
			if food_in_inv > 0.0:
				var eat_amount: float = minf(food_in_inv, 2.0)
				entity.remove_item("food", eat_amount)
				entity.hunger += eat_amount * GameConfig.FOOD_HUNGER_RESTORE

		# Clamp all needs
		entity.hunger = clampf(entity.hunger, 0.0, 1.0)
		if entity.age_stage == "infant" or entity.age_stage == "toddler" or entity.age_stage == "child" or entity.age_stage == "teen":
			if entity.settlement_id > 0 and _get_settlement_food(entity.settlement_id) > 0.0:
				entity.hunger = maxf(entity.hunger, 0.05)
		entity.energy = clampf(entity.energy, 0.0, 1.0)
		entity.social = clampf(entity.social, 0.0, 1.0)
		entity.thirst = clampf(entity.thirst, 0.0, 1.0)
		entity.warmth = clampf(entity.warmth, 0.0, 1.0)
		entity.safety = clampf(entity.safety, 0.0, 1.0)

		## [Lazarus & Folkman (1984) — 욕구 미충족 stressor]
		if GameConfig.NEEDS_EXPANSION_ENABLED and entity.emotion_data != null:
			var has_rust_severity: bool = false
			var rust_sev_thirst: float = 0.0
			var rust_sev_warmth: float = 0.0
			var rust_sev_safety: float = 0.0
			_critical_severity_scalar_inputs[0] = entity.thirst
			_critical_severity_scalar_inputs[1] = entity.warmth
			_critical_severity_scalar_inputs[2] = entity.safety
			_critical_severity_scalar_inputs[3] = GameConfig.THIRST_CRITICAL
			_critical_severity_scalar_inputs[4] = GameConfig.WARMTH_CRITICAL
			_critical_severity_scalar_inputs[5] = GameConfig.SAFETY_CRITICAL
			var severity_variant: Variant = SimBridge.body_needs_critical_severity_step_packed(
				_critical_severity_scalar_inputs
			)
			if severity_variant is PackedFloat32Array:
				var packed_severity: PackedFloat32Array = severity_variant
				if packed_severity.size() >= 3:
					rust_sev_thirst = float(packed_severity[0])
					rust_sev_warmth = float(packed_severity[1])
					rust_sev_safety = float(packed_severity[2])
					has_rust_severity = true
			if entity.thirst < GameConfig.THIRST_CRITICAL:
				var sev_thirst: float = 1.0 - (entity.thirst / GameConfig.THIRST_CRITICAL)
				if has_rust_severity:
					sev_thirst = rust_sev_thirst
				if _stress_system != null:
					_stress_system.inject_stress_event(entity.emotion_data, "dehydration", 3.0 * sev_thirst)
				else:
					entity.emotion_data.stress = clampf(entity.emotion_data.stress + 3.0 * sev_thirst, 0.0, 100.0)
			if entity.warmth < GameConfig.WARMTH_CRITICAL:
				var sev_warmth: float = 1.0 - (entity.warmth / GameConfig.WARMTH_CRITICAL)
				if has_rust_severity:
					sev_warmth = rust_sev_warmth
				if _stress_system != null:
					_stress_system.inject_stress_event(entity.emotion_data, "hypothermia", 4.0 * sev_warmth)
				else:
					entity.emotion_data.stress = clampf(entity.emotion_data.stress + 4.0 * sev_warmth, 0.0, 100.0)
			if entity.safety < GameConfig.SAFETY_CRITICAL:
				var sev_safety: float = 1.0 - (entity.safety / GameConfig.SAFETY_CRITICAL)
				if has_rust_severity:
					sev_safety = rust_sev_safety
				if _stress_system != null:
					_stress_system.inject_stress_event(entity.emotion_data, "constant_threat", 2.0 * sev_safety)
				else:
					entity.emotion_data.stress = clampf(entity.emotion_data.stress + 2.0 * sev_safety, 0.0, 100.0)
			# [Cassidy & Berlin 1994 — Anxious attachment: chronic low-level stress when social need unmet]
			if str(entity.get_meta("attachment_type", "secure")) == "anxious":
				var attachment_stress_delta: float = 0.0
				var attachment_stress_variant: Variant = SimBridge.body_anxious_attachment_stress_delta(
					entity.social,
					GameConfig.ATTACHMENT_ANXIOUS_STRESS_THRESHOLD,
					GameConfig.ATTACHMENT_ANXIOUS_STRESS_RATE
				)
				if attachment_stress_variant != null:
					attachment_stress_delta = float(attachment_stress_variant)
				elif entity.social < GameConfig.ATTACHMENT_ANXIOUS_STRESS_THRESHOLD:
					attachment_stress_delta = GameConfig.ATTACHMENT_ANXIOUS_STRESS_RATE
				if attachment_stress_delta > 0.0:
					entity.emotion_data.stress = minf(
						entity.emotion_data.stress + attachment_stress_delta,
						100.0
					)

		# Starvation check with grace period (children get longer grace)
		if entity.hunger <= 0.0:
			var age_years: float = GameConfig.get_age_years(entity.age)
			if age_years < 15.0:
				# Child conditional protection: check settlement food
				var sett_food: float = 0.0
				if entity.settlement_id > 0:
					sett_food = _get_settlement_food(entity.settlement_id)
				if sett_food > 0.0:
					# Food exists but child is starving -> emergency feed from stockpile
					var feed_amount: float = minf(0.3, sett_food)
					var withdrawn: float = _withdraw_food(entity.settlement_id, feed_amount)
					if withdrawn > 0.0:
						entity.hunger = withdrawn * GameConfig.FOOD_HUNGER_RESTORE
					entity.starving_timer = 0
				else:
					# True famine (no settlement food) -> grace period, then starvation allowed
					entity.starving_timer += 1
					var grace: int = GameConfig.CHILD_STARVATION_GRACE_TICKS.get(entity.age_stage, GameConfig.STARVATION_GRACE_TICKS)
					if entity.starving_timer >= grace:
						emit_event("entity_starved", {
							"entity_id": entity.id,
							"entity_name": entity.entity_name,
							"starving_ticks": entity.starving_timer,
							"tick": tick,
						})
						var deceased_entity = entity
						_entity_manager.kill_entity(entity.id, "starvation", tick)
						if _mortality_system != null:
							if _mortality_system.has_method("register_death"):
								var death_age_years: float = GameConfig.get_age_years(deceased_entity.age)
								_mortality_system.register_death(death_age_years < 1.0, deceased_entity.age_stage, death_age_years, "starvation")
							if _mortality_system.has_method("inject_bereavement_stress"):
								_mortality_system.inject_bereavement_stress(deceased_entity)
					else:
						entity.hunger = 0.01  # Keep barely alive during grace
			else:
				# Adult starvation: grace period then death
				entity.starving_timer += 1
				var grace: int = GameConfig.CHILD_STARVATION_GRACE_TICKS.get(entity.age_stage, GameConfig.STARVATION_GRACE_TICKS)
				if entity.starving_timer >= grace:
					emit_event("entity_starved", {
						"entity_id": entity.id,
						"entity_name": entity.entity_name,
						"starving_ticks": entity.starving_timer,
						"tick": tick,
					})
					var deceased_entity = entity
					_entity_manager.kill_entity(entity.id, "starvation", tick)
					if _mortality_system != null:
						if _mortality_system.has_method("register_death"):
							var death_age_years: float = GameConfig.get_age_years(deceased_entity.age)
							_mortality_system.register_death(death_age_years < 1.0, deceased_entity.age_stage, death_age_years, "starvation")
						if _mortality_system.has_method("inject_bereavement_stress"):
							_mortality_system.inject_bereavement_stress(deceased_entity)
		else:
			entity.starving_timer = 0
		_update_erg_frustration(entity)


## Get total food in stockpiles belonging to a settlement
func _get_settlement_food(settlement_id: int) -> float:
	if _building_manager == null:
		return 0.0
	var total_food: float = 0.0
	var stockpiles: Array = _building_manager.get_buildings_by_type("stockpile")
	for i in range(stockpiles.size()):
		var stockpile: RefCounted = stockpiles[i]
		if stockpile.settlement_id != settlement_id or not stockpile.is_built:
			continue
		total_food += float(stockpile.storage.get("food", 0.0))
	return total_food


## Withdraw food from stockpiles belonging to a settlement
func _withdraw_food(settlement_id: int, amount: float) -> float:
	if _building_manager == null or amount <= 0.0:
		return 0.0
	var remaining: float = amount
	var withdrawn: float = 0.0
	var stockpiles: Array = _building_manager.get_buildings_by_type("stockpile")
	for i in range(stockpiles.size()):
		if remaining <= 0.0:
			break
		var stockpile: RefCounted = stockpiles[i]
		if stockpile.settlement_id != settlement_id or not stockpile.is_built:
			continue
		var available: float = float(stockpile.storage.get("food", 0.0))
		if available <= 0.0:
			continue
		var take: float = minf(available, remaining)
		stockpile.storage["food"] = available - take
		remaining -= take
		withdrawn += take
	return withdrawn


## Setter for external injection of StressSystem reference
func set_stress_system(ss) -> void:
	_stress_system = ss


## [Alderfer 1969 ERG Theory] Track sustained need frustration.
## If growth needs are chronically unmet → regression to existence-level obsession.
## If relatedness needs are chronically unmet → regression to existence-level obsession.
func _update_erg_frustration(entity: RefCounted) -> void:
	if not entity.is_alive:
		return
	var started_growth_regression: bool = false
	var started_relatedness_regression: bool = false
	var rust_applied: bool = false

	_erg_frustration_scalar_inputs[0] = entity.competence
	_erg_frustration_scalar_inputs[1] = entity.autonomy
	_erg_frustration_scalar_inputs[2] = entity.self_actualization
	_erg_frustration_scalar_inputs[3] = entity.belonging
	_erg_frustration_scalar_inputs[4] = entity.intimacy
	_erg_frustration_scalar_inputs[5] = GameConfig.ERG_GROWTH_FRUSTRATION_THRESHOLD
	_erg_frustration_scalar_inputs[6] = GameConfig.ERG_RELATEDNESS_FRUSTRATION_THRESHOLD
	_erg_frustration_scalar_inputs[7] = float(GameConfig.ERG_FRUSTRATION_WINDOW)
	_erg_frustration_scalar_inputs[8] = float(entity.erg_growth_frustration_ticks)
	_erg_frustration_scalar_inputs[9] = float(entity.erg_relatedness_frustration_ticks)
	_erg_frustration_flag_inputs[0] = 1 if entity.erg_regressing_to_existence else 0
	_erg_frustration_flag_inputs[1] = 1 if entity.erg_regressing_to_relatedness else 0

	var erg_step_variant: Variant = SimBridge.body_erg_frustration_step_packed(
		_erg_frustration_scalar_inputs,
		_erg_frustration_flag_inputs
	)
	if erg_step_variant is PackedInt32Array:
		var packed_erg_step: PackedInt32Array = erg_step_variant
		if packed_erg_step.size() >= 6:
			entity.erg_growth_frustration_ticks = int(packed_erg_step[0])
			entity.erg_relatedness_frustration_ticks = int(packed_erg_step[1])
			entity.erg_regressing_to_existence = int(packed_erg_step[2]) != 0
			started_growth_regression = int(packed_erg_step[3]) != 0
			entity.erg_regressing_to_relatedness = int(packed_erg_step[4]) != 0
			started_relatedness_regression = int(packed_erg_step[5]) != 0
			rust_applied = true

	if not rust_applied:
		# --- Growth frustration check ---
		var growth_frustrated: bool = (
			entity.competence < GameConfig.ERG_GROWTH_FRUSTRATION_THRESHOLD and
			entity.autonomy < GameConfig.ERG_GROWTH_FRUSTRATION_THRESHOLD and
			entity.self_actualization < GameConfig.ERG_GROWTH_FRUSTRATION_THRESHOLD
		)
		if growth_frustrated:
			entity.erg_growth_frustration_ticks += 1
		else:
			entity.erg_growth_frustration_ticks = maxi(0, entity.erg_growth_frustration_ticks - 10)

		var was_regressing_growth: bool = entity.erg_regressing_to_existence
		entity.erg_regressing_to_existence = (
			entity.erg_growth_frustration_ticks >= GameConfig.ERG_FRUSTRATION_WINDOW
		)
		started_growth_regression = entity.erg_regressing_to_existence and not was_regressing_growth

		# --- Relatedness frustration check ---
		var relatedness_frustrated: bool = (
			entity.belonging < GameConfig.ERG_RELATEDNESS_FRUSTRATION_THRESHOLD and
			entity.intimacy < GameConfig.ERG_RELATEDNESS_FRUSTRATION_THRESHOLD
		)
		if relatedness_frustrated:
			entity.erg_relatedness_frustration_ticks += 1
		else:
			entity.erg_relatedness_frustration_ticks = maxi(0, entity.erg_relatedness_frustration_ticks - 10)

		var was_regressing_rel: bool = entity.erg_regressing_to_relatedness
		entity.erg_regressing_to_relatedness = (
			entity.erg_relatedness_frustration_ticks >= GameConfig.ERG_FRUSTRATION_WINDOW
		)
		started_relatedness_regression = entity.erg_regressing_to_relatedness and not was_regressing_rel

	if started_growth_regression:
		if entity.emotion_data != null and _stress_system != null:
			_stress_system.inject_stress_event(entity.emotion_data, "erg_growth_regression", 5.0)
		emit_event("erg_regression_started", {
			"entity_id": entity.id,
			"entity_name": entity.entity_name,
			"regression_type": "growth_to_existence",
			"tick": _last_tick,
		})

	if started_relatedness_regression:
		if entity.emotion_data != null and _stress_system != null:
			_stress_system.inject_stress_event(entity.emotion_data, "erg_relatedness_regression", 4.0)
		emit_event("erg_regression_started", {
			"entity_id": entity.id,
			"entity_name": entity.entity_name,
			"regression_type": "relatedness_to_existence",
			"tick": _last_tick,
		})
