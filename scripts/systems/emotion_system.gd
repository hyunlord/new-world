extends "res://scripts/core/simulation_system.gd"

## Updates emotion values daily based on needs, proximity, and personality.
## Runs once per day (12 ticks).

var _entity_manager: RefCounted


func _init() -> void:
	system_name = "emotions"
	priority = 32
	tick_interval = 12  # Once per game day


func init(entity_manager: RefCounted) -> void:
	_entity_manager = entity_manager


func execute_tick(_tick: int) -> void:
	var alive: Array = _entity_manager.get_alive_entities()
	for i in range(alive.size()):
		var entity: RefCounted = alive[i]
		_update_happiness(entity)
		_update_loneliness(entity)
		_update_stress(entity)
		_update_grief(entity)
		_update_love(entity)


func _update_happiness(entity: RefCounted) -> void:
	var target: float = (entity.hunger + entity.energy + entity.social) / 3.0
	entity.emotions["happiness"] = lerpf(entity.emotions.get("happiness", 0.5), target, 0.1)


func _update_loneliness(entity: RefCounted) -> void:
	var loneliness: float = entity.emotions.get("loneliness", 0.0)
	if entity.social < 0.3:
		loneliness += 0.02
	# Check family/partner nearby (within 3 tiles)
	var has_close_family: bool = false
	if entity.partner_id != -1:
		var partner: RefCounted = _entity_manager.get_entity(entity.partner_id)
		if partner != null and partner.is_alive:
			var dx: int = absi(entity.position.x - partner.position.x)
			var dy: int = absi(entity.position.y - partner.position.y)
			if dx <= 3 and dy <= 3:
				has_close_family = true
	if not has_close_family:
		for j in range(entity.parent_ids.size()):
			var parent: RefCounted = _entity_manager.get_entity(entity.parent_ids[j])
			if parent != null and parent.is_alive:
				var dx: int = absi(entity.position.x - parent.position.x)
				var dy: int = absi(entity.position.y - parent.position.y)
				if dx <= 3 and dy <= 3:
					has_close_family = true
					break
	if has_close_family:
		loneliness -= 0.05
	entity.emotions["loneliness"] = clampf(loneliness, 0.0, 1.0)


func _update_stress(entity: RefCounted) -> void:
	var stress: float = entity.emotions.get("stress", 0.0)
	var stability: float = 1.0 - entity.personality.axes.get("E", 0.5)
	if entity.hunger < 0.2:
		stress += 0.03
	else:
		stress -= 0.01 * stability
	entity.emotions["stress"] = clampf(stress, 0.0, 1.0)


func _update_grief(entity: RefCounted) -> void:
	var grief: float = entity.emotions.get("grief", 0.0)
	var stability: float = 1.0 - entity.personality.axes.get("E", 0.5)
	grief -= 0.002 * stability
	entity.emotions["grief"] = clampf(grief, 0.0, 1.0)


func _update_love(entity: RefCounted) -> void:
	var love: float = entity.emotions.get("love", 0.0)
	if entity.partner_id != -1:
		var partner: RefCounted = _entity_manager.get_entity(entity.partner_id)
		if partner != null and partner.is_alive:
			var dx: int = absi(entity.position.x - partner.position.x)
			var dy: int = absi(entity.position.y - partner.position.y)
			if dx <= 3 and dy <= 3:
				love += 0.03
			else:
				love -= 0.01
		else:
			love -= 0.01
	else:
		love -= 0.01
	entity.emotions["love"] = clampf(love, 0.0, 1.0)
