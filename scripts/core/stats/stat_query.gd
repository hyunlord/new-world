extends Node
## StatQuery: 모든 시스템의 단일 스탯 접점 API.
## Autoload 이름: StatQuery
##
## Phase 0: 인프라 stub. 기존 entity 필드에서 직접 읽어 반환.
##          기존 시스템들은 변경 없이 동작.
## Phase 2: 캐시/커브/modifier 완전 활성화.

const StatDefinitionScript = preload("res://scripts/core/stats/stat_definition.gd")
const _TraitEffectCache = preload("res://scripts/systems/psychology/trait_effect_cache.gd")
const StatGraphScript = preload("res://scripts/core/stats/stat_graph.gd")
const StatCacheScript = preload("res://scripts/core/stats/stat_cache.gd")
const StatCurveScript = preload("res://scripts/core/stats/stat_curve.gd")
const StatEvaluatorRegistryScript = preload("res://scripts/core/stats/stat_evaluator_registry.gd")
const StatModifierScript = preload("res://scripts/core/stats/stat_modifier.gd")

const PHASE: int = 2  ## 현재 구현 Phase.

var _affinity_cache: Dictionary = {}

## [Human Definition v3 §11] Tech era ordering for required_tech gate
## Must be kept in sync with SettlementData.tech_era progression.
## If eras are added to the tech tree, append them here in order.
const ERA_ORDER = ["stone_age", "tribal", "bronze_age", "iron_age"]

## Settlement manager reference — set via init_settlement_manager() after scene is ready
var _settlement_manager = null

## Called by main.gd (or SimulationEngine) after all autoloads are ready.
## Stores the SettlementManager reference used for tech-era and prerequisite gating in add_xp.
func init_settlement_manager(mgr) -> void:
	_settlement_manager = mgr

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
## Returns the current cached stat value for the entity, or fallback if missing or dirty.
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
## Returns the stat value normalized to [0.0, 1.0] based on its defined min/max range.
func get_normalized(entity: RefCounted, stat_id: StringName) -> float:
	var range_arr: Array = StatDefinitionScript.get_range(stat_id)
	var rmin: int = range_arr[0] if range_arr.size() > 0 else 0
	var rmax: int = range_arr[1] if range_arr.size() > 1 else 1000
	if rmax == rmin:
		return 0.0
	return float(get_stat(entity, stat_id, rmin) - rmin) / float(rmax - rmin)

## 영향력 값 반환 (InfluenceCurve 적용된 float)
## Phase 0: stub, 항상 1.0 반환
## Returns the multiplicative influence of a stat on a given context, applying curve and evaluator affects.
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

## Compute skill → gameplay multiplier from JSON affects[] for the given context.
##
## Correctly handles:
##   - Range-agnostic evaluation via get_normalized() (0.0–1.0)
##   - direction:"positive" → multiplier > 1.0 (bonus above baseline)
##   - direction:"negative" → multiplier < 1.0 (penalty below baseline)
##   - Multiple affects in same skill — multiplicative chaining
##
## Why NOT use get_influence():
##   get_influence() uses raw stat value / 1000 (StatCurveScript.apply), wrong for skill range [0,100].
##   At level 100: (100/1000)^1.3 = 0.063 → produces a 66% penalty instead of bonus.
##   This function uses get_normalized() which correctly maps [0,100] to [0.0,1.0].
##
## References: Newell & Rosenbloom (1981) Power Law of Practice,
##             Mincer (1958) returns to human capital
##
## Example result for SKILL_FORAGING (exponent=1.3, weight=0.7, direction=positive):
##   level  0 → 1.000× |  level 25 → 1.116× |  level 50 → 1.284×
##   level 75 → 1.482× |  level 100 → 1.700×
func get_skill_multiplier(entity: RefCounted, skill_id: StringName,
		context: StringName = &"") -> float:
	if entity == null:
		return 1.0
	var affects: Array = StatDefinitionScript.get_affects(skill_id, context)
	if affects.is_empty():
		return 1.0

	## get_normalized handles range [0,100] → 0.0~1.0 correctly
	var norm: float = get_normalized(entity, skill_id)
	var total: float = 1.0

	for aff in affects:
		var evaluator: String = aff.get("evaluator", "CURVE")
		if evaluator != "CURVE":
			continue  ## Only CURVE evaluator supported here
		var curve: String = aff.get("curve", "POWER")
		if curve != "POWER":
			continue  ## Only POWER curve supported here; extend later if needed
		var exponent: float = float(aff.get("params", {}).get("exponent", 1.3))
		var weight: float = float(aff.get("weight", 1.0))
		var direction: String = aff.get("direction", "positive")

		## POWER influence on normalized value: norm^exponent
		## norm=0.0 → 0.0, norm=1.0 → 1.0
		var curve_val: float = pow(norm, exponent)

		if direction == "positive":
			## Add bonus proportional to skill level
			total += curve_val * weight
		elif direction == "negative":
			## Apply penalty proportional to skill level
			total -= curve_val * weight

	# v3 trait effect: skill/mult (all_work × all_learning × specific skill)
	var skill_name: String = str(skill_id).replace("SKILL_", "").to_lower()
	total *= _TraitEffectCache.get_skill_mult(entity, skill_name)

	return maxf(total, 0.01)  ## Never multiply to zero or negative

