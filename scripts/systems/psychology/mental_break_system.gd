extends "res://scripts/core/simulation/simulation_system.gd"

# NO class_name — headless compatibility

const _TraitEffectCache = preload("res://scripts/systems/psychology/trait_effect_cache.gd")
const BASE_BREAK_THRESHOLD: float = 520.0
const THRESHOLD_MIN: float = 420.0
const THRESHOLD_MAX: float = 900.0
const BREAK_SCALE: float = 6000.0
const BREAK_CAP_PER_TICK: float = 0.25
const _SIM_BRIDGE_NODE_NAME: String = "SimBridge"
const _SIM_BRIDGE_THRESHOLD_METHOD: String = "body_mental_break_threshold"
const _SIM_BRIDGE_CHANCE_METHOD: String = "body_mental_break_chance"
const SHAKEN_WORK_PENALTY_MINOR: float = -0.05
const SHAKEN_WORK_PENALTY_MAJOR: float = -0.12
const SHAKEN_WORK_PENALTY_EXTREME: float = -0.20
const SHAKEN_DURATION_MINOR: int = 24
const SHAKEN_DURATION_MAJOR: int = 60
const SHAKEN_DURATION_EXTREME: int = 120

var _entity_manager: RefCounted
var _rng: RandomNumberGenerator
var _break_defs: Dictionary = {}
var _trauma_scar_system = null  # TraumaScarSystem (RefCounted), set by main.gd
var _current_tick: int = 0      # Phase 4: stored for _end_break signal emission
var _bridge_checked: bool = false
var _sim_bridge: Object = null


func _init() -> void:
	system_name = "mental_break"
	priority = 35  # stress_system(34) 바로 다음
	tick_interval = 1


## Initialize with references
func init(entity_manager: RefCounted, rng: RandomNumberGenerator) -> void:
	_entity_manager = entity_manager
	_rng = rng
	_load_break_definitions()


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
	if node != null \
	and node.has_method(_SIM_BRIDGE_THRESHOLD_METHOD) \
	and node.has_method(_SIM_BRIDGE_CHANCE_METHOD):
		_sim_bridge = node
	return _sim_bridge


func set_trauma_scar_system(tss) -> void:
	_trauma_scar_system = tss


## Load break definitions from JSON
func _load_break_definitions() -> void:
	var path: String = "res://data/mental_breaks.json"
	if not FileAccess.file_exists(path):
		push_error("[MentalBreakSystem] Cannot find " + path)
		return
	var f = FileAccess.open(path, FileAccess.READ)
	if f == null:
		push_error("[MentalBreakSystem] Cannot open " + path)
		return
	var text: String = f.get_as_text()
	f.close()
	var json = JSON.new()
	var err: int = json.parse(text)
	if err != OK:
		push_error("[MentalBreakSystem] JSON parse error: " + json.get_error_message())
		return
	_break_defs = json.get_data()


## Per-tick: 발동 확률 체크
func _check_mental_break(entity: RefCounted, ed: RefCounted, tick: int) -> void:
	var threshold: float = _calc_threshold(entity, ed)
	var p: float = _calc_break_chance(ed.stress, threshold, ed.reserve, ed.allostatic)
	if p <= 0.0:
		return
	if _rng.randf() < p:
		_trigger_break(entity, ed, tick)


