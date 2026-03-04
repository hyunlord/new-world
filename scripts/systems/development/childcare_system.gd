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