## Returns XP progress info for a skill with LOG_DIMINISHING growth.
##
## Used by EntityDetailPanel to display:
##   - Current level and talent ceiling
##   - XP progress within current level (e.g. "28 / 722 XP")
##
## Returns empty Dictionary if the skill has no LOG_DIMINISHING definition.
func get_skill_xp_info(entity: RefCounted, skill_id: StringName) -> Dictionary:
	if entity == null:
		return {}
	var def: Dictionary = StatDefinitionScript.get_def(skill_id)
	if def.is_empty():
		return {}
	var growth: Dictionary = def.get("growth", {})
	if growth.get("type", "") != "LOG_DIMINISHING":
		return {}

	var current_xp: float = float(entity.skill_xp.get(skill_id, 0.0))
	var level: int = int(entity.skill_levels.get(skill_id, 0))
	var max_level: int = _compute_talent_ceiling(entity, def)

	## Compute cumulative XP at the start of current level
	## (sum of XP required for all levels up to but not including current)
	var params: Dictionary = growth.get("params", {})

	var xp_at_level: float = 0.0
	for l in range(1, level + 1):
		xp_at_level += StatCurveScript.log_xp_required(l, params)

	## XP needed to complete current level (reach level + 1)
	var xp_to_next: float = 0.0
	if level < max_level:
		xp_to_next = StatCurveScript.log_xp_required(level + 1, params)
	## If at max level, xp_to_next = 0 (no further progress possible)

	return {
		"level": level,
		"max_level": max_level,
		"current_xp": current_xp,
		"xp_at_level": xp_at_level,
		"xp_to_next": xp_to_next,
		"progress_in_level": current_xp - xp_at_level,
	}

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# WRITE API
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## 스탯 값 직접 설정 (초기화/이벤트)
## Clamps value to the stat's defined range and writes it directly into the entity's stat cache.
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
## Adds a StatModifier to the entity's stat cache, replacing or stack-contesting as needed.
func apply_modifier(entity: RefCounted, modifier: RefCounted) -> void:
	if entity == null or modifier == null:
		return
	var cache = entity.get("stat_cache")
	if cache == null or not cache is Dictionary:
		return
	StatCacheScript.add_modifier(cache as Dictionary, modifier)

