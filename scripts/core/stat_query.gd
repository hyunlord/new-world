extends Node
## StatQuery: 모든 시스템의 단일 스탯 접점 API.
## Autoload 이름: StatQuery
##
## Phase 0: 인프라 stub. 기존 entity 필드에서 직접 읽어 반환.
##          기존 시스템들은 변경 없이 동작.
## Phase 2: 캐시/커브/modifier 완전 활성화.

const StatDefinitionScript = preload("res://scripts/core/stat_definition.gd")
const StatGraphScript = preload("res://scripts/core/stat_graph.gd")
const StatCacheScript = preload("res://scripts/core/stat_cache.gd")
const StatCurveScript = preload("res://scripts/core/stat_curve.gd")
const StatEvaluatorRegistryScript = preload("res://scripts/core/stat_evaluator_registry.gd")
const StatModifierScript = preload("res://scripts/core/stat_modifier.gd")

const PHASE: int = 2  ## 현재 구현 Phase.

func _ready() -> void:
	StatDefinitionScript.load_all("res://stats/")
	var ok: bool = StatGraphScript.build()
	if not ok:
		push_error("StatQuery: StatGraph build failed — check for cycles")

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# READ API
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## 현재 스탯 값 반환 (int)
## Phase 0: entity.stat_cache에 있으면 그것, 없으면 fallback
func get_stat(entity: RefCounted, stat_id: StringName,
		fallback: int = 0) -> int:
	if entity == null:
		return fallback
	if not StatDefinitionScript.has_def(stat_id):
		return fallback
	var cache = entity.get("stat_cache")
	if cache == null or not cache is Dictionary:
		return fallback
	var cache_dict: Dictionary = cache as Dictionary
	if cache_dict.has(stat_id) and not bool(cache_dict[stat_id].get("dirty", true)):
		return int(cache_dict[stat_id].get("value", fallback))
	return fallback

## 정규화 값 반환 (0.0~1.0)
func get_normalized(entity: RefCounted, stat_id: StringName) -> float:
	var range_arr: Array = StatDefinitionScript.get_range(stat_id)
	var rmin: int = range_arr[0] if range_arr.size() > 0 else 0
	var rmax: int = range_arr[1] if range_arr.size() > 1 else 1000
	if rmax == rmin:
		return 0.0
	return float(get_stat(entity, stat_id, rmin) - rmin) / float(rmax - rmin)

## 영향력 값 반환 (InfluenceCurve 적용된 float)
## Phase 0: stub, 항상 1.0 반환
func get_influence(entity: RefCounted, stat_id: StringName,
		context: StringName, extra_context: Dictionary = {}) -> float:
	if entity == null:
		return 1.0
	var affects: Array = StatDefinitionScript.get_affects(stat_id, context)
	if affects.is_empty():
		return 1.0

	var total: float = 1.0
	var val: int = get_stat(entity, stat_id)

	for affect in affects:
		if not _check_condition(entity, affect.get("condition", {}), extra_context):
			continue
		var evaluator: String = affect.get("evaluator", "CURVE")
		var influence: float
		if evaluator == "CURVE":
			influence = StatCurveScript.apply(val, affect)
		else:
			var ctx: Dictionary = extra_context.duplicate()
			ctx["source_stat"] = stat_id
			ctx["source_value"] = val
			influence = StatEvaluatorRegistryScript.evaluate(
				StringName(evaluator), entity, ctx)
		var weight: float = float(affect.get("weight", 1.0))
		total *= (1.0 + (influence - 1.0) * weight)

	return total

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# WRITE API
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## 스탯 값 직접 설정 (초기화/이벤트)
func set_value(entity: RefCounted, stat_id: StringName,
		value: int, tick: int = 0) -> void:
	if entity == null:
		return
	var range_arr: Array = StatDefinitionScript.get_range(stat_id)
	var clamped: int = clampi(value,
		range_arr[0] if range_arr.size() > 0 else 0,
		range_arr[1] if range_arr.size() > 1 else 9999)
	var cache = entity.get("stat_cache")
	if cache == null or not cache is Dictionary:
		return
	StatCacheScript.set_value(cache as Dictionary, stat_id, clamped, tick)

## StatModifier 추가
func apply_modifier(entity: RefCounted, modifier: RefCounted) -> void:
	if entity == null or modifier == null:
		return
	var cache = entity.get("stat_cache")
	if cache == null or not cache is Dictionary:
		return
	StatCacheScript.add_modifier(cache as Dictionary, modifier)

## StatModifier 제거
func remove_modifier(entity: RefCounted, stat_id: StringName,
		modifier_id: StringName) -> void:
	if entity == null:
		return
	var cache = entity.get("stat_cache")
	if cache == null or not cache is Dictionary:
		return
	StatCacheScript.remove_modifier(cache as Dictionary, stat_id, modifier_id)

