extends RefCounted

## StatGraph 빌드 및 순환 감지 테스트
## StatQuery._ready()가 먼저 실행된 상태에서 호출해야 함

const StatGraphScript = preload("res://scripts/core/stat_graph.gd")
const StatDefinitionScript = preload("res://scripts/core/stat_definition.gd")

static func run_all() -> bool:
	var ok: bool = true
	ok = _test_build_succeeds() and ok
	ok = _test_eval_order_valid() and ok
	if ok:
		print("[test_stat_graph] ALL PASS")
	return ok

static func _test_build_succeeds() -> bool:
	# StatDefinition이 이미 로드됐다고 가정 (StatQuery._ready()에서 호출됨)
	# 또는 직접 로드
	if StatDefinitionScript.get_all_ids().is_empty():
		StatDefinitionScript.load_all("res://stats/")
	var result: bool = StatGraphScript.build()
	assert(result, "StatGraph.build() should succeed with no cycles")
	print("[PASS] build_succeeds")
	return result

static func _test_eval_order_valid() -> bool:
	var order: Array = StatGraphScript.get_eval_order()
	assert(order.size() > 0, "eval_order should not be empty")
	var seen: Dictionary = {}
	for sid in order:
		assert(not seen.has(sid),
			"duplicate in eval_order: " + str(sid))
		seen[sid] = true
	print("[PASS] eval_order_valid (" + str(order.size()) + " stats)")
	return true
