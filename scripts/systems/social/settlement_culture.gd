## [Axelrod 1997, Haidt 2012] 정착지 공유 가치관 및 문화 동조 압력 관리
## 참조: const SettlementCulture = preload("res://scripts/systems/settlement_culture.gd")
extends RefCounted

const ValueDefs = preload("res://scripts/core/social/value_defs.gd")
const _SIM_BRIDGE_NODE_NAME: String = "SimBridge"
const _SIM_BRIDGE_VALUE_PLASTICITY_METHOD: String = "body_value_plasticity"

## 정착지 리더 가중치 (리더의 가치관이 문화에 주는 추가 영향)
const LEADER_INFLUENCE: float = 0.20
## 문화 이탈 임계값 — 이 이상 벗어난 가치관만 동조 압력 적용
const DEVIATION_THRESHOLD: float = 0.40
## 동조 압력 tick당 드리프트량
const CONFORMITY_DRIFT_RATE: float = 0.003
static var _bridge_checked_static: bool = false
static var _sim_bridge_static: Object = null


## 구성원 가치관 평균 + 리더 보정으로 shared_values 계산
## members: Array of entity_data objects (each has .values Dictionary)
## leader: entity_data or null
static func compute_shared_values(members: Array, leader = null) -> Dictionary:
	if members.is_empty():
		return _neutral()

	var sum: Dictionary = {}
	for vkey in ValueDefs.KEYS:
		sum[vkey] = 0.0

	var count: float = float(members.size())
	for member in members:
		for vkey in ValueDefs.KEYS:
			sum[vkey] += member.values.get(vkey, 0.0)

	var avg: Dictionary = {}
	for vkey in ValueDefs.KEYS:
		avg[vkey] = sum[vkey] / count

	# 리더 보정: 구성원 평균과 리더 가치관을 가중 합산
	if leader != null:
		var member_w: float = 1.0 - LEADER_INFLUENCE
		for vkey in ValueDefs.KEYS:
			avg[vkey] = avg[vkey] * member_w \
				+ leader.values.get(vkey, 0.0) * LEADER_INFLUENCE

	return avg


## 문화 동조 압력 적용
## 문화와 크게 다른 가치관을 서서히 문화 방향으로 끌어당김
## 반환: 이 tick에 발생한 스트레스량 (0.0이면 스트레스 없음)
static func apply_conformity_pressure(
	entity_data,
	culture_values: Dictionary,
	enforcement_strength: float,
	age_years: float,
) -> float:
	var plasticity: float = _compute_plasticity(age_years)
	var total_deviation: float = 0.0

	for vkey in ValueDefs.KEYS:
		var my_val: float = entity_data.values.get(vkey, 0.0)
		var cult_val: float = culture_values.get(vkey, 0.0)
		var dev: float = absf(my_val - cult_val)

		if dev > DEVIATION_THRESHOLD:
			total_deviation += dev
			var drift: float = (cult_val - my_val) \
				* enforcement_strength * plasticity * CONFORMITY_DRIFT_RATE
			entity_data.values[vkey] = clampf(my_val + drift, -1.0, 1.0)

	# 이탈도 높을수록 사회적 스트레스 발생
	if total_deviation > 1.0:
		return total_deviation * enforcement_strength * 3.0
	return 0.0


static func _compute_plasticity(age_years: float) -> float:
	var bridge: Object = _get_sim_bridge_static()
	if bridge != null:
		var rust_variant: Variant = bridge.call(_SIM_BRIDGE_VALUE_PLASTICITY_METHOD, age_years)
		if rust_variant != null:
			return clampf(float(rust_variant), 0.10, 1.0)
	if age_years < 7.0:
		return 1.0
	elif age_years < 15.0:
		return lerpf(1.0, 0.70, (age_years - 7.0) / 8.0)
	elif age_years < 25.0:
		return lerpf(0.70, 0.30, (age_years - 15.0) / 10.0)
	elif age_years < 50.0:
		return lerpf(0.30, 0.10, (age_years - 25.0) / 25.0)
	return 0.10


static func _get_sim_bridge_static() -> Object:
	if _bridge_checked_static:
		return _sim_bridge_static
	_bridge_checked_static = true
	var tree: SceneTree = Engine.get_main_loop() as SceneTree
	if tree == null:
		return null
	var root: Node = tree.get_root()
	if root == null:
		return null
	var node: Node = root.get_node_or_null(_SIM_BRIDGE_NODE_NAME)
	if node != null and node.has_method(_SIM_BRIDGE_VALUE_PLASTICITY_METHOD):
		_sim_bridge_static = node
	return _sim_bridge_static


## 전체 가치관이 0.0인 중립 딕셔너리 반환 (내부 헬퍼)
static func _neutral() -> Dictionary:
	var v: Dictionary = {}
	for vkey in ValueDefs.KEYS:
		v[vkey] = 0.0
	return v
