extends "res://scripts/core/simulation/simulation_system.gd"

## [Tilly 1978, Keeley 1996, Hardin 1968]
## Tracks inter-settlement resource tension and triggers skirmish events.
##
## Tension per pair = 0.0 (neutral) → 1.0 (raid imminent)
## Sources: resource scarcity, proximity
## Decay: natural over time if no conflict
## Trigger: tension > TENSION_SKIRMISH_THRESHOLD → possible skirmish
## priority=64, tick_interval=TENSION_CHECK_INTERVAL_TICKS (bi-annual)

const CombatResolverScript = preload("res://scripts/core/combat/combat_resolver.gd")
const StatCacheScript = preload("res://scripts/core/stats/stat_cache.gd")
const _SIM_BRIDGE_NODE_NAME: String = "SimBridge"
const _SIM_BRIDGE_TENSION_PRESSURE_METHOD: String = "body_tension_scarcity_pressure"
const _SIM_BRIDGE_TENSION_NEXT_METHOD: String = "body_tension_next_value"

var _entity_manager: RefCounted
var _settlement_manager: RefCounted
var _world_data: RefCounted
var _chronicle
var _combat_rng: RandomNumberGenerator
var _bridge_checked: bool = false
var _sim_bridge: Object = null

## Settlement pair tension: "min_id:max_id" -> float
var _tension: Dictionary = {}
## Cooldown: "min_id:max_id" -> last_skirmish_tick
var _skirmish_cooldown: Dictionary = {}


func _init() -> void:
	system_name = "tension"
	priority = 64
	tick_interval = GameConfig.TENSION_CHECK_INTERVAL_TICKS


func init(p_entity_manager: RefCounted, p_settlement_manager: RefCounted,
		p_world_data: RefCounted, p_chronicle,
		p_rng: RandomNumberGenerator) -> void:
	_entity_manager = p_entity_manager
	_settlement_manager = p_settlement_manager
	_world_data = p_world_data
	_chronicle = p_chronicle
	_combat_rng = RandomNumberGenerator.new()
	_combat_rng.seed = p_rng.randi()


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
	and node.has_method(_SIM_BRIDGE_TENSION_PRESSURE_METHOD) \
	and node.has_method(_SIM_BRIDGE_TENSION_NEXT_METHOD):
		_sim_bridge = node
	return _sim_bridge


func execute_tick(tick: int) -> void:
	var settlements: Array = _settlement_manager.get_all_settlements()
	if settlements.size() < 2:
		return

	## Update tension for all nearby pairs
	for i in range(settlements.size()):
		for j in range(i + 1, settlements.size()):
			_update_pair_tension(settlements[i], settlements[j], tick)

	## Check skirmish triggers
	for key in _tension.keys():
		if float(_tension[key]) > GameConfig.TENSION_SKIRMISH_THRESHOLD:
			_maybe_trigger_skirmish(key, tick)


func _update_pair_tension(s1: RefCounted, s2: RefCounted, _tick: int) -> void:
	## Proximity gate
	var dist: float = Vector2(float(s1.center_x), float(s1.center_y)).distance_to(
		Vector2(float(s2.center_x), float(s2.center_y)))
	if dist > float(GameConfig.TENSION_PROXIMITY_RADIUS):
		return

	var key: String = _pair_key(s1.id, s2.id)
	var current: float = float(_tension.get(key, 0.0))
	var current_base: float = current

	## Resource scarcity driver
	var s1_deficit: bool = _is_food_scarce(s1)
	var s2_deficit: bool = _is_food_scarce(s2)
	var scarcity_pressure: float = 0.0
	if s1_deficit or s2_deficit:
		scarcity_pressure = GameConfig.TENSION_PER_SHARED_RESOURCE * 2.0
	var bridge: Object = _get_sim_bridge()
	if bridge != null:
		var pressure_variant: Variant = bridge.call(
			_SIM_BRIDGE_TENSION_PRESSURE_METHOD,
			s1_deficit,
			s2_deficit,
			float(GameConfig.TENSION_PER_SHARED_RESOURCE),
		)
		if pressure_variant != null:
			scarcity_pressure = float(pressure_variant)

	## Natural decay
	var dt_years: float = float(tick_interval) / float(GameConfig.TICKS_PER_YEAR)
	var decay: float = GameConfig.TENSION_DECAY_PER_YEAR * dt_years

	var fallback_next: float = clampf(current + scarcity_pressure - decay, 0.0, 1.0)
	current = fallback_next
	if bridge != null:
		var next_variant: Variant = bridge.call(
			_SIM_BRIDGE_TENSION_NEXT_METHOD,
			current_base,
			scarcity_pressure,
			float(GameConfig.TENSION_DECAY_PER_YEAR),
			dt_years,
		)
		if next_variant != null:
			current = clampf(float(next_variant), 0.0, 1.0)
	_tension[key] = current


