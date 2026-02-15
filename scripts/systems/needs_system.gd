class_name NeedsSystem
extends SimulationSystem

var _entity_manager: EntityManager


func _init() -> void:
	system_name = "needs"
	priority = 10
	tick_interval = 1


## Initialize with entity manager reference
func init(entity_manager: EntityManager) -> void:
	_entity_manager = entity_manager


func execute_tick(tick: int) -> void:
	var alive: Array[EntityData] = _entity_manager.get_alive_entities()
	for entity: EntityData in alive:
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
