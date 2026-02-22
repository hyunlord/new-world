extends RefCounted

## StatEvaluatorRegistry: 커브로 표현할 수 없는 복잡한 로직 등록소.
##
## JSON의 "evaluator" 필드가 "CURVE"가 아닌 경우 이 레지스트리에서 실행.
## 등록은 main.gd _init_systems() 시점에 수행.
## Rust 전환 시: 이 파일을 GDExtension 바인딩으로 교체.
##
## Callable 시그니처:
##   func(entity: RefCounted, context: Dictionary) -> float

static var _evaluators: Dictionary = {}  # StringName → Callable

## evaluator 등록
static func register(id: StringName, fn: Callable) -> void:
	if _evaluators.has(id):
		push_warning("StatEvaluatorRegistry: overwriting evaluator: " + str(id))
	_evaluators[id] = fn

## evaluator 실행
## 없으면 push_error + 1.0 반환 (중립값)
static func evaluate(id: StringName, entity: RefCounted,
		context: Dictionary = {}) -> float:
	if not _evaluators.has(id):
		push_error("StatEvaluatorRegistry: unknown evaluator: " + str(id))
		return 1.0
	return float(_evaluators[id].call(entity, context))

static func has_evaluator(id: StringName) -> bool:
	return _evaluators.has(id)

static func get_all_ids() -> Array:
	return _evaluators.keys()