func _is_food_scarce(settlement: RefCounted) -> bool:
	var total_food: float = 0.0
	var pop: int = 0
	for mid in settlement.member_ids:
		var e: RefCounted = _entity_manager.get_entity(mid)
		if e != null and e.is_alive:
			total_food += float(e.inventory.get("food", 0.0))
			pop += 1
	if pop == 0:
		return false
	var need_30d: float = float(pop) * GameConfig.HUNGER_DECAY_RATE \
		* float(GameConfig.TICKS_PER_DAY) * 30.0
	return (total_food / maxf(need_30d, 1.0)) < GameConfig.TENSION_RESOURCE_DEFICIT_TRIGGER


func _maybe_trigger_skirmish(pair_key: String, tick: int) -> void:
	## Cooldown check
	var last_skirmish: int = int(_skirmish_cooldown.get(pair_key, -99999))
	if tick - last_skirmish < GameConfig.TENSION_SKIRMISH_COOLDOWN:
		return

	if _combat_rng.randf() > GameConfig.TENSION_SKIRMISH_CHANCE:
		return

	## Identify the two settlements
	var parts: PackedStringArray = pair_key.split(":")
	var id1: int = int(parts[0])
	var id2: int = int(parts[1])
	var s1: RefCounted = _settlement_manager.get_settlement(id1)
	var s2: RefCounted = _settlement_manager.get_settlement(id2)
	if s1 == null or s2 == null:
		return

	## Attacker = food-scarce side, or s1 if both/neither
	var attacker: RefCounted = s1
	var defender: RefCounted = s2
	if _is_food_scarce(s2) and not _is_food_scarce(s1):
		attacker = s2
		defender = s1

	_execute_skirmish(attacker, defender, pair_key, tick)


func _execute_skirmish(attacker: RefCounted, defender: RefCounted,
		pair_key: String, tick: int) -> void:

	_skirmish_cooldown[pair_key] = tick

	var atk_fighters: Array = _get_fighters(attacker, 5)
	var def_fighters: Array = _get_fighters(defender, 5)

	if atk_fighters.is_empty() or def_fighters.is_empty():
		return

	var atk_leader_charisma: float = _get_leader_charisma(attacker)
	var def_leader_charisma: float = _get_leader_charisma(defender)

	var atk_casualties: int = 0
	var def_casualties: int = 0
	var atk_routed: bool = false
	var def_routed: bool = false

	## Run up to 3 rounds of duels (or until one side routes)
	for _round in range(3):
		if atk_fighters.is_empty() or def_fighters.is_empty():
			break

		var atk_morale: float = _compute_side_morale(atk_fighters, atk_leader_charisma, attacker)
		var def_morale: float = _compute_side_morale(def_fighters, def_leader_charisma, defender)

		if CombatResolverScript.check_morale_state(atk_morale) == "rout":
			atk_routed = true
			break
		if CombatResolverScript.check_morale_state(def_morale) == "rout":
			def_routed = true
			break

		var pair_count: int = mini(atk_fighters.size(), def_fighters.size())
		for i in range(pair_count):
			var atk_e: RefCounted = atk_fighters[i]
			var def_e: RefCounted = def_fighters[i]

			if CombatResolverScript.check_morale_state(atk_morale) == "shaken" and \
					_combat_rng.randf() < 0.50:
				continue

			var duel: Dictionary = CombatResolverScript.resolve_duel(atk_e, def_e, _combat_rng)
			var loser_id: int = int(duel.get("loser_id", -1))
			var loser_status: String = duel.get("loser_status", "fled")
			if loser_id == atk_e.id:
				_apply_combat_outcome(atk_e, loser_status, tick)
				if loser_status == "dead":
					atk_casualties += 1
					atk_fighters.erase(atk_e)
			elif loser_id == def_e.id:
				_apply_combat_outcome(def_e, loser_status, tick)
				if loser_status == "dead":
					def_casualties += 1
					def_fighters.erase(def_e)

	var attacker_won: bool = def_routed or \
		(def_casualties > atk_casualties and not atk_routed)

	if attacker_won:
		_transfer_raid_resources(attacker, defender)
		_tension[pair_key] = clampf(
			float(_tension.get(pair_key, 0.5)) - GameConfig.TENSION_WINNER_REDUCTION, 0.0, 1.0)
	else:
		_tension[pair_key] = clampf(
			float(_tension.get(pair_key, 0.5)) + GameConfig.TENSION_LOSER_INCREASE, 0.0, 1.0)

	## War veteran scar for all participants
	var all_participants: Array = atk_fighters + def_fighters
	for e in all_participants:
		_add_war_veteran_scar(e, tick)

	if _chronicle != null:
		_chronicle.log_event("skirmish", -1,
			"Settlement %d raided Settlement %d — %s" % [
				attacker.id, defender.id,
				"attacker won" if attacker_won else "defender repelled"
			], 4, [], tick,
			{"key": "SKIRMISH_EVENT_FMT", "params": {
				"attacker_id": attacker.id, "defender_id": defender.id,
				"atk_cas": atk_casualties, "def_cas": def_casualties,
				"winner": "attacker" if attacker_won else "defender"
			}})

	SimulationBus.emit_event("skirmish", {
		"attacker_id": attacker.id,
		"defender_id": defender.id,
		"attacker_won": attacker_won,
		"attacker_casualties": atk_casualties,
		"defender_casualties": def_casualties,
		"tick": tick,
	})


