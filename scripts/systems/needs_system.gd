extends "res://scripts/core/simulation_system.gd"

var _entity_manager: RefCounted


func _init() -> void:
	system_name = "needs"
	priority = 10
	tick_interval = GameConfig.NEEDS_TICK_INTERVAL


## Initialize with entity manager reference
func init(entity_manager: RefCounted) -> void:
	_entity_manager = entity_manager


func execute_tick(tick: int) -> void:
	var alive: Array = _entity_manager.get_alive_entities()
	for i in range(alive.size()):
		var entity = alive[i]
		# Decay needs
		entity.hunger -= GameConfig.HUNGER_DECAY_RATE
		entity.energy -= GameConfig.ENERGY_DECAY_RATE
		entity.social -= GameConfig.SOCIAL_DECAY_RATE

		# Extra energy cost when performing actions
		if entity.current_action != "idle" and entity.current_action != "rest":
			entity.energy -= GameConfig.ENERGY_ACTION_COST

		# Age
		entity.age += 1

		# Clamp all needs
		entity.hunger = clampf(entity.hunger, 0.0, 1.0)
		entity.energy = clampf(entity.energy, 0.0, 1.0)
		entity.social = clampf(entity.social, 0.0, 1.0)

		# Starvation check
		if entity.hunger <= 0.0:
			emit_event("entity_starved", {
				"entity_id": entity.id,
				"entity_name": entity.entity_name,
				"tick": tick,
			})
			_entity_manager.kill_entity(entity.id, "starvation")
