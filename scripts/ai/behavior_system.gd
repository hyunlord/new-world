extends "res://scripts/core/simulation_system.gd"

var _entity_manager: RefCounted
var _world_data: RefCounted
var _rng: RandomNumberGenerator
var _resource_map: RefCounted
var _building_manager: RefCounted
var _settlement_manager: RefCounted
## TraitViolationSystem 참조 (Phase 3B), main.gd에서 주입
var _trait_violation_system = null
const TraitSystem = preload("res://scripts/systems/trait_system.gd")
var _morale_system = null
## Debug-only: override hysteresis threshold for testing (-1 = use default HYSTERESIS_THRESHOLD)
var _debug_hysteresis_threshold_override: float = -1.0

## [Hysteresis — Utility AI standard pattern]
## Action inertia threshold: prevents micro-score oscillation from flip-flopping.
const HYSTERESIS_THRESHOLD: float = 0.85


func _init() -> void:
	system_name = "behavior"
	priority = 20
	tick_interval = GameConfig.BEHAVIOR_TICK_INTERVAL


## Initialize with references (resource_map and building_manager optional for backward compat)
func init(entity_manager: RefCounted, world_data: RefCounted, rng: RandomNumberGenerator, resource_map: RefCounted = null, building_manager: RefCounted = null, settlement_manager: RefCounted = null) -> void:
	_entity_manager = entity_manager
	_world_data = world_data
	_rng = rng
	_resource_map = resource_map
	_building_manager = building_manager
	_settlement_manager = settlement_manager


func set_trait_violation_system(tvs) -> void:
	_trait_violation_system = tvs


func set_morale_system(ms) -> void:
	_morale_system = ms


## Debug helper: temporarily override hysteresis threshold. Pass -1.0 to restore default.
func debug_set_hysteresis_threshold(value: float) -> void:
	_debug_hysteresis_threshold_override = value


func execute_tick(tick: int) -> void:
	var alive: Array = _entity_manager.get_alive_entities()
	for i in range(alive.size()):
		var entity = alive[i]
		if entity.current_action == "migrate":
			continue
		# Mental break override: 브레이크 중이면 utility AI 스킵
		if entity.emotion_data != null and entity.emotion_data.mental_break_type != "":
			if entity.action_timer <= 0:
				_assign_break_action(entity, entity.emotion_data.mental_break_type, tick)
			continue
		if entity.action_timer > 0:
			continue
		var scores: Dictionary = _evaluate_actions(entity)
		var best_action: String = "wander"
		var best_score: float = -1.0
		for action: String in scores:
			var score: float = scores[action]
			if score > best_score:
				best_score = score
				best_action = action
		## [Hysteresis] If current action scores >= best * threshold, keep current action.
		var _eff_threshold: float = _debug_hysteresis_threshold_override \
			if _debug_hysteresis_threshold_override >= 0.0 else HYSTERESIS_THRESHOLD
		if entity.current_action != "" \
				and entity.current_action in scores \
				and scores[entity.current_action] >= best_score * _eff_threshold:
			best_action = entity.current_action
		_assign_action(entity, best_action, tick)


