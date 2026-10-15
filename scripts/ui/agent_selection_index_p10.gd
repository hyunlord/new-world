extends RefCounted
## Snapshot ECS entity bits map to agent IDs; the renderer owns the sprite map.
var _agents_by_entity: Dictionary = {}

func begin_frame() -> void:
	_agents_by_entity.clear()

func observe(entity_bits: int, agent_id: int) -> void:
	_agents_by_entity[entity_bits] = agent_id

func find(sprites: Dictionary, entity_bits: int) -> Sprite2D:
	if not _agents_by_entity.has(entity_bits):
		return null
	var candidate: Variant = sprites.get(_agents_by_entity[entity_bits])
	if not is_instance_valid(candidate) or not candidate is Sprite2D:
		return null
	var sprite: Sprite2D = candidate
	if not sprite.visible:
		return null
	return sprite
