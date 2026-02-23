extends "res://scripts/core/simulation_system.gd"
## StatSyncSystem: entity 필드 → stat_cache 동기화 브릿지.
## priority=1 — 매 tick 모든 시스템보다 먼저 실행.

var _entity_manager: RefCounted

func _init() -> void:
	system_name = "stat_sync"
	priority = 1
	tick_interval = 1


## entity_manager를 받아 초기화
func init(entity_manager: RefCounted) -> void:
	_entity_manager = entity_manager


func execute_tick(_tick: int) -> void:
	if _entity_manager == null:
		return
	var alive: Array = _entity_manager.get_alive_entities()
	for i in range(alive.size()):
		_sync_entity(alive[i])


func _sync_entity(entity: RefCounted) -> void:
	# Needs: float 0~1 → int 0~1000
	StatQuery.set_value(entity, &"NEED_HUNGER", int(entity.hunger * 1000), 0)
	StatQuery.set_value(entity, &"NEED_THIRST", int(entity.thirst * 1000), 0)
	StatQuery.set_value(entity, &"NEED_ENERGY", int(entity.energy * 1000), 0)
	StatQuery.set_value(entity, &"NEED_WARMTH", int(entity.warmth * 1000), 0)
	StatQuery.set_value(entity, &"NEED_SAFETY", int(entity.safety * 1000), 0)
	StatQuery.set_value(entity, &"NEED_SOCIAL", int(entity.social * 1000), 0)

	# HEXACO axes: float 0~1 → int 0~1000
	var pd = entity.personality
	if pd == null:
		return
	StatQuery.set_value(entity, &"HEXACO_H", int(pd.axes.get("H", 0.5) * 1000), 0)
	StatQuery.set_value(entity, &"HEXACO_E", int(pd.axes.get("E", 0.5) * 1000), 0)
	StatQuery.set_value(entity, &"HEXACO_X", int(pd.axes.get("X", 0.5) * 1000), 0)
	StatQuery.set_value(entity, &"HEXACO_A", int(pd.axes.get("A", 0.5) * 1000), 0)
	StatQuery.set_value(entity, &"HEXACO_C", int(pd.axes.get("C", 0.5) * 1000), 0)
	StatQuery.set_value(entity, &"HEXACO_O", int(pd.axes.get("O", 0.5) * 1000), 0)

	# Emotion meta stats
	var ed = entity.emotion_data
	if ed == null:
		return
	StatQuery.set_value(entity, &"EMOTION_STRESS",     int(ed.stress * 20.0), 0)
	StatQuery.set_value(entity, &"EMOTION_ALLOSTATIC",  int(ed.allostatic), 0)
	StatQuery.set_value(entity, &"EMOTION_RESERVE",     int(ed.reserve), 0)