func _evaluate_actions(entity: RefCounted) -> Dictionary:
	var hunger_deficit: float = 1.0 - entity.hunger
	var energy_deficit: float = 1.0 - entity.energy
	var social_deficit: float = 1.0 - entity.social
	var stage: String = entity.age_stage

	# Infant/toddler: wander, rest, socialize only
	if stage == "infant" or stage == "toddler":
		return {
			"wander": 0.3 + _rng.randf() * 0.1,
			"rest": _urgency_curve(energy_deficit) * 1.2,
			"socialize": _urgency_curve(social_deficit) * 0.8,
		}

	# Child: basic actions + reduced food/wood gathering
	if stage == "child":
		var child_scores: Dictionary = {
			"wander": 0.3 + _rng.randf() * 0.1,
			"rest": _urgency_curve(energy_deficit) * 1.2,
			"socialize": _urgency_curve(social_deficit) * 0.8,
			"gather_food": _urgency_curve(hunger_deficit) * 1.5 * 0.4,
		}
		if _resource_map != null and _has_nearby_resource(entity.position, GameConfig.ResourceType.WOOD, 15):
			child_scores["gather_wood"] = (0.3 + _rng.randf() * 0.1) * 0.3
		return child_scores

	var scores: Dictionary = {
		"wander": 0.2 + _rng.randf() * 0.1,
		"gather_food": _urgency_curve(hunger_deficit) * 1.5,
		"rest": _urgency_curve(energy_deficit) * 1.2,
		"socialize": _urgency_curve(social_deficit) * 0.8,
	}

	# Visit partner: seek proximity with partner for love/pregnancy
	if entity.partner_id != -1 and (stage == "adult" or stage == "elder"):
		var partner: RefCounted = _entity_manager.get_entity(entity.partner_id)
		if partner != null and partner.is_alive:
			var pdx: int = absi(entity.position.x - partner.position.x)
			var pdy: int = absi(entity.position.y - partner.position.y)
			if pdx > 3 or pdy > 3:
				# Partner is far, want to visit
				scores["visit_partner"] = 0.4 + _rng.randf() * 0.1
				if entity.emotions.get("love", 0.0) > 0.3:
					scores["visit_partner"] = 0.6  # Higher when in love

	# ── Hunger override: ALL jobs prioritize food when starving ──
	if entity.hunger < 0.3:
		scores["gather_food"] = 1.0
		# If entity has food in inventory, just eat (will auto-eat in needs_system)
		if entity.inventory.get("food", 0.0) > 0.5:
			scores["gather_food"] = 0.5  # lower because auto-eat handles it

	# Resource gathering (requires resource_map)
	if _resource_map != null:
		if _has_nearby_resource(entity.position, GameConfig.ResourceType.WOOD, 15):
			scores["gather_wood"] = 0.3 + _rng.randf() * 0.1
		if _has_nearby_resource(entity.position, GameConfig.ResourceType.STONE, 15):
			scores["gather_stone"] = 0.2 + _rng.randf() * 0.1

	if stage == "teen":
		if scores.has("gather_wood"):
			scores["gather_wood"] *= 0.7
		scores.erase("gather_stone")

	# Building-related actions (requires building_manager) — adults only
	if _building_manager != null and stage == "adult":
		var sid: int = entity.settlement_id
		# Deliver to stockpile — gradual threshold
		var carry: float = entity.get_total_carry()
		if carry > 3.0:
			var stockpile: RefCounted = _find_nearest_building_in_settlement(
				entity.position, "stockpile", sid, true
			)
			if stockpile != null:
				if carry > 6.0:
					scores["deliver_to_stockpile"] = 0.9
				else:
					scores["deliver_to_stockpile"] = 0.6

		# Build when there are unbuilt buildings or we can place new ones
		var unbuilt: RefCounted = _find_unbuilt_building(entity)
		if unbuilt != null:
			scores["build"] = 0.4 + _rng.randf() * 0.1
		elif entity.job == "builder" and _should_place_building(entity):
			scores["build"] = 0.4 + _rng.randf() * 0.1

		# Take from stockpile when hungry
		if hunger_deficit > 0.3:
			var stockpile: RefCounted = _find_nearest_building_in_settlement(
				entity.position, "stockpile", sid, true
			)
			if stockpile != null and stockpile.storage.get("food", 0.0) > 0.5:
				scores["take_from_stockpile"] = _urgency_curve(hunger_deficit) * 1.3

	# Apply job bonuses
	match entity.job:
		"gatherer":
			if scores.has("gather_food"):
				scores["gather_food"] *= 1.5
		"lumberjack":
			if scores.has("gather_wood"):
				scores["gather_wood"] *= 1.5
		"builder":
			if scores.has("build"):
				scores["build"] *= 1.5
			# Builder should gather wood when can't afford any building
			if _building_manager != null and _should_place_building(entity):
				if not _builder_can_afford_anything(entity):
					if scores.has("gather_wood"):
						scores["gather_wood"] *= 2.0
					elif _resource_map != null and _has_nearby_resource(entity.position, GameConfig.ResourceType.WOOD, 20):
						scores["gather_wood"] = 0.7
		"miner":
			if scores.has("gather_stone"):
				scores["gather_stone"] *= 1.5

	# Apply trait / morale / stress weights
	for action in scores.keys():
		scores[action] *= TraitSystem.get_behavior_weight(entity, action)
		if _morale_system != null:
			scores[action] *= _morale_system.get_behavior_weight_multiplier(entity.id)
		if action == "rest" and entity.emotion_data != null and entity.emotion_data.stress > 0.6:
			scores[action] *= 1.0 + entity.emotion_data.stress

	## [Boredom] Apply repetition penalty before hysteresis.
	for action in scores.keys():
		scores[action] *= _calc_boredom_penalty(entity, action)

	return scores


