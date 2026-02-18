extends "res://scripts/core/simulation_system.gd"

# NO class_name — headless compatibility

const BASE_BREAK_THRESHOLD: float = 520.0
const THRESHOLD_MIN: float = 420.0
const THRESHOLD_MAX: float = 900.0
const BREAK_SCALE: float = 6000.0
const BREAK_CAP_PER_TICK: float = 0.25
const SHAKEN_WORK_PENALTY_MINOR: float = -0.05
const SHAKEN_WORK_PENALTY_MAJOR: float = -0.12
const SHAKEN_WORK_PENALTY_EXTREME: float = -0.20
const SHAKEN_DURATION_MINOR: int = 24
const SHAKEN_DURATION_MAJOR: int = 60
const SHAKEN_DURATION_EXTREME: int = 120

var _entity_manager: RefCounted
var _rng: RandomNumberGenerator
var _break_defs: Dictionary = {}


func _init() -> void:
	system_name = "mental_break"
	priority = 35  # stress_system(34) 바로 다음
	tick_interval = 1


## Initialize with references
func init(entity_manager: RefCounted, rng: RandomNumberGenerator) -> void:
	_entity_manager = entity_manager
	_rng = rng
	_load_break_definitions()


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


func execute_tick(tick: int) -> void:
	if _entity_manager == null:
		return
	var alive: Array = _entity_manager.get_alive_entities()
	for i in range(alive.size()):
		var entity = alive[i]
		if entity.emotion_data == null:
			continue
		var ed = entity.emotion_data
		# 이미 브레이크 중이면 카운트다운만
		if ed.mental_break_type != "":
			_tick_active_break(entity, ed)
			continue
		# break_risk 상태가 아니면 스킵
		if ed.stress_state < 3:
			continue
		_check_mental_break(entity, ed, tick)


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
	var pd = entity.personality
	var E: float = 0.5
	var C: float = 0.5
	if pd != null:
		E = pd.axes.get("E", 0.5)
		C = pd.axes.get("C", 0.5)

	var threshold: float = BASE_BREAK_THRESHOLD
	threshold *= (1.0 + 0.40 * (ed.resilience - 0.5) * 2.0)
	threshold *= (1.0 + 0.25 * (C - 0.5) * 2.0)
	threshold *= (1.0 - 0.35 * (E - 0.5) * 2.0)
	threshold *= (1.0 - 0.25 * (ed.allostatic / 100.0))
	threshold *= (0.85 + 0.15 * entity.energy)
	threshold *= (0.85 + 0.15 * entity.hunger)
	threshold = clampf(threshold, THRESHOLD_MIN, THRESHOLD_MAX)

	# GAS Exhaustion 보정
	if ed.reserve < 30.0:
		threshold -= 40.0
	if ed.reserve < 15.0:
		threshold -= 80.0

	return maxf(threshold, THRESHOLD_MIN)


## 발동 확률 계산 (stress가 threshold 초과량에 비례)
func _calc_break_chance(stress: float, threshold: float,
		reserve: float, allostatic: float) -> float:
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
	var pd = entity.personality
	var weights: Dictionary = {}
	for break_id in _break_defs:
		var bdef = _break_defs[break_id]
		var w: float = 1.0
		var pw = bdef.get("personality_weights", {})
		for axis in ["H", "E", "X", "A", "C", "O"]:
			var axis_val: float = 0.5
			if pd != null:
				axis_val = pd.axes.get(axis, 0.5)
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

	print("[MENTAL_BREAK] %s → %s (%.0f ticks, stress=%.0f)" % [
		entity.entity_name, break_type, duration, ed.stress
	])


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

	print("[MENTAL_BREAK_END] %s: %s ended, catharsis %.0f%%, shaken %d ticks" % [
		entity.entity_name, break_type, catharsis * 100.0, shaken_ticks
	])

	# 4) 상태 클리어
	ed.mental_break_type = ""
	ed.mental_break_remaining = 0.0
