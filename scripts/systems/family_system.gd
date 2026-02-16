extends "res://scripts/core/simulation_system.gd"

## Handles pregnancy, birth, widowhood, and maternal complications.
## Gaussian gestation duration with preterm birth mechanics (T-2000).

const GameCalendar = preload("res://scripts/core/game_calendar.gd")

var _entity_manager: RefCounted
var _relationship_manager: RefCounted
var _building_manager: RefCounted
var _settlement_manager: RefCounted
var _mortality_system: RefCounted  # For registering births in demography log
var _rng: RandomNumberGenerator

## 45 years in ticks (fertility cutoff for females)
var _fertility_end: int = 45 * GameConfig.TICKS_PER_YEAR

## Twins probability (~0.9% natural rate)
const TWINS_CHANCE: float = 0.009

## Maternal mortality base (tech=0: ~1.5%)
const MATERNAL_DEATH_BASE: float = 0.015

## Obstructed labor chance
const OBSTRUCTED_LABOR_CHANCE: float = 0.05

## Per-entity gestation duration (entity_id -> ticks), cleared on birth
var _gestation_map: Dictionary = {}


func _init() -> void:
	system_name = "family"
	priority = 52
	tick_interval = 50


func init(entity_manager: RefCounted, relationship_manager: RefCounted, building_manager: RefCounted, settlement_manager: RefCounted, rng: RandomNumberGenerator, mortality_system: RefCounted = null) -> void:
	_entity_manager = entity_manager
	_relationship_manager = relationship_manager
	_building_manager = building_manager
	_settlement_manager = settlement_manager
	_rng = rng
	_mortality_system = mortality_system


func execute_tick(tick: int) -> void:
	var alive: Array = _entity_manager.get_alive_entities()
	_check_widowhood(alive, tick)
	_process_births(alive, tick)
	_check_pregnancies(alive, tick)


## ─── Widowhood: detect dead partners ──────────────────────

func _check_widowhood(alive: Array, tick: int) -> void:
	for i in range(alive.size()):
		var entity: RefCounted = alive[i]
		if entity.partner_id == -1:
			continue
		var partner: RefCounted = _entity_manager.get_entity(entity.partner_id)
		if partner == null or not partner.is_alive:
			entity.partner_id = -1
			entity.emotions["grief"] = minf(entity.emotions.get("grief", 0.0) + 0.8, 1.0)
			emit_event("partner_died", {
				"entity_id": entity.id,
				"entity_name": entity.entity_name,
				"tick": tick,
			})


## ─── Process active pregnancies ───────────────────────────

func _process_births(alive: Array, tick: int) -> void:
	for i in range(alive.size()):
		var mother: RefCounted = alive[i]
		if mother.pregnancy_tick < 0:
			continue
		if mother.gender != "female":
			continue
		# Look up individual gestation duration or use default
		var gestation_ticks: int = _gestation_map.get(mother.id, GameConfig.PREGNANCY_DURATION)
		var elapsed: int = tick - mother.pregnancy_tick
		if elapsed < gestation_ticks:
			continue
		# Birth!
		_gestation_map.erase(mother.id)
		_give_birth(mother, tick, gestation_ticks)


func _generate_gestation_days(mother_nutrition: float, mother_age_years: float) -> int:
	var base: float = _rng.randfn(280.0, 10.0)

	# Malnutrition → higher preterm risk
	if mother_nutrition < 0.3:
		base -= _rng.randf_range(0.0, 21.0)

	# Young (<18) or old (>40) mother → higher preterm risk
	if mother_age_years < 18.0 or mother_age_years > 40.0:
		base -= _rng.randf_range(0.0, 14.0)

	# Clamp: 154 days (22 weeks, viability limit) to 308 days (44 weeks)
	return clampi(int(base), 154, 308)