## Exponential urgency: higher deficit = much higher urgency
func _urgency_curve(deficit: float) -> float:
	return pow(deficit, 2.0)


## [Boredom / Exploration-Exploitation — Cohen et al. (2007)]
## [HEXACO O_inquisitiveness — Lee & Ashton (2004)]
## Returns penalty multiplier based on consecutive repetitions of action in history.
## High O_inquisitiveness → stronger penalty (seeks novelty).
## Low O_inquisitiveness → weaker penalty (prefers routine).
func _calc_boredom_penalty(entity: RefCounted, action: String) -> float:
	var repeat_count: int = 0
	var history: Array = entity.action_history
	for i in range(history.size() - 1, -1, -1):
		if history[i].get("action", "") == action:
			repeat_count += 1
		else:
			break
	if repeat_count < 3:
		return 1.0
	var inquisitiveness: float = 0.5
	if entity.personality != null:
		inquisitiveness = entity.personality.facets.get("O_inquisitiveness", 0.5)
	var base_penalty: float = 0.85
	if repeat_count >= 5:
		base_penalty = 0.70
	var inq_modifier: float = inquisitiveness * 0.30
	return maxf(0.30, base_penalty - inq_modifier)


## 멘탈 브레이크 행동 오버라이드
func _assign_break_action(entity: RefCounted, break_type: String, tick: int) -> void:
	var action: String = "rest"
	match break_type:
		"panic":
			# 도주/은신: 빠른 wander
			action = "wander"
		"rage", "outrage_violence":
			# 공격/파괴: wander (실제 전투는 미래 구현)
			action = "wander"
		"shutdown":
			# 멈춤: 제자리 rest
			action = "rest"
		"purge":
			# 폭식: 비축품 접근
			action = "take_from_stockpile"
		"grief_withdrawal":
			# 은둔: rest
			action = "rest"
		"fugue":
			# 방황: wander
			action = "wander"
		"paranoia":
			# 의심/고립: wander
			action = "wander"
		"compulsive_ritual":
			# 반복 행동: rest (제자리)
			action = "rest"
		"hysterical_bonding":
			# 집착: socialize
			action = "socialize"
		_:
			action = "rest"
	_assign_action(entity, action, tick)