## 역치 계산 (Connor-Davidson resilience + HEXACO)
func _calc_threshold(entity: RefCounted, ed: RefCounted) -> float:
	var E: float = StatQuery.get_normalized(entity, &"HEXACO_E")
	var C: float = StatQuery.get_normalized(entity, &"HEXACO_C")
	var energy_norm: float = StatQuery.get_normalized(entity, &"NEED_ENERGY")
	var hunger_norm: float = StatQuery.get_normalized(entity, &"NEED_HUNGER")
	# Phase 5: ACE history permanently lowers break threshold (Teicher & Samson 2016)
	var ace_break_threshold_mult: float = float(entity.get_meta("ace_break_threshold_mult", 1.0))
	# v3 trait effect: stress/add target=mental_break_threshold
	var trait_break_threshold_add: float = _TraitEffectCache.get_stress_break_threshold_add(entity)
	var scar_threshold_reduction: float = 0.0
	if _trauma_scar_system != null:
		scar_threshold_reduction = _trauma_scar_system.get_scar_threshold_reduction(entity)

	var bridge: Object = _get_sim_bridge()
	if bridge != null:
		var rust_threshold_variant: Variant = bridge.call(
			_SIM_BRIDGE_THRESHOLD_METHOD,
			BASE_BREAK_THRESHOLD,
			ed.resilience,
			C,
			E,
			ed.allostatic,
			energy_norm,
			hunger_norm,
			ace_break_threshold_mult,
			trait_break_threshold_add,
			THRESHOLD_MIN,
			THRESHOLD_MAX,
			ed.reserve,
			scar_threshold_reduction
		)
		if rust_threshold_variant is float:
			return float(rust_threshold_variant)

	var threshold: float = BASE_BREAK_THRESHOLD
	threshold *= (1.0 + 0.40 * (ed.resilience - 0.5) * 2.0)
	threshold *= (1.0 + 0.25 * (C - 0.5) * 2.0)
	threshold *= (1.0 - 0.35 * (E - 0.5) * 2.0)
	threshold *= (1.0 - 0.25 * (ed.allostatic / 100.0))
	threshold *= (0.85 + 0.15 * energy_norm)
	threshold *= (0.85 + 0.15 * hunger_norm)
	threshold *= ace_break_threshold_mult
	threshold += trait_break_threshold_add
	threshold = clampf(threshold, THRESHOLD_MIN, THRESHOLD_MAX)

	# GAS Exhaustion 보정
	if ed.reserve < 30.0:
		threshold -= 40.0
	if ed.reserve < 15.0:
		threshold -= 80.0

	# 트라우마 흉터 역치 감소
	threshold -= scar_threshold_reduction

	return maxf(threshold, THRESHOLD_MIN)


## 발동 확률 계산 (stress가 threshold 초과량에 비례)
func _calc_break_chance(stress: float, threshold: float,
		reserve: float, allostatic: float) -> float:
	var bridge: Object = _get_sim_bridge()
	if bridge != null:
		var rust_chance_variant: Variant = bridge.call(
			_SIM_BRIDGE_CHANCE_METHOD,
			stress,
			threshold,
			reserve,
			allostatic,
			BREAK_SCALE,
			BREAK_CAP_PER_TICK
		)
		if rust_chance_variant is float:
			return float(rust_chance_variant)
	if stress <= threshold:
		return 0.0
	var p: float = clampf((stress - threshold) / BREAK_SCALE, 0.0, BREAK_CAP_PER_TICK)
	if reserve < 30.0:
		p *= 1.3
	if allostatic > 60.0:
		p *= 1.2
	return p


## 유형 선택 (HEXACO 가중치 기반 가중 랜덤)
func _select_break_type(entity: RefCounted) -> String:
	if _break_defs.is_empty():
		return "shutdown"

	# v3 trait effect: stress/replace target=break_types — override distribution
	var override_types: Dictionary = _TraitEffectCache.get_stress_break_types(entity)
	if not override_types.is_empty():
		var total_ov: float = 0.0
		for k in override_types:
			total_ov += float(override_types[k])
		if total_ov > 0.0:
			var roll_ov: float = _rng.randf() * total_ov
			var cum_ov: float = 0.0
			for k in override_types:
				cum_ov += float(override_types[k])
				if roll_ov <= cum_ov:
					return str(k)

	var weights: Dictionary = {}
	for break_id in _break_defs:
		var bdef = _break_defs[break_id]
		var w: float = 1.0
		var pw = bdef.get("personality_weights", {})
		for axis in ["H", "E", "X", "A", "C", "O"]:
			var axis_val: float = StatQuery.get_normalized(entity, StringName("HEXACO_" + axis))
			var axis_weight: float = pw.get(axis, 0.0)
			if axis_weight > 0:
				w *= lerpf(1.0, 1.0 + axis_weight, axis_val)
			elif axis_weight < 0:
				w *= lerpf(1.0, 1.0 + absf(axis_weight), 1.0 - axis_val)
		weights[break_id] = maxf(w, 0.01)

	# 가중 랜덤 선택
	var total: float = 0.0
	for k in weights:
		total += weights[k]
	var roll: float = _rng.randf() * total
	var cum: float = 0.0
	for k in weights:
		cum += weights[k]
		if roll <= cum:
			return k
	return _break_defs.keys()[0]


