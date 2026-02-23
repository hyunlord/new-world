extends "res://scripts/core/simulation_system.gd"
## StatThresholdSystem: stat JSON thresholds 배열을 평가하고 게임플레이에 반영.
## priority=12 — NeedsSystem(10) 이후, BehaviorSystem(20) 이전 실행.
## tick_interval=5 — 매 5tick 평가 (성능/정확도 트레이드오프).

const StatDefinitionScript = preload("res://scripts/core/stat_definition.gd")
const StatModifierScript = preload("res://scripts/core/stat_modifier.gd")

var _entity_manager: RefCounted
## entity_id → { "STAT_ID": ["EFFECT_1", "EFFECT_2"] }
## 현재 조건 진입 상태인 threshold 효과 목록
var _active_effects: Dictionary = {}


func _init() -> void:
	system_name = "stat_threshold"
	priority = 12
	tick_interval = 5


func init(entity_manager: RefCounted) -> void:
	_entity_manager = entity_manager


func execute_tick(tick: int) -> void:
	if _entity_manager == null:
		return
	var alive: Array = _entity_manager.get_alive_entities()
	for entity in alive:
		_evaluate_entity(entity, tick)


func _evaluate_entity(entity: RefCounted, tick: int) -> void:
	var eid: int = entity.id
	if not _active_effects.has(eid):
		_active_effects[eid] = {}

	var stat_ids: Array = StatDefinitionScript.get_all_ids()
	for stat_id in stat_ids:
		var stat_name: StringName = StringName(stat_id)
		var thresholds: Array = StatDefinitionScript.get_thresholds(stat_name)
		if thresholds.is_empty():
			continue

		var current_val: int = StatQuery.get_stat(entity, stat_name, -1)
		if current_val < 0:
			continue

		if not _active_effects[eid].has(stat_name):
			_active_effects[eid][stat_name] = []

		for thr in thresholds:
			var thr_val: int = int(thr.get("value", 0))
			var direction: String = thr.get("direction", "below")
			var effect: String = thr.get("effect", "")
			var hysteresis: int = int(thr.get("hysteresis", 0))
			if effect.is_empty():
				continue

			var currently_active: bool = _active_effects[eid][stat_name].has(effect)
			var should_be_active: bool = _check_threshold(
				current_val, thr_val, direction, hysteresis, currently_active
			)

			if should_be_active and not currently_active:
				_active_effects[eid][stat_name].append(effect)
				_apply_effect(entity, str(stat_name), effect, true, tick)
				SimulationBus.stat_threshold_crossed.emit(eid, str(stat_name), effect, "entered")
			elif not should_be_active and currently_active:
				_active_effects[eid][stat_name].erase(effect)
				_apply_effect(entity, str(stat_name), effect, false, tick)
				SimulationBus.stat_threshold_crossed.emit(eid, str(stat_name), effect, "exited")


func _check_threshold(
		val: int, thr: int, direction: String, hysteresis: int, currently_active: bool
) -> bool:
	match direction:
		"below":
			if currently_active:
				return val < thr + hysteresis
			return val < thr
		"above":
			if currently_active:
				return val > thr - hysteresis
			return val > thr
	return false


func _apply_effect(
		entity: RefCounted, stat_id: String, effect: String, entering: bool, tick: int
) -> void:
	if effect.begins_with("MODIFIER_"):
		_handle_modifier_effect(entity, stat_id, effect, entering, tick)
	elif effect.begins_with("STRESS_STATE_"):
		## stress_system이 이미 stress_state를 관리함. 시그널만 emit (위에서 완료).
		pass
	elif effect.begins_with("BEHAVIOR_"):
		## Phase 4에서 BehaviorSystem 연동. 현재는 시그널만 emit.
		pass
	elif effect.begins_with("ALLOSTATIC_"):
		## 미래 확장용. 시그널만 emit.
		pass
	elif effect.begins_with("RESERVE_"):
		## 미래 확장용. 시그널만 emit.
		pass
	## TRAIT_VALUE_* 등 미분류 효과는 시그널만 emit


func _handle_modifier_effect(
		entity: RefCounted, stat_id: String, effect: String, entering: bool, _tick: int
) -> void:
	## modifier_id = "threshold_{stat_id}_{effect}" 고유 식별자
	var modifier_id: StringName = StringName("threshold_%s_%s" % [stat_id, effect])

	if not entering:
		## 효과 해제 — 어떤 stat에서도 제거 시도
		StatQuery.remove_modifier(entity, StringName(stat_id), modifier_id)
		## NEED_HUNGER에도 제거 (MODIFIER_WORK_PENALTY_* 임시 타겟)
		StatQuery.remove_modifier(entity, &"NEED_HUNGER", modifier_id)
		return

	## effect별 modifier 값 설정
	var modifier_value: float = 0.0
	var target_stat: StringName

	match effect:
		"MODIFIER_WORK_PENALTY_MILD":
			modifier_value = -200.0
			target_stat = &"NEED_HUNGER"
		"MODIFIER_WORK_PENALTY_SEVERE":
			modifier_value = -500.0
			target_stat = &"NEED_HUNGER"
		_:
			return

	## StatModifier 객체 생성 (make_add 사용)
	var mod: RefCounted = StatModifierScript.make_add(
		modifier_id, target_stat, modifier_value, "threshold", -1
	)
	StatQuery.apply_modifier(entity, mod)