func _assign_action(entity: RefCounted, action: String, tick: int) -> void:
	var old_action: String = entity.current_action
	entity.current_action = action
	# ── Trait 위반 체크 (Phase 3B) ─────────────────────────────
	# 에이전트의 Trait에 반하는 행동 실행 시 스트레스 발생
	# 학술: Cognitive Dissonance Theory (Festinger, 1957)
	# 게임 레퍼런스: CK3 stress on trait-violating actions
	if _trait_violation_system != null:
		var vctx: Dictionary = {
			"forced_by_authority": false,
			"survival_necessity": entity.hunger < 0.3,
			"witness_relationship": "none",
			"victim_relationship": "stranger",
			"is_habit": false,
			"tick": tick,
		}
		_trait_violation_system.on_action_performed(entity, action, vctx)
	# Clear cached path when action changes
	if action != old_action:
		entity.cached_path = []
		entity.path_index = 0
		# Track action history (max 20 entries)
		if entity.action_history.size() >= 20:
			entity.action_history.pop_front()
		entity.action_history.append({"tick": tick, "action": action})
		emit_event("action_changed", {
			"entity_id": entity.id,
			"entity_name": entity.entity_name,
			"from": old_action,
			"to": action,
			"tick": tick,
		})

	var sid: int = entity.settlement_id

	match action:
		"wander":
			entity.action_target = _find_random_walkable_nearby(entity.position, 5)
			entity.action_timer = 5
		"gather_food":
			if _resource_map != null:
				entity.action_target = _find_resource_tile(entity.position, GameConfig.ResourceType.FOOD, 15)
			else:
				entity.action_target = _find_food_tile(entity.position, 10)
			entity.action_timer = 20
		"gather_wood":
			entity.action_target = _find_resource_tile(entity.position, GameConfig.ResourceType.WOOD, 15)
			entity.action_timer = 20
		"gather_stone":
			entity.action_target = _find_resource_tile(entity.position, GameConfig.ResourceType.STONE, 15)
			entity.action_timer = 20
		"deliver_to_stockpile":
			var stockpile: RefCounted = _find_nearest_building_in_settlement(
				entity.position, "stockpile", sid, true
			)
			if stockpile != null:
				entity.action_target = Vector2i(stockpile.tile_x, stockpile.tile_y)
			else:
				entity.action_target = entity.position
			entity.action_timer = 30
		"build":
			var building: RefCounted = _find_unbuilt_building(entity)
			if building == null and _building_manager != null:
				building = _try_place_building(entity)
			if building != null:
				entity.action_target = Vector2i(building.tile_x, building.tile_y)
			else:
				entity.action_target = entity.position
			entity.action_timer = 25
		"take_from_stockpile":
			var stockpile: RefCounted = _find_nearest_building_in_settlement(
				entity.position, "stockpile", sid, true
			)
			if stockpile != null:
				entity.action_target = Vector2i(stockpile.tile_x, stockpile.tile_y)
			else:
				entity.action_target = entity.position
			entity.action_timer = 15
		"rest":
			if _building_manager != null:
				var shelter: RefCounted = _find_nearest_building_in_settlement(
					entity.position, "shelter", sid, true
				)
				if shelter != null:
					entity.action_target = Vector2i(shelter.tile_x, shelter.tile_y)
				else:
					entity.action_target = entity.position
			else:
				entity.action_target = entity.position
			entity.action_timer = 10
		"socialize":
			## [Maslow (1943) — Social belonging need must be satisfied after contact.]
			## Partial recovery (+0.15) on action start: single contact partially fills social need.
			## social_event_system handles relationship strength separately.
			entity.action_target = _find_nearest_entity(entity)
			entity.action_timer = 8
			entity.social = minf(1.0, entity.social + 0.15)
		"visit_partner":
			var partner: RefCounted = _entity_manager.get_entity(entity.partner_id)
			if partner != null:
				entity.action_target = partner.position
			else:
				entity.action_target = entity.position
			entity.action_timer = 15

	emit_event("action_chosen", {
		"entity_id": entity.id,
		"entity_name": entity.entity_name,
		"action": action,
		"tick": tick,
	})


## ─── Tile/Entity Finders ─────────────────────────────────

func _find_random_walkable_nearby(pos: Vector2i, radius: int) -> Vector2i:
	var candidates: Array[Vector2i] = []
	for dy in range(-radius, radius + 1):
		for dx in range(-radius, radius + 1):
			if dx == 0 and dy == 0:
				continue
			var nx: int = pos.x + dx
			var ny: int = pos.y + dy
			if _world_data.is_walkable(nx, ny):
				candidates.append(Vector2i(nx, ny))
	if candidates.is_empty():
		return pos
	return candidates[_rng.randi() % candidates.size()]


func _find_food_tile(pos: Vector2i, radius: int) -> Vector2i:
	var candidates: Array[Vector2i] = []
	for dy in range(-radius, radius + 1):
		for dx in range(-radius, radius + 1):
			var nx: int = pos.x + dx
			var ny: int = pos.y + dy
			if not _world_data.is_valid(nx, ny):
				continue
			var biome: int = _world_data.get_biome(nx, ny)
			if biome == GameConfig.Biome.GRASSLAND or biome == GameConfig.Biome.FOREST:
				candidates.append(Vector2i(nx, ny))
	if candidates.is_empty():
		return _find_random_walkable_nearby(pos, radius)
	return candidates[_rng.randi() % candidates.size()]