## StatModifier 제거
## Removes the modifier with the given modifier_id from the entity's cache entry for stat_id.
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

	## [Human Definition v3 §11] Tech era gate — XP blocked if settlement era < required_tech
	## Unresolvable settlement (nomads, settlement_id=0) blocks conservatively —
	## an unsettled entity is never more privileged than a settled one.
	var req_tech: String = def.get("required_tech", "")
	if req_tech != "":
		if _settlement_manager == null:
			return result  ## manager not ready — block conservatively
		var settlement = _settlement_manager.get_settlement(entity.settlement_id)
		if settlement == null:
			return result  ## no settlement resolved — block conservatively
		var req_idx: int = ERA_ORDER.find(req_tech)
		var cur_idx: int = ERA_ORDER.find(settlement.tech_era)
		if req_idx >= 0 and cur_idx >= 0 and cur_idx < req_idx:
			return result

	## [Human Definition v3 §11] Prerequisites gate — XP blocked if prerequisite skill level too low
	var prereqs: Array = def.get("prerequisites", [])
	for prereq in prereqs:
		var prereq_id: StringName = StringName(prereq.get("skill_id", ""))
		var min_level: int = int(prereq.get("min_level", 0))
		if prereq_id != &"" and entity.skill_levels.get(prereq_id, 0) < min_level:
			return result

	## Accumulate XP
	var current_xp: float = float(entity.skill_xp.get(stat_id, 0.0))
	var old_level: int = int(entity.skill_levels.get(stat_id, 0))
	var _intel_mult: float = _compute_intel_xp_mult(entity, def)
	current_xp += xp * _intel_mult
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
	var level: int = StatCurveScript.xp_to_level(total_xp, params, max_level)
	return clampi(level, 0, max_level)


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
## Returns a debug Dictionary describing the current value, dirty state, modifiers, and definition metadata for a stat.
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


## ── Intelligence-based XP multiplier [Gardner, CHC, Lupien] ────────
func _compute_intel_xp_mult(entity: RefCounted, def: Dictionary) -> float:
	if entity.intelligences.is_empty():
		return 1.0
	var category: String = def.get("category", "")
	var intel_mult: float = 1.0
	if category != "":
		var affinity: Dictionary = _get_affinity_for_category(category)
		if not affinity.is_empty():
			var score: float = 0.0
			for key in affinity:
				score += entity.intelligences.get(key, 0.5) * affinity[key]
			var centered: float = score - 0.5
			intel_mult = 1.0 + GameConfig.INTEL_LEARN_MULT_M * tanh(GameConfig.INTEL_LEARN_MULT_K * centered)
	## Conscientiousness effort [Chamorro-Premuzic 2004]
	var c_mult: float = 1.0
	if entity.personality != null and not entity.personality.facets.is_empty():
		var facets: Dictionary = entity.personality.facets
		var c_mean: float = (
			float(facets.get("C_organization", 0.5))
			+ float(facets.get("C_diligence", 0.5))
			+ float(facets.get("C_perfectionism", 0.5))
			+ float(facets.get("C_prudence", 0.5))
		) / 4.0
		c_mult = 1.0 + GameConfig.INTEL_CONSCIENTIOUSNESS_WEIGHT * (c_mean - 0.5)
	## Stress penalty [Lupien 2009]
	var stress_mult: float = 1.0
	var allostatic: float = get_normalized(entity, &"EMOTION_ALLOSTATIC")
	if allostatic > GameConfig.INTEL_STRESS_LEARNING_THRESHOLD_HIGH:
		stress_mult = GameConfig.INTEL_STRESS_LEARNING_PENALTY_HIGH
	elif allostatic > GameConfig.INTEL_STRESS_LEARNING_THRESHOLD_LOW:
		stress_mult = GameConfig.INTEL_STRESS_LEARNING_PENALTY_LOW
	return intel_mult * c_mult * stress_mult


func _get_affinity_for_category(category: String) -> Dictionary:
	if _affinity_cache.has(category):
		return _affinity_cache[category]
	var file = FileAccess.open("res://data/intelligence/affinity_defaults.json", FileAccess.READ)
	if file == null:
		return {}
	var json = JSON.new()
	if json.parse(file.get_as_text()) != OK:
		return {}
	var all_data: Dictionary = json.get_data()
	_affinity_cache = all_data
	return _affinity_cache.get(category, {})
