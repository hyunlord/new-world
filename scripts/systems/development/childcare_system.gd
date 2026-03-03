extends "res://scripts/core/simulation/simulation_system.gd"

const _SIM_BRIDGE_NODE_NAME: String = "SimBridge"
const _SIM_BRIDGE_TAKE_FOOD_METHOD: String = "body_childcare_take_food"
const _SIM_BRIDGE_HUNGER_AFTER_METHOD: String = "body_childcare_hunger_after"

var _entity_manager: RefCounted
var _building_manager: RefCounted
var _settlement_manager: RefCounted
var _bridge_checked: bool = false
var _sim_bridge: Object = null

const CHILDCARE_DEBUG: bool = false


func _init() -> void:
	system_name = "childcare"
	priority = 8  # Run before NeedsSystem (priority 10)
	tick_interval = 2  # Deliberately override config to match NeedsSystem frequency


## Initializes the childcare system with entity, building, and settlement managers.
func init(entity_manager: RefCounted, building_manager: RefCounted, settlement_manager: RefCounted) -> void:
	_entity_manager = entity_manager
	_building_manager = building_manager
	_settlement_manager = settlement_manager


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
	and node.has_method(_SIM_BRIDGE_TAKE_FOOD_METHOD) \
	and node.has_method(_SIM_BRIDGE_HUNGER_AFTER_METHOD):
		_sim_bridge = node
	return _sim_bridge


## Feeds hungry infants, toddlers, children, and teens from settlement stockpiles each tick, prioritizing the hungriest.
func execute_tick(tick: int) -> void:
	var alive: Array = _entity_manager.get_alive_entities()
	var hungry_children: Array = []
	for i in range(alive.size()):
		var entity: RefCounted = alive[i]
		if entity.age_stage != "infant" and entity.age_stage != "toddler" and entity.age_stage != "child" and entity.age_stage != "teen":
			continue
		var threshold: float = float(GameConfig.CHILDCARE_HUNGER_THRESHOLDS.get(entity.age_stage, 0.7))
		if StatQuery.get_normalized(entity, &"NEED_HUNGER") >= threshold:
			continue
		hungry_children.append(entity)

	hungry_children.sort_custom(Callable(self, "_sort_hunger_ascending"))

	for i in range(hungry_children.size()):
		var child: RefCounted = hungry_children[i]
		var settlement_id: int = child.settlement_id
		if settlement_id <= 0:
			continue

		var feed_amount: float = float(GameConfig.CHILDCARE_FEED_AMOUNTS.get(child.age_stage, 0.0))
		if feed_amount <= 0.0:
			continue

		var available_food: float = _get_settlement_food(settlement_id)
		if available_food <= 0.0:
			if CHILDCARE_DEBUG:
				print("[CHILDCARE_DEBUG] Tick %d | %s SKIP: no food at all" % [tick, child.entity_name])
			continue
		if available_food < feed_amount:
			feed_amount = available_food
			if CHILDCARE_DEBUG:
				print("[CHILDCARE_DEBUG] Tick %d | %s PARTIAL: need more but giving %.2f" % [
					tick, child.entity_name, feed_amount,
				])

		var old_hunger: float = StatQuery.get_normalized(child, &"NEED_HUNGER")
		var withdrawn: float = _withdraw_food(settlement_id, feed_amount)
		if withdrawn <= 0.0:
			continue

		var bridge: Object = _get_sim_bridge()
		if bridge != null:
			var rust_variant: Variant = bridge.call(
				_SIM_BRIDGE_HUNGER_AFTER_METHOD,
				float(child.hunger),
				withdrawn,
				float(GameConfig.FOOD_HUNGER_RESTORE)
			)
			if rust_variant is float:
				child.hunger = float(rust_variant)
			else:
				child.hunger = minf(child.hunger + withdrawn * GameConfig.FOOD_HUNGER_RESTORE, 1.0)
		else:
			child.hunger = minf(child.hunger + withdrawn * GameConfig.FOOD_HUNGER_RESTORE, 1.0)
		if CHILDCARE_DEBUG:
			var age_str: String = "%dy %dm" % [int(float(child.age) / 4380.0), int(fmod(float(child.age) / 365.0, 12.0))]
			var sett_food: float = _get_settlement_food(settlement_id)
			print("[CHILDCARE_DEBUG] Tick %d | %s (age %s) hunger=%.2f | sett_food=%.1f | fed=%.2f -> hunger=%.2f" % [
				tick, child.entity_name, age_str, old_hunger, sett_food, withdrawn, StatQuery.get_normalized(child, &"NEED_HUNGER"),
			])
		emit_event("child_fed", {
			"entity_id": child.id,
			"entity_name": child.entity_name,
			"amount": withdrawn,
			"settlement_id": settlement_id,
			"hunger_after": StatQuery.get_normalized(child, &"NEED_HUNGER"),
			"tick": tick,
		})


func _get_settlement_food(settlement_id: int) -> float:
	var total_food: float = 0.0
	var stockpiles: Array = _building_manager.get_buildings_by_type("stockpile")
	for i in range(stockpiles.size()):
		var stockpile: RefCounted = stockpiles[i]
		if stockpile.settlement_id != settlement_id or not stockpile.is_built:
			continue
		total_food += float(stockpile.storage.get("food", 0.0))
	return total_food


func _withdraw_food(settlement_id: int, amount: float) -> float:
	if amount <= 0.0:
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
		var bridge: Object = _get_sim_bridge()
		if bridge != null:
			var rust_variant: Variant = bridge.call(_SIM_BRIDGE_TAKE_FOOD_METHOD, available, remaining)
			if rust_variant is float:
				take = float(rust_variant)
		stockpile.storage["food"] = available - take
		remaining -= take
		withdrawn += take
	return withdrawn


func _sort_hunger_ascending(a: RefCounted, b: RefCounted) -> bool:
	return StatQuery.get_normalized(a, &"NEED_HUNGER") < StatQuery.get_normalized(b, &"NEED_HUNGER")
