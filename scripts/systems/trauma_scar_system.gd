extends "res://scripts/core/simulation_system.gd"

const TRAUMA_SCARS_PATH: String = "res://data/trauma_scars.json"
## Kindling effect: each existing stack increases next acquisition chance by this factor
const KINDLING_FACTOR: float = 0.30
## Global scale for tuning scar acquisition rates
const SCAR_CHANCE_SCALE: float = 1.0

var _scar_defs: Dictionary = {}
var _entity_manager = null


func _init() -> void:
	system_name = "trauma_scar"
	priority = 36
	tick_interval = 10


## Initialize with dependencies dictionary
func init(deps: Dictionary) -> void:
	priority = 36
	tick_interval = 10
	_entity_manager = deps.get("entity_manager")
	_load_scar_defs()


func _load_scar_defs() -> void:
	var file = FileAccess.open(TRAUMA_SCARS_PATH, FileAccess.READ)
	if file == null:
		push_error("[TraumaScarSystem] Cannot open %s" % TRAUMA_SCARS_PATH)
		return
	var json = JSON.new()
	var err: int = json.parse(file.get_as_text())
	file.close()
	if err != OK:
		push_error("[TraumaScarSystem] JSON parse error in %s" % TRAUMA_SCARS_PATH)
		return
	_scar_defs = json.get_data()
	print("[TraumaScarSystem] Loaded %d scar definitions" % _scar_defs.size())


## 정신붕괴 종료 시 MentalBreakSystem이 호출 — 확률적으로 흉터 획득
## scar_id: trauma_scars.json의 키, base_chance: 0.0~1.0
func try_acquire_scar(entity: RefCounted, scar_id: String, base_chance: float, tick: int) -> void:
	if scar_id.is_empty() or _scar_defs.is_empty():
		return
	var sdef: Dictionary = _scar_defs.get(scar_id, {})
	if sdef.is_empty():
		return

	var max_stacks: int = int(sdef.get("max_stacks", 3))
	var existing_stacks: int = _get_scar_stacks(entity, scar_id)
	if existing_stacks >= max_stacks:
		return

	# Kindling Theory: 기존 스택이 있을수록 다음 획득 확률 증가
	var chance: float = base_chance * SCAR_CHANCE_SCALE
	if existing_stacks > 0:
		chance *= (1.0 + KINDLING_FACTOR * float(existing_stacks))
	chance = clampf(chance, 0.0, 1.0)

	if randf() >= chance:
		return

	# 흉터 추가 또는 기존 스택 증가
	if existing_stacks > 0:
		for i in range(entity.trauma_scars.size()):
			if entity.trauma_scars[i].get("scar_id") == scar_id:
				entity.trauma_scars[i]["stacks"] += 1
				break
	else:
		entity.trauma_scars.append({"scar_id": scar_id, "stacks": 1, "acquired_tick": tick})

	var new_stacks: int = existing_stacks + 1
	print("[TraumaScarSystem] %s acquired scar: %s (stacks: %d, chance: %.1f%%)" % [
		entity.entity_name, scar_id, new_stacks, chance * 100.0
	])

	if SimulationBus.has_signal("scar_acquired"):
		SimulationBus.scar_acquired.emit({
			"entity_id": entity.id,
			"entity_name": entity.entity_name,
			"scar_id": scar_id,
			"scar_name_kr": sdef.get("name_kr", scar_id),
			"stacks": new_stacks,
			"tick": tick,
		})


## StressSystem의 inject_event()에서 호출 — 흉터 재활성화 체크
func check_reactivation(entity: RefCounted, context_type: String, tick: int) -> void:
	if entity.trauma_scars.is_empty():
		return
	for scar_entry in entity.trauma_scars:
		var scar_id: String = scar_entry.get("scar_id", "")
		if scar_id.is_empty():
			continue
		var sdef: Dictionary = _scar_defs.get(scar_id, {})
		var triggers: Array = sdef.get("reactivation_triggers", [])
		if context_type in triggers:
			print("[TraumaScarSystem] %s scar reactivated: %s (trigger: %s)" % [
				entity.entity_name, scar_id, context_type
			])
			if SimulationBus.has_signal("scar_reactivated"):
				SimulationBus.scar_reactivated.emit({
					"entity_id": entity.id,
					"entity_name": entity.entity_name,
					"scar_id": scar_id,
					"scar_name_kr": sdef.get("name_kr", scar_id),
					"trigger": context_type,
					"tick": tick,
				})