## 스킬/훈련 XP 추가
## 반환: {"leveled_up": bool, "old_level": int, "new_level": int}
## References: Newell & Rosenbloom (1981) Power Law of Practice
##             Ericsson (1993) deliberate practice — talent ceiling
func add_xp(entity: RefCounted, stat_id: StringName,
		xp: float) -> Dictionary:
	var result: Dictionary = {
		"leveled_up": false, "old_level": 0, "new_level": 0}
	if entity == null or xp <= 0.0:
		return result

	var def: Dictionary = StatDefinitionScript.get_def(stat_id)
	if def.is_empty():
		return result

	var growth: Dictionary = def.get("growth", {})
	if growth.get("type", "") != "LOG_DIMINISHING":
		return result  ## Only handle LOG_DIMINISHING for now

	## Accumulate XP
	var current_xp: float = float(entity.skill_xp.get(stat_id, 0.0))
	var old_level: int = int(entity.skill_levels.get(stat_id, 0))
	current_xp += xp
	entity.skill_xp[stat_id] = current_xp

	## Compute talent ceiling from body trainability
	var max_level: int = _compute_talent_ceiling(entity, def)

	## Compute new level from total XP
	var new_level: int = _compute_level_from_xp(current_xp, growth, max_level)

	result["old_level"] = old_level
	result["new_level"] = new_level

	if new_level != old_level:
		entity.skill_levels[stat_id] = new_level
		result["leveled_up"] = (new_level > old_level)
		## Sync to stat_cache immediately so other systems see the new level
		set_value(entity, stat_id, new_level, 0)

	return result


## Compute level from cumulative XP using LOG_DIMINISHING curve.
## XP required to reach level L from level L-1:
##   xp_per_level(L) = base_xp * L^exponent * breakpoint_multiplier(L)
## Total XP to reach level L = sum_{i=1}^{L} xp_per_level(i)
func _compute_level_from_xp(total_xp: float, growth: Dictionary, max_level: int) -> int:
	var params: Dictionary = growth.get("params", {})
	var base_xp: float = float(params.get("base_xp", 100))
	var exponent: float = float(params.get("exponent", 1.8))
	var breakpoints: Array = params.get("level_breakpoints", [])
	var multipliers: Array = params.get("breakpoint_multipliers", [1.0])

	var cumulative_xp: float = 0.0
	var level: int = 0

	for l in range(1, max_level + 1):
		var mult: float = _get_breakpoint_multiplier(l, breakpoints, multipliers)
		var xp_this_level: float = base_xp * pow(float(l), exponent) * mult
		if cumulative_xp + xp_this_level > total_xp:
			break
		cumulative_xp += xp_this_level
		level = l

	return clampi(level, 0, max_level)


## Get breakpoint multiplier for a given level.
## breakpoints: [25, 50, 75], multipliers: [1.0, 1.5, 2.0, 3.0]
## Level < 25 → 1.0, Level 25-49 → 1.5, Level 50-74 → 2.0, Level 75+ → 3.0
func _get_breakpoint_multiplier(level: int, breakpoints: Array, multipliers: Array) -> float:
	var mult_idx: int = 0
	for bp in breakpoints:
		if level >= int(bp):
			mult_idx += 1
		else:
			break
	if multipliers.is_empty():
		return 1.0
	return float(multipliers[clampi(mult_idx, 0, multipliers.size() - 1)])


## Compute the talent ceiling (max achievable level) for this entity+skill.
## talent_ceiling_map: {"0": 40, "200": 60, "400": 80, "600": 90, "800": 100}
## Finds the highest threshold where entity's trainability >= threshold → returns that ceiling.
func _compute_talent_ceiling(entity: RefCounted, def: Dictionary) -> int:
	var growth: Dictionary = def.get("growth", {})
	var talent_key: String = growth.get("talent_key", "")
	var ceiling_map: Dictionary = growth.get("talent_ceiling_map", {})
	var range_arr: Array = def.get("range", [0, 100])
	var hard_max: int = range_arr[1] if range_arr.size() > 1 else 100

	if talent_key == "" or ceiling_map.is_empty():
		return hard_max

	## Read trainability from stat_cache (e.g. BODY_AGI_TRAINABILITY → 0–1000)
	var trainability: int = get_stat(entity, StringName(talent_key), 500)

	## Find the highest threshold ≤ trainability
	var best_ceiling: int = 40  ## minimum ceiling even at 0 trainability
	var sorted_thresholds: Array = []
	for k in ceiling_map:
		sorted_thresholds.append(int(k))
	sorted_thresholds.sort()

	for thresh in sorted_thresholds:
		if trainability >= thresh:
			best_ceiling = int(ceiling_map[str(thresh)])
		else:
			break

	return clampi(best_ceiling, 0, hard_max)


# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# DEBUG API
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## 스탯이 왜 이 값인지 전체 추적
func explain(entity: RefCounted, stat_id: StringName) -> Dictionary:
	if entity == null:
		return {}
	var cache = entity.get("stat_cache")
	var entry: Dictionary = {}
	if cache != null and cache is Dictionary:
		entry = (cache as Dictionary).get(stat_id, {})
	var def: Dictionary = StatDefinitionScript.get_def(stat_id)

	var mod_breakdown: Array = []
	for m in entry.get("modifiers", []):
		mod_breakdown.append({
			"source": m.get("source", ""),
			"type": m.get("mod_type", 0),
			"value": m.get("value", 0.0),
			"duration": m.get("duration", -1)
		})

	return {
		"stat_id": str(stat_id),
		"current_value": entry.get("value", 0),
		"dirty": entry.get("dirty", true),
		"last_computed_tick": entry.get("last_computed_tick", 0),
		"modifiers": mod_breakdown,
		"range": def.get("range", [0, 1000]),
		"tier": def.get("tier", 2)
	}

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# INTERNAL
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

func _check_condition(entity: RefCounted, condition: Dictionary,
		extra: Dictionary) -> bool:
	if condition.is_empty():
		return true
	var requires_states: Array = condition.get("requires_states", [])
	for state in requires_states:
		if not extra.get(state, false):
			return false
	var gte: Dictionary = condition.get("requires_stat_gte", {})
	for sid in gte:
		if get_stat(entity, StringName(sid)) < int(gte[sid]):
			return false
	return true
