extends "res://scripts/core/simulation/simulation_system.gd"
## StatThresholdSystem: stat JSON thresholds 배열을 평가하고 게임플레이에 반영.
## priority=12 — NeedsSystem(10) 이후, BehaviorSystem(20) 이전 실행.
## tick_interval=5 — 매 5tick 평가 (성능/정확도 트레이드오프).

const StatDefinitionScript = preload("res://scripts/core/stats/stat_definition.gd")
const StatModifierScript = preload("res://scripts/core/stats/stat_modifier.gd")
const _SIM_BRIDGE_NODE_NAME: String = "SimBridge"
const _SIM_BRIDGE_THRESHOLD_METHOD: String = "body_stat_threshold_is_active"

var _entity_manager: RefCounted
var _bridge_checked: bool = false
var _sim_bridge: Object = null
## entity_id → { "STAT_ID": ["EFFECT_1", "EFFECT_2"] }
## 현재 조건 진입 상태인 threshold 효과 목록
var _active_effects: Dictionary = {}


func _init() -> void:
	system_name = "stat_threshold"
	priority = 12
	tick_interval = 5


## Initializes the threshold system with the entity manager.
func init(entity_manager: RefCounted) -> void:
	_entity_manager = entity_manager


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
	if node != null and node.has_method(_SIM_BRIDGE_THRESHOLD_METHOD):
		_sim_bridge = node
	return _sim_bridge


## Evaluates all stat thresholds for every alive entity and applies or removes threshold effects.

func _evaluate_entity(entity: RefCounted, tick: int) -> void:
	var eid: int = entity.id
	if entity.is_alive == false:
		_active_effects.erase(eid)
		return
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
	var bridge: Object = _get_sim_bridge()
	if bridge != null:
		var direction_code: int = -1
		if direction == "below":
			direction_code = 0
		elif direction == "above":
			direction_code = 1
		if direction_code >= 0:
			var rust_variant: Variant = bridge.call(
				_SIM_BRIDGE_THRESHOLD_METHOD,
				val,
				thr,
				direction_code,
				hysteresis,
				currently_active,
			)
			if rust_variant != null:
				return bool(rust_variant)
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
	elif effect.begins_with("UNLOCK_ACTION_"):
		_handle_unlock_action_effect(entity, stat_id, effect, entering, tick)
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


## [Anderson 1982 ACT*] Grant action unlock based on threshold state.
## entering=true  → add action to entity.unlocked_actions (permanent grant)
## entering=false → threshold dropped below; do NOT revoke (knowledge persists)
func _handle_unlock_action_effect(
		entity: RefCounted, stat_id: String, effect: String,
		entering: bool, tick: int) -> void:
	if not entering:
		## Deliberately ignored — actions are not revoked on level drop.
		## Rationale: Anderson (1982) ACT* — procedural knowledge is not erased
		## by temporary stat decline (injury, aging).
		return

	var action_id: StringName = StringName(effect)
	## Idempotent: if already unlocked, skip signal and return
	if entity.unlocked_actions.get(action_id, false):
		return

	entity.unlocked_actions[action_id] = true

	## Emit bus signal so UI / chronicle / future teaching system can react
	SimulationBus.skill_action_unlocked.emit(
		entity.id,
		entity.entity_name,
		action_id,
		StringName(stat_id),
		StatQuery.get_stat(entity, StringName(stat_id), 0),
		tick
	)

	## Fire toast notification
	var toast_key: String = "SKILL_UNLOCK_%s" % effect
	SimulationBus.ui_notification.emit(
		Locale.ltr(toast_key).format({"name": entity.entity_name}),
		"skill_unlock"
	)