func _find_nearest_entity(entity: RefCounted) -> Vector2i:
	var nearby: Array = _entity_manager.get_entities_near(entity.position, 10)
	var best_dist: int = 999999
	var best_pos: Vector2i = entity.position
	for i in range(nearby.size()):
		var other = nearby[i]
		if other.id == entity.id:
			continue
		var dist: int = absi(other.position.x - entity.position.x) + absi(other.position.y - entity.position.y)
		if dist < best_dist:
			best_dist = dist
			best_pos = other.position
	if best_dist == 999999:
		return _find_random_walkable_nearby(entity.position, 5)
	return best_pos


## ─── Resource Finders ────────────────────────────────────

func _has_nearby_resource(pos: Vector2i, resource_type: int, radius: int) -> bool:
	if _resource_map == null:
		return false
	for dy in range(-radius, radius + 1, 3):
		for dx in range(-radius, radius + 1, 3):
			if _resource_map.get_resource(pos.x + dx, pos.y + dy, resource_type) >= 0.5:
				return true
	return false


func _find_resource_tile(pos: Vector2i, resource_type: int, radius: int) -> Vector2i:
	if _resource_map == null:
		return _find_random_walkable_nearby(pos, radius)
	var best_pos: Vector2i = pos
	var best_dist: int = 999999
	var best_amount: float = 0.0
	for dy in range(-radius, radius + 1):
		for dx in range(-radius, radius + 1):
			var x: int = pos.x + dx
			var y: int = pos.y + dy
			var amount: float = _resource_map.get_resource(x, y, resource_type)
			if amount >= 0.5:
				var dist: int = absi(dx) + absi(dy)
				if dist < best_dist or (dist == best_dist and amount > best_amount):
					best_dist = dist
					best_pos = Vector2i(x, y)
					best_amount = amount
	if best_dist == 999999:
		return _find_random_walkable_nearby(pos, radius)
	return best_pos


## ─── Settlement-Aware Building Helpers ─────────────────────

func _find_nearest_building_in_settlement(pos: Vector2i, btype: String, settlement_id: int, built_only: bool) -> RefCounted:
	if _building_manager == null:
		return null
	var all_buildings: Array = _building_manager.get_all_buildings()
	var nearest: RefCounted = null
	var best_dist: int = 999999
	for i in range(all_buildings.size()):
		var building: RefCounted = all_buildings[i]
		if building.building_type != btype:
			continue
		if settlement_id > 0 and building.settlement_id != settlement_id:
			continue
		if built_only and not building.is_built:
			continue
		var dist: int = absi(building.tile_x - pos.x) + absi(building.tile_y - pos.y)
		if dist < best_dist:
			best_dist = dist
			nearest = building
	return nearest


func _count_settlement_buildings(btype: String, settlement_id: int) -> int:
	if _building_manager == null:
		return 0
	var buildings: Array = _building_manager.get_buildings_by_type(btype)
	if settlement_id <= 0:
		return buildings.size()
	var count: int = 0
	for i in range(buildings.size()):
		if buildings[i].settlement_id == settlement_id:
			count += 1
	return count


func _count_settlement_alive(settlement_id: int) -> int:
	if settlement_id <= 0:
		return _entity_manager.get_alive_count()
	if _settlement_manager == null:
		return _entity_manager.get_alive_count()
	return _settlement_manager.get_settlement_population(settlement_id)


func _find_unbuilt_building(entity: RefCounted) -> RefCounted:
	if _building_manager == null:
		return null
	var sid: int = entity.settlement_id
	var all_buildings: Array = _building_manager.get_all_buildings()
	var nearest: RefCounted = null
	var best_dist: float = 999999.0
	for i in range(all_buildings.size()):
		var building: RefCounted = all_buildings[i]
		if building.is_built:
			continue
		if sid > 0 and building.settlement_id != sid:
			continue
		var dist: float = float(absi(building.tile_x - entity.position.x) + absi(building.tile_y - entity.position.y))
		if dist < best_dist:
			best_dist = dist
			nearest = building
	return nearest