func _give_birth(mother: RefCounted, tick: int, gestation_ticks: int) -> void:
	mother.pregnancy_tick = -1

	# Find father
	var father: RefCounted = null
	if mother.partner_id != -1:
		father = _entity_manager.get_entity(mother.partner_id)

	# Gestation in weeks for health calculation
	var gestation_days: int = gestation_ticks / GameCalendar.TICKS_PER_DAY
	var gestation_weeks: int = gestation_days / 7

	# Check maternal complications
	var mother_age_years: float = GameConfig.get_age_years(mother.age)
	var complications: Dictionary = _check_birth_complications(mother, gestation_weeks)

	# Consume food
	var food_cost: float = GameConfig.BIRTH_FOOD_COST
	var from_inv: float = mother.remove_item("food", food_cost)
	food_cost -= from_inv
	if food_cost > 0.0 and _building_manager != null:
		var stockpiles: Array = _building_manager.get_buildings_by_type("stockpile")
		for j in range(stockpiles.size()):
			var sp: RefCounted = stockpiles[j]
			if sp.is_built and sp.settlement_id == mother.settlement_id:
				var avail: float = sp.storage.get("food", 0.0)
				var take: float = minf(avail, food_cost)
				sp.storage["food"] = avail - take
				food_cost -= take
				if food_cost <= 0.0:
					break

	# Determine number of babies (twins check)
	var num_babies: int = 2 if _rng.randf() < TWINS_CHANCE else 1

	for baby_idx in range(num_babies):
		_spawn_baby(mother, father, tick, gestation_weeks, mother_age_years, baby_idx)

	# Handle maternal death
	if not complications.mother_survived:
		_entity_manager.kill_entity(mother.id, "maternal_death", tick)
		if _mortality_system != null and _mortality_system.has_method("register_death"):
			_mortality_system.register_death(false)
		emit_event("maternal_death", {
			"entity_id": mother.id,
			"entity_name": mother.entity_name,
			"tick": tick,
		})
		SimulationBus.emit_signal("ui_notification",
			"%s died in childbirth" % mother.entity_name, "death")


func _spawn_baby(mother: RefCounted, father: RefCounted, tick: int, gestation_weeks: int, mother_age_years: float, baby_idx: int) -> void:
	var child: RefCounted = _entity_manager.spawn_entity(mother.position)
	child.age = 0
	child.age_stage = "infant"
	child.birth_tick = tick
	child.settlement_id = mother.settlement_id
	child.parent_ids = [mother.id]
	if father != null:
		child.parent_ids.append(father.id)

	# Calculate newborn health → frailty
	var mother_nutrition: float = clampf(mother.hunger, 0.0, 1.0)
	var health: float = _calc_newborn_health(gestation_weeks, mother_nutrition, mother_age_years, child.frailty)
	child.frailty = lerpf(2.0, 0.8, health)

	# Very unhealthy newborns die immediately
	if health < 0.1:
		_entity_manager.kill_entity(child.id, "stillborn", tick)
		if _mortality_system != null and _mortality_system.has_method("register_death"):
			_mortality_system.register_death(true)
		emit_event("stillborn", {
			"entity_id": child.id,
			"entity_name": child.entity_name,
			"health": health,
			"gestation_weeks": gestation_weeks,
			"tick": tick,
		})
		return

	# Register child with parents
	mother.children_ids.append(child.id)
	if father != null:
		father.children_ids.append(child.id)

	# Assign to settlement
	if _settlement_manager != null and child.settlement_id > 0:
		_settlement_manager.add_member(child.settlement_id, child.id)

	# Register birth in demography log and global counter
	if _mortality_system != null and _mortality_system.has_method("register_birth"):
		_mortality_system.register_birth()
	_entity_manager.register_birth()

	var father_name: String = father.entity_name if father != null else "?"
	var twin_label: String = " (twin)" if baby_idx > 0 else ""
	emit_event("child_born", {
		"entity_id": child.id,
		"entity_name": child.entity_name,
		"mother_id": mother.id,
		"mother_name": mother.entity_name,
		"father_id": mother.partner_id,
		"father_name": father_name,
		"gestation_weeks": gestation_weeks,
		"health": health,
		"frailty": child.frailty,
		"tick": tick,
	})
	SimulationBus.emit_signal("ui_notification",
		"%s%s born to %s & %s!" % [child.entity_name, twin_label, mother.entity_name, father_name], "birth")


## ─── Newborn health calculation ──────────────────────────

func _calc_newborn_health(gestation_weeks: int, mother_nutrition: float, mother_age: float, genetics_z: float) -> float:
	# 1. Gestational age survival (logistic curve)
	# w50 = weeks at 50% survival; tech=0 → 35, tech=10 → 24
	var tech: float = 0.0  # Will be connected to tech system later
	var w50: float = lerpf(35.0, 24.0, tech / 10.0)
	var survival_base: float = 1.0 / (1.0 + exp(-(float(gestation_weeks) - w50) / 2.0))

	# 2. Long-term damage from prematurity
	var damage: float = 0.0
	if gestation_weeks < 28:
		damage = lerpf(0.9, 0.3, tech / 10.0)
	elif gestation_weeks < 32:
		damage = lerpf(0.5, 0.1, tech / 10.0)
	elif gestation_weeks < 37:
		damage = lerpf(0.2, 0.02, tech / 10.0)
	else:
		damage = 0.01

	# 3. Maternal nutrition factor
	var nutrition_factor: float = lerpf(0.6, 1.1, clampf(mother_nutrition, 0.0, 1.0))

	# 4. Maternal age factor (U-curve: teens and >40 higher risk)
	var age_factor: float = 1.0
	if mother_age < 16.0:
		age_factor = 0.7
	elif mother_age < 18.0:
		age_factor = 0.85
	elif mother_age > 45.0:
		age_factor = 0.7
	elif mother_age > 40.0:
		age_factor = 0.85

	# 5. Genetics
	var genetics_factor: float = clampf(genetics_z, 0.7, 1.3)

	var health: float = survival_base * (1.0 - damage) * nutrition_factor * age_factor * genetics_factor
	return clampf(health, 0.0, 1.0)


