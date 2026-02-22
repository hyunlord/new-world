extends RefCounted

## StatCache: 엔티티별 스탯 캐시 관리.
## entity.stat_cache Dictionary를 직접 조작.
## StatQuery가 이 클래스를 통해 캐시를 읽고 씀.

## 캐시 엔트리 구조 (각 stat_id에 대해):
## {
##   "value": int,
##   "dirty": bool,
##   "modifiers": Array,
##   "last_computed_tick": int
## }

const StatModifierScript = preload("res://scripts/core/stat_modifier.gd")
const StatGraphScript = preload("res://scripts/core/stat_graph.gd")

## 엔트리 초기화 (존재하지 않으면 생성)
static func ensure(stat_cache: Dictionary, stat_id: StringName,
		default_value: int) -> void:
	if not stat_cache.has(stat_id):
		stat_cache[stat_id] = {
			"value": default_value,
			"dirty": false,
			"modifiers": [],
			"last_computed_tick": 0
		}

## 값 읽기 (dirty 체크 없음 — StatQuery가 담당)
static func get_value(stat_cache: Dictionary, stat_id: StringName,
		fallback: int = 0) -> int:
	if not stat_cache.has(stat_id):
		return fallback
	return int(stat_cache[stat_id].get("value", fallback))

## 값 쓰기 + downstream dirty 전파
static func set_value(stat_cache: Dictionary, stat_id: StringName,
		value: int, tick: int = 0) -> void:
	ensure(stat_cache, stat_id, value)
	stat_cache[stat_id]["value"] = value
	stat_cache[stat_id]["dirty"] = false
	stat_cache[stat_id]["last_computed_tick"] = tick
	_propagate_dirty(stat_cache, stat_id)

## dirty 마킹
static func mark_dirty(stat_cache: Dictionary, stat_id: StringName) -> void:
	if stat_cache.has(stat_id):
		stat_cache[stat_id]["dirty"] = true
	_propagate_dirty(stat_cache, stat_id)

## dirty 여부 확인
static func is_dirty(stat_cache: Dictionary, stat_id: StringName) -> bool:
	if not stat_cache.has(stat_id):
		return true
	return bool(stat_cache[stat_id].get("dirty", true))

## Modifier 추가 (같은 id → 교체, stack_group → max 비교)
static func add_modifier(stat_cache: Dictionary, modifier: RefCounted) -> void:
	var sid: StringName = modifier.target
	ensure(stat_cache, sid, 0)
	var mods: Array = stat_cache[sid]["modifiers"]

	# 같은 id → 교체
	for i in range(mods.size()):
		if StringName(mods[i].get("id", "")) == modifier.id:
			mods[i] = modifier.to_dict()
			mark_dirty(stat_cache, sid)
			return

	# stack_group → max(|value|) 비교
	if modifier.stack_group != "":
		for i in range(mods.size()):
			if mods[i].get("stack_group", "") == modifier.stack_group:
				if absf(float(mods[i].get("value", 0.0))) >= absf(modifier.value):
					return
				else:
					mods[i] = modifier.to_dict()
					mark_dirty(stat_cache, sid)
					return

	mods.append(modifier.to_dict())
	mark_dirty(stat_cache, sid)

## Modifier 제거
static func remove_modifier(stat_cache: Dictionary,
		stat_id: StringName, modifier_id: StringName) -> void:
	if not stat_cache.has(stat_id):
		return
	var mods: Array = stat_cache[stat_id]["modifiers"]
	for i in range(mods.size() - 1, -1, -1):
		if StringName(mods[i].get("id", "")) == modifier_id:
			mods.remove_at(i)
			mark_dirty(stat_cache, stat_id)
			return

## Modifier 만료 처리 (매 틱 호출)
## 반환: 만료된 modifier id 배열
static func tick_modifiers(stat_cache: Dictionary,
		stat_id: StringName) -> Array:
	if not stat_cache.has(stat_id):
		return []
	var mods: Array = stat_cache[stat_id]["modifiers"]
	var expired: Array = []
	var dirty_flag: bool = false

	for i in range(mods.size() - 1, -1, -1):
		var m: Dictionary = mods[i]
		var dur: int = int(m.get("duration", -1))
		if dur == -1:
			continue

		var decay: float = float(m.get("decay_rate", 0.0))
		if decay > 0.0:
			var new_val: float = absf(float(m.get("value", 0.0))) - decay
			if new_val <= 0.0:
				expired.append(StringName(m.get("id", "")))
				mods.remove_at(i)
				dirty_flag = true
				continue
			mods[i]["value"] = new_val * signf(float(m.get("value", 1.0)))

		mods[i]["duration"] = dur - 1
		if dur - 1 <= 0:
			expired.append(StringName(m.get("id", "")))
			mods.remove_at(i)
			dirty_flag = true

	if dirty_flag:
		mark_dirty(stat_cache, stat_id)
	return expired

## Modifier 적용 (ADD → MULTIPLY → CLAMP → OVERRIDE 순서)
static func apply_modifiers(base_value: int, mods: Array,
		stat_range: Array) -> int:
	var result: float = float(base_value)
	var override_val: float = -1.0
	var override_priority: int = -999

	for m in mods:
		var mod_type: int = int(m.get("mod_type", StatModifierScript.ModType.ADD))
		var val: float = float(m.get("value", 0.0))
		match mod_type:
			StatModifierScript.ModType.ADD:
				result += val
			StatModifierScript.ModType.MULTIPLY:
				result *= val
			StatModifierScript.ModType.CLAMP:
				var mn: float = float(m.get("clamp_min", 0.0))
				var mx: float = float(m.get("clamp_max", 99999.0))
				result = clampf(result, mn, mx)
			StatModifierScript.ModType.OVERRIDE:
				var prio: int = int(m.get("priority", 0))
				if prio > override_priority:
					override_val = val
					override_priority = prio

	if override_val >= 0.0:
		result = override_val

	var rmin: int = int(stat_range[0]) if stat_range.size() > 0 else 0
	var rmax: int = int(stat_range[1]) if stat_range.size() > 1 else 1000
	return clampi(int(result), rmin, rmax)

## downstream dirty 전파 (내부)
static func _propagate_dirty(stat_cache: Dictionary,
		changed_id: StringName) -> void:
	var deps: Array = StatGraphScript.get_dependents(changed_id)
	for dep in deps:
		if stat_cache.has(dep):
			stat_cache[dep]["dirty"] = true
