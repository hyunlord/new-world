extends "res://scripts/core/simulation_system.gd"

var _entity_manager: RefCounted
var _mortality_system: RefCounted


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
		var hunger_mult: float = GameConfig.CHILD_HUNGER_DECAY_MULT.get(entity.age_stage, 1.0)
		entity.hunger -= GameConfig.HUNGER_DECAY_RATE * hunger_mult
		entity.energy -= GameConfig.ENERGY_DECAY_RATE
		entity.social -= GameConfig.SOCIAL_DECAY_RATE

		# Extra energy cost when performing actions
		if entity.current_action != "idle" and entity.current_action != "rest":
			entity.energy -= GameConfig.ENERGY_ACTION_COST

		# Age: derive from birth_tick (not incremental — avoids drift)
		entity.age = tick - entity.birth_tick

		# Auto-eat from inventory when hungry
		if entity.hunger < GameConfig.HUNGER_EAT_THRESHOLD:
			var food_in_inv: float = entity.inventory.get("food", 0.0)
			if food_in_inv > 0.0:
				var eat_amount: float = minf(food_in_inv, 2.0)
				entity.remove_item("food", eat_amount)
				entity.hunger += eat_amount * GameConfig.FOOD_HUNGER_RESTORE

		# Clamp all needs
		entity.hunger = clampf(entity.hunger, 0.0, 1.0)
		entity.energy = clampf(entity.energy, 0.0, 1.0)
		entity.social = clampf(entity.social, 0.0, 1.0)

		# Starvation check with grace period
		if entity.hunger <= 0.0:
			entity.starving_timer += 1
			if entity.starving_timer >= GameConfig.STARVATION_GRACE_TICKS:
				emit_event("entity_starved", {
					"entity_id": entity.id,
					"entity_name": entity.entity_name,
					"starving_ticks": entity.starving_timer,
					"tick": tick,
				})
				_entity_manager.kill_entity(entity.id, "starvation", tick)
				if _mortality_system != null and _mortality_system.has_method("register_death"):
					var age_years: float = GameConfig.get_age_years(entity.age)
					_mortality_system.register_death(age_years < 1.0)
		else:
			entity.starving_timer = 0
