extends "res://scripts/core/simulation/simulation_system.gd"

var _entity_manager: RefCounted
var _building_manager: RefCounted
var _sim_engine: RefCounted
const _SIM_BRIDGE_NODE_NAME: String = "SimBridge"
const _SIM_BRIDGE_CAMPFIRE_BOOST_METHOD: String = "body_building_campfire_social_boost"
const _SIM_BRIDGE_ADD_CAPPED_METHOD: String = "body_building_add_capped"
var _bridge_checked: bool = false
var _sim_bridge: Object = null


func init(entity_manager: RefCounted, building_manager: RefCounted, sim_engine: RefCounted) -> void:
	system_name = "building_effect"
	priority = 15
	tick_interval = GameConfig.BUILDING_EFFECT_TICK_INTERVAL
	_entity_manager = entity_manager
	_building_manager = building_manager
	_sim_engine = sim_engine


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
	and node.has_method(_SIM_BRIDGE_CAMPFIRE_BOOST_METHOD) \
	and node.has_method(_SIM_BRIDGE_ADD_CAPPED_METHOD):
		_sim_bridge = node
	return _sim_bridge

func _apply_campfire(building: RefCounted) -> void:
	var time_data: Dictionary = _sim_engine.get_game_time()
	var hour: int = time_data.get("hour", 12)
	var is_night: bool = hour >= 20 or hour < 6
	var social_boost: float = 0.02 if is_night else 0.01
	var bridge: Object = _get_sim_bridge()
	if bridge != null:
		var boost_variant: Variant = bridge.call(
			_SIM_BRIDGE_CAMPFIRE_BOOST_METHOD,
			is_night,
			0.01,
			0.02,
		)
		if boost_variant != null:
			social_boost = float(boost_variant)
	var radius: int = GameConfig.BUILDING_TYPES["campfire"]["radius"]
	var nearby: Array = _entity_manager.get_entities_near(
		Vector2i(building.tile_x, building.tile_y), radius
	)
	for j in range(nearby.size()):
		var entity = nearby[j]
		var next_social: float = minf(entity.social + social_boost, 1.0)
		if bridge != null:
			var social_variant: Variant = bridge.call(
				_SIM_BRIDGE_ADD_CAPPED_METHOD,
				float(entity.social),
				social_boost,
				1.0,
			)
			if social_variant != null:
				next_social = float(social_variant)
		entity.social = next_social
		## [Cannon (1932) 항상성 — 불 근처 체온 회복]
		var next_warmth: float = minf(entity.warmth + GameConfig.WARMTH_FIRE_RESTORE, 1.0)
		if bridge != null:
			var warmth_variant: Variant = bridge.call(
				_SIM_BRIDGE_ADD_CAPPED_METHOD,
				float(entity.warmth),
				float(GameConfig.WARMTH_FIRE_RESTORE),
				1.0,
			)
			if warmth_variant != null:
				next_warmth = float(warmth_variant)
		entity.warmth = next_warmth


func _apply_shelter(building: RefCounted) -> void:
	var bridge: Object = _get_sim_bridge()
	var nearby: Array = _entity_manager.get_entities_near(
		Vector2i(building.tile_x, building.tile_y), 0
	)
	for j in range(nearby.size()):
		var entity = nearby[j]
		var next_energy: float = minf(entity.energy + 0.01, 1.0)
		if bridge != null:
			var energy_variant: Variant = bridge.call(
				_SIM_BRIDGE_ADD_CAPPED_METHOD,
				float(entity.energy),
				0.01,
				1.0,
			)
			if energy_variant != null:
				next_energy = float(energy_variant)
		entity.energy = next_energy
		## [Cannon (1932) 항상성 — shelter 체류 중 체온 회복]
		var next_warmth: float = minf(entity.warmth + GameConfig.WARMTH_SHELTER_RESTORE, 1.0)
		if bridge != null:
			var warmth_variant: Variant = bridge.call(
				_SIM_BRIDGE_ADD_CAPPED_METHOD,
				float(entity.warmth),
				float(GameConfig.WARMTH_SHELTER_RESTORE),
				1.0,
			)
			if warmth_variant != null:
				next_warmth = float(warmth_variant)
		entity.warmth = next_warmth
		## [Maslow (1943) L2 — shelter 내 안전감 지속 회복]
		var next_safety: float = minf(entity.safety + GameConfig.SAFETY_SHELTER_RESTORE, 1.0)
		if bridge != null:
			var safety_variant: Variant = bridge.call(
				_SIM_BRIDGE_ADD_CAPPED_METHOD,
				float(entity.safety),
				float(GameConfig.SAFETY_SHELTER_RESTORE),
				1.0,
			)
			if safety_variant != null:
				next_safety = float(safety_variant)
		entity.safety = next_safety
