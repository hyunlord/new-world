extends "res://scripts/core/simulation_system.gd"

var _entity_manager: RefCounted
var _building_manager: RefCounted
var _mortality_system: RefCounted
var _world_data: RefCounted = null
var _stress_system = null


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
	var alive: Array = _entity_manager.get_alive_entities()
	for i in range(alive.size()):
		var entity = alive[i]
		# Decay needs
		var hunger_mult: float = GameConfig.CHILD_HUNGER_DECAY_MULT.get(entity.age_stage, 1.0)
		var metabolic_factor: float = GameConfig.HUNGER_METABOLIC_MIN + GameConfig.HUNGER_METABOLIC_RANGE * entity.hunger
		entity.hunger -= GameConfig.HUNGER_DECAY_RATE * hunger_mult * metabolic_factor
		entity.energy -= GameConfig.ENERGY_DECAY_RATE
		entity.social -= GameConfig.SOCIAL_DECAY_RATE
		var tile_temp: float = GameConfig.WARMTH_TEMP_NEUTRAL
		var has_tile_temp: bool = false
		if _world_data != null:
			tile_temp = _world_data.get_temperature(int(entity.position.x), int(entity.position.y))
			has_tile_temp = true

		## [Maslow (1943) L1 — 갈증 소모]
		## 기본 소모 + 더운 타일에서 가속 (최대 2배)
		var thirst_decay: float = GameConfig.THIRST_DECAY_RATE
		if has_tile_temp and tile_temp > GameConfig.WARMTH_TEMP_NEUTRAL:
			thirst_decay *= 1.0 + (tile_temp - GameConfig.WARMTH_TEMP_NEUTRAL) * 2.0
		entity.thirst = maxf(0.0, entity.thirst - thirst_decay)

		## [Cannon (1932) 항상성 — 체온 소모]
		## 중립 온도(0.5) 이상이면 소모 없음, 추울수록 가속
		var warmth_decay: float = 0.0
		if has_tile_temp:
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

		## [Maslow (1943) L2 — 안전감 소모]
		entity.safety = maxf(0.0, entity.safety - GameConfig.SAFETY_DECAY_RATE)

		# Extra energy cost when performing actions
		if entity.current_action != "idle" and entity.current_action != "rest":
			entity.energy -= GameConfig.ENERGY_ACTION_COST

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
		if entity.emotion_data != null:
			if entity.thirst < GameConfig.THIRST_CRITICAL:
				var sev_thirst: float = 1.0 - (entity.thirst / GameConfig.THIRST_CRITICAL)
				if _stress_system != null:
					_stress_system.inject_stress_event(entity.emotion_data, "dehydration", 3.0 * sev_thirst)
				else:
					entity.emotion_data.stress = clampf(entity.emotion_data.stress + 3.0 * sev_thirst, 0.0, 100.0)
			if entity.warmth < GameConfig.WARMTH_CRITICAL:
				var sev_warmth: float = 1.0 - (entity.warmth / GameConfig.WARMTH_CRITICAL)
				if _stress_system != null:
					_stress_system.inject_stress_event(entity.emotion_data, "hypothermia", 4.0 * sev_warmth)
				else:
					entity.emotion_data.stress = clampf(entity.emotion_data.stress + 4.0 * sev_warmth, 0.0, 100.0)
			if entity.safety < GameConfig.SAFETY_CRITICAL:
				var sev_safety: float = 1.0 - (entity.safety / GameConfig.SAFETY_CRITICAL)
				if _stress_system != null:
					_stress_system.inject_stress_event(entity.emotion_data, "constant_threat", 2.0 * sev_safety)
				else:
					entity.emotion_data.stress = clampf(entity.emotion_data.stress + 2.0 * sev_safety, 0.0, 100.0)

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