## ── Helpers ──────────────────────────────────────────────────────────────────

func _get_fighters(settlement: RefCounted, max_count: int) -> Array:
	var result: Array = []
	for mid in settlement.member_ids:
		var e: RefCounted = _entity_manager.get_entity(mid)
		if e == null or not e.is_alive:
			continue
		if e.age_stage != "adult" and e.age_stage != "elder":
			continue
		result.append(e)
		if result.size() >= max_count:
			break
	return result


func _get_leader_charisma(settlement: RefCounted) -> float:
	if settlement.leader_id < 0:
		return 0.4
	var leader: RefCounted = _entity_manager.get_entity(settlement.leader_id)
	if leader == null or not leader.is_alive:
		return 0.3
	return float(StatCacheScript.get_value(leader.stat_cache, &"DERIVED_CHARISMA", 500)) / 1000.0


func _compute_side_morale(fighters: Array, leader_charisma: float,
		settlement: RefCounted) -> float:
	var sv: Dictionary = settlement.shared_values
	var martial = sv.get(&"MARTIAL_PROWESS", 0.0)
	var cause: float = (float(martial) + 1.0) / 2.0  ## remap -1~1 to 0~1
	var avg_morale: float = 0.0
	for e in fighters:
		avg_morale += CombatResolverScript.compute_morale(e, leader_charisma, cause)
	return avg_morale / maxf(float(fighters.size()), 1.0)


func _apply_combat_outcome(entity: RefCounted, status: String, tick: int) -> void:
	match status:
		"dead":
			entity.is_alive = false
			SimulationBus.entity_died.emit(entity.id, entity.entity_name, "combat",
				float(entity.age) / float(GameConfig.TICKS_PER_YEAR), tick)
		"incapacitated":
			if entity.body != null:
				var pd: Dictionary = entity.body.part_damage
				var limb_dmg: float = float(pd.get("limb_left", 0.0)) \
					+ float(pd.get("limb_right", 0.0))
				entity.body.set_meta("combat_speed_penalty",
					clampf(limb_dmg * GameConfig.COMBAT_LIMB_SPEED_PENALTY, 0.0, 0.60))


func _transfer_raid_resources(attacker_settlement: RefCounted,
		defender_settlement: RefCounted) -> void:
	var total_stolen: float = 0.0
	for mid in defender_settlement.member_ids:
		var e: RefCounted = _entity_manager.get_entity(mid)
		if e == null or not e.is_alive:
			continue
		var steal: float = float(e.inventory.get("food", 0.0)) * 0.20
		e.inventory["food"] = maxf(float(e.inventory.get("food", 0.0)) - steal, 0.0)
		total_stolen += steal

	if total_stolen > 0.0 and not attacker_settlement.member_ids.is_empty():
		var share: float = total_stolen / float(attacker_settlement.member_ids.size())
		for mid in attacker_settlement.member_ids:
			var e: RefCounted = _entity_manager.get_entity(mid)
			if e != null and e.is_alive:
				e.inventory["food"] = float(e.inventory.get("food", 0.0)) + share


func _add_war_veteran_scar(entity: RefCounted, tick: int) -> void:
	for scar in entity.trauma_scars:
		if scar.get("scar_id") == "war_veteran":
			return
	entity.trauma_scars.append({
		"scar_id": "war_veteran",
		"stacks": 1,
		"acquired_tick": tick,
	})


static func _pair_key(id1: int, id2: int) -> String:
	if id1 < id2:
		return "%d:%d" % [id1, id2]
	return "%d:%d" % [id2, id1]


## Serialize for save
func to_save_data() -> Dictionary:
	return {
		"tension": _tension.duplicate(),
		"skirmish_cooldown": _skirmish_cooldown.duplicate(),
	}


func load_save_data(data: Dictionary) -> void:
	_tension = data.get("tension", {}).duplicate()
	_skirmish_cooldown = data.get("skirmish_cooldown", {}).duplicate()
