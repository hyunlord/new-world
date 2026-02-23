extends RefCounted

## StatGraph: 스탯 의존성 그래프.
## - StatDefinition.affects로부터 그래프 구축
## - 순환 의존성 감지 (같은 Tier 내)
## - Topological sort → eval_order 생성
## StatQuery 초기화 시 build() 호출.

const StatDefinitionScript = preload("res://scripts/core/stat_definition.gd")

## 의존성 그래프. key=stat_id, value=Array[stat_id]
static var _dependents: Dictionary = {}  # A → [B, C]: A 변하면 B, C dirty
static var _dependencies: Dictionary = {} # A → [B, C]: A는 B, C에 의존
static var _eval_order: Array = []
static var _built: bool = false

## 그래프 구축. StatDefinition.load_all() 이후 호출.
static func build() -> bool:
	_dependents.clear()
	_dependencies.clear()
	_eval_order.clear()
	_built = false

	var all_ids: Array = StatDefinitionScript.get_all_ids()

	# 모든 노드 초기화
	for sid in all_ids:
		_dependents[sid] = []
		_dependencies[sid] = []

	# affects로부터 엣지 구성
	for sid in all_ids:
		var affects: Array = StatDefinitionScript.get_affects(sid)
		for affect in affects:
			var evaluator: String = affect.get("evaluator", "CURVE")
			if evaluator != "CURVE":
				continue  # Evaluator 타입은 그래프 엣지 제외
			var target: StringName = StringName(affect.get("target", ""))
			if target == &"" or not StatDefinitionScript.has_def(target):
				continue
			if not _dependents.has(sid):
				_dependents[sid] = []
			if not _dependencies.has(target):
				_dependencies[target] = []
			if not _dependents[sid].has(target):
				_dependents[sid].append(target)
			if not _dependencies[target].has(sid):
				_dependencies[target].append(sid)

	# Topological sort (Kahn's algorithm)
	var in_degree: Dictionary = {}
	for sid in all_ids:
		in_degree[sid] = (_dependencies[sid] as Array).size()

	var queue: Array = []
	for sid in all_ids:
		if in_degree[sid] == 0:
			queue.append(sid)

	while queue.size() > 0:
		queue.sort()  # 결정론적 순서
		var node: StringName = queue.pop_front()
		_eval_order.append(node)
		for dep in _dependents.get(node, []):
			in_degree[dep] -= 1
			if in_degree[dep] == 0:
				queue.append(dep)

	# 순환 감지
	if _eval_order.size() != all_ids.size():
		var remaining: Array = []
		for sid in all_ids:
			if not _eval_order.has(sid):
				remaining.append(str(sid))
		push_error("StatGraph: cycle detected among: " + ", ".join(remaining))
		return false

	_built = true
	return true

## A가 변할 때 dirty 마킹이 필요한 스탯들 반환
static func get_dependents(stat_id: StringName) -> Array:
	return _dependents.get(stat_id, [])

## 계산 순서 반환
static func get_eval_order() -> Array:
	return _eval_order

## 정의 조회 proxy
static func get_definition(stat_id: StringName) -> Dictionary:
	return StatDefinitionScript.get_def(stat_id)

static func get_affects(stat_id: StringName,
		context: StringName = &"") -> Array:
	return StatDefinitionScript.get_affects(stat_id, context)