func _should_place_building(entity: RefCounted) -> bool:
	if _building_manager == null:
		return false
	var sid: int = entity.settlement_id
	var stockpile_count: int = _count_settlement_buildings("stockpile", sid)
	if stockpile_count == 0:
		return true
	var alive_count: int = _count_settlement_alive(sid)
	var shelter_count: int = _count_settlement_buildings("shelter", sid)
	# Preemptive: build when within 6 of shelter cap (not just at cap)
	if shelter_count * 6 < alive_count + 6:
		return true
	var campfire_count: int = _count_settlement_buildings("campfire", sid)
	if campfire_count == 0:
		return true
	# More stockpiles as population grows
	if stockpile_count < alive_count / 10 + 1:
		return true
	return false


func _builder_can_afford_anything(entity: RefCounted) -> bool:
	var btypes: Array = ["stockpile", "campfire", "shelter"]
	for i in range(btypes.size()):
		var btype: String = btypes[i]
		var cost: Dictionary = GameConfig.BUILDING_TYPES[btype]["cost"]
		if _can_afford_building(entity, cost):
			return true
	return false


func _try_place_building(entity: RefCounted) -> RefCounted:
	if _building_manager == null:
		return null
	var sid: int = entity.settlement_id
	# Determine what to build
	var btype: String = ""
	var stockpile_count: int = _count_settlement_buildings("stockpile", sid)
	var shelter_count: int = _count_settlement_buildings("shelter", sid)
	var campfire_count: int = _count_settlement_buildings("campfire", sid)
	var alive_count: int = _count_settlement_alive(sid)

	if stockpile_count == 0:
		btype = "stockpile"
	elif shelter_count * 6 < alive_count + 6:
		btype = "shelter"
	elif campfire_count == 0:
		btype = "campfire"
	elif stockpile_count < alive_count / 10 + 1:
		btype = "stockpile"
	else:
		return null

	var cost: Dictionary = GameConfig.BUILDING_TYPES[btype]["cost"]
	if not _can_afford_building(entity, cost):
		return null

	var site: Vector2i = _find_building_site(entity.position)
	if site == Vector2i(-1, -1):
		return null

	_consume_building_cost(entity, cost)
	var building: RefCounted = _building_manager.place_building(btype, site.x, site.y)
	if building != null and sid > 0:
		building.settlement_id = sid
		if _settlement_manager != null:
			_settlement_manager.add_building(sid, building.id)
	return building


func _can_afford_building(entity: RefCounted, cost: Dictionary) -> bool:
	var sid: int = entity.settlement_id
	var cost_keys: Array = cost.keys()
	for i in range(cost_keys.size()):
		var res: String = cost_keys[i]
		var needed: float = cost[res]
		var have: float = entity.inventory.get(res, 0.0)
		if _building_manager != null:
			var stockpile: RefCounted = _find_nearest_building_in_settlement(
				entity.position, "stockpile", sid, true
			)
			if stockpile != null:
				have += stockpile.storage.get(res, 0.0)
		if have < needed:
			return false
	return true


func _consume_building_cost(entity: RefCounted, cost: Dictionary) -> void:
	var sid: int = entity.settlement_id
	var cost_keys: Array = cost.keys()
	for i in range(cost_keys.size()):
		var res: String = cost_keys[i]
		var needed: float = cost[res]
		var from_entity: float = entity.remove_item(res, needed)
		needed -= from_entity
		if needed > 0.0 and _building_manager != null:
			var stockpile: RefCounted = _find_nearest_building_in_settlement(
				entity.position, "stockpile", sid, true
			)
			if stockpile != null:
				var available: float = stockpile.storage.get(res, 0.0)
				var from_storage: float = minf(available, needed)
				stockpile.storage[res] = available - from_storage


func _find_building_site(pos: Vector2i) -> Vector2i:
	for radius in range(1, 8):
		for dy in range(-radius, radius + 1):
			for dx in range(-radius, radius + 1):
				if absi(dx) != radius and absi(dy) != radius:
					continue
				var x: int = pos.x + dx
				var y: int = pos.y + dy
				if _world_data.is_walkable(x, y) and _building_manager.get_building_at(x, y) == null:
					return Vector2i(x, y)
	return Vector2i(-1, -1)
