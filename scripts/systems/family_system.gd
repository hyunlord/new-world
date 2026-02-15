extends "res://scripts/core/simulation_system.gd"

## Handles pregnancy, birth, and widowhood.
## All reproduction goes through this system (asexual reproduction disabled).

var _entity_manager: RefCounted
var _relationship_manager: RefCounted
var _building_manager: RefCounted
var _settlement_manager: RefCounted
var _rng: RandomNumberGenerator

## 45 years in ticks (fertility cutoff for females)
var _fertility_end: int = 45 * GameConfig.TICKS_PER_YEAR


func _init() -> void:
	system_name = "family"
	priority = 52
	tick_interval = 50


func init(entity_manager: RefCounted, relationship_manager: RefCounted, building_manager: RefCounted, settlement_manager: RefCounted, rng: RandomNumberGenerator) -> void:
	_entity_manager = entity_manager
	_relationship_manager = relationship_manager
	_building_manager = building_manager
	_settlement_manager = settlement_manager
	_rng = rng


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
		# Check if pregnancy duration has passed
		var elapsed: int = tick - mother.pregnancy_tick
		if elapsed < GameConfig.PREGNANCY_DURATION:
			continue
		# Birth!
		_give_birth(mother, tick)


func _give_birth(mother: RefCounted, tick: int) -> void:
	mother.pregnancy_tick = -1

	# Find father
	var father: RefCounted = null
	if mother.partner_id != -1:
		father = _entity_manager.get_entity(mother.partner_id)

	# Consume food (from mother inventory or nearest stockpile)
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

	# Spawn child at mother's position
	var child: RefCounted = _entity_manager.spawn_entity(mother.position)
	child.age = 0
	child.age_stage = "child"
	child.birth_tick = tick
	child.settlement_id = mother.settlement_id
	child.parent_ids = [mother.id]
	if father != null:
		child.parent_ids.append(father.id)

	# Register child with parents
	mother.children_ids.append(child.id)
	if father != null:
		father.children_ids.append(child.id)

	# Assign to settlement
	if _settlement_manager != null and child.settlement_id > 0:
		_settlement_manager.add_member(child.settlement_id, child.id)

	var father_name: String = father.entity_name if father != null else "?"
	emit_event("child_born", {
		"entity_id": child.id,
		"entity_name": child.entity_name,
		"mother_id": mother.id,
		"mother_name": mother.entity_name,
		"father_id": mother.partner_id,
		"father_name": father_name,
		"tick": tick,
	})
	SimulationBus.emit_signal("ui_notification",
		"%s born to %s & %s!" % [child.entity_name, mother.entity_name, father_name], "birth")


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

		# Love >= 0.3
		if entity.emotions.get("love", 0.0) < 0.3:
			continue

		# Settlement food check: food >= population * 0.5
		if not _settlement_has_enough_food(entity.settlement_id):
			continue

		# 5% chance per check
		if _rng.randf() >= 0.05:
			continue

		# Start pregnancy!
		entity.pregnancy_tick = tick
		emit_event("pregnancy_started", {
			"entity_id": entity.id,
			"entity_name": entity.entity_name,
			"partner_id": entity.partner_id,
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
