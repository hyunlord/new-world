extends "res://scripts/core/simulation_system.gd"

var _entity_manager: RefCounted
var _building_manager: RefCounted


func init(entity_manager: RefCounted, building_manager: RefCounted) -> void:
	system_name = "construction"
	priority = 28
	tick_interval = GameConfig.CONSTRUCTION_TICK_INTERVAL
	_entity_manager = entity_manager
	_building_manager = building_manager


func execute_tick(tick: int) -> void:
	var entities: Array = _entity_manager.get_alive_entities()
	for i in range(entities.size()):
		var entity = entities[i]
		if entity.current_action != "build":
			continue

		var tx: int = entity.action_target.x
		var ty: int = entity.action_target.y
		var building = _building_manager.get_building_at(tx, ty)
		if building == null or building.is_built:
			continue

		# Check entity is at or adjacent to building tile
		var dx: int = absi(entity.position.x - tx)
		var dy: int = absi(entity.position.y - ty)
		if dx > 1 or dy > 1:
			continue

		# Calculate progress per tick from build_ticks config
		var build_ticks: int = GameConfig.BUILDING_TYPES.get(
			building.building_type, {}
		).get("build_ticks", 50)
		var ticks_per_cycle: int = build_ticks / GameConfig.CONSTRUCTION_TICK_INTERVAL
		if ticks_per_cycle < 1:
			ticks_per_cycle = 1
		var progress_per_tick: float = 1.0 / float(ticks_per_cycle)

		building.build_progress += progress_per_tick
		if building.build_progress >= 1.0:
			building.build_progress = 1.0
			building.is_built = true
			emit_event("building_completed", {
				"building_id": building.id,
				"building_type": building.building_type,
				"tile_x": building.tile_x,
				"tile_y": building.tile_y,
				"tick": tick,
			})