## ─── Birth complications ─────────────────────────────────

func _check_birth_complications(mother: RefCounted, gestation_weeks: int) -> Dictionary:
	var result: Dictionary = {"mother_survived": true, "complication": "none"}

	var tech: float = 0.0
	var base_risk: float = lerpf(MATERNAL_DEATH_BASE, 0.0002, tech / 10.0)

	# Risk modifiers
	var mother_age: float = GameConfig.get_age_years(mother.age)
	if gestation_weeks < 34:
		base_risk *= 2.0
	if mother_age > 40.0:
		base_risk *= 1.5
	if mother_age < 16.0:
		base_risk *= 1.8
	if mother.hunger < 0.3:
		base_risk *= 2.0

	if _rng.randf() < base_risk:
		result.mother_survived = false
		result.complication = "maternal_death"
	elif _rng.randf() < OBSTRUCTED_LABOR_CHANCE:
		result.complication = "obstructed_labor"
		# Obstructed labor: mother takes health penalty
		mother.energy = maxf(mother.energy - 0.3, 0.0)

	return result


## ─── Check pregnancy conditions ───────────────────────────

func _check_pregnancies(alive: Array, tick: int) -> void:
	for i in range(alive.size()):
		var entity: RefCounted = alive[i]
		# Only females
		if entity.gender != "female":
			continue
		# Must be adult, not elder
		if entity.age_stage != "adult":
			continue
		# Fertility age limit (18~45)
		if entity.age >= _fertility_end:
			continue
		# Must have partner
		if entity.partner_id == -1:
			continue
		# Not already pregnant
		if entity.pregnancy_tick >= 0:
			continue
		# Max 4 children
		if entity.children_ids.size() >= 4:
			continue

		var partner: RefCounted = _entity_manager.get_entity(entity.partner_id)
		if partner == null or not partner.is_alive:
			continue

		# Partner within 3 tiles
		var dx: int = absi(entity.position.x - partner.position.x)
		var dy: int = absi(entity.position.y - partner.position.y)
		if dx > 3 or dy > 3:
			continue

		# Love >= 0.15 (lowered from 0.3 for faster first births)
		if entity.emotions.get("love", 0.0) < 0.15:
			continue

		# Food check: stockpile food OR personal hunger > 0.4
		if not _settlement_has_enough_food(entity.settlement_id):
			if entity.hunger < 0.4:
				continue

		# 8% chance per check (increased from 5% for viable population growth)
		if _rng.randf() >= 0.08:
			continue

		# Start pregnancy with Gaussian gestation duration
		entity.pregnancy_tick = tick
		var mother_nutrition: float = clampf(entity.hunger, 0.0, 1.0)
		var mother_age_years: float = GameConfig.get_age_years(entity.age)
		var gestation_days: int = _generate_gestation_days(mother_nutrition, mother_age_years)
		var gestation_ticks: int = gestation_days * GameCalendar.TICKS_PER_DAY
		# Store gestation duration in system map
		_gestation_map[entity.id] = gestation_ticks

		emit_event("pregnancy_started", {
			"entity_id": entity.id,
			"entity_name": entity.entity_name,
			"partner_id": entity.partner_id,
			"gestation_days": gestation_days,
			"tick": tick,
		})


func _settlement_has_enough_food(settlement_id: int) -> bool:
	if _building_manager == null:
		return false
	var pop: int = 0
	if _settlement_manager != null and settlement_id > 0:
		pop = _settlement_manager.get_settlement_population(settlement_id)
	else:
		pop = _entity_manager.get_alive_count()
	var total_food: float = 0.0
	var stockpiles: Array = _building_manager.get_buildings_by_type("stockpile")
	for i in range(stockpiles.size()):
		var sp: RefCounted = stockpiles[i]
		if sp.is_built:
			if settlement_id <= 0 or sp.settlement_id == settlement_id:
				total_food += sp.storage.get("food", 0.0)
	return total_food >= float(pop) * 0.5