## 브레이크 발동
func _trigger_break(entity: RefCounted, ed: RefCounted, tick: int) -> void:
	var break_type: String = _select_break_type(entity)
	var bdef = _break_defs.get(break_type, {})

	var base: int = bdef.get("duration_base_ticks", 4)
	var variance: int = bdef.get("duration_variance_ticks", 4)
	var duration: float = float(base + _rng.randi() % (variance + 1))
	duration = clampf(duration, 1.0, 168.0)

	ed.mental_break_type = break_type
	ed.mental_break_remaining = duration

	SimulationBus.mental_break_started.emit(entity.id, break_type, tick)

	if GameConfig.DEBUG_MENTAL_BREAK_LOG:
		print("[MENTAL_BREAK] %s → %s (%.0f ticks, stress=%.0f)" % [
			entity.entity_name, break_type, duration, ed.stress
		])

## Debug: 강제 멘탈 브레이크 발동 (debug build 전용)
## _trigger_break()의 public wrapper
func force_break(entity: RefCounted, tick: int) -> void:
	if entity == null:
		return
	var ed = entity.emotion_data
	if ed == null:
		return
	if ed.mental_break_type != "":
		push_warning("[MentalBreakSystem] force_break: %s already in break (%s)" % [entity.entity_name, ed.mental_break_type])
		return
	_trigger_break(entity, ed, tick)


## 진행 중인 브레이크 틱 처리
func _tick_active_break(entity: RefCounted, ed: RefCounted) -> void:
	ed.mental_break_remaining -= 1.0
	if ed.mental_break_remaining <= 0.0:
		_end_break(entity, ed)


## 브레이크 종료 → 카타르시스 + Shaken 설정
func _end_break(entity: RefCounted, ed: RefCounted) -> void:
	var break_type: String = ed.mental_break_type
	var bdef = _break_defs.get(break_type, {})

	# 1) 카타르시스: stress 감소
	var catharsis: float = bdef.get("stress_catharsis_factor", 0.80)
	ed.stress *= catharsis

	# 2) 에너지 소모
	var energy_cost: float = bdef.get("energy_cost", 0.10)
	entity.energy = maxf(0.0, entity.energy - energy_cost)

	# 3) Shaken 상태 설정
	var severity: String = bdef.get("severity", "minor")
	var shaken_ticks: int = SHAKEN_DURATION_MINOR
	var shaken_penalty: float = SHAKEN_WORK_PENALTY_MINOR
	match severity:
		"major":
			shaken_ticks = SHAKEN_DURATION_MAJOR
			shaken_penalty = SHAKEN_WORK_PENALTY_MAJOR
		"extreme":
			shaken_ticks = SHAKEN_DURATION_EXTREME
			shaken_penalty = SHAKEN_WORK_PENALTY_EXTREME

	ed.set_meta("shaken_remaining", shaken_ticks)
	ed.set_meta("shaken_work_penalty", shaken_penalty)

	if GameConfig.DEBUG_MENTAL_BREAK_LOG:
		print("[MENTAL_BREAK_END] %s: %s ended, catharsis %.0f%%, shaken %d ticks" % [
			entity.entity_name, break_type, catharsis * 100.0, shaken_ticks
		])

	# 3.5) 트라우마 흉터 획득 시도 (TraumaScarSystem에 위임)
	if _trauma_scar_system != null:
		var scar_id: String = bdef.get("scar_id", "")
		var scar_chance: float = bdef.get("scar_chance_base", 0.0)
		if scar_id != "" and scar_chance > 0.0:
			_trauma_scar_system.try_acquire_scar(entity, scar_id, scar_chance, ed.get_meta("current_tick", 0))

	# 4) 상태 클리어
	ed.mental_break_type = ""
	ed.mental_break_remaining = 0.0
	SimulationBus.mental_break_recovered.emit(entity.id, _current_tick)