## MentalBreakSystem._calc_threshold()에서 호출 — 총 역치 감소량 반환
func get_scar_threshold_reduction(entity: RefCounted) -> float:
	var total: float = 0.0
	for scar_entry in entity.trauma_scars:
		var scar_id: String = scar_entry.get("scar_id", "")
		var stacks: int = int(scar_entry.get("stacks", 1))
		var sdef: Dictionary = _scar_defs.get(scar_id, {})
		var per_stack: float = float(sdef.get("break_threshold_reduction", 0.0))
		total += per_stack * float(stacks)
	return total


## StressSystem.inject_event()에서 호출 — 스트레스 민감도 배수 반환
func get_scar_stress_sensitivity(entity: RefCounted) -> float:
	var mult: float = 1.0
	for scar_entry in entity.trauma_scars:
		var scar_id: String = scar_entry.get("scar_id", "")
		var stacks: int = int(scar_entry.get("stacks", 1))
		var sdef: Dictionary = _scar_defs.get(scar_id, {})
		var base_mult: float = float(sdef.get("stress_sensitivity_mult", 1.0))
		# 스택 초과분은 50% 감쇠 (diminishing returns)
		var delta: float = base_mult - 1.0
		mult *= (1.0 + delta * (1.0 + 0.5 * float(stacks - 1)))
	return clampf(mult, 0.5, 3.0)


## StressSystem._update_resilience()에서 호출 — 회복력 재생 모디파이어 반환
func get_scar_resilience_mod(entity: RefCounted) -> float:
	var total: float = 0.0
	for scar_entry in entity.trauma_scars:
		var scar_id: String = scar_entry.get("scar_id", "")
		var stacks: int = int(scar_entry.get("stacks", 1))
		var sdef: Dictionary = _scar_defs.get(scar_id, {})
		var per_stack: float = float(sdef.get("resilience_mod", 0.0))
		total += per_stack * float(stacks)
	return total


## UI에서 사용 — 단일 흉터 정의 반환
func get_scar_def(scar_id: String) -> Dictionary:
	return _scar_defs.get(scar_id, {})


## 감정 기준선 드리프트 적용 (매 tick_interval 틱마다)
func execute_tick(tick: int) -> void:
	if _entity_manager == null:
		return
	var entities = _entity_manager.get_all_entities()
	for entity in entities:
		if not entity.is_alive:
			continue
		if entity.trauma_scars.is_empty():
			continue
		_apply_emotion_drift(entity)


## 흉터의 감정 기준선 드리프트를 매우 작은 양으로 적용
func _apply_emotion_drift(entity: RefCounted) -> void:
	for scar_entry in entity.trauma_scars:
		var scar_id: String = scar_entry.get("scar_id", "")
		var stacks: int = int(scar_entry.get("stacks", 1))
		var sdef: Dictionary = _scar_defs.get(scar_id, {})
		var shifts: Dictionary = sdef.get("emotion_baseline_shifts", {})
		for emotion in shifts:
			var shift_per_tick: float = float(shifts[emotion]) * 0.001 * float(stacks)
			var current: float = float(entity.emotions.get(emotion, 0.0))
			entity.emotions[emotion] = clampf(current + shift_per_tick, 0.0, 1.0)


func _get_scar_stacks(entity: RefCounted, scar_id: String) -> int:
	for scar_entry in entity.trauma_scars:
		if scar_entry.get("scar_id") == scar_id:
			return int(scar_entry.get("stacks", 0))
	return 0
